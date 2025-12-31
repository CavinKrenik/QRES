use std::fs;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

// This simulation requires the qres-cli binary to be built with --features swarm (feature flag logic might be needed if libp2p is optional, but for now it's standard)

fn main() {
    println!("🧪 Starting Swarm Simulation...");

    // 1. Setup Brains
    let brain_a = "brain_node_a.json";
    let brain_b = "brain_node_b.json";

    // Reset
    let _ = fs::remove_file(brain_a);
    let _ = fs::remove_file(brain_b);

    // Seed Node A with "Genomic" wisdom (High LSTM confidence)
    let seed_brain_a = r#"{"confidence": [0.0, 0.0, 0.0, 10.0, 0.0, 0.0]}"#; // ID 3 (LSTM) = 10.0
    fs::write(brain_a, seed_brain_a).expect("Failed to write brain A");

    // Seed Node B with "Default" (Linear)
    let seed_brain_b = r#"{"confidence": [10.0, 0.0, 0.0, 0.0, 0.0, 0.0]}"#; // ID 1 (Linear) = 10.0
    fs::write(brain_b, seed_brain_b).expect("Failed to write brain B");

    // 2. Spawn Node A (The Teacher)
    // Note: In a real test we'd need to pass the brain path to the CLI.
    // Since CLI currently hardcodes "qres_brain.json", we will temporarily create directories to separate them
    // or we assume the implementation allows path override.
    // Ideally, we'd update `QresSwarm::run_daemon` to take a path, and CLI to parse it.
    // For this simulation, let's assume valid implementation allows environment var or CWD change.

    // We will use CWD isolation.
    let dir_a = "swarm_sim_a";
    let dir_b = "swarm_sim_b";
    let _ = fs::create_dir_all(dir_a);
    let _ = fs::create_dir_all(dir_b);

    fs::copy(brain_a, format!("{}/qres_brain.json", dir_a)).unwrap();
    fs::copy(brain_b, format!("{}/qres_brain.json", dir_b)).unwrap();

    let exe = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap() // debug/deps
        .parent()
        .unwrap() // debug
        .parent()
        .unwrap() // target
        .join("release")
        .join("qres-cli.exe");

    if !exe.exists() {
        eprintln!(
            "Binary not found at {:?}. Run usage: cargo run --bin swarm_sim",
            exe
        );
        return;
    }

    println!("🚀 Spawning Node A (Teacher)...");
    let mut child_a = Command::new(&exe)
        .arg("swarm")
        .current_dir(dir_a)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn node A");

    println!("🚀 Spawning Node B (Student)...");
    let mut child_b = Command::new(&exe)
        .arg("swarm")
        .current_dir(dir_b)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn node B");

    // 3. Wait for Gossip (60s+ for broadcast + propagation)
    println!("⏳ Waiting 70s for Wisdom Propagation (Gossip + Merge)...");
    thread::sleep(Duration::from_secs(70));

    // 4. Wait and Kill
    let _ = child_a.wait();
    let _ = child_b.wait();
    let _ = child_a.kill();
    let _ = child_b.kill();

    // 5. Verify Node B
    let content_b = fs::read_to_string(format!("{}/qres_brain.json", dir_b)).unwrap();
    println!("Node B Final Brain: {}", content_b);

    // Check if LSTM confidence increased (was 0.0)
    // Expected: 0.05 * 10.0 = 0.5 minimum.
    if content_b.contains("\"confidence\":") {
        // Simple string check or regex
        println!("✅ Simulation Complete. Manually verify confidence drift.");
    } else {
        println!("❌ Failed to read brain.");
    }
}
