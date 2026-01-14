use alloc::vec;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Regime {
    Calm,
    Storm,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RegimeChange {
    None,
    Drift { current_error: f32, threshold: f32 },
}

pub struct RegimeDetector {
    window_size: usize,
    history: Vec<f32>,
    /// Running sum of values in the window
    sum: f32,
    /// Running sum of squares in the window
    sum_sq: f32,
    /// Current write index in the ring buffer
    idx: usize,
    /// Number of samples observed so far
    count: usize,
    /// Entropy threshold for storm detection
    entropy_threshold: f32,
    /// Throughput threshold (bytes/sec) for storm detection
    throughput_threshold: f32,
    /// Current regime
    current_regime: Regime,

    // --- Throughput Tracking Fields ---
    /// Last update timestamp (ms)
    last_update_ms: u64,
    /// Accumulated bytes since last update
    accumulated_bytes: u64,
    /// Current throughput metric (bytes/sec)
    current_throughput: f32,
}

impl RegimeDetector {
    pub fn new(window_size: usize, entropy_threshold: f32, throughput_threshold: f32) -> Self {
        Self {
            window_size,
            history: vec![0.0; window_size],
            sum: 0.0,
            sum_sq: 0.0,
            idx: 0,
            count: 0,
            entropy_threshold,
            throughput_threshold,
            current_regime: Regime::Calm,
            last_update_ms: 0,
            accumulated_bytes: 0,
            current_throughput: 0.0,
        }
    }

    pub fn current_regime(&self) -> Regime {
        self.current_regime
    }

    /// Update regime based on entropy and throughput
    /// entropy: current entropy value
    /// packet_size: size of the current packet in bytes
    /// now_ms: current system timestamp in milliseconds
    pub fn update(&mut self, entropy: f32, packet_size: usize, now_ms: u64) {
        // 1. Initialize timer on first run
        if self.last_update_ms == 0 {
            self.last_update_ms = now_ms;
        }

        // 2. Accumulate bytes (READS accumulated_bytes)
        self.accumulated_bytes += packet_size as u64;

        // 3. Check Time Window (READS last_update_ms)
        let elapsed = now_ms.saturating_sub(self.last_update_ms);

        // Update throughput metric every 1 second (1000ms)
        if elapsed >= 1000 {
            // Calculate bytes/sec
            self.current_throughput = (self.accumulated_bytes as f32) / (elapsed as f32 / 1000.0);

            // Reset Window
            self.last_update_ms = now_ms;
            self.accumulated_bytes = 0;
        }

        // 4. Dual trigger: Storm if entropy > threshold OR throughput > threshold
        let new_regime = if entropy > self.entropy_threshold
            || self.current_throughput > self.throughput_threshold
        {
            Regime::Storm
        } else {
            Regime::Calm
        };

        self.current_regime = new_regime;
    }

    /// Observe a new residual (absolute error).
    /// Returns a RegimeChange event if anomaly detected.
    pub fn observe(&mut self, error: f32) -> RegimeChange {
        let abs_error = error.abs();

        // 1. Check for anomaly BEFORE updating stats (compare against *historical* baseline)
        // Only check if we have enough data (full window)
        let result = if self.count >= self.window_size {
            let mean = self.sum / self.window_size as f32;
            let mean_sq = self.sum_sq / self.window_size as f32;
            // Variance = E[X^2] - (E[X])^2
            let variance = (mean_sq - mean * mean).max(0.0);
            let std_dev = variance.sqrt();

            // Threshold: Mean + 3 * StdDev
            let threshold = mean + 3.0 * std_dev;

            if abs_error > threshold {
                RegimeChange::Drift {
                    current_error: abs_error,
                    threshold,
                }
            } else {
                RegimeChange::None
            }
        } else {
            RegimeChange::None
        };

        // 2. Update Window (Ring Buffer)
        let old_val = self.history[self.idx];
        self.history[self.idx] = abs_error;

        // Update running stats
        self.sum = self.sum - old_val + abs_error;
        self.sum_sq = self.sum_sq - (old_val * old_val) + (abs_error * abs_error);

        // Advance index
        self.idx = (self.idx + 1) % self.window_size;
        self.count += 1;

        result
    }

    pub fn reset(&mut self) {
        self.sum = 0.0;
        self.sum_sq = 0.0;
        self.idx = 0;
        self.count = 0;
        for x in &mut self.history {
            *x = 0.0;
        }
        // Reset throughput tracking
        self.last_update_ms = 0;
        self.accumulated_bytes = 0;
        self.current_throughput = 0.0;
    }
}
