use crate::inference::onnx::NeuralPredictor;
use anyhow::{anyhow, Result};
use std::collections::VecDeque;
use std::path::Path;

/// Simple heuristic predictor using Weighted Moving Average
pub struct MovingAveragePredictor {
    window_size: usize,
}

impl MovingAveragePredictor {
    pub fn new(window_size: usize) -> Self {
        Self { window_size }
    }

    pub fn predict(&self, window: &[f32]) -> f32 {
        if window.is_empty() {
            return 0.0;
        }

        let len = window.len();
        let mut sum = 0.0;
        let mut weight_sum = 0.0;

        // Give more weight to recent values (Linear decay)
        for (i, &val) in window.iter().enumerate() {
            let weight = (i + 1) as f32;
            sum += val * weight;
            weight_sum += weight;
        }

        if weight_sum > 0.0 {
            sum / weight_sum
        } else {
            0.0
        }
    }
}

pub struct ResourceUsagePredictor {
    neural: Option<NeuralPredictor>,
    heuristic: MovingAveragePredictor,
}

impl ResourceUsagePredictor {
    pub fn new<P: AsRef<Path>>(onnx_path: Option<P>) -> Self {
        let neural = if let Some(path) = onnx_path {
            match NeuralPredictor::load(path.as_ref()) {
                Ok(p) => Some(p),
                Err(e) => {
                    eprintln!("Warning: Failed to load NeuralPredictor: {}. Using heuristic only.", e);
                    None
                }
            }
        } else {
            None
        };

        Self {
            neural,
            heuristic: MovingAveragePredictor::new(32),
        }
    }

    /// Hybrid prediction: Try Neural, fallback to Heuristic
    pub fn predict(&self, window: &[f32]) -> f32 {
        // Try Neural first
        if let Some(neural) = &self.neural {
            if window.len() == NeuralPredictor::WINDOW_SIZE {
                if let Ok(val) = neural.predict(window) {
                    return val;
                }
            }
        }

        // Fallback
        self.heuristic.predict(window)
    }

    /// Force use of heuristic (good for benchmarking baseline)
    pub fn predict_heuristic(&self, window: &[f32]) -> f32 {
        self.heuristic.predict(window)
    }

    /// Force use of neural (good for benchmarking overhead)
    /// Returns None if neural model not loaded or window size mismatch
    pub fn predict_neural(&self, window: &[f32]) -> Option<f32> {
        if let Some(neural) = &self.neural {
             if window.len() == NeuralPredictor::WINDOW_SIZE {
                 return neural.predict(window).ok();
             }
        }
        None
    }
}

// --- Worker Pool Simulation ---
pub struct WorkerPool {
    pub current_capacity: usize,
}

impl Default for WorkerPool {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkerPool {
    pub fn new() -> Self {
        Self {
            current_capacity: 2, // Min threads
        }
    }

    /// Adjust capacity based on predicted load.
    /// Load is 0.0 to 1.0 (or higher if overloaded)
    /// Mapping: 0.0 -> 2 threads, 1.0 -> 16 threads.
    pub fn adjust_capacity(&mut self, predicted_load: f32) -> usize {
        // Linear mapping: y = 2 + (x * 14)
        // Clamp x to [0.0, 1.0] for safety approx
        let load = predicted_load.max(0.0).min(1.2); // Allow slight overprovision if > 1.0
        
        // Calculate raw target
        let raw_target = 2.0 + (load * 14.0);
        
        let target = raw_target.round() as usize;
        let clamped = target.clamp(2, 16);
        
        self.current_capacity = clamped;
        clamped
    }
}
