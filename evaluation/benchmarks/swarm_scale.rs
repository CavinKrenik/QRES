use std::fs::{self, OpenOptions};
use std::io::Write;
use std::time::Instant;
use sysinfo::{System, SystemExt};
use tokio::task::JoinSet;

// Simulation parameters
const TEST_DURATION_SECS: u64 = 10; // Increased to 10s for stability at scale

// Mock node behavior
async fn spawn_node(id: usize, duration: u64) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));
    let start = Instant::now();

    loop {
        interval.tick().await;
        if start.elapsed().as_secs() >= duration {
            break;
        }

        // Simulate activity:
        // 1. Compute update (Model Dim 1000)
        let mut _data = vec![0.5f32; 1000];

        // 2. Mock "sync" (sleep)
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        if id.is_multiple_of(10) {
            // Periodic "heavy" task
            let _ = (0..10_000).map(|i| (i as f32).sqrt()).sum::<f32>();
        }
    }
}

#[tokio::main]
async fn main() {
    println!("🚀 QRES Massive Scale Benchmark (v18.0)");
    println!("========================================");

    // Ensure output directory
    let output_dir = "reproducibility/results";
    let _ = fs::create_dir_all(output_dir);

    // Open CSV file
    let csv_path = format!("{}/scalability_massive.csv", output_dir);
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&csv_path)
        .expect("Failed to open CSV");

    writeln!(
        file,
        "nodes,memory_mb,memory_per_node_kb,cpu_usage_est,success_rate"
    )
    .unwrap();

    let mut sys = System::new_all();

    // UPDATED: Test scenarios: 100 to 10,000 nodes
    let scenarios = [100, 500, 1000, 2500, 5000, 10_000];

    for node_count in scenarios {
        println!("\nTesting {} nodes...", node_count);

        // 1. Snapshot Baseline Memory
        sys.refresh_all();
        let start_mem = sys.used_memory();

        let start_time = Instant::now();
        let mut set = JoinSet::new();

        // 2. Spawn the Swarm
        for i in 0..node_count {
            set.spawn(spawn_node(i, TEST_DURATION_SECS));
        }

        // 3. Wait for stability (Warm-up)
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // 4. Measure Peak Memory
        sys.refresh_all();
        let peak_mem = sys.used_memory();

        // Calculate Delta
        // Note: Casting to f64 avoids overflow
        let mem_delta_mb = (peak_mem.saturating_sub(start_mem)) as f64 / 1024.0 / 1024.0;
        let mem_per_node_kb = if node_count > 0 {
            (mem_delta_mb * 1024.0) / node_count as f64
        } else {
            0.0
        };

        // 5. Wait for completion
        let mut success_count = 0;
        while let Some(res) = set.join_next().await {
            if res.is_ok() {
                success_count += 1;
            }
        }

        let duration = start_time.elapsed();
        let success_rate = (success_count as f64 / node_count as f64) * 100.0;
        let cpu_est = 1.0; // Mock placeholder

        println!("   ✅ Complete in {:.2}s", duration.as_secs_f64());
        println!(
            "   🧠 Memory Delta: {:.2} MB ({:.2} KB/node)",
            mem_delta_mb, mem_per_node_kb
        );
        println!("   🎯 Success Rate: {:.1}%", success_rate);

        // Write to CSV
        writeln!(
            file,
            "{},{:.2},{:.2},{:.2},{:.1}",
            node_count, mem_delta_mb, mem_per_node_kb, cpu_est, success_rate
        )
        .unwrap();

        // Cooldown to let OS reclaim memory
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    println!("\n💾 Results saved to {}", csv_path);
}
