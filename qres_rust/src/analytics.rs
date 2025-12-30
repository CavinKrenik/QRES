use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainSnapshot {
    pub timestamp: DateTime<Utc>,
    pub confidence: Vec<f32>,
    pub wisdom_level: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BrainHistory {
    pub snapshots: Vec<BrainSnapshot>,
}

impl BrainHistory {
    fn get_history_path() -> PathBuf {
        let mut path = dirs::home_dir().expect("Could not find home directory");
        path.push(".qres");
        fs::create_dir_all(&path).expect("Could not create .qres directory");
        path.push("brain_history.json");
        path
    }

    pub fn load() -> Self {
        let path = Self::get_history_path();
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(history) = serde_json::from_str(&content) {
                return history;
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::get_history_path();
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn add_snapshot(&mut self, confidence: Vec<f32>) {
        let wisdom_level = if !confidence.is_empty() {
            confidence.iter().sum::<f32>() / confidence.len() as f32
        } else {
            0.0
        };

        self.snapshots.push(BrainSnapshot {
            timestamp: Utc::now(),
            confidence,
            wisdom_level,
        });

        // Keep last 100 snapshots
        if self.snapshots.len() > 100 {
            self.snapshots.drain(0..self.snapshots.len() - 100);
        }

        let _ = self.save();
    }

    pub fn get_trend(&self) -> String {
        if self.snapshots.len() < 2 {
            return "insufficient_data".to_string();
        }

        let recent = &self.snapshots[self.snapshots.len() - 10.min(self.snapshots.len())..];
        let avg_wisdom: f32 =
            recent.iter().map(|s| s.wisdom_level).sum::<f32>() / recent.len() as f32;

        if avg_wisdom > 0.9 {
            "expert".to_string()
        } else if avg_wisdom > 0.7 {
            "learning".to_string()
        } else {
            "exploring".to_string()
        }
    }
}
