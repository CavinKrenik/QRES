use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

fn main() {
    println!("[Sim] Starting QRES v4 Hive Simulation (FedProx)...");

    // 1. Locate Python Environment
    let python_path = "c:\\Dev\\QRES\\.venv\\Scripts\\python.exe";
    if !Path::new(python_path).exists() {
        eprintln!("[Error] Python not found at {}", python_path);
        return;
    }

    // 2. Setup Hive Server
    println!("[Setup] Spawning Hive Server...");
    let mut server = Command::new(python_path)
        .arg("../utils/hive_server.py")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to start Hive Server");

    // Compute absolute CLI path
    // We are running from qres_rust/target/debug/swarm_sim.exe usually
    // We want qres_rust/target/release/qres-cli.exe
    let mut cli_path = std::env::current_exe().unwrap(); // .../target/debug/swarm_sim.exe
    cli_path.pop(); // debug
    cli_path.pop(); // target
    cli_path.push("release");
    cli_path.push("qres-cli.exe");

    // Explicit check
    if !cli_path.exists() {
        // Try current dir fallback if run via cargo run in root might differ (unlikely)
        // Or assume user built release.
        println!(
            "[Warning] Release binary not found at {:?}. Trying debug...",
            cli_path
        );
        cli_path.pop();
        cli_path.pop();
        cli_path.push("debug");
        cli_path.push("qres-cli.exe");
    }

    let cli_path_str = cli_path.to_str().unwrap();
    println!("[Setup] Using CLI: {}", cli_path_str);

    // Wait for startup
    thread::sleep(Duration::from_secs(2));

    // 3. Setup Agents
    let dir_a = "swarm_sim_a";
    let dir_b = "swarm_sim_b";
    let _ = fs::create_dir_all(dir_a);
    let _ = fs::create_dir_all(dir_b);

    // Seed Agent A (Expert - High Confidence in ID 3/Spectral)
    let brain_a_json = r#"{"confidence": [0.5, 0.5, 0.5, 10.0], "stats": {"compressions": 5000}}"#;
    fs::write(format!("{}/qres_brain.json", dir_a), brain_a_json).unwrap();

    // Seed Agent B (Novice - Default)
    let brain_b_json = r#"{"confidence": [0.5, 0.5, 0.5, 0.5], "stats": {"compressions": 10}}"#;
    fs::write(format!("{}/qres_brain.json", dir_b), brain_b_json).unwrap();

    // 4. Agent A Syncs (Push)
    println!("[Agent A] Expert Connecting to Hive...");
    let status_a = Command::new(python_path)
        .arg("../../utils/hive_sync.py")
        .current_dir(dir_a)
        .env("HIVE_URL", "http://localhost:5000")
        .env("QRES_CLI", cli_path_str)
        .output()
        .expect("Agent A sync failed");

    println!(
        "Agent A Stdout:\n{}",
        String::from_utf8_lossy(&status_a.stdout)
    );
    println!(
        "Agent A Stderr:\n{}",
        String::from_utf8_lossy(&status_a.stderr)
    );

    // 5. Agent B Syncs (Pull/FedProx)
    println!("[Agent B] Novice Connecting to Hive...");
    let status_b = Command::new(python_path)
        .arg("../../utils/hive_sync.py")
        .current_dir(dir_b)
        .env("HIVE_URL", "http://localhost:5000")
        .env("QRES_CLI", cli_path_str)
        .output()
        .expect("Agent B sync failed");

    println!(
        "Agent B Stdout:\n{}",
        String::from_utf8_lossy(&status_b.stdout)
    );
    println!(
        "Agent B Stderr:\n{}",
        String::from_utf8_lossy(&status_b.stderr)
    );

    // 6. Verification
    let final_brain_b = fs::read_to_string(format!("{}/qres_brain.json", dir_b)).unwrap();
    println!("[Verify] Checking Agent B's Brain:\n{}", final_brain_b);

    if final_brain_b.contains("10.0") || final_brain_b.contains("9.") {
        println!("[SUCCESS] Agent B acquired Expert Knowledge (Zero-Shot)!");
    } else if final_brain_b.contains("Confidence") {
        // Fallback check
        println!("[Partial] Check values manually.");
    } else {
        println!("[FAILURE] Agent B did not evolve.");
    }

    // Cleanup
    let _ = server.kill();
    let _ = fs::remove_dir_all(dir_a);
    let _ = fs::remove_dir_all(dir_b);
}
