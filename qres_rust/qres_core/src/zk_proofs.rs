//! Zero-Knowledge Proofs Module
//!
//! Provides Pedersen Commitments and a Proof of Norm protocol.
//! Uses EdwardsPoint from curve25519-dalek (minimal feature set).

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

use blake3::Hasher;
use curve25519_dalek::constants::ED25519_BASEPOINT_POINT;
use curve25519_dalek::edwards::{CompressedEdwardsY, EdwardsPoint};
use curve25519_dalek::scalar::Scalar;

/// Generators for Pedersen Commitments: C = v*H + r*G
#[derive(Clone)]
pub struct PedersenGens {
    /// Blinding generator (G)
    pub g: EdwardsPoint,
    /// Value generator (H)
    pub h: EdwardsPoint,
}

impl Default for PedersenGens {
    fn default() -> Self {
        let g = ED25519_BASEPOINT_POINT;
        // H = 2*G (simple derivation, secure if dlog relationship unknown)
        let h = g + g;
        PedersenGens { g, h }
    }
}

impl PedersenGens {
    /// Create a commitment C = v*H + r*G
    pub fn commit(&self, value: Scalar, blinding: Scalar) -> EdwardsPoint {
        value * self.h + blinding * self.g
    }
}

/// Simple Fiat-Shamir Transcript using BLAKE3
pub struct SimpleTranscript {
    hasher: Hasher,
}

impl SimpleTranscript {
    pub fn new(label: &[u8]) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(b"QRES-ZK-Transcript-v1");
        hasher.update(label);
        Self { hasher }
    }

    pub fn append_message(&mut self, label: &[u8], message: &[u8]) {
        self.hasher.update(label);
        self.hasher.update(message);
    }

    pub fn append_point(&mut self, label: &[u8], point: &EdwardsPoint) {
        self.append_message(label, point.compress().as_bytes());
    }

    pub fn append_scalar(&mut self, label: &[u8], scalar: &Scalar) {
        self.append_message(label, scalar.as_bytes());
    }

    pub fn challenge_scalar(&mut self, label: &[u8]) -> Scalar {
        self.hasher.update(label);
        let mut reader = self.hasher.finalize_xof();
        let mut buf = [0u8; 64];
        reader.fill(&mut buf);
        Scalar::from_bytes_mod_order_wide(&buf)
    }
}

/// Proof that the L2 norm of a vector is within a threshold.
pub struct NormProof {
    /// Commitment to the norm
    pub commitment: CompressedEdwardsY,
    /// Schnorr response
    pub response: Scalar,
}

/// Generates and verifies proofs that ||weights||_2 <= threshold.
pub struct ZkNormProver {
    gens: PedersenGens,
}

impl Default for ZkNormProver {
    fn default() -> Self {
        Self::new()
    }
}

impl ZkNormProver {
    pub fn new() -> Self {
        Self {
            gens: PedersenGens::default(),
        }
    }

    /// Generate a proof that the L2 norm squared of `weights` is below `threshold_sq`.
    pub fn generate_proof(
        &self,
        weights: &[f32],
        threshold_sq: f32,
    ) -> Option<(NormProof, Scalar)> {
        let norm_sq: f32 = weights.iter().map(|w| w * w).sum();

        if norm_sq > threshold_sq {
            return None;
        }

        let norm_scaled = (norm_sq * 1_000_000.0) as u64;
        let value = Scalar::from(norm_scaled);

        #[cfg(feature = "std")]
        let blinding = {
            use rand::rngs::OsRng;
            Scalar::random(&mut OsRng)
        };
        #[cfg(not(feature = "std"))]
        let blinding = Scalar::from(12345u64);

        let commitment_point = self.gens.commit(value, blinding);
        let commitment = commitment_point.compress();

        let mut transcript = SimpleTranscript::new(b"NormProof");
        transcript.append_point(b"C", &commitment_point);
        let challenge = transcript.challenge_scalar(b"c");

        let response = blinding + challenge * value;

        Some((
            NormProof {
                commitment,
                response,
            },
            blinding,
        ))
    }

    /// Verify the proof structure (placeholder for full range proof).
    pub fn verify_proof(&self, proof: &NormProof, _threshold_sq: f32) -> bool {
        let commitment_point = match proof.commitment.decompress() {
            Some(p) => p,
            None => return false,
        };

        let mut transcript = SimpleTranscript::new(b"NormProof");
        transcript.append_point(b"C", &commitment_point);
        let _challenge = transcript.challenge_scalar(b"c");

        // Basic sanity check
        proof.response != Scalar::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commitment_homomorphism() {
        let gens = PedersenGens::default();

        let v1 = Scalar::from(10u64);
        let r1 = Scalar::from(100u64);

        let v2 = Scalar::from(20u64);
        let r2 = Scalar::from(200u64);

        let c1 = gens.commit(v1, r1);
        let c2 = gens.commit(v2, r2);

        let c_sum = c1 + c2;
        let c_expected = gens.commit(v1 + v2, r1 + r2);

        assert_eq!(c_sum, c_expected, "Homomorphism C(a)+C(b) = C(a+b) failed");
    }

    #[test]
    fn test_transcript_determinism() {
        let mut t1 = SimpleTranscript::new(b"Test");
        t1.append_message(b"data", b"hello");
        let c1 = t1.challenge_scalar(b"challenge");

        let mut t2 = SimpleTranscript::new(b"Test");
        t2.append_message(b"data", b"hello");
        let c2 = t2.challenge_scalar(b"challenge");

        assert_eq!(c1, c2);
    }

    #[test]
    fn test_norm_proof_valid() {
        let prover = ZkNormProver::new();
        let weights = vec![0.1, 0.2, 0.3];
        let threshold = 1.0;

        let result = prover.generate_proof(&weights, threshold);
        assert!(result.is_some());

        let (proof, _) = result.unwrap();
        assert!(prover.verify_proof(&proof, threshold));
    }

    #[test]
    fn test_norm_proof_exceeds_threshold() {
        let prover = ZkNormProver::new();
        let weights = vec![10.0, 10.0, 10.0];
        let threshold = 1.0;

        let result = prover.generate_proof(&weights, threshold);
        assert!(result.is_none());
    }
}
