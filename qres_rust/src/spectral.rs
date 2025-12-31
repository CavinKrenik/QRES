use rustfft::{FftPlanner, num_complex::Complex};
use std::sync::Arc;

// QRES v4.0 Spectral Predictor
// Uses FFT to identify dominant frequencies in a sliding window
// and predicts the next value by projecting those phases forward.

pub struct SpectralPredictor {
    window_size: usize,
    buffer: Vec<f32>,
    planner: FftPlanner<f32>,
}

impl SpectralPredictor {
    pub fn new(window_size: usize) -> Self {
        SpectralPredictor {
            window_size,
            buffer: Vec::with_capacity(window_size),
            planner: FftPlanner::new(),
        }
    }

    pub fn update(&mut self, val: u8) {
        if self.buffer.len() >= self.window_size {
            self.buffer.remove(0);
        }
        self.buffer.push(val as f32);
    }

    pub fn predict(&mut self) -> u8 {
        if self.buffer.len() < self.window_size {
            return 128; // Not enough data
        }

        // 1. Prepare FFT Input
        let mut input: Vec<Complex<f32>> = self.buffer.iter()
            .map(|&val| Complex::new(val, 0.0))
            .collect();

        // 2. Perform FFT
        let fft = self.planner.plan_fft_forward(self.window_size);
        fft.process(&mut input);

        // 3. Find Dominant Frequency
        // Simple strategy: Pick the bin with max magnitude (ignoring DC)
        let mut max_mag = 0.0;
        let mut max_idx = 0;
        
        // Only search first half (Nyquist)
        for i in 1..(self.window_size / 2) {
            let mag = input[i].norm_sqr();
            if mag > max_mag {
                max_mag = mag;
                max_idx = i;
            }
        }

        // 4. Predict
        // If we found a strong signal -> Project it
        // x[n] = A * cos(2*pi*f*n + phi)
        // input[k] gives us A and phi for frequency k.
        // We want to predict x[N] (the next point).
        
        if max_mag > 1000.0 { // Threshold filter
             let bin = &input[max_idx];
             let ampl = bin.norm() / (self.window_size as f32) * 2.0; // Normalize
             let phase = bin.arg();
             let freq = max_idx as f32; // Cycles per window
             
             // Project forward by 1 step (index = window_size)
             let t = self.window_size as f32;
             let angle = (2.0 * std::f32::consts::PI * freq * t / (self.window_size as f32)) + phase;
             let pred_val = ampl * angle.cos();
             
             // Add DC offset back (approx 128 or buffer mean)
             let dc = input[0].re / (self.window_size as f32);
             
             return (dc + pred_val).clamp(0.0, 255.0) as u8;
        }

        // Fallback
        *self.buffer.last().unwrap() as u8
    }
}
