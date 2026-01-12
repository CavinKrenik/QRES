use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time;

/// Telemetry data sent by the sensor node.
/// Matches the `SensorData` struct in aggregator.rs
#[derive(Debug, Serialize, Deserialize)]
pub struct SensorData {
    pub id: String,
    pub value: f64,
    pub timestamp: u64,
}

/// A simulated sensor node.
pub struct SensorNode {
    pub id: String,
    pub aggregator_url: String,
    pub sample_rate_hz: u64,
}

impl SensorNode {
    /// New sensor node.
    pub fn new(id: String, aggregator_url: String, sample_rate_hz: u64) -> Self {
        Self {
            id,
            aggregator_url,
            sample_rate_hz,
        }
    }

    /// Runs the simulation loop.
    pub async fn run(&self) {
        let client = reqwest::Client::new();
        let mut interval = time::interval(Duration::from_secs(1) / self.sample_rate_hz as u32);
        let start_time = std::time::SystemTime::now();

        println!("Sensor {} started.", self.id);

        loop {
            interval.tick().await;

            let elapsed_secs = start_time.elapsed().unwrap_or_default().as_secs_f64();

            // Generate synthetic value (temperature-like)
            let value = {
                let mut rng = rand::thread_rng();
                25.0 + (elapsed_secs * 0.1).sin() * 5.0 + rng.gen_range(-0.5..0.5)
            };

            let data = SensorData {
                id: self.id.clone(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                value,
            };

            // Send data
            match client.post(&self.aggregator_url).json(&data).send().await {
                Ok(_) => {
                    // Quiet success
                }
                Err(e) => {
                    eprintln!("Sensor {} failed to send data: {}", self.id, e);
                }
            }
        }
    }
}
