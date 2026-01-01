use rustfft::{num_complex::Complex, FftPlanner};

// QRES v4.1 Enhanced Spectral Predictor
// Improvements:
// - Larger window (2048) for better frequency resolution
// - Harmonic detection (2nd, 3rd harmonics)
// - Adaptive threshold based on signal strength
// Target: 60%+ compression on sine waves

pub struct SpectralPredictor {
    window_size: usize,
    buffer: Vec<f32>,
    cursor: usize,
    count: usize,
    planner: FftPlanner<f32>,
    // Adaptive threshold
    signal_strength_history: Vec<f32>,
    // Cached model for extrapolation
    cached_model: Option<SpectralModel>,
    steps_since_update: usize,
}

struct SpectralModel {
    dc: f32,
    components: Vec<FreqComponent>,
}

struct FreqComponent {
    amplitude: f32,
    frequency: f32,
    phase: f32,
}

impl SpectralPredictor {
    pub fn new(window_size: usize) -> Self {
        SpectralPredictor {
            window_size,
            buffer: vec![0.0; window_size], // Pre-allocated zeroed
            cursor: 0,
            count: 0,
            planner: FftPlanner::new(),
            signal_strength_history: Vec::with_capacity(10),
            cached_model: None,
            steps_since_update: 0,
        }
    }

    pub fn update(&mut self, val: u8) {
        // Circular buffer update: O(1)
        self.buffer[self.cursor] = val as f32;
        self.cursor = (self.cursor + 1) % self.window_size;
        if self.count < self.window_size {
            self.count += 1;
        }
        self.steps_since_update += 1;
    }

    pub fn predict(&mut self) -> u8 {
        if self.count < self.window_size {
            return 128; // Not enough data
        }

        // Lazy Update Strategy:
        // Only re-calculate FFT every 64 steps (reduces overhead by 64x)
        // For sine waves, frequency doesn't change rapidly.
        if self.cached_model.is_none() || self.steps_since_update >= 64 {
            self.recalc_model();
            self.steps_since_update = 0;
        }

        if let Some(model) = &self.cached_model {
            let mut pred_val = 0.0;

            // Project forward
            // t = window_size (end of window) + steps_since_last_fft
            // Phase is relative to window start (t=0)
            let t = (self.window_size + self.steps_since_update) as f32;

            for comp in &model.components {
                let angle = (2.0 * std::f32::consts::PI * comp.frequency * t
                    / (self.window_size as f32))
                    + comp.phase;
                pred_val += comp.amplitude * angle.cos();
            }

            let result = model.dc + pred_val;
            return result.clamp(0.0, 255.0) as u8;
        }

        // Fallback: Use most recent value
        // Cursor points to oldest, so cursor-1 (modulo size) is newest
        let last_idx = if self.cursor == 0 {
            self.window_size - 1
        } else {
            self.cursor - 1
        };
        self.buffer[last_idx] as u8
    }

    fn recalc_model(&mut self) {
        // 1. Prepare FFT Input (Unroll circular buffer)
        // Order: Oldest -> Newest
        // Start reading from self.cursor (oldest)
        let mut input: Vec<Complex<f32>> = (0..self.window_size)
            .map(|i| {
                let idx = (self.cursor + i) % self.window_size;
                Complex::new(self.buffer[idx], 0.0)
            })
            .collect();

        // 2. Perform FFT
        let fft = self.planner.plan_fft_forward(self.window_size);
        fft.process(&mut input);

        // 3. Find Dominant Frequencies
        let mut components = Vec::new();

        let mut max_mag = 0.0;
        let mut fundamental_idx = 0;

        // Search Nyquist
        for (i, bin) in input.iter().enumerate().take(self.window_size / 2).skip(1) {
            let mag = bin.norm_sqr();
            if mag > max_mag {
                max_mag = mag;
                fundamental_idx = i;
            }
        }

        let threshold = max_mag * 0.1;
        self.signal_strength_history.push(max_mag);
        if self.signal_strength_history.len() > 10 {
            self.signal_strength_history.remove(0);
        }

        if max_mag > 100.0 {
            // Helper to add component
            let add_comp = |idx: usize, bins: &[Complex<f32>], out: &mut Vec<FreqComponent>| {
                let bin = bins[idx];
                out.push(FreqComponent {
                    amplitude: bin.norm() / (self.window_size as f32) * 2.0,
                    frequency: idx as f32,
                    phase: bin.arg(),
                });
            };

            add_comp(fundamental_idx, &input, &mut components);

            // Harmonics
            for harmonic in 2..=3 {
                let h_idx = fundamental_idx * harmonic;
                if h_idx < self.window_size / 2 && input[h_idx].norm_sqr() > threshold {
                    add_comp(h_idx, &input, &mut components);
                }
            }
        }

        let dc = input[0].re / (self.window_size as f32);
        self.cached_model = Some(SpectralModel { dc, components });
    }

    /// Returns confidence in prediction (0.0 to 1.0)
    pub fn confidence(&self) -> f32 {
        if self.signal_strength_history.is_empty() {
            return 0.0;
        }
        let avg: f32 = self.signal_strength_history.iter().sum::<f32>()
            / self.signal_strength_history.len() as f32;
        (avg / 1_000_000.0).min(1.0)
    }
}
