use rustfft::{FftPlanner, num_complex::Complex};
use std::sync::Arc;

// QRES v4.1 Enhanced Spectral Predictor
// Improvements:
// - Larger window (2048) for better frequency resolution
// - Harmonic detection (2nd, 3rd harmonics)
// - Adaptive threshold based on signal strength
// Target: 60%+ compression on sine waves

pub struct SpectralPredictor {
    window_size: usize,
    buffer: Vec<f32>,
    planner: FftPlanner<f32>,
    // Adaptive threshold
    signal_strength_history: Vec<f32>,
}

impl SpectralPredictor {
    pub fn new(window_size: usize) -> Self {
        SpectralPredictor {
            window_size,
            buffer: Vec::with_capacity(window_size),
            planner: FftPlanner::new(),
            signal_strength_history: Vec::with_capacity(10),
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

        // 3. Find Dominant Frequencies (fundamental + harmonics)
        let mut frequencies = Vec::new();
        
        // Find fundamental (strongest frequency)
        let mut max_mag = 0.0;
        let mut fundamental_idx = 0;
        
        // Only search first half (Nyquist)
        for i in 1..(self.window_size / 2) {
            let mag = input[i].norm_sqr();
            if mag > max_mag {
                max_mag = mag;
                fundamental_idx = i;
            }
        }
        
        // Calculate adaptive threshold (10% of max)
        let threshold = max_mag * 0.1;
        
        // Track signal strength for adaptive behavior
        self.signal_strength_history.push(max_mag);
        if self.signal_strength_history.len() > 10 {
            self.signal_strength_history.remove(0);
        }
        
        // Add fundamental
        if max_mag > 100.0 { // Minimum threshold
            frequencies.push((fundamental_idx, input[fundamental_idx]));
            
            // Look for harmonics (2x, 3x fundamental frequency)
            for harmonic in 2..=3 {
                let harmonic_idx = fundamental_idx * harmonic;
                if harmonic_idx < self.window_size / 2 {
                    let harmonic_mag = input[harmonic_idx].norm_sqr();
                    if harmonic_mag > threshold {
                        frequencies.push((harmonic_idx, input[harmonic_idx]));
                    }
                }
            }
        }

        // 4. Predict using all detected frequencies
        if !frequencies.is_empty() {
            let dc = input[0].re / (self.window_size as f32);
            let mut pred_val = 0.0;
            
            for (freq_idx, bin) in frequencies {
                let ampl = bin.norm() / (self.window_size as f32) * 2.0;
                let phase = bin.arg();
                let freq = freq_idx as f32;
                
                // Project forward by 1 step
                let t = self.window_size as f32;
                let angle = (2.0 * std::f32::consts::PI * freq * t / (self.window_size as f32)) + phase;
                pred_val += ampl * angle.cos();
            }
            
            // Combine DC offset and predicted AC component
            let result = dc + pred_val;
            return result.clamp(0.0, 255.0) as u8;
        }

        // Fallback: Use last value
        *self.buffer.last().unwrap() as u8
    }
    
    /// Returns confidence in prediction (0.0 to 1.0)
    pub fn confidence(&self) -> f32 {
        if self.signal_strength_history.is_empty() {
            return 0.0;
        }
        
        let avg_strength: f32 = self.signal_strength_history.iter().sum::<f32>() 
                                / self.signal_strength_history.len() as f32;
        
        // Normalize to 0-1 range (assuming max strength ~1M)
        (avg_strength / 1_000_000.0).min(1.0)
    }
}
