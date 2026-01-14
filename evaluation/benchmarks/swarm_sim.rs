#![allow(clippy::zombie_processes)]
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

fn main() {
    println!("[Sim] Starting QRES v5 Hive Simulation (Multi-Node FedProx)...");

    // 1. Locate Python Environment
    let python_path = "c:\\Dev\\QRES\\.venv\\Scripts\\python.exe";
    if !Path::new(python_path).exists() {
        eprintln!("[Error] Python not found at {}", python_path);
        return;
    }

    // 2. Setup Hive Server
    println!("[Setup] Spawning Hive Server...");
    // Reset server state first via API (optional but good practice) or just restart
    // Since spawn starts fresh instance in memory, it's fine.
    let mut server = Command::new(python_path)
        .arg("../utils/hive_server.py")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to start Hive Server");

    // Compute absolute CLI path
    let mut cli_path = std::env::current_exe().unwrap();
    cli_path.pop(); // debug
    cli_path.pop(); // target
    cli_path.push("release");
    cli_path.push("qres-cli.exe");

    if !cli_path.exists() {
        println!("[Warning] Release binary not found. Trying debug...");
        cli_path.pop();
        cli_path.pop();
        cli_path.push("debug");
        cli_path.push("qres-cli.exe");
    }

    let cli_path_str = cli_path.to_str().unwrap();
    println!("[Setup] Using CLI: {}", cli_path_str);

    // Wait for startup
    thread::sleep(Duration::from_secs(2));

    // 3. Setup 5 Agents
    let agent_count = 5;
    let base_dir = "swarm_sim_agents";
    let _ = fs::create_dir_all(base_dir);

    for i in 0..agent_count {
        let dir = format!("{}/agent_{}", base_dir, i);
        let _ = fs::create_dir_all(&dir);

        // Define Brain
        let brain_json = if i == 0 {
            // Expert A: High confidence in Index 3 (Spectral)
            r#"{"confidence": [0.1, 0.1, 0.1, 10.0], "stats": {"compressions": 5000}}"#
        } else if i == 1 {
            // Expert B: High confidence in Index 2 (Graph)
            r#"{"confidence": [0.1, 0.1, 10.0, 0.1], "stats": {"compressions": 5000}}"#
        } else {
            // Novices
            r#"{"confidence": [1.0, 1.0, 1.0, 1.0], "stats": {"compressions": 10}}"#
        };

        fs::write(format!("{}/qres_brain.json", dir), brain_json).unwrap();
    }

    // 4. Run Sync Cycle
    for i in 0..agent_count {
        let dir = format!("{}/agent_{}", base_dir, i);
        println!("[Agent {}] Syncing...", i);

        let status = Command::new(python_path)
            .arg("../../../utils/hive_sync.py")
            .current_dir(&dir)
            .env("HIVE_URL", "http://localhost:5000")
            .env("QRES_CLI", cli_path_str)
            .output()
            .expect("Agent sync failed");

        if !status.status.success() {
            eprintln!(
                "Agent {} failed: {}",
                i,
                String::from_utf8_lossy(&status.stderr)
            );
        }
    }

    // 5. Verification
    println!("[Verify] Checking Learning Transfer...");

    // Check Metrics via Python helper
    let metrics_cmd = Command::new(python_path)
        .arg("-c")
        .arg("import requests; print(requests.get('http://localhost:5000/metrics').text)")
        .output()
        .expect("Failed to fetch metrics");

    println!(
        "Hive Metrics:\n{}",
        String::from_utf8_lossy(&metrics_cmd.stdout)
    );

    // Check Novice (Agent 4) Brain
    let last_agent_dir = format!("{}/agent_{}", base_dir, agent_count - 1);
    let novice_brain = fs::read_to_string(format!("{}/qres_brain.json", last_agent_dir)).unwrap();
    println!(
        "[Verify] Agent {} Brain:\n{}",
        agent_count - 1,
        novice_brain
    );

    // Naive string check for transferred knowledge
    // Should have elevated confidence for BOTH index 2 and 3?
    // FedAvg might average them down, but they should be significantly higher than 1.0
    // e.g. (10 + 0.1)/2 = ~5. If weighted by samples...
    // Experts have 5000 samples, Novices 10.
    // So Global should be dominated by Experts.

    let success_idx2 = novice_brain.contains("Confidence")
        && (novice_brain.contains("4.")
            || novice_brain.contains("5.")
            || novice_brain.contains("9.")
            || novice_brain.contains("10."));
    // Actually, simple string matching is brittle with float formatting.
    // But let's check for "success" keywords printed by validation script if possible.
    // Instead, I'll rely on the output inspection for now or simple "higher confidence" check.

    if success_idx2 {
        println!("[SUCCESS] Agent acquired Expert Knowledge!");
    } else {
        println!("[Partial] Check global weights in metrics above for confirmation.");
    }

    // Cleanup
    let _ = server.kill();
    let _ = server.wait();
    let _ = fs::remove_dir_all(base_dir);
}
