//! Differential Privacy Module
//!
//! Implements mechanisms to add noise to model updates, providing (epsilon, delta)-differential privacy.
//! Supports `opendp` for rigorous accounting (optional feature) and a manual fallback.

// Vec is only needed when dp feature is enabled (for OpenDP implementation)
#[cfg(all(not(feature = "std"), feature = "dp"))]
use alloc::vec::Vec;
#[cfg(all(feature = "std", feature = "dp"))]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(feature = "std")]
use std::string::String;

#[cfg(feature = "dp")]
use opendp::domains::{AtomDomain, VectorDomain};
#[cfg(feature = "dp")]
use opendp::measurements::make_gaussian;
#[cfg(feature = "dp")]
use opendp::metrics::L2Distance;

use fixed::types::I16F16;
#[cfg(all(not(feature = "dp"), feature = "std"))]
use rand::{thread_rng, Rng};

// Math imports for no_std manual implementation
#[cfg(not(feature = "std"))]
use libm::{cos, log as ln, sin, sqrt};
#[cfg(feature = "std")]
fn sqrt(x: f64) -> f64 {
    x.sqrt()
}
#[cfg(feature = "std")]
fn ln(x: f64) -> f64 {
    x.ln()
}
#[cfg(feature = "std")]
fn cos(x: f64) -> f64 {
    x.cos()
}
#[cfg(feature = "std")]
fn sin(x: f64) -> f64 {
    x.sin()
}

/// Differential Privacy configuration and mechanism
#[derive(Clone, Debug)]
pub struct DifferentialPrivacy {
    /// Privacy budget (lower is more private)
    epsilon: f64,
    /// Probability of privacy violation
    delta: f64,
    /// Maximum L2 norm for update vectors
    clipping_threshold: f64,
}

impl DifferentialPrivacy {
    /// Create a new Differential Privacy handler
    pub fn new(epsilon: f64, delta: f64, clipping_threshold: f64) -> Self {
        Self {
            epsilon,
            delta,
            clipping_threshold,
        }
    }

    /// Clip the L2 norm of the update vector to the threshold
    /// Returns true if clipping was applied
    pub fn clip_update(&self, update: &mut [f32]) -> bool {
        // Calculate L2 norm
        let mut sum_sq = 0.0;
        for &x in update.iter() {
            sum_sq += x * x;
        }
        let norm = (sum_sq as f64).sqrt();

        if norm > self.clipping_threshold {
            let scale = (self.clipping_threshold / norm) as f32;
            for x in update.iter_mut() {
                *x *= scale;
            }
            true
        } else {
            false
        }
    }

    /// Add Gaussian noise to the update vector
    ///
    /// Uses OpenDP if the "dp" feature is enabled, otherwise falls back to manual Box-Muller.
    pub fn add_noise(&self, update: &mut [f32]) -> Result<(), String> {
        #[cfg(feature = "dp")]
        {
            // OpenDP Implementation
            // Convert f32 vec to f64 for OpenDP
            let mut data_f64: Vec<f64> = update.iter().map(|&x| x as f64).collect();

            let domain = VectorDomain::new(AtomDomain::<f64>::default());
            let metric = L2Distance::default();

            // Sensitivity is the clipping threshold (L2 sensitivity)
            let sensitivity = self.clipping_threshold;

            // Create measurement
            let meas = make_gaussian(domain, metric, sensitivity, self.epsilon, Some(self.delta))
                .map_err(|e| e.to_string())?;

            // Invoke measurement
            let noisy = meas.invoke(&data_f64).map_err(|e| e.to_string())?;

            // Copy back to update vector
            if noisy.len() != update.len() {
                return Err("Noise generation changed vector length".into());
            }

            for (i, &val) in noisy.iter().enumerate() {
                update[i] = val as f32;
            }

            Ok(())
        }

        #[cfg(all(not(feature = "dp"), feature = "std"))]
        {
            // Manual Fallback Implementation (Box-Muller Transform) - std version
            let c = 2.0 * ln(1.25 / self.delta);
            let sigma = (self.clipping_threshold * sqrt(c)) / self.epsilon;

            let mut rng = thread_rng();

            let len = update.len();
            let mut i = 0;

            while i < len {
                let u1: f64 = rng.gen();
                let u2: f64 = rng.gen();

                let u1 = if u1 < 1e-10 { 1e-10 } else { u1 };

                let r = sqrt(-2.0 * ln(u1));
                let theta = 2.0 * core::f64::consts::PI * u2;

                let z0 = r * cos(theta);
                let z1 = r * sin(theta);

                update[i] += (z0 * sigma) as f32;

                if i + 1 < len {
                    update[i + 1] += (z1 * sigma) as f32;
                }

                i += 2;
            }

            Ok(())
        }

        #[cfg(all(not(feature = "dp"), not(feature = "std")))]
        {
            // no_std fallback: Use a simple seeded PRNG
            // WARNING: This is NOT cryptographically secure!
            // In production, the caller should provide entropy.
            use rand_chacha::rand_core::{RngCore, SeedableRng};
            use rand_chacha::ChaCha20Rng;

            let c = 2.0 * ln(1.25 / self.delta);
            let sigma = (self.clipping_threshold * sqrt(c)) / self.epsilon;

            // Use a deterministic seed for reproducibility in no_std
            // In real use, seed should come from external entropy source
            let mut rng = ChaCha20Rng::from_seed([42u8; 32]);

            let len = update.len();
            let mut i = 0;

            while i < len {
                // Generate uniform [0, 1) from u64
                let u1: f64 = (rng.next_u64() as f64) / (u64::MAX as f64);
                let u2: f64 = (rng.next_u64() as f64) / (u64::MAX as f64);

                let u1 = if u1 < 1e-10 { 1e-10 } else { u1 };

                let r = sqrt(-2.0 * ln(u1));
                let theta = 2.0 * core::f64::consts::PI * u2;

                let z0 = r * cos(theta);
                let z1 = r * sin(theta);

                update[i] += (z0 * sigma) as f32;

                if i + 1 < len {
                    update[i + 1] += (z1 * sigma) as f32;
                }

                i += 2;
            }

            Ok(())
        }
    }

    /// Add Gaussian noise to fixed-point weights (I16F16)
    ///
    /// Converts to f64 for noise addition, then saturates back to I16F16.
    pub fn add_noise_fixed(&self, weights: &mut [I16F16]) -> Result<(), String> {
        // 1. Convert to float context
        // In a real no_std environment without allocation, we might operate in chunks
        // or implement a fixed-point Gaussian sampler.
        // For v16.5, we use a temporary float buffer (requiring alloc).

        #[cfg(feature = "std")]
        let mut float_weights: Vec<f32> = weights.iter().map(|w| w.to_num::<f32>()).collect();
        #[cfg(not(feature = "std"))]
        let mut float_weights: alloc::vec::Vec<f32> =
            weights.iter().map(|w| w.to_num::<f32>()).collect();

        // 2. Add Noise (using existing float impl)
        self.add_noise(&mut float_weights)?;

        // 3. Convert back
        for (i, &fw) in float_weights.iter().enumerate() {
            weights[i] = I16F16::from_num(fw);
        }

        Ok(())
    }

    /// Calculate the theoretical noise scale (sigma)
    pub fn sigma(&self) -> f64 {
        let c = 2.0 * ln(1.25 / self.delta);
        (self.clipping_threshold * sqrt(c)) / self.epsilon
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "std")]
    #[test]
    fn test_sigma_calculation() {
        let dp = DifferentialPrivacy::new(1.0, 1e-5, 1.0);
        let sigma = dp.sigma();
        // sigma = 1.0 * sqrt(2 * ln(1.25/1e-5)) / 1.0
        // ln(125000) ≈ 11.736
        // sqrt(2 * 11.736) ≈ sqrt(23.472) ≈ 4.84
        assert!(sigma > 4.8 && sigma < 4.9);
    }

    #[test]
    fn test_clipping() {
        let dp = DifferentialPrivacy::new(1.0, 1e-5, 1.0);
        // Vector with norm 2.0 (sqrt(2^2))
        let mut update = vec![2.0f32];
        assert!(dp.clip_update(&mut update));
        assert!((update[0] - 1.0).abs() < 1e-6);

        // Vector with norm 0.5 (no clip)
        let mut update_small = vec![0.5f32];
        assert!(!dp.clip_update(&mut update_small));
        assert!((update_small[0] - 0.5).abs() < 1e-6);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_noise_distribution() {
        // High epsilon (low noise) to check mean
        let dp = DifferentialPrivacy::new(100.0, 1e-5, 1.0);
        let mut data = vec![0.0f32; 1000];

        dp.add_noise(&mut data).expect("Failed to add noise");

        let sum: f32 = data.iter().sum();
        let mean = sum / 1000.0;

        // Mean should be close to 0
        assert!(mean.abs() < 0.1, "Mean noise should be ~0, got {}", mean);

        // Variance check
        let variance: f32 = data.iter().map(|x| x * x).sum::<f32>() / 1000.0;
        let sigma = dp.sigma() as f32;
        let expected_var = sigma * sigma;

        assert!(
            (variance - expected_var).abs() < expected_var * 0.5,
            "Variance {} vs expected {}",
            variance,
            expected_var
        );
    }
}
