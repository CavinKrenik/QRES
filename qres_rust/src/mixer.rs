// QRES v5.0 Mixer: Hybrid Neural-Statistical Ensemble (SIMD)
// Features:
// 1. Weighted Ensemble of Predictors (Linear, Simple, Graph, Spectral, LzMatch)
// 2. Dynamic AR(2) Auto-regressor for Waveforms
// 3. Variance-based Algorithm Switching (Stable -> AR2, Chaotic -> Ensemble)

use std::arch::x86_64::*;

pub const NUM_MODELS: usize = 5; // Added LzMatch

pub struct Mixer {
    // Ensemble Weights (Aligned __m256 for SIMD)
    // [Linear, Simple, Graph, Spectral, LzMatch, 0, 0, 0]
    pub weights: __m256,
    learning_rate: f32,

    // AR(2) Components (Recursive IIR - Scalar)
    ar_coeffs: [f32; 2],  // [phi1, phi2]
    history: [f32; 2],     // [x_{t-1}, x_{t-2}]
    ar_learning_rate: f32,
    
    // Variance Tracking
    running_mean: f32,
    running_var: f32,
    count: usize,
}

impl Mixer {
    pub fn new() -> Self {
        // Initial weights: Uniform for 5 models (0.2 each)
        // Pad with zeros for 8 lanes
        let weights = _mm256_set_ps(0.0, 0.0, 0.05, 0.05, 0.05, 0.05, 0.1, 0.2, 0.5);

        Mixer {
            weights,
            learning_rate: 0.01,
            // Initialize AR(2)
            ar_coeffs: [0.7, -0.2], 
            history: [128.0, 128.0],
            ar_learning_rate: 0.05,
            
            running_mean: 128.0,
            running_var: 1000.0, 
            count: 0,
        }
    }

    pub fn mix(&self, preds: &[u8; NUM_MODELS]) -> u8 {
        // 1. Calculate Ensemble Prediction (SIMD)
        // Load predictions into f32 array
        let mut p_arr = [0.0f32; 8];
        for i in 0..NUM_MODELS {
            p_arr[i] = preds[i] as f32;
        }
        let p_simd = _mm256_loadu_ps(p_arr.as_ptr());

        // Dot Product: weights * preds
        let weighted = _mm256_mul_ps(self.weights, p_simd);
        let h1 = _mm256_hadd_ps(weighted, weighted);
        let h2 = _mm256_hadd_ps(h1, h1);
        let ensemble_sum = _mm256_cvtss_f32(h2);

        // 2. Calculate AR(2) Prediction (Scalar)
        // x_t = phi1*x_{t-1} + phi2*x_{t-2}
        let ar_pred = self.ar_coeffs[0] * self.history[0] + self.ar_coeffs[1] * self.history[1];

        // 3. Dynamic Selection
        let std = (self.running_var / (self.count.max(1) as f32)).sqrt();
        
        // Critical: The "Ratio Killer" needs to choose efficiently.
        // If std is low, AR(2) is great for waveforms.
        // If std is high, Ensemble (especially LZ/Graph) logic wins.
        let prediction = if std < 45.0 {
            0.8 * ar_pred + 0.2 * ensemble_sum
        } else {
            ensemble_sum
        };

        if prediction > 255.0 { 255 } else if prediction < 0.0 { 0 } else { prediction.round() as u8 }
    }

    pub fn mix_batch(&self, preds_batch: &[[u8; NUM_MODELS]; 8]) -> [u8; 8] {
        let mut results = [0u8; 8];
        
        for i in 0..8 {
            // For each byte in batch, compute mix
            let preds = &preds_batch[i];
            
            // 1. Calculate Ensemble Prediction (SIMD)
            let mut p_arr = [0.0f32; 8];
            for j in 0..NUM_MODELS {
                p_arr[j] = preds[j] as f32;
            }
            let p_simd = Simd::from_array(p_arr);

            // Dot Product: weights * preds
            let weighted = self.weights * p_simd;
            let ensemble_sum = weighted.reduce_sum();

            // 2. Calculate AR(2) Prediction (Scalar, per byte? For batch, perhaps average or something, but for simplicity, use same AR for all)
            let ar_pred = self.ar_coeffs[0] * self.history[0] + self.ar_coeffs[1] * self.history[1];

            // 3. Dynamic Selection
            let std = (self.running_var / (self.count.max(1) as f32)).sqrt();
            
            let prediction = if std < 45.0 {
                0.8 * ar_pred + 0.2 * ensemble_sum
            } else {
                ensemble_sum
            };

            results[i] = if prediction > 255.0 { 255 } else if prediction < 0.0 { 0 } else { prediction.round() as u8 };
        }
        
        results
    }

    pub fn update(&mut self, actual: u8, preds: &[u8; NUM_MODELS]) {
        let y = actual as f32;

        // --- A. Update Statistics ---
        self.count += 1;
        let delta = y - self.running_mean;
        self.running_mean += delta / self.count as f32;
        let delta2 = y - self.running_mean;
        self.running_var = self.running_var * 0.95 + (delta * delta2) * 0.05;

        // --- B. Update Ensemble Weights (SIMD) ---
        // Load preds
        let mut p_arr = [0.0f32; 8];
        for i in 0..NUM_MODELS {
            p_arr[i] = preds[i] as f32;
        }
        let p_simd = _mm256_loadu_ps(p_arr.as_ptr());
        let y_simd = _mm256_set1_ps(y);

        // Error = |pred - y|
        let diff = _mm256_sub_ps(p_simd, y_simd);
        let error = _mm256_abs_ps(diff);

        // Weight Update: w = w * (1 - lr * err_norm)
        let err_norm = _mm256_div_ps(error, _mm256_set1_ps(255.0));
        let err_norm = _mm256_min_ps(_mm256_max_ps(err_norm, _mm256_set1_ps(0.0)), _mm256_set1_ps(1.0));
        let factor = _mm256_sub_ps(_mm256_set1_ps(1.0), _mm256_mul_ps(_mm256_set1_ps(self.learning_rate), err_norm));
        
        self.weights = _mm256_mul_ps(self.weights, factor);

        // Regeneration (prevent death)
        self.weights = _mm256_add_ps(self.weights, _mm256_set1_ps(0.001));

        // Verify/Mask padded lanes (indices 5,6,7 should stay 0 or irrelevant, but regeneration adds 0.001)
        // Let's create a mask.
        let mask = _mm256_set_ps(0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0);
        self.weights = _mm256_mul_ps(self.weights, mask);

        // Normalize
        let h1 = _mm256_hadd_ps(self.weights, self.weights);
        let h2 = _mm256_hadd_ps(h1, h1);
        let sum_w = _mm256_cvtss_f32(h2);
        if sum_w > 0.0001 {
             self.weights = _mm256_div_ps(self.weights, _mm256_set1_ps(sum_w));
        }

        // --- C. Update AR(2) Coefficients (Scalar LMS) ---
        let ar_est = self.ar_coeffs[0] * self.history[0] + self.ar_coeffs[1] * self.history[1];
        let ar_error = y - ar_est;
        const NORM: f32 = 1.0 / 10000.0; 
        
        // Update phi
        self.ar_coeffs[0] += self.ar_learning_rate * ar_error * self.history[0] * NORM;
        self.ar_coeffs[1] += self.ar_learning_rate * ar_error * self.history[1] * NORM;
        
        // Clamp (Scalar)
        self.ar_coeffs[0] = self.ar_coeffs[0].clamp(-1.9, 1.9);
        self.ar_coeffs[1] = self.ar_coeffs[1].clamp(-0.99, 0.99);

        // Shift History
        self.history[1] = self.history[0];
        self.history[0] = y;
    }

    pub fn update_batch(&mut self, actuals: &[u8; 8], preds_batch: &[[u8; NUM_MODELS]; 8]) {
        for i in 0..8 {
            let y = actuals[i] as f32;
            let preds = &preds_batch[i];

            // --- A. Update Statistics ---
            self.count += 1;
            let delta = y - self.running_mean;
            self.running_mean += delta / self.count as f32;
            let delta2 = y - self.running_mean;
            self.running_var = self.running_var * 0.95 + (delta * delta2) * 0.05;

            // --- B. Update Ensemble Weights (SIMD) ---
            let mut p_arr = [0.0f32; 8];
            for j in 0..NUM_MODELS {
                p_arr[j] = preds[j] as f32;
            }
            let p_simd = Simd::from_array(p_arr);
            let y_simd = Simd::splat(y);

            // Error = |pred - y|
            let diff = p_simd - y_simd;
            let error = diff.abs();

            // Weight Update: w = w * (1 - lr * err_norm)
            let err_norm = (error / Simd::splat(255.0)).simd_clamp(Simd::splat(0.0), Simd::splat(1.0));
            let factor = Simd::splat(1.0) - (Simd::splat(self.learning_rate) * err_norm);
            
            self.weights = self.weights * factor;

            // Regeneration (prevent death)
            self.weights += Simd::splat(0.001);

            // Mask padded lanes
            let mask = Simd::from_array([1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0]);
            self.weights = self.weights * mask;

            // Normalize
            let sum_w = self.weights.reduce_sum();
            if sum_w > 0.0001 {
                 self.weights = self.weights / Simd::splat(sum_w);
            }

            // --- C. Update AR(2) Coefficients (Scalar LMS) ---
            let ar_est = self.ar_coeffs[0] * self.history[0] + self.ar_coeffs[1] * self.history[1];
            let ar_error = y - ar_est;
            const NORM: f32 = 1.0 / 10000.0; 
            
            self.ar_coeffs[0] += self.ar_learning_rate * ar_error * self.history[0] * NORM;
            self.ar_coeffs[1] += self.ar_learning_rate * ar_error * self.history[1] * NORM;
            
            self.ar_coeffs[0] = self.ar_coeffs[0].clamp(-1.9, 1.9);
            self.ar_coeffs[1] = self.ar_coeffs[1].clamp(-0.99, 0.99);

            // Shift History
            self.history[1] = self.history[0];
            self.history[0] = y;
        }
    }
}
