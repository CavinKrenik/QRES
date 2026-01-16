use crate::zk_proofs::NormProof;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

pub mod fragmentation;

/// The "Ghost" Update Packet
///
/// This structure brings together the three layers of the Phase 3 Security architecture:
/// 1. **Differential Privacy**: `masked_weights` contain noise. `dp_epsilon` tracks the budget.
/// 2. **Secure Aggregation**: `masked_weights` are masked with pairwise keys.
/// 3. **Zero-Knowledge Proofs**: `zk_proof` ensures the potentially garbage-looking masked data
///    came from a valid, bounded update.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GhostUpdate {
    /// Sender Identity (Ed25519 Public Key Bytes)
    pub peer_id: [u8; 32],
    /// The payload: Q16.16 weights + Gaussian Noise + Pairwise Masks
    /// Store as i32 bits to accommodate the wrapping arithmetic of Secure Aggregation.
    pub masked_weights: Vec<i32>,
    /// Zero-Knowledge Proof that ||original_weights|| < Threshold
    pub zk_proof: NormProof,
    /// Privacy budget consumed by this update
    pub dp_epsilon: f32,
}
