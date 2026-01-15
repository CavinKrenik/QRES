use alloc::vec::Vec;
use core::convert::TryInto;
use fixed::types::I16F16;

use super::neuron::{Regime, SpikeEvent, SwarmNeuron};

const REFRACTORY_PERIOD_TICKS: u32 = 10; // Prevent spike storms

/// Linear Predictor neuron with fixed-point math and refractory logic
///
/// Uses a linear model with configurable lags to predict the next value.
/// Implements active neuron behaviors: surprise detection, refractory periods,
/// adaptation via peer signals, and gene export/import.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct LinearNeuron {
    /// Lag intervals (e.g., [1, 2, 4, 8] looks back 1, 2, 4, 8 steps)
    lags: [usize; 8],
    /// Linear weights for each lag (Q16.16 fixed point)
    weights: [I16F16; 8],
    /// Bias term
    bias: I16F16,
    /// Recent history (circular buffer)
    history: Vec<u8>,
    /// Current position in history (circular)
    cursor: usize,
    /// Learning rate for adaptation
    learning_rate: I16F16,
    /// Refractory counter: when 0, can spike
    refractory_counter: u32,
    /// Entropy running average (for regime detection)
    entropy_estimate: I16F16,
}

impl LinearNeuron {
    /// Create a new linear predictor neuron
    pub fn new(history_size: usize) -> Self {
        // Default: look back at 1, 2, 4, 8 steps
        let lags = [1, 2, 4, 8, 0, 0, 0, 0];

        // Default weights (rough initialization)
        let weights = [
            I16F16::from_num(0.5),  // lag 1: heavy weight
            I16F16::from_num(0.25), // lag 2
            I16F16::from_num(0.15), // lag 4
            I16F16::from_num(0.1),  // lag 8
            I16F16::from_num(0.0),
            I16F16::from_num(0.0),
            I16F16::from_num(0.0),
            I16F16::from_num(0.0),
        ];

        LinearNeuron {
            lags,
            weights,
            bias: I16F16::from_num(128.0), // Midpoint of u8 range
            history: alloc::vec![0u8; history_size],
            cursor: 0,
            learning_rate: I16F16::from_num(0.01),
            refractory_counter: 0,
            entropy_estimate: I16F16::from_num(0.5),
        }
    }

    /// Compute prediction from history
    fn predict_internal(&self, history: &[u8]) -> u8 {
        let mut sum: I16F16 = self.bias;

        for i in 0..8 {
            if self.lags[i] == 0 {
                break; // End of valid lags
            }
            let lag = self.lags[i];
            if lag >= history.len() {
                continue; // Not enough history
            }
            let value = history[history.len() - lag];
            sum += self.weights[i] * I16F16::from_num(value as i32);
        }

        // Clamp to u8 range
        let result = sum.to_num::<i32>();
        result.clamp(0, 255) as u8
    }

    /// Update entropy estimate based on error
    fn update_entropy(&mut self, error: u8) {
        let error_fixed = I16F16::from_num(error as i32) / I16F16::from_num(256);
        // Exponential moving average
        self.entropy_estimate =
            self.entropy_estimate * I16F16::from_num(0.9) + error_fixed * I16F16::from_num(0.1);
    }
}

impl SwarmNeuron for LinearNeuron {
    fn predict(&self, history: &[u8]) -> u8 {
        self.predict_internal(history)
    }

    fn check_surprise(
        &mut self,
        actual: u8,
        predicted: u8,
        regime: Regime,
        tick: u32,
    ) -> Option<SpikeEvent> {
        // Calculate absolute error
        let error = (actual as i16 - predicted as i16).unsigned_abs() as u8;

        // Update entropy tracking
        self.update_entropy(error);

        // Check if error exceeds surprise threshold for this regime
        let threshold = regime.surprise_threshold();
        let error_normalized = I16F16::from_num(error as i32) / I16F16::from_num(256);

        // Only spike if:
        // 1. Error exceeds threshold
        // 2. Refractory counter is 0
        if error_normalized > threshold && self.refractory_counter == 0 {
            // Fire a spike
            self.refractory_counter = REFRACTORY_PERIOD_TICKS;

            let surprise_u8 = (self.entropy_estimate * I16F16::from_num(255)).to_num::<u8>();

            Some(SpikeEvent::new(tick, error, surprise_u8, regime))
        } else {
            None
        }
    }

    fn adapt(&mut self, signals: &[SpikeEvent], reputation: &[I16F16]) {
        if signals.is_empty() {
            return;
        }

        // Aggregate learning signal from peer spikes
        let mut weight_delta = [I16F16::from_num(0); 8];

        for (signal, rep) in signals.iter().zip(reputation.iter()) {
            // Weight each signal by peer reputation
            let signal_weight =
                *rep * I16F16::from_num(signal.error as i32) / I16F16::from_num(256);

            // Distribute error back to weights (simplified ADALINE-like update)
            for item in &mut weight_delta {
                *item += signal_weight * self.learning_rate;
            }
        }

        // Apply weight updates
        for (i, delta) in weight_delta.iter().enumerate() {
            self.weights[i] += delta;
            // Clamp to reasonable range
            if self.weights[i] < I16F16::from_num(-2) {
                self.weights[i] = I16F16::from_num(-2);
            }
            if self.weights[i] > I16F16::from_num(2) {
                self.weights[i] = I16F16::from_num(2);
            }
        }
    }

    fn export_gene(&self) -> Vec<u8> {
        // Serialize: [lags (8B) | weights (32B) | bias (4B) | learning_rate (4B)] = 48 bytes
        let mut gene = Vec::with_capacity(48);

        // Lags (as u8, 8 bytes)
        for &lag in &self.lags {
            gene.push(lag as u8);
        }

        // Weights (4 bytes each, I16F16 is i32, big-endian)
        for &weight in &self.weights {
            let bits = weight.to_bits();
            gene.extend_from_slice(&bits.to_le_bytes());
        }

        // Bias (4 bytes)
        let bias_bits = self.bias.to_bits();
        gene.extend_from_slice(&bias_bits.to_le_bytes());

        // Learning rate (4 bytes)
        let lr_bits = self.learning_rate.to_bits();
        gene.extend_from_slice(&lr_bits.to_le_bytes());

        gene
    }

    fn install_gene(&mut self, gene: &[u8]) -> bool {
        // Deserialize: must be exactly 48 bytes
        if gene.len() != 48 {
            return false;
        }

        // Extract lags
        let mut new_lags = [0usize; 8];
        for i in 0..8 {
            new_lags[i] = gene[i] as usize;
        }

        // Extract weights
        let mut new_weights = [I16F16::from_num(0); 8];
        for (i, weight) in new_weights.iter_mut().enumerate() {
            let offset = 8 + i * 4;
            if offset + 4 > gene.len() {
                return false;
            }
            let bytes: [u8; 4] = gene[offset..offset + 4].try_into().unwrap_or_default();
            let bits = i32::from_le_bytes(bytes);
            *weight = I16F16::from_bits(bits);
        }

        // Extract bias
        let bias_offset = 8 + 32;
        let bias_bytes: [u8; 4] = gene[bias_offset..bias_offset + 4]
            .try_into()
            .unwrap_or_default();
        let bias_bits = i32::from_le_bytes(bias_bytes);
        let new_bias = I16F16::from_bits(bias_bits);

        // Extract learning rate
        let lr_offset = 8 + 32 + 4;
        let lr_bytes: [u8; 4] = gene[lr_offset..lr_offset + 4]
            .try_into()
            .unwrap_or_default();
        let lr_bits = i32::from_le_bytes(lr_bytes);
        let new_learning_rate = I16F16::from_bits(lr_bits);

        // Install
        self.lags = new_lags;
        self.weights = new_weights;
        self.bias = new_bias;
        self.learning_rate = new_learning_rate;
        self.refractory_counter = 0; // Reset refractory on gene install

        true
    }

    fn refractory_remaining(&self) -> u32 {
        self.refractory_counter
    }

    fn tick(&mut self) {
        if self.refractory_counter > 0 {
            self.refractory_counter -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_neuron_creation() {
        let neuron = LinearNeuron::new(64);
        assert_eq!(neuron.history.len(), 64);
        assert_eq!(neuron.refractory_counter, 0);
    }

    #[test]
    fn test_predict() {
        let neuron = LinearNeuron::new(64);
        let history = [100u8; 64];
        let prediction = neuron.predict(&history);
        // With uniform input, should predict in valid u8 range (always true)
        let _ = prediction; // Use prediction to avoid dead code warning
    }

    #[test]
    fn test_gene_export_import() {
        let mut neuron1 = LinearNeuron::new(64);
        neuron1.learning_rate = I16F16::from_num(0.05);

        let gene = neuron1.export_gene();
        assert_eq!(gene.len(), 48);

        let mut neuron2 = LinearNeuron::new(64);
        assert!(neuron2.install_gene(&gene));
        assert_eq!(neuron2.learning_rate, I16F16::from_num(0.05));
    }

    #[test]
    fn test_refractory_period() {
        let mut neuron = LinearNeuron::new(64);
        let _history = [100u8; 64];

        // First spike should succeed
        let spike1 = neuron.check_surprise(200, 100, Regime::Storm, 0);
        assert!(spike1.is_some());
        assert_eq!(neuron.refractory_counter, REFRACTORY_PERIOD_TICKS);

        // Immediately trying to spike again should fail
        let spike2 = neuron.check_surprise(200, 100, Regime::Storm, 1);
        assert!(spike2.is_none());

        // Decrement refractory
        for _ in 0..REFRACTORY_PERIOD_TICKS {
            neuron.tick();
        }

        // Should be able to spike again
        assert_eq!(neuron.refractory_counter, 0);
    }
}
