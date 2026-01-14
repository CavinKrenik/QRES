use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;

/// Represents the hardware constraints of a simulated edge device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceProfile {
    /// The name of the device profile (e.g., "Raspberry Pi Zero").
    pub name: String,
    /// Number of available CPU cores.
    pub cpu_cores: usize,
    /// Clock speed in MHz.
    pub clock_speed_mhz: u64,
    /// Memory limit in Megabytes.
    pub memory_limit_mb: u64,
}

impl DeviceProfile {
    /// Loads a DeviceProfile from a YAML file.
    ///
    /// Args:
    ///     path: The file path to the YAML profile.
    ///
    /// Returns:
    ///     A Result containing the loaded DeviceProfile or an error.
    pub fn load_from_yaml<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(&path)
            .with_context(|| format!("Failed to open profile file: {:?}", path.as_ref()))?;
        let profile: DeviceProfile = serde_yaml::from_reader(file)
            .with_context(|| format!("Failed to parse YAML profile: {:?}", path.as_ref()))?;
        Ok(profile)
    }
}
