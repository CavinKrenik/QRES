use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrivacyConfig {
    /// Whether differential privacy is enabled
    #[serde(default)]
    pub enabled: bool,
    /// Privacy budget (epsilon)
    #[serde(default = "default_epsilon")]
    pub epsilon: f32,
    /// Failure probability (delta)
    #[serde(default = "default_delta")]
    pub delta: f32,
    /// L2 clipping threshold
    #[serde(default = "default_clipping")]
    pub clipping_threshold: f32,
    /// Whether secure aggregation (pairwise masking) is enabled
    #[serde(default)]
    pub secure_aggregation: bool,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            epsilon: 1.0,
            delta: 1e-5,
            clipping_threshold: 1.0,
            secure_aggregation: false,
        }
    }
}

fn default_epsilon() -> f32 {
    1.0
}
fn default_delta() -> f32 {
    1e-5
}
fn default_clipping() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub swarm: SwarmConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub aggregation: AggregationConfig,
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub privacy: PrivacyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmConfig {
    pub gossip_interval: u64,
    pub wan_mode: bool,
    pub max_peers: usize,
}

impl Default for SwarmConfig {
    fn default() -> Self {
        Self {
            gossip_interval: 600,
            wan_mode: false,
            max_peers: 50,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub ban_duration: u64,
    pub max_violations: u8,
    /// Whether to require ed25519 signatures on model updates
    pub require_signatures: bool,
    /// Path to the ed25519 private key file
    pub key_path: Option<String>,
    /// List of trusted peer IDs (e.g., "12D3KooW...")
    #[serde(default)]
    pub trusted_peers: Vec<String>,
    /// List of trusted public keys in hex format (32-byte ed25519)
    #[serde(default)]
    pub trusted_pubkeys: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            ban_duration: 3600,
            max_violations: 2,
            require_signatures: false,
            key_path: None,
            trusted_peers: Vec::new(),
            trusted_pubkeys: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub port: u16,
    pub enabled: bool,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            port: 3030,
            enabled: true,
        }
    }
}

/// Aggregation settings for robust federated averaging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationConfig {
    /// Aggregation mode: "mean", "krum", "multi_krum", "trimmed_mean", "median"
    #[serde(default = "default_agg_mode")]
    pub mode: String,
    /// Expected fraction of Byzantine (malicious) nodes (for Krum)
    #[serde(default = "default_expected_byz")]
    pub expected_byzantines_fraction: f32,
    /// Number of updates to buffer before aggregating (for Multi-Krum)
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
    /// Trim fraction for trimmed mean (e.g., 0.2 = trim 10% from each side)
    #[serde(default)]
    pub trim_fraction: f32,
}

fn default_agg_mode() -> String {
    "mean".to_string()
}

fn default_expected_byz() -> f32 {
    0.2
}

fn default_buffer_size() -> usize {
    5
}

impl Default for AggregationConfig {
    fn default() -> Self {
        Self {
            mode: default_agg_mode(),
            expected_byzantines_fraction: default_expected_byz(),
            buffer_size: default_buffer_size(),
            trim_fraction: 0.2,
        }
    }
}

impl Config {
    pub fn get_config_path() -> PathBuf {
        let mut path = dirs::home_dir().expect("Could not find home directory");
        path.push(".qres");
        fs::create_dir_all(&path).expect("Could not create .qres directory");
        path.push("config.toml");
        path
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::get_config_path();

        if !path.exists() {
            // Create default config
            let config = Config::default();
            config.save()?;
            return Ok(config);
        }

        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::get_config_path();
        let toml = toml::to_string_pretty(self)?;
        fs::write(path, toml)?;
        Ok(())
    }
}
