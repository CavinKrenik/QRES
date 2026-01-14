//! Secure Aggregation Module
//!
//! Implements a pairwise masking protocol for privacy-preserving aggregation of model updates.
//! Based on the principle that sum(Masked_i) = sum(Weights_i) if masks sum to zero globally.

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap;
#[cfg(feature = "std")]
use std::collections::BTreeMap;

use rand_chacha::rand_core::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use x25519_dalek::{PublicKey, StaticSecret};

/// Handles secure aggregation via pairwise masking
pub struct SecureAggregator {
    /// Local secret key (keep private!)
    my_secret: StaticSecret,
    /// My public key (shared with peers)
    my_public_key: PublicKey,
    /// Map of Peer Public Key Bytes -> Peer Public Key Object
    /// We use BTreeMap for deterministic iteration order (important for consistency, though not strictly required for correctness if logic is robust)
    peers: BTreeMap<[u8; 32], PublicKey>,
}

impl Default for SecureAggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl SecureAggregator {
    /// Create a new SecureAggregator with a random secret
    ///
    /// In a real P2P system, the secret should ideally persist for the round
    /// or be derived from a session key.
    pub fn new() -> Self {
        // We need a CSPRNG. In no_std, the caller typically provides one,
        // but StaticSecret::random_from_rng requires rand_core::CryptoRng.
        // For simplicity in this library, we'll assume we can use OsRng in std
        // or a passed-in seed.
        //
        // CRITICAL TODO: effectively handling RNG in strict no_std environments
        // without OS entropy is tricky. Here we use a standard approach if available.

        #[cfg(feature = "std")]
        let secret = StaticSecret::random_from_rng(rand::rngs::OsRng);

        #[cfg(not(feature = "std"))]
        // In no_std tests/wasm, usage might differ.
        // For now, we allow creating from a seed to support deterministic testing/embedded usage.
        let secret = StaticSecret::from([0u8; 32]); // Placeholder for strictly manual init, usually requires seed

        let public = PublicKey::from(&secret);

        Self {
            my_secret: secret,
            my_public_key: public,
            peers: BTreeMap::new(),
        }
    }

    /// Create from a specific seed (useful for deterministic tests or no_std)
    pub fn from_seed(seed: [u8; 32]) -> Self {
        let secret = StaticSecret::from(seed);
        let public = PublicKey::from(&secret);
        Self {
            my_secret: secret,
            my_public_key: public,
            peers: BTreeMap::new(),
        }
    }

    /// Get my public key bytes
    pub fn get_public_key(&self) -> [u8; 32] {
        *self.my_public_key.as_bytes()
    }

    /// Add a peer to the aggregation group
    pub fn add_peer(&mut self, public_key_bytes: [u8; 32]) {
        // Don't add ourselves
        if public_key_bytes == self.get_public_key() {
            return;
        }
        let pk = PublicKey::from(public_key_bytes);
        self.peers.insert(public_key_bytes, pk);
    }

    /// Generate a masked update vector for Fixed Point weights
    ///
    /// The mask is the sum of pairwise masks:
    /// Mask_i = Sum_{j > i} (PRNG(S_ij)) - Sum_{j < i} (PRNG(S_ij))
    pub fn mask_update_fixed(&self, update: &[fixed::types::I16F16]) -> Vec<fixed::types::I16F16> {
        let mut masked_update = update.to_vec();
        let my_pk_bytes = self.get_public_key();

        for (peer_pk_bytes, peer_pk) in &self.peers {
            // Compute shared secret using ECDH
            let shared_secret = self.my_secret.diffie_hellman(peer_pk);

            // Expand shared secret into a mask vector using ChaCha20
            // We use the shared secret bytes as the seed
            let mut rng = ChaCha20Rng::from_seed(*shared_secret.as_bytes());

            // Apply mask
            // If MyID < PeerID: Add Mask
            // If MyID > PeerID: Subtract Mask
            // Lexicographical comparison of public keys serves as consistent ordering
            let add_mask = my_pk_bytes < *peer_pk_bytes;

            for val in masked_update.iter_mut() {
                // Generate a random u32
                let rnd_u32 = rng.next_u32();
                // We treat this u32 as an I16F16 raw representation?
                // No, that would overflow. We need to generate a valid I16F16 mask within range.
                // For simplicity, we generate a float-like perturbation in [-1, 1].

                // Construct mask from bits to ensure exact reproducibility across platforms
                // We mask lower 16 bits to stay within reason, or just wrap?
                // Secure Aggregation usually works on Modular Arithmetic (Fields).
                // Doing it on Q16.16 with wrapping_add is actually mathematically sound
                // IF we allow overflows (Modular Arithmetic on 2^32).
                // Let's use wrapping arithmetic on the raw bits (i32).

                let mask_bits = rnd_u32 as i32;
                let mask_val = fixed::types::I16F16::from_bits(mask_bits);

                // Use wrapping add/sub to effectively use modulo 2^32 arithmetic
                // This prevents clipping issues and guarantees perfect cancellation.
                if add_mask {
                    *val = val.wrapping_add(mask_val);
                } else {
                    *val = val.wrapping_sub(mask_val);
                }
            }
        }

        masked_update
    }

    /// Generate a masked update vector (Legacy Float)
    pub fn mask_update(&self, update: &[f32]) -> Vec<f32> {
        let mut masked_update = update.to_vec();
        let my_pk_bytes = self.get_public_key();

        for (peer_pk_bytes, peer_pk) in &self.peers {
            let shared_secret = self.my_secret.diffie_hellman(peer_pk);
            let mut rng = ChaCha20Rng::from_seed(*shared_secret.as_bytes());
            let add_mask = my_pk_bytes < *peer_pk_bytes;

            for val in masked_update.iter_mut() {
                let rnd_u32 = rng.next_u32();
                let rnd_f32 = (rnd_u32 as f32) / (u32::MAX as f32);
                let mask_val = rnd_f32;

                if add_mask {
                    *val += mask_val;
                } else {
                    *val -= mask_val;
                }
            }
        }
        masked_update
    }

    /// Aggregate a set of masked updates
    ///
    /// If all peers from the clique contributed, the masks cancel out,
    /// leaving Sum(Original_Updates).
    pub fn aggregate(&self, masked_updates: &[Vec<f32>]) -> Option<Vec<f32>> {
        if masked_updates.is_empty() {
            return None;
        }

        let len = masked_updates[0].len();
        let mut sum = vec![0.0f32; len];

        for update in masked_updates {
            if update.len() != len {
                return None; // Dimension mismatch
            }
            for (i, val) in update.iter().enumerate() {
                sum[i] += val;
            }
        }

        Some(sum)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_masking_cancellation_3_peers() {
        // Setup 3 peers
        let seed1 = [1u8; 32];
        let seed2 = [2u8; 32];
        let seed3 = [3u8; 32];

        let mut p1 = SecureAggregator::from_seed(seed1);
        let mut p2 = SecureAggregator::from_seed(seed2);
        let mut p3 = SecureAggregator::from_seed(seed3);

        // Exchange keys
        let pk1 = p1.get_public_key();
        let pk2 = p2.get_public_key();
        let pk3 = p3.get_public_key();

        p1.add_peer(pk2);
        p1.add_peer(pk3);
        p2.add_peer(pk1);
        p2.add_peer(pk3);
        p3.add_peer(pk1);
        p3.add_peer(pk2);

        // Create dummy updates
        let u1 = vec![1.0, 2.0, 3.0];
        let u2 = vec![4.0, 5.0, 6.0];
        let u3 = vec![7.0, 8.0, 9.0];

        // Mask updates
        let m1 = p1.mask_update(&u1);
        let m2 = p2.mask_update(&u2);
        let m3 = p3.mask_update(&u3);

        // Verify individual masked updates look "random" (simple check)
        assert_ne!(m1, u1);
        assert_ne!(m2, u2);
        assert_ne!(m3, u3);

        // Aggregate
        let agg_masked = p1.aggregate(&[m1, m2, m3]).unwrap();

        // Calculate expected sum of original updates
        let expected = [12.0, 15.0, 18.0];

        // Assert equality with small epsilon for float precision
        for (a, e) in agg_masked.iter().zip(expected.iter()) {
            assert!((a - e).abs() < 1e-4, "Got {}, expected {}", a, e);
        }
    }

    #[test]
    fn test_partial_failure() {
        // If one peer drops out, the masks shouldn't cancel, and the result should be garbage/unusable.
        // This is a property of the protocol: it's fragile to dropouts without recovery phases (Shamir Secret Sharing).
        // For Phase 3 Item 2, we implement basic masking. The fact that it fails on dropout is expected behavior for this simple version.

        let seed1 = [1u8; 32];
        let seed2 = [2u8; 32];
        let mut p1 = SecureAggregator::from_seed(seed1);
        let mut p2 = SecureAggregator::from_seed(seed2);

        p1.add_peer(p2.get_public_key());
        p2.add_peer(p1.get_public_key());

        let u1 = [10.0f32];
        let _u2 = [20.0f32];

        let m1 = p1.mask_update(&u1);
        // p2 fails to send m2

        // Attempts to aggregate just m1? Result is just m1 (random garbage relative to u1)
        // m1 = u1 + mask_12
        // Since mask_12 is random, m1 should not be close to u1 (unless we got incredibly unlucky with a ~0 mask)
        assert!((m1[0] - u1[0]).abs() > 0.1);
    }
}
