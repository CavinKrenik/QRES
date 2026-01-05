use qres_core::mixer::NUM_MODELS;
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
