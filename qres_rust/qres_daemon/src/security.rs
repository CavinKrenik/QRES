//! Security module for QRES daemon
//!
//! Provides ed25519 signing and verification for model updates,
//! implementing Phase 1 Item 1 of the security roadmap.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Size of an ed25519 signature in bytes
pub const SIGNATURE_SIZE: usize = 64;

/// A signed message containing payload and signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedPayload {
    /// The original message data (serialized brain/delta)
    pub data: Vec<u8>,
    /// ed25519 signature in hex format
    pub signature: String,
    /// Hex-encoded public key of the signer
    pub signer_pubkey: String,
    /// Timestamp for replay prevention (Unix epoch seconds)
    pub timestamp: u64,
    /// Nonce for replay prevention
    pub nonce: u64,
}

/// Security manager for handling keys and verification
pub struct SecurityManager {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    require_signatures: bool,
    /// Set of recently seen nonces to prevent replay
    seen_nonces: std::collections::HashSet<u64>,
    /// Maximum age of messages in seconds (for timestamp validation)
    max_message_age_secs: u64,
}

impl SecurityManager {
    /// Create a new SecurityManager, loading or generating keys
    pub fn new(
        key_path: &PathBuf,
        require_signatures: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let signing_key = if key_path.exists() {
            // Load existing key
            let key_bytes = fs::read(key_path)?;
            if key_bytes.len() != 32 {
                return Err("Invalid key file size".into());
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&key_bytes);
            SigningKey::from_bytes(&arr)
        } else {
            // Generate new key
            let mut csprng = OsRng;
            let key = SigningKey::generate(&mut csprng);
            // Save the secret key
            if let Some(parent) = key_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(key_path, key.to_bytes())?;
            key
        };

        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            signing_key,
            verifying_key,
            require_signatures,
            seen_nonces: std::collections::HashSet::new(),
            max_message_age_secs: 300, // 5 minutes
        })
    }

    /// Get the hex-encoded public key for sharing
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.verifying_key.to_bytes())
    }

    /// Sign a payload, returning a SignedPayload
    pub fn sign(&self, data: &[u8]) -> SignedPayload {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let nonce = rand::random::<u64>();

        // Create message to sign: data + timestamp + nonce
        let mut message = data.to_vec();
        message.extend_from_slice(&timestamp.to_le_bytes());
        message.extend_from_slice(&nonce.to_le_bytes());

        let signature: Signature = self.signing_key.sign(&message);

        SignedPayload {
            data: data.to_vec(),
            signature: hex::encode(signature.to_bytes()),
            signer_pubkey: self.public_key_hex(),
            timestamp,
            nonce,
        }
    }

    /// Verify a signed payload
    /// Returns the original data if valid, or an error
    pub fn verify(&mut self, payload: &SignedPayload) -> Result<Vec<u8>, SecurityError> {
        // Skip verification if not required
        if !self.require_signatures {
            return Ok(payload.data.clone());
        }

        // Check timestamp (prevent replay of old messages)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if payload.timestamp > now + 60 {
            return Err(SecurityError::FutureTimestamp);
        }

        if now - payload.timestamp > self.max_message_age_secs {
            return Err(SecurityError::ExpiredMessage);
        }

        // Check nonce (prevent replay within window)
        if self.seen_nonces.contains(&payload.nonce) {
            return Err(SecurityError::ReplayDetected);
        }

        // Decode public key
        let pubkey_bytes =
            hex::decode(&payload.signer_pubkey).map_err(|_| SecurityError::InvalidPublicKey)?;

        if pubkey_bytes.len() != 32 {
            return Err(SecurityError::InvalidPublicKey);
        }

        let mut pubkey_arr = [0u8; 32];
        pubkey_arr.copy_from_slice(&pubkey_bytes);

        let verifying_key =
            VerifyingKey::from_bytes(&pubkey_arr).map_err(|_| SecurityError::InvalidPublicKey)?;

        // Reconstruct signed message
        let mut message = payload.data.clone();
        message.extend_from_slice(&payload.timestamp.to_le_bytes());
        message.extend_from_slice(&payload.nonce.to_le_bytes());

        // Verify signature
        let sig_bytes =
            hex::decode(&payload.signature).map_err(|_| SecurityError::InvalidSignature)?;
        if sig_bytes.len() != SIGNATURE_SIZE {
            return Err(SecurityError::InvalidSignature);
        }
        let mut sig_arr = [0u8; SIGNATURE_SIZE];
        sig_arr.copy_from_slice(&sig_bytes);
        let signature = Signature::from_bytes(&sig_arr);
        verifying_key
            .verify(&message, &signature)
            .map_err(|_| SecurityError::InvalidSignature)?;

        // Record nonce to prevent replay
        self.seen_nonces.insert(payload.nonce);

        // Prune old nonces periodically (simple approach: limit size)
        if self.seen_nonces.len() > 10000 {
            self.seen_nonces.clear();
        }

        Ok(payload.data.clone())
    }
}

/// Security-related errors
#[derive(Debug, Clone)]
pub enum SecurityError {
    InvalidSignature,
    InvalidPublicKey,
    ExpiredMessage,
    FutureTimestamp,
    ReplayDetected,
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityError::InvalidSignature => write!(f, "Invalid signature"),
            SecurityError::InvalidPublicKey => write!(f, "Invalid public key"),
            SecurityError::ExpiredMessage => write!(f, "Message expired"),
            SecurityError::FutureTimestamp => write!(f, "Future timestamp detected"),
            SecurityError::ReplayDetected => write!(f, "Replay attack detected"),
        }
    }
}

impl std::error::Error for SecurityError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_sign_verify() {
        let temp_dir = std::env::temp_dir();
        let key_path = temp_dir.join("test_qres_key");

        // Clean up any existing key
        let _ = fs::remove_file(&key_path);

        let mut manager = SecurityManager::new(&key_path, true).unwrap();
        let data = b"test model weights";

        let signed = manager.sign(data);
        let verified = manager.verify(&signed).unwrap();

        assert_eq!(verified, data.to_vec());

        // Clean up
        let _ = fs::remove_file(&key_path);
    }

    #[test]
    fn test_invalid_signature() {
        let temp_dir = std::env::temp_dir();
        let key_path = temp_dir.join("test_qres_key2");

        let _ = fs::remove_file(&key_path);

        let mut manager = SecurityManager::new(&key_path, true).unwrap();
        let data = b"test model weights";

        let mut signed = manager.sign(data);
        // Corrupt the signature (modify hex string)
        let mut sig_bytes = hex::decode(&signed.signature).unwrap();
        sig_bytes[0] ^= 0xFF;
        signed.signature = hex::encode(&sig_bytes);

        let result = manager.verify(&signed);
        assert!(result.is_err());

        let _ = fs::remove_file(&key_path);
    }

    #[test]
    fn test_replay_prevention() {
        let temp_dir = std::env::temp_dir();
        let key_path = temp_dir.join("test_qres_key3");

        let _ = fs::remove_file(&key_path);

        let mut manager = SecurityManager::new(&key_path, true).unwrap();
        let data = b"test model weights";

        let signed = manager.sign(data);

        // First verification should succeed
        assert!(manager.verify(&signed).is_ok());

        // Second verification with same nonce should fail (replay)
        let result = manager.verify(&signed);
        assert!(matches!(result, Err(SecurityError::ReplayDetected)));

        let _ = fs::remove_file(&key_path);
    }
}
