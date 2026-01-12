use qres_core::multivariate::correlation::PearsonCorrelation;
use qres_core::multivariate::manager::MultivariateManager;
use std::collections::HashMap;

fn main() {
    println!(">> QRES v16 Multivariate Analysis Demo");
    println!("--------------------------------------");

    // 1. Generate Synthetic Data
    let n = 1000;
    let mut t_series = Vec::with_capacity(n);
    let mut humidity = Vec::with_capacity(n); // Correlated
    let mut noise = Vec::with_capacity(n); // Uncorrelated

    for i in 0..n {
        let t = i as f32 * 0.1;

        // Base signal (Temperature)
        let val = t.sin();
        t_series.push(val);

        // Correlated signal (Humidity - inverse relation + noise)
        humidity.push(val * -0.8 + (t * 13.0).sin() * 0.05);

        // Random noise
        noise.push((t * 50.0).cos());
    }

    // 2. Test Single Correlation
    let score = PearsonCorrelation::calculate(&t_series, &humidity);
    println!(
        "Correlation (Temp vs Humidity): {:.4} (Expected: High Negative)",
        score
    );

    let noise_score = PearsonCorrelation::calculate(&t_series, &noise);
    println!(
        "Correlation (Temp vs Noise):    {:.4} (Expected: Low)",
        noise_score
    );

    // 3. Test Group Finding
    println!("\n>> Detecting Groups (Threshold: 0.7)...");
    let mut streams = HashMap::new();
    streams.insert("sensor_temp".to_string(), t_series);
    streams.insert("sensor_humid".to_string(), humidity);
    streams.insert("sensor_noise".to_string(), noise);

    let groups = MultivariateManager::find_groups(&streams, 0.7);

    for group in &groups {
        println!("  [Group Found] Leader: {}", group.leader);
        println!("    Members: {:?}", group.members);
    }

    if groups.is_empty() {
        println!("  No groups found (Unexpected!).");
    }
}
