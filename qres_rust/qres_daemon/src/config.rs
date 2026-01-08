use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub swarm: SwarmConfig,
    pub security: SecurityConfig,
    pub api: ApiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmConfig {
    pub gossip_interval: u64,
    pub wan_mode: bool,
    pub max_peers: usize,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub port: u16,
    pub enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            swarm: SwarmConfig {
                gossip_interval: 600,
                wan_mode: false,
                max_peers: 50,
            },
            security: SecurityConfig {
                ban_duration: 3600,
                max_violations: 2,
                require_signatures: false, // Disabled by default for backward compat
                key_path: None,
                trusted_peers: Vec::new(),
                trusted_pubkeys: Vec::new(),
            },
            api: ApiConfig {
                port: 3030,
                enabled: true,
            },
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
