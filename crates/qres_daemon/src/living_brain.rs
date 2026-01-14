use qres_core::mixer::NUM_MODELS;
use qres_core::zk_proofs::ProofBundle;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LivingBrain {
    pub version: u8,
    pub predictors: Vec<String>,
    pub stats: serde_json::Value,
    pub confidence: Vec<f32>,
    pub global_confidence: Option<Vec<f32>>, // Phase 2: FedProx Anchor
    pub best_engine_weights: Option<Vec<u8>>,
}

impl Default for LivingBrain {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum BrainMessage {
    Full(LivingBrain),
    Delta(BrainDelta),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BrainDelta {
    pub timestamp: u64,
    pub updates: Vec<(usize, f32)>,
}

/// A wrapper for the LivingBrain that includes a ZK Proof and a Signature.
/// This matches the "Ghost Protocol" Update Format.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignedEpiphany {
    pub brain: LivingBrain,
    pub proof_bundle: Option<ProofBundle>,
    pub signature: String,   // Hex encoded signature
    pub sender_id: String,   // PeerID or Public Key
    pub timestamp: u64,      // Replay protection
    pub nonce: u64,          // Replay protection
    pub is_storm_mode: bool, // True if weights are I8F8 quantized
}

impl SignedEpiphany {
    pub fn new(
        brain: LivingBrain,
        proof_bundle: Option<ProofBundle>,
        signature: String,
        sender_id: String,
        timestamp: u64,
        nonce: u64,
        is_storm_mode: bool,
    ) -> Self {
        Self {
            brain,
            proof_bundle,
            signature,
            sender_id,
            timestamp,
            nonce,
            is_storm_mode,
        }
    }

    /// Serialize just the payload (brain + proof) for signing
    pub fn payload_bytes(&self) -> Vec<u8> {
        // We re-serialize the components to get the canonical bytes for signing
        // Note: In production, use a stable serialization (like bincode or canonical JSON)
        // Here we use serde_json for simplicity/consistency with existing protocol
        let mut payload = serde_json::to_vec(&self.brain).unwrap_or_default();
        if let Some(proof) = &self.proof_bundle {
            payload.extend(serde_json::to_vec(proof).unwrap_or_default());
        }
        payload.extend(self.timestamp.to_le_bytes());
        payload.extend(self.nonce.to_le_bytes());
        payload.extend([self.is_storm_mode as u8]);
        payload
    }
}

impl LivingBrain {
    pub fn new() -> Self {
        LivingBrain {
            version: 1,
            predictors: vec![
                "lstm".to_string(),
                "graph".to_string(),
                "transformer".to_string(),
            ],
            stats: serde_json::json!({"compressions": 0}),
            confidence: vec![0.5; NUM_MODELS.max(4)], // Ensure enough space
            global_confidence: None,
            best_engine_weights: None,
        }
    }

    pub fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or("{}".to_string())
    }

    pub fn merge(&mut self, other: &LivingBrain, alpha: f32) {
        for i in 0..self.confidence.len().min(other.confidence.len()) {
            self.confidence[i] = self.confidence[i] * (1.0 - alpha) + other.confidence[i] * alpha;
        }
        // Always derive global anchor from the imported brain (truth)
        if other.global_confidence.is_some() {
            self.global_confidence = other.global_confidence.clone();
        }
    }

    pub fn diff(&self, other: &LivingBrain) -> Option<BrainDelta> {
        let mut updates = Vec::new();
        // Check for significant differences in confidence
        for (i, (&a, &b)) in self
            .confidence
            .iter()
            .zip(other.confidence.iter())
            .enumerate()
        {
            if (a - b).abs() > 0.05 {
                // 5% change threshold for "Epiphany"
                updates.push((i, a));
            }
        }

        if updates.is_empty() {
            None
        } else {
            Some(BrainDelta {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                updates,
            })
        }
    }

    pub fn apply_delta(&mut self, delta: &BrainDelta) {
        for &(i, val) in &delta.updates {
            if i < self.confidence.len() {
                // Alpha blend the delta (safely absorb knowledge)
                let alpha = 0.2;
                self.confidence[i] = self.confidence[i] * (1.0 - alpha) + val * alpha;
            }
        }
    }
}
