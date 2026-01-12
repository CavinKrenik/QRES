use super::heuristic::MovingAveragePredictor;
use super::onnx::NeuralPredictor;
use std::path::Path;

/// Hybrid predictor that switches between Heuristic and Neural based on signal complexity.
pub struct HybridPredictor {
    heuristic: MovingAveragePredictor,
    neural: Option<NeuralPredictor>,
    /// Variance threshold to trigger neural inference.
    /// Below this: Signal is "stable" -> Use Heuristic
    /// Above this: Signal is "complex" -> Use Neural
    threshold: f32,
}

impl HybridPredictor {
    pub fn new<P: AsRef<Path>>(onnx_path: Option<P>, threshold: f32) -> Self {
        let neural = if let Some(path) = onnx_path {
            match NeuralPredictor::load(path.as_ref()) {
                Ok(p) => Some(p),
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to load NeuralPredictor: {}. Using heuristic only.",
                        e
                    );
                    None
                }
            }
        } else {
            None
        };

        Self {
            heuristic: MovingAveragePredictor::new(NeuralPredictor::WINDOW_SIZE),
            neural,
            threshold,
        }
    }

    /// Predict next value using dynamic routing
    pub fn predict(&self, window: &[f32]) -> f32 {
        // 1. Calculate Complexity (Variance)
        // This is O(N) but N=32 so it's extremely fast (~20ns)
        // Optimization: We could use zero-crossing rate or just check if last delta is large
        let variance = self.calculate_variance(window);

        // 2. Route
        if variance > self.threshold {
            // Signal is volatile/complex - use Brain
            if let Some(neural) = &self.neural {
                // Ensure window size matches
                if window.len() == NeuralPredictor::WINDOW_SIZE {
                    if let Ok(val) = neural.predict(window) {
                        return val;
                    }
                }
            }
        }

        // 3. Fallback (Stable signal or Neural failed)
        self.heuristic.predict(window)
    }

    pub fn calculate_variance(&self, window: &[f32]) -> f32 {
        if window.is_empty() {
            return 0.0;
        }
        let mean = window.iter().sum::<f32>() / window.len() as f32;
        let variance = window
            .iter()
            .map(|&x| {
                let diff = x - mean;
                diff * diff
            })
            .sum::<f32>()
            / window.len() as f32;
        variance
    }

    /// Force use of heuristic (for benchmarking)
    pub fn predict_heuristic(&self, window: &[f32]) -> f32 {
        self.heuristic.predict(window)
    }

    /// Force use of neural (for benchmarking)
    pub fn predict_neural(&self, window: &[f32]) -> Option<f32> {
        if let Some(neural) = &self.neural {
            if window.len() == NeuralPredictor::WINDOW_SIZE {
                return neural.predict(window).ok();
            }
        }
        None
    }
}
