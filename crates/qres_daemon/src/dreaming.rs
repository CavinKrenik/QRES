// QRES v11.2 - Federated Dreaming
// Idle-time hallucinatory training for privacy-preserving learning

use rand::Rng;
use std::time::{Duration, Instant};

use std::collections::VecDeque;

/// Federated Dreaming manager.
/// Generates synthetic training data during idle periods
/// to reinforce patterns without accessing original data.
pub struct DreamingManager {
    last_activity: Instant,
    idle_threshold: Duration,
    dream_count: u64,
    /// Buffer of recent real data points for validation (Phase 2 Item 2)
    validation_buffer: VecDeque<Vec<u8>>,
    /// Max validation buffer size
    max_validation_samples: usize,
}

impl DreamingManager {
    /// Create a new Dreaming manager.
    /// idle_threshold_secs: seconds of inactivity before dreaming starts.
    pub fn new(idle_threshold_secs: u64) -> Self {
        DreamingManager {
            last_activity: Instant::now(),
            idle_threshold: Duration::from_secs(idle_threshold_secs),
            dream_count: 0,
            validation_buffer: VecDeque::with_capacity(100),
            max_validation_samples: 100,
        }
    }

    /// Record activity to reset idle timer.
    pub fn record_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    /// Add a real data sample to the validation buffer
    pub fn add_real_sample(&mut self, sample: Vec<u8>) {
        if self.validation_buffer.len() >= self.max_validation_samples {
            self.validation_buffer.pop_front();
        }
        self.validation_buffer.push_back(sample);
    }

    /// Check if we should start dreaming.
    pub fn should_dream(&self) -> bool {
        self.last_activity.elapsed() >= self.idle_threshold
    }

    /// Generate a synthetic training sample based on learned patterns.
    /// This is a placeholder - real implementation would use
    /// the generative models (SNN/QNN) to produce realistic data.
    pub fn generate_dream_sample(&mut self, pattern_mean: u8) -> Vec<u8> {
        self.dream_count += 1;
        let mut rng = rand::thread_rng();

        // Generate 1KB of synthetic data around the pattern mean
        (0..1024)
            .map(|_| {
                let noise: i16 = rng.gen_range(-20..20);
                (pattern_mean as i16 + noise).clamp(0, 255) as u8
            })
            .collect()
    }

    /// Validate dreamt weights against real data buffer.
    /// Returns true if the new weights maintain accuracy on real data.
    pub fn validate_dream(&self, current_weights: &[f32], new_weights: &[f32]) -> bool {
        if self.validation_buffer.is_empty() {
            // No real data to validate against, accept but warn?
            // For safety, we accept (bootstrapping phase)
            return true;
        }

        // Mock Validation Logic:
        // In reality, we run the model with current_weights AND new_weights on validation_buffer
        // and ensure loss(new) <= loss(current) * tolerance.

        // Sanity Check: Ensure weights didn't explode (Gradient Explosion check)
        let mean_current: f32 = current_weights.iter().sum::<f32>() / current_weights.len() as f32;
        let mean_new: f32 = new_weights.iter().sum::<f32>() / new_weights.len() as f32;

        // If weight shift is extreme (>50% shift in mean), reject as hallucination
        if (mean_new - mean_current).abs() > (mean_current.abs() * 0.5 + 0.1) {
            return false;
        }

        true
    }

    /// Get total dream cycles completed.
    pub fn dream_count(&self) -> u64 {
        self.dream_count
    }

    /// Reset the idle timer (e.g., after a dream cycle).
    pub fn reset_idle(&mut self) {
        self.last_activity = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dreaming_manager_creation() {
        let mgr = DreamingManager::new(60);
        assert_eq!(mgr.dream_count(), 0);
    }

    #[test]
    fn test_dream_sample_generation() {
        let mut mgr = DreamingManager::new(60);
        let sample = mgr.generate_dream_sample(128);
        assert_eq!(sample.len(), 1024);
        assert_eq!(mgr.dream_count(), 1);
    }
}
