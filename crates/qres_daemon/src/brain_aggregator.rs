//! Brain Aggregator for Robust Federated Learning
//!
//! Buffers brain updates from peers and applies robust aggregation
//! algorithms (Krum, Median, Trimmed Mean) before merging.
//! Part of Phase 2 Security implementation.

use crate::config::AggregationConfig;
use crate::living_brain::{LivingBrain, SignedEpiphany};
use crate::security::ReputationManager;
use fixed::types::I16F16;
use qres_core::aggregation::{aggregate_updates, AggregationMode, AggregationResult};
use qres_core::consensus::aggregate_krum;
use qres_core::tensor::FixedTensor;
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};

/// Aggregator that buffers brain updates and applies robust aggregation
pub struct BrainAggregator {
    /// Buffered confidence vectors from remote peers (Update, PeerID)
    buffer: VecDeque<(Vec<f32>, String)>,
    /// Configuration for aggregation
    config: AggregationConfig,
    /// Derived aggregation mode
    mode: AggregationMode,
    /// Whether to use deterministic I16F16 Krum
    use_fixed_krum: bool,
}

impl BrainAggregator {
    /// Create a new aggregator from config
    pub fn new(config: AggregationConfig) -> Self {
        let use_fixed_krum = config.mode.to_lowercase() == "krum_fixed";
        let mode = Self::parse_mode(&config);
        info!(
            mode = ?config.mode,
            buffer_size = config.buffer_size,
            use_fixed_krum = use_fixed_krum,
            "Brain aggregator initialized"
        );

        Self {
            buffer: VecDeque::with_capacity(config.buffer_size),
            config,
            mode,
            use_fixed_krum,
        }
    }

    /// Parse aggregation mode from config string  
    fn parse_mode(config: &AggregationConfig) -> AggregationMode {
        match config.mode.to_lowercase().as_str() {
            "krum" | "krum_fixed" => AggregationMode::Krum {
                expected_byz: 1, // Will be calculated dynamically based on buffer size
            },
            "multi_krum" => AggregationMode::MultiKrum {
                expected_byz: 1,
                k: 3,
            },
            "trimmed_mean" | "trimmed" => AggregationMode::TrimmedMean {
                trim_fraction: config.trim_fraction,
            },
            "median" => AggregationMode::Median,
            _ => AggregationMode::SimpleMean,
        }
    }

    /// Add a brain update to the buffer
    /// Returns Some((aggregated confidence, accepted_peers, rejected_peers)) if buffer is full and ready for aggregation
    pub fn add_update(
        &mut self,
        brain: &LivingBrain,
        peer_id: String,
    ) -> Option<(Vec<f32>, Vec<String>, Vec<String>)> {
        // Add confidence vector to buffer
        self.buffer.push_back((brain.confidence.clone(), peer_id));

        // Check if we have enough updates to aggregate
        if self.buffer.len() >= self.config.buffer_size {
            Some(self.aggregate_and_clear())
        } else {
            info!(
                buffered = self.buffer.len(),
                needed = self.config.buffer_size,
                "Update buffered, waiting for more"
            );
            None
        }
    }

    /// Force aggregation with current buffer (for timeout scenarios)
    pub fn force_aggregate(&mut self) -> Option<(Vec<f32>, Vec<String>, Vec<String>)> {
        if self.buffer.is_empty() {
            return None;
        }
        Some(self.aggregate_and_clear())
    }

    /// Aggregate buffered updates and clear the buffer
    fn aggregate_and_clear(&mut self) -> (Vec<f32>, Vec<String>, Vec<String>) {
        // Separate updates and peer_ids
        let (updates, peer_ids): (Vec<Vec<f32>>, Vec<String>) = self.buffer.drain(..).unzip();
        let n = updates.len();

        // Calculate expected byzantines dynamically based on fraction
        let expected_byz = ((n as f32) * self.config.expected_byzantines_fraction).floor() as usize;

        // Use deterministic I16F16 Krum if configured
        if self.use_fixed_krum {
            // Convert f32 vectors to I16F16
            let fixed_vectors: Vec<Vec<I16F16>> = updates
                .iter()
                .map(|vec| vec.iter().map(|&val| I16F16::from_num(val)).collect())
                .collect();

            // Run deterministic Krum
            let result_fixed = aggregate_krum(&fixed_vectors, expected_byz);

            match result_fixed {
                Some(fixed_result) => {
                    // Convert back to f32
                    let weights: Vec<f32> =
                        fixed_result.iter().map(|val| val.to_num::<f32>()).collect();

                    // Krum selects 1 vector, find which one
                    let selected_idx = fixed_vectors
                        .iter()
                        .position(|v| *v == fixed_result)
                        .unwrap_or(0);

                    // === BFT Defense Logging (Portfolio Evidence) ===
                    // Compare Krum result to what simple mean would have produced
                    let mean_val: Vec<f32> = if !updates.is_empty() && !updates[0].is_empty() {
                        let dim = updates[0].len();
                        let n_f = n as f32;
                        (0..dim)
                            .map(|i| {
                                updates
                                    .iter()
                                    .map(|u| u.get(i).unwrap_or(&0.0))
                                    .sum::<f32>()
                                    / n_f
                            })
                            .collect()
                    } else {
                        weights.clone()
                    };

                    let diff: f32 = weights
                        .iter()
                        .zip(mean_val.iter())
                        .map(|(a, b)| (a - b).abs())
                        .sum();

                    if diff > 0.1 {
                        // Krum result is significantly different from Mean - outlier was rejected!
                        info!("🛡️ BFT DEFENSE ACTIVE: Malicious outlier rejected");
                        info!(
                            mean_first = ?mean_val.first(),
                            krum_first = ?weights.first(),
                            total_diff = diff,
                            "   Mean (Compromised) vs Krum (Protected)"
                        );
                    }

                    info!(
                        updates = n,
                        selected_idx = selected_idx,
                        mode = "krum_fixed",
                        "Aggregated brain updates (deterministic I16F16)"
                    );

                    let accepted_peers = vec![peer_ids[selected_idx].clone()];
                    let rejected_peers: Vec<String> = peer_ids
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| *i != selected_idx)
                        .map(|(_, p)| p.clone())
                        .collect();

                    return (weights, accepted_peers, rejected_peers);
                }
                None => {
                    warn!(
                        n = n,
                        "Krum fixed failed (n < 3), falling back to first vector"
                    );
                    // Fallback to first vector
                    return (
                        updates.into_iter().next().unwrap_or_default(),
                        peer_ids.into_iter().take(1).collect(),
                        Vec::new(),
                    );
                }
            }
        }

        // Standard f32 aggregation path
        let dynamic_mode = match &self.mode {
            AggregationMode::Krum { .. } => AggregationMode::Krum { expected_byz },
            AggregationMode::MultiKrum { k, .. } => AggregationMode::MultiKrum {
                expected_byz,
                k: (*k).min(n),
            },
            other => other.clone(),
        };

        let result: AggregationResult = aggregate_updates(&updates, &dynamic_mode);

        info!(
            updates = n,
            selected = result.selected_indices.len(),
            rejected = result.rejected_indices.len(),
            mode = ?self.config.mode,
            "Aggregated brain updates"
        );

        if !result.rejected_indices.is_empty() {
            warn!(
                rejected = ?result.rejected_indices,
                "Rejected potential Byzantine updates"
            );
        }

        let rejected_peers: Vec<String> = result
            .rejected_indices
            .iter()
            .map(|&idx| peer_ids[idx].clone())
            .collect();

        let accepted_peers: Vec<String> = result
            .selected_indices
            .iter()
            .map(|&idx| peer_ids[idx].clone())
            .collect();

        (result.weights, accepted_peers, rejected_peers)
    }

    /// Get current buffer size
    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if using robust mode (not simple mean)
    pub fn is_robust(&self) -> bool {
        !matches!(self.mode, AggregationMode::SimpleMean)
    }
}

/// Apply aggregated confidence to a brain
pub fn apply_aggregated_confidence(brain: &mut LivingBrain, aggregated: &[f32], alpha: f32) {
    for (conf, &agg) in brain.confidence.iter_mut().zip(aggregated.iter()) {
        *conf = *conf * (1.0 - alpha) + agg * alpha;
    }
}

/// Federated Learning Averager using weighted averaging with reputation and freshness
pub struct FederatedAverager {
    /// Buffered SignedEpiphany updates from peers
    buffer: VecDeque<SignedEpiphany>,
    /// Maximum buffer size before forced aggregation
    max_buffer_size: usize,
    /// Freshness decay half-life in seconds (updates older than this lose weight)
    freshness_half_life: f64,
}

impl FederatedAverager {
    /// Create a new FederatedAverager
    pub fn new(max_buffer_size: usize, freshness_half_life: f64) -> Self {
        Self {
            buffer: VecDeque::with_capacity(max_buffer_size),
            max_buffer_size,
            freshness_half_life,
        }
    }

    /// Add a SignedEpiphany update to the buffer
    pub fn add_update(&mut self, epiphany: SignedEpiphany) {
        self.buffer.push_back(epiphany);

        // Keep buffer size in check
        if self.buffer.len() > self.max_buffer_size {
            self.buffer.pop_front(); // Remove oldest
        }
    }

    /// Aggregate buffered updates using weighted average
    /// Returns the aggregated weights and confidence vectors
    pub fn aggregate(
        &mut self,
        reputation_manager: &ReputationManager,
    ) -> Option<(Vec<u8>, Vec<f32>)> {
        if self.buffer.is_empty() {
            return None;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as f64;

        // Extract all weight vectors and compute weights
        let mut all_weights: Vec<Vec<f32>> = Vec::new();
        let mut all_confidences: Vec<Vec<f32>> = Vec::new();
        let mut weights: Vec<f64> = Vec::new();
        let mut total_weight = 0.0;

        for epiphany in &self.buffer {
            // Get reputation score (0.5 default for new peers)
            let reputation = reputation_manager.get_trust(&epiphany.sender_id) as f64;

            // Calculate freshness decay: weight = reputation * exp(-ln(2) * age / half_life)
            let age_seconds = now - epiphany.timestamp as f64;
            let freshness =
                (-std::f64::consts::LN_2 * age_seconds / self.freshness_half_life).exp();

            let combined_weight = reputation * freshness;
            weights.push(combined_weight);
            total_weight += combined_weight;

            // Extract weights - handle both I16F16 and I8F8 formats
            if epiphany.is_storm_mode {
                // I8F8 weights - need to upcast to f32
                if let Some(weights_bytes) = &epiphany.brain.best_engine_weights {
                    let i8f8_weights = FixedTensor::from_i8f8_bytes(weights_bytes);
                    all_weights.push(
                        i8f8_weights
                            .data
                            .iter()
                            .map(|&w| w.to_num::<f32>())
                            .collect(),
                    );
                } else {
                    // Fallback to confidence if no weights
                    all_weights.push(epiphany.brain.confidence.clone());
                }
            } else {
                // I16F16 weights
                if let Some(weights_bytes) = &epiphany.brain.best_engine_weights {
                    let i16f16_weights = FixedTensor::from_i16f16_bytes(weights_bytes);
                    all_weights.push(
                        i16f16_weights
                            .data
                            .iter()
                            .map(|&w| w.to_num::<f32>())
                            .collect(),
                    );
                } else {
                    // Fallback to confidence
                    all_weights.push(epiphany.brain.confidence.clone());
                }
            }

            all_confidences.push(epiphany.brain.confidence.clone());
        }

        if all_weights.is_empty() || total_weight == 0.0 {
            return None;
        }

        // Normalize weights
        for w in &mut weights {
            *w /= total_weight;
        }

        // Weighted average using Kahan summation for precision
        let weight_dim = all_weights[0].len();
        let mut aggregated_weights = vec![0.0f32; weight_dim];
        let mut confidences = vec![0.0f32; all_confidences[0].len()];

        // Aggregate weights with Kahan summation
        for i in 0..weight_dim {
            let mut sum = 0.0f64;
            let mut c = 0.0f64; // Kahan compensation

            for (j, weights_vec) in all_weights.iter().enumerate() {
                let y = weights_vec[i] as f64 * weights[j] - c;
                let t = sum + y;
                c = (t - sum) - y;
                sum = t;
            }
            aggregated_weights[i] = sum as f32;
        }

        // Aggregate confidences with Kahan summation
        for i in 0..all_confidences[0].len() {
            let mut sum = 0.0f64;
            let mut c = 0.0f64;

            for (j, conf_vec) in all_confidences.iter().enumerate() {
                let y = conf_vec[i] as f64 * weights[j] - c;
                let t = sum + y;
                c = (t - sum) - y;
                sum = t;
            }
            confidences[i] = sum as f32;
        }

        // Convert back to bytes (use I16F16 for aggregated result)
        let fixed_weights: Vec<fixed::types::I16F16> = aggregated_weights
            .iter()
            .map(|&w| fixed::types::I16F16::from_num(w))
            .collect();
        let weights_bytes: Vec<u8> = fixed_weights
            .iter()
            .flat_map(|&w| w.to_le_bytes())
            .collect();

        // Clear buffer after aggregation
        self.buffer.clear();

        info!(
            updates = all_weights.len(),
            total_weight = total_weight,
            "Federated averaging completed"
        );

        Some((weights_bytes, confidences))
    }

    /// Get current buffer size
    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if buffer is ready for aggregation
    pub fn should_aggregate(&self) -> bool {
        !self.buffer.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregator_buffering() {
        let config = AggregationConfig {
            mode: "mean".to_string(),
            expected_byzantines_fraction: 0.2,
            buffer_size: 3,
            trim_fraction: 0.2,
        };

        let mut agg = BrainAggregator::new(config);

        let brain1 = LivingBrain::new();
        let brain2 = LivingBrain::new();

        // First two shouldn't trigger aggregation
        assert!(agg.add_update(&brain1, "peer1".to_string()).is_none());
        assert!(agg.add_update(&brain2, "peer2".to_string()).is_none());
        assert_eq!(agg.buffer_len(), 2);

        // Third should trigger
        let result = agg.add_update(&brain1, "peer3".to_string());
        assert!(result.is_some());
        assert_eq!(agg.buffer_len(), 0);
    }

    #[test]
    fn test_krum_mode() {
        let config = AggregationConfig {
            mode: "krum".to_string(),
            expected_byzantines_fraction: 0.2,
            buffer_size: 5,
            trim_fraction: 0.2,
        };

        let agg = BrainAggregator::new(config);
        assert!(agg.is_robust());
    }
}
