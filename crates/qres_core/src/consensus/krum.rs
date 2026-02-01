//! Byzantine Fault Tolerant Krum Aggregation with Fixed-Point Arithmetic
//!
//! Implements the Krum algorithm (Blanchard et al., 2017) using I16F16 fixed-point
//! arithmetic for deterministic, bit-perfect consensus across heterogeneous architectures.
//!
//! Reference: "Machine Learning with Adversaries: Byzantine Tolerant Gradient Descent"

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

use fixed::types::I16F16;
use serde::{Deserialize, Serialize};

/// Block Floating Point Vector (BFP-16)
/// Solves the "vanishing update" problem for low learning rates by using
/// a shared exponent for the entire vector.
/// Value = mantissa * 2^exponent
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bfp16Vec {
    /// Shared exponent for the block
    pub exponent: i8,
    /// 16-bit signed mantissas
    pub mantissas: Vec<i16>,
}

impl Bfp16Vec {
    /// Create Bfp16Vec from f32 slice
    pub fn from_f32_slice(data: &[f32]) -> Self {
        if data.is_empty() {
            return Self {
                exponent: 0,
                mantissas: Vec::new(),
            };
        }

        // 1. Find max absolute value
        let max_abs = data.iter().map(|x| x.abs()).fold(0.0f32, |a, b| a.max(b));

        if max_abs == 0.0 {
            return Self {
                exponent: 0,
                mantissas: vec![0; data.len()],
            };
        }

        // 2. Calculate optimal exponent
        // We want max_abs * 2^(-exp) <= 32767
        // exp >= log2(max_abs) - log2(32767)
        // exp = ceil(log2(max_abs) - 14.99993)
        // Using -15.0 ensures we maximize dynamic range usage (matches Python)
        let exp_f32 = max_abs.log2().ceil() - 15.0;

        // Clamp exponent to valid range if needed
        let exponent = (exp_f32 as i8).clamp(-126, 126);

        // 3. Quantize
        // scale = 2^(-exponent)
        let scale = 2.0f32.powi(-(exponent as i32));
        let mantissas = data
            .iter()
            .map(|&x| {
                let scaled = x * scale;
                // Clamp to i16 range (saturating)
                scaled.clamp(-32767.0, 32767.0).round() as i16
            })
            .collect();

        Self {
            exponent,
            mantissas,
        }
    }

    /// Convert back to Vec<f32>
    pub fn to_vec_f32(&self) -> Vec<f32> {
        let scale = 2.0f32.powi(self.exponent as i32);
        self.mantissas.iter().map(|&m| m as f32 * scale).collect()
    }
}

/// Calculates the squared Euclidean distance between two fixed-point vectors.
///
/// Uses `(a - b) * (a - b)` since I16F16 doesn't support `.powi()`.
/// Returns `I16F16::MAX` on length mismatch or saturation.
pub fn dist_sq(a: &[I16F16], b: &[I16F16]) -> I16F16 {
    if a.len() != b.len() {
        return I16F16::MAX;
    }

    let mut sum = I16F16::ZERO;
    for (x, y) in a.iter().zip(b.iter()) {
        let diff = x.saturating_sub(*y);
        let sq = diff.saturating_mul(diff);
        sum = sum.saturating_add(sq);
    }
    sum
}

/// Selects the "best" vector using the Krum rule.
///
/// Selects the vector that minimizes the sum of squared distances to its
/// (n - f - 2) nearest neighbors.
///
/// # Arguments
/// * `vectors` - Slice of model update vectors (each is a `Vec<I16F16>`)
/// * `f` - Maximum number of Byzantine (malicious) nodes expected
///
/// # Returns
/// * `Some(Vec<I16F16>)` - The selected representative vector (cloned)
/// * `None` - If aggregation is mathematically impossible (n < 3)
///
/// # Krum Condition
/// The algorithm requires `n >= 2f + 3`. If this condition is not met,
/// the algorithm proceeds with best-effort selection using clamped neighbor count.
pub fn aggregate_krum(vectors: &[Vec<I16F16>], f: usize) -> Option<Vec<I16F16>> {
    let n = vectors.len();

    // Krum requires at least n=3 (with f=0, needs n-f-2=1 neighbor)
    if n < 3 {
        return None;
    }

    // Theoretical requirement is n >= 2f + 3.
    // If not met, we proceed best-effort but warn (if std available)
    #[cfg(feature = "std")]
    if n < 2 * f + 3 {
        eprintln!(
            "WARN: Krum condition not met (n={} < 2*f+3={}). Proceeding with best-effort.",
            n,
            2 * f + 3
        );
    }

    // Number of neighbors to consider for the score: k = n - f - 2
    // If n is too small relative to f, ensure we define a valid range.
    let k = if n > f + 2 { n - f - 2 } else { 1 };

    let mut min_score = I16F16::MAX;
    let mut best_idx = 0;

    // Score calculation for each candidate
    for i in 0..n {
        let mut distances: Vec<I16F16> = Vec::with_capacity(n - 1);

        for j in 0..n {
            if i == j {
                continue;
            }
            distances.push(dist_sq(&vectors[i], &vectors[j]));
        }

        // Sort to find the k nearest neighbors
        distances.sort_unstable();

        // Sum the smallest k distances
        let mut current_score = I16F16::ZERO;
        for d in distances.iter().take(k) {
            current_score = current_score.saturating_add(*d);
        }

        // Selection: keep track of minimum score
        if current_score < min_score {
            min_score = current_score;
            best_idx = i;
        }
    }

    // Return a clone of the winning vector
    Some(vectors[best_idx].clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use fixed::types::I16F16;

    #[test]
    fn test_dist_sq_identical() {
        let a = vec![I16F16::from_num(1.0), I16F16::from_num(2.0)];
        let b = vec![I16F16::from_num(1.0), I16F16::from_num(2.0)];
        assert_eq!(dist_sq(&a, &b), I16F16::ZERO);
    }

    #[test]
    fn test_dist_sq_simple() {
        let a = vec![I16F16::from_num(0.0), I16F16::from_num(0.0)];
        let b = vec![I16F16::from_num(3.0), I16F16::from_num(4.0)];
        // Expected: 3^2 + 4^2 = 9 + 16 = 25
        assert_eq!(dist_sq(&a, &b), I16F16::from_num(25));
    }

    #[test]
    fn test_dist_sq_length_mismatch() {
        let a = vec![I16F16::from_num(1.0)];
        let b = vec![I16F16::from_num(1.0), I16F16::from_num(2.0)];
        assert_eq!(dist_sq(&a, &b), I16F16::MAX);
    }

    #[test]
    fn test_krum_returns_none_for_small_n() {
        let vectors: Vec<Vec<I16F16>> = vec![];
        assert!(aggregate_krum(&vectors, 0).is_none());

        let vectors = vec![vec![I16F16::from_num(1.0)]];
        assert!(aggregate_krum(&vectors, 0).is_none());

        let vectors = vec![vec![I16F16::from_num(1.0)], vec![I16F16::from_num(2.0)]];
        assert!(aggregate_krum(&vectors, 0).is_none());
    }

    #[test]
    fn test_krum_selects_honest() {
        // Honest value: ~1.0. Malicious: 100.0.
        let v1 = vec![I16F16::from_num(1.0), I16F16::from_num(1.1)];
        let v2 = vec![I16F16::from_num(0.9), I16F16::from_num(1.0)];
        let v3 = vec![I16F16::from_num(1.05), I16F16::from_num(0.95)];
        let v4 = vec![I16F16::from_num(1.0), I16F16::from_num(1.0)]; // Perfect
        let malicious = vec![I16F16::from_num(100.0), I16F16::from_num(100.0)];

        let vectors = vec![v1, v2, v3, v4, malicious];
        // n=5, f=1. k = 5 - 1 - 2 = 2 nearest neighbors.

        let result = aggregate_krum(&vectors, 1).expect("Should return result");

        // The malicious vector is far from everyone.
        // Honest vectors are close to each other.
        // Krum should pick one of the honest ones, NOT malicious.
        assert_ne!(result[0], I16F16::from_num(100.0));
        assert!(result[0] < I16F16::from_num(2.0));
    }

    #[test]
    fn test_krum_determinism() {
        // Run Krum multiple times, should always get same result
        let vectors = vec![
            vec![I16F16::from_num(1.0), I16F16::from_num(2.0)],
            vec![I16F16::from_num(1.1), I16F16::from_num(2.1)],
            vec![I16F16::from_num(0.9), I16F16::from_num(1.9)],
            vec![I16F16::from_num(1.05), I16F16::from_num(2.05)],
            vec![I16F16::from_num(50.0), I16F16::from_num(50.0)],
        ];

        let result1 = aggregate_krum(&vectors, 1);
        let result2 = aggregate_krum(&vectors, 1);
        let result3 = aggregate_krum(&vectors, 1);

        assert_eq!(result1, result2);
        assert_eq!(result2, result3);
    }

    #[test]
    fn test_krum_f_zero_edge_case() {
        // Test with f=0: reduces to "closest to all neighbors" logic
        // k = n - 0 - 2 = n - 2
        let vectors = vec![
            vec![I16F16::from_num(1.0), I16F16::from_num(1.0)],
            vec![I16F16::from_num(1.1), I16F16::from_num(1.1)],
            vec![I16F16::from_num(0.9), I16F16::from_num(0.9)],
            vec![I16F16::from_num(100.0), I16F16::from_num(100.0)], // Outlier
        ];

        // n=4, f=0 -> k = 4 - 0 - 2 = 2 neighbors
        let result = aggregate_krum(&vectors, 0).expect("Should return result");

        // Should still reject the outlier even with f=0
        let val: f32 = result[0].to_num();
        assert!(
            val < 10.0,
            "f=0 Krum should still avoid obvious outlier: {}",
            val
        );
    }
}
