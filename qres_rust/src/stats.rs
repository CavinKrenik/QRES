use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionStats {
    pub total_compressions: u64,
    pub total_decompressions: u64,
    pub engines_used: HashMap<String, u64>,
    pub total_bytes_in: u64,
    pub total_bytes_out: u64,
    pub avg_ratio: f64,
}

impl Default for CompressionStats {
    fn default() -> Self {
        let mut engines = HashMap::new();
        engines.insert("zstd".to_string(), 0);
        engines.insert("ipeps".to_string(), 0);
        engines.insert("lstm".to_string(), 0);
        engines.insert("linear".to_string(), 0);

        CompressionStats {
            total_compressions: 0,
            total_decompressions: 0,
            engines_used: engines,
            total_bytes_in: 0,
            total_bytes_out: 0,
            avg_ratio: 0.0,
        }
    }
}

impl CompressionStats {
    fn get_stats_path() -> PathBuf {
        let mut path = dirs::home_dir().expect("Could not find home directory");
        path.push(".qres");
        fs::create_dir_all(&path).expect("Could not create .qres directory");
        path.push("stats.json");
        path
    }

    pub fn load() -> Self {
        let path = Self::get_stats_path();
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(stats) = serde_json::from_str(&content) {
                return stats;
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::get_stats_path();
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn record_compression(&mut self, engine: &str, bytes_in: u64, bytes_out: u64) {
        self.total_compressions += 1;
        *self.engines_used.entry(engine.to_string()).or_insert(0) += 1;
        self.total_bytes_in += bytes_in;
        self.total_bytes_out += bytes_out;

        if self.total_bytes_in > 0 {
            self.avg_ratio = self.total_bytes_out as f64 / self.total_bytes_in as f64;
        }

        let _ = self.save();
    }

    pub fn bytes_saved(&self) -> u64 {
        self.total_bytes_in.saturating_sub(self.total_bytes_out)
    }
}

// Global stats instance
lazy_static::lazy_static! {
    pub static ref GLOBAL_STATS: Arc<Mutex<CompressionStats>> = Arc::new(Mutex::new(CompressionStats::load()));
}
