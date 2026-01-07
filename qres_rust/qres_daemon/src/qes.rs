// QRES v11.2 - Quantum-Entangled Swarms (QES)
// PRNG-seeded weight synchronization for zero-bandwidth model updates

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

/// QES synchronization manager.
/// Uses a shared PRNG seed to generate identical weight updates
/// across swarm nodes without explicit communication.
pub struct QesSyncManager {
    rng: ChaCha20Rng,
    epoch: u64,
}

impl QesSyncManager {
    /// Create a new QES sync manager with a shared seed.
    pub fn new(seed: u64) -> Self {
        let rng = ChaCha20Rng::seed_from_u64(seed);
        QesSyncManager { rng, epoch: 0 }
    }

    /// Generate synchronized weight deltas for a given epoch.
    /// All nodes with the same seed will produce identical deltas.
    pub fn generate_weight_deltas(&mut self, num_weights: usize) -> Vec<f32> {
        self.epoch += 1;
        (0..num_weights)
            .map(|_| self.rng.gen_range(-0.01..0.01))
            .collect()
    }

    /// Apply synchronized deltas to mixer weights.
    pub fn apply_to_weights(&mut self, weights: &mut [f32]) {
        let deltas = self.generate_weight_deltas(weights.len());
        for (w, d) in weights.iter_mut().zip(deltas.iter()) {
            *w = (*w + d).clamp(0.0, 1.0);
        }
        // Normalize
        let sum: f32 = weights.iter().sum();
        if sum > 0.001 {
            for w in weights.iter_mut() {
                *w /= sum;
            }
        }
    }

    /// Get current synchronization epoch.
    pub fn current_epoch(&self) -> u64 {
        self.epoch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qes_determinism() {
        let mut mgr1 = QesSyncManager::new(12345);
        let mut mgr2 = QesSyncManager::new(12345);

        let deltas1 = mgr1.generate_weight_deltas(6);
        let deltas2 = mgr2.generate_weight_deltas(6);

        assert_eq!(deltas1, deltas2, "QES deltas should be identical");
    }
}
