use crate::edge_realistic::constraint_simulator::ConstraintSimulator;
use crate::edge_realistic::device_profiles::DeviceProfile;
use anyhow::{Context, Result};
use std::path::Path;
use std::thread;
use std::time::Duration;

/// Harness to run benchmarks under simulated edge constraints.
pub struct BenchmarkRunner {
    simulator: ConstraintSimulator,
}

impl BenchmarkRunner {
    /// Initializes a new runner with a specific device profile.
    ///
    /// Args:
    ///     profile_path: Path to the YAML profile configuration.
    ///
    /// Returns:
    ///     A Result containing the new BenchmarkRunner.
    pub fn from_profile<P: AsRef<Path>>(profile_path: P) -> Result<Self> {
        let profile = DeviceProfile::load_from_yaml(&profile_path)
            .with_context(|| "Failed to load device profile for runner")?;

        println!("Initializing BenchmarkRunner for device: {}", profile.name);

        Ok(Self {
            simulator: ConstraintSimulator::new(profile),
        })
    }

    /// Runs a dummy compression benchmark to test the simulation.
    ///
    /// This simulates a workload by sleeping and allocating memory.
    ///
    /// Returns:
    ///     Result indicating success.
    pub fn run_dummy_benchmark(&self) -> Result<()> {
        let task_name = "Dummy Compression";

        self.simulator.run_cpu_constrained(task_name, || {
            // Simulate memory allocation for a buffer (e.g., 50MB)
            let alloc_size_mb = 50;
            self.simulator.allocate_memory(alloc_size_mb)?;
            println!("Allocated {} MB for dummy task.", alloc_size_mb);

            // Simulate CPU work
            // Ideally this would be real work, but for now we sleep to simulate 'native' duration
            // The simulator will add EXTRA sleep on top of this.
            // Let's pretend the work takes 100ms on a reference machine.
            thread::sleep(Duration::from_millis(100));

            // Clean up
            self.simulator.free_memory(alloc_size_mb);
            println!("Freed {} MB.", alloc_size_mb);

            Ok(())
        })
    }
}
