//! Peer Key Store for QRES daemon
//!
//! Manages the mapping of PeerIds to their public keys,
//! populated via libp2p Identify protocol or manual configuration.
//! Part of Phase 1 Item 2 of the security roadmap.

use libp2p::identity::PublicKey;
use libp2p::PeerId;
use std::collections::{HashMap, HashSet};
use tracing::{info, warn};

/// Store for peer public keys, used for identity verification
pub struct PeerKeyStore {
    /// Map of peer ID to their verified public key
    keys: HashMap<PeerId, PublicKey>,
    /// Set of trusted peer IDs (from config)
    trusted_peer_ids: HashSet<PeerId>,
    /// Whether to allow any peer or only trusted ones
    whitelist_mode: bool,
}

impl PeerKeyStore {
    /// Create a new PeerKeyStore from config values
    pub fn new(trusted_peers: &[String], trusted_pubkeys: &[String]) -> Self {
        let mut store = Self {
            keys: HashMap::new(),
            trusted_peer_ids: HashSet::new(),
            whitelist_mode: !trusted_peers.is_empty(),
        };

        // Parse trusted peer IDs from config
        for peer_str in trusted_peers {
            if let Ok(peer_id) = peer_str.parse::<PeerId>() {
                store.trusted_peer_ids.insert(peer_id);
                info!(peer_id = %peer_id, "Added trusted peer from config");
            } else {
                warn!(peer_str = %peer_str, "Failed to parse trusted peer ID");
            }
        }

        // Parse trusted pubkeys and derive PeerIds
        for pubkey_hex in trusted_pubkeys {
            if let Ok(pubkey_bytes) = hex::decode(pubkey_hex) {
                // Try to parse as ed25519 public key (32 bytes)
                if pubkey_bytes.len() == 32 {
                    if let Ok(ed_key) =
                        libp2p::identity::ed25519::PublicKey::try_from_bytes(&pubkey_bytes)
                    {
                        let public_key = PublicKey::from(ed_key);
                        let peer_id = PeerId::from_public_key(&public_key);
                        store.keys.insert(peer_id, public_key);
                        store.trusted_peer_ids.insert(peer_id);
                        info!(peer_id = %peer_id, "Added trusted pubkey from config");
                    } else {
                        warn!(hex = %pubkey_hex, "Invalid ed25519 public key");
                    }
                } else {
                    warn!(hex = %pubkey_hex, len = pubkey_bytes.len(), "Invalid pubkey length (expected 32)");
                }
            } else {
                warn!(hex = %pubkey_hex, "Failed to decode hex pubkey");
            }
        }

        store
    }

    /// Add a public key for a peer (called when Identify event is received)
    /// Only adds if in whitelist mode and peer is trusted, or whitelist mode is off
    pub fn add_peer_key(&mut self, peer_id: PeerId, public_key: PublicKey) -> bool {
        if self.whitelist_mode && !self.trusted_peer_ids.contains(&peer_id) {
            warn!(peer_id = %peer_id, "Rejecting key from non-whitelisted peer");
            return false;
        }

        // Verify the public key matches the peer ID
        let derived_peer_id = PeerId::from_public_key(&public_key);
        if derived_peer_id != peer_id {
            warn!(
                peer_id = %peer_id,
                derived = %derived_peer_id,
                "Public key does not match peer ID"
            );
            return false;
        }

        self.keys.insert(peer_id, public_key);
        info!(peer_id = %peer_id, "Added verified peer key");
        true
    }

    /// Get the public key for a peer
    pub fn get_key(&self, peer_id: &PeerId) -> Option<PublicKey> {
        self.keys.get(peer_id).cloned()
    }

    /// Check if a peer is trusted
    pub fn is_trusted(&self, peer_id: &PeerId) -> bool {
        if self.whitelist_mode {
            self.trusted_peer_ids.contains(peer_id)
        } else {
            // If no whitelist, accept any peer with a known key
            self.keys.contains_key(peer_id)
        }
    }

    /// Check if we have a key for a peer
    pub fn has_key(&self, peer_id: &PeerId) -> bool {
        self.keys.contains_key(peer_id)
    }

    /// Get hex-encoded public key for a peer (for display/logging)
    pub fn get_key_hex(&self, peer_id: &PeerId) -> Option<String> {
        self.keys.get(peer_id).map(|pk| {
            // Extract ed25519 bytes if possible
            match pk.clone().try_into_ed25519() {
                Ok(ed_key) => hex::encode(ed_key.to_bytes()),
                Err(_) => "non-ed25519".to_string(),
            }
        })
    }

    /// Get number of known peers
    pub fn peer_count(&self) -> usize {
        self.keys.len()
    }

    /// Get number of trusted peers
    pub fn trusted_count(&self) -> usize {
        self.trusted_peer_ids.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::identity::Keypair;

    #[test]
    fn test_peer_key_store_add_and_lookup() {
        let mut store = PeerKeyStore::new(&[], &[]);

        // Generate a test keypair
        let keypair = Keypair::generate_ed25519();
        let public_key = keypair.public();
        let peer_id = PeerId::from_public_key(&public_key);

        // Add the key
        assert!(store.add_peer_key(peer_id, public_key.clone()));

        // Verify lookup
        assert!(store.has_key(&peer_id));
        assert!(store.get_key(&peer_id).is_some());
    }

    #[test]
    fn test_whitelist_mode() {
        // Create store with a trusted peer
        let keypair = Keypair::generate_ed25519();
        let public_key = keypair.public();
        let peer_id = PeerId::from_public_key(&public_key);

        let store = PeerKeyStore::new(&[peer_id.to_string()], &[]);

        // This peer should be trusted
        assert!(store.is_trusted(&peer_id));

        // Random peer should not be trusted
        let random_keypair = Keypair::generate_ed25519();
        let random_peer_id = PeerId::from_public_key(&random_keypair.public());
        assert!(!store.is_trusted(&random_peer_id));
    }

    #[test]
    fn test_key_mismatch_rejected() {
        let mut store = PeerKeyStore::new(&[], &[]);

        // Generate two different keypairs
        let keypair1 = Keypair::generate_ed25519();
        let keypair2 = Keypair::generate_ed25519();

        let peer_id1 = PeerId::from_public_key(&keypair1.public());

        // Try to add keypair2's public key for peer_id1 (should fail)
        assert!(!store.add_peer_key(peer_id1, keypair2.public()));
    }
}
