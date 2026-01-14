use crate::edge_realistic::device_profiles::DeviceProfile;
use anyhow::{anyhow, Result};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Simulates hardware constraints by throttling execution and tracking memory.
pub struct ConstraintSimulator {
    profile: DeviceProfile,
    current_memory_usage_mb: Arc<AtomicUsize>,
}

impl ConstraintSimulator {
    /// Creates a new ConstraintSimulator with the given profile.
    ///
    /// Args:
    ///     profile: The DeviceProfile to simulate.
    ///
    /// Returns:
    ///     A new ConstraintSimulator instance.
    pub fn new(profile: DeviceProfile) -> Self {
        Self {
            profile,
            current_memory_usage_mb: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Runs a task with simulated CPU throttling.
    ///
    /// The slowing factor is calculated based on a reference clock speed of 3000 MHz.
    /// If the profile's clock speed is lower, the task is delayed proportionally.
    ///
    /// Args:
    ///     task_name: Name of the task for logging.
    ///     work_fn: The closure containing the actual work.
    ///
    /// Returns:
    ///     The result of the closure execution.
    pub fn run_cpu_constrained<F, T>(&self, task_name: &str, work_fn: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        let start_time = Instant::now();
        let reference_clock_mhz = 3000.0;
        let device_clock_mhz = self.profile.clock_speed_mhz as f64;

        // Calculate slowdown factor. E.g., if device is 1000MHz vs 3000MHz ref, factor is 3.0.
        // We sleep for (factor - 1) * actual_duration.
        let slowdown_factor = if device_clock_mhz > 0.0 {
            reference_clock_mhz / device_clock_mhz
        } else {
            1.0
        };

        println!(
            "Starting task '{}' on device '{}'...",
            task_name, self.profile.name
        );

        // Execute value
        let result = work_fn();

        let elapsed = start_time.elapsed();
        let simulated_delay = if slowdown_factor > 1.0 {
            elapsed.mul_f64(slowdown_factor - 1.0)
        } else {
            Duration::from_secs(0)
        };

        if simulated_delay > Duration::from_millis(0) {
            println!(
                "Throttling task '{}' by {:?} to match {} MHz...",
                task_name, simulated_delay, self.profile.clock_speed_mhz
            );
            thread::sleep(simulated_delay);
        }

        result
    }

    /// Allocates virtual memory to simulate usage, erroring if limit is exceeded.
    ///
    /// Args:
    ///     amount_mb: Amount of memory to allocate in MB.
    ///
    /// Returns:
    ///     Result indicating success or OutOfMemory error.
    pub fn allocate_memory(&self, amount_mb: usize) -> Result<()> {
        let current = self.current_memory_usage_mb.load(Ordering::SeqCst);
        let new_usage = current + amount_mb;

        if new_usage as u64 > self.profile.memory_limit_mb {
            return Err(anyhow!(
                "Out of Error: Device '{}' limit {} MB, requested total {} MB",
                self.profile.name,
                self.profile.memory_limit_mb,
                new_usage
            ));
        }

        self.current_memory_usage_mb
            .store(new_usage, Ordering::SeqCst);
        Ok(())
    }

    /// Frees simulated memory.
    ///
    /// Args:
    ///     amount_mb: Amount of memory to free in MB.
    pub fn free_memory(&self, amount_mb: usize) {
        let _ =
            self.current_memory_usage_mb
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |val| {
                    Some(val.saturating_sub(amount_mb))
                });
    }
}
