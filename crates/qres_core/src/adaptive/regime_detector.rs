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
    /// Last update timestamp (ms)
    last_update_ms: u64,
    /// Accumulated bytes since last update
    accumulated_bytes: u64,
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
        }
    }

    pub fn current_regime(&self) -> Regime {
        self.current_regime
    }

    /// Update regime based on entropy and throughput
    /// elapsed_ms: time since last update
    /// bytes: bytes processed in this interval
    pub fn update(&mut self, entropy: f32, elapsed_ms: u64, bytes: u64) {
        // Update throughput
        let bytes_per_sec = if elapsed_ms > 0 {
            (bytes as f64 / elapsed_ms as f64 * 1000.0) as f32
        } else {
            0.0
        };

        // Dual trigger: Storm if entropy > threshold OR throughput > threshold
        let new_regime =
            if entropy > self.entropy_threshold || bytes_per_sec > self.throughput_threshold {
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
        self.count += 1; // Saturating add could be safer if running forever, but usize is huge.

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
    }
}
