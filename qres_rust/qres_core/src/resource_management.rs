use crate::inference::hybrid_predictor::HybridPredictor;
use std::path::Path;

// Re-export MovingAveragePredictor for compatibility if needed, 
// but primarily it is used internally relative to this module in benchmarks?
// benchmarks usually import `qres_core::resource_management::ResourceUsagePredictor`.
// If they imported `MovingAveragePredictor`, we might break them. 
// Let's re-export it.
// pub use crate::inference::heuristic::MovingAveragePredictor;

pub struct ResourceUsagePredictor {
    inner: HybridPredictor,
}

impl ResourceUsagePredictor {
    pub fn new<P: AsRef<Path>>(onnx_path: Option<P>) -> Self {
        // Default threshold of 0.01 (1% variance)
        // If variance is < 0.01 (very smooth), use Heuristic
        // If variance is > 0.01 (chaotic), use Neural
        Self {
            inner: HybridPredictor::new(onnx_path, 0.01),
        }
    }

    /// Hybrid prediction: Try Neural, fallback to Heuristic
    pub fn predict(&self, window: &[f32]) -> f32 {
        self.inner.predict(window)
    }

    /// Force use of heuristic (good for benchmarking baseline)
    pub fn predict_heuristic(&self, window: &[f32]) -> f32 {
        self.inner.predict_heuristic(window)
    }

    /// Force use of neural (good for benchmarking overhead)
    /// Returns None if neural model not loaded or window size mismatch
    pub fn predict_neural(&self, window: &[f32]) -> Option<f32> {
        self.inner.predict_neural(window)
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
        let load = predicted_load.clamp(0.0, 1.2); // Allow slight overprovision if > 1.0

        // Calculate raw target
        let raw_target = 2.0 + (load * 14.0);

        let target = raw_target.round() as usize;
        let clamped = target.clamp(2, 16);

        self.current_capacity = clamped;
        clamped
    }
}
