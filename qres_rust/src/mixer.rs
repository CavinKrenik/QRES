// QRES v5.0 Mixer: Hybrid Neural-Statistical Ensemble (SIMD/Scalar)
// Features:
// 1. Weighted Ensemble of Predictors (Linear, Simple, Graph, Spectral, LzMatch)
// 2. Dynamic AR(2) Auto-regressor for Waveforms
// 3. Variance-based Algorithm Switching (Stable -> AR2, Chaotic -> Ensemble)

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

pub const NUM_MODELS: usize = 5;

// Define storage type based on architecture
#[cfg(target_arch = "x86_64")]
type WeightStorage = __m256;

#[cfg(not(target_arch = "x86_64"))]
type WeightStorage = [f32; 8];

pub struct Mixer {
    // Ensemble Weights (Aligned __m256 for SIMD, [f32;8] for Scalar)
    pub weights: WeightStorage,
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
    #[cfg(target_arch = "x86_64")]
    pub fn new() -> Self {
        // Initial weights: Uniform for 5 models (0.2 each)
        // Pad with zeros for 8 lanes
        let weights = unsafe { _mm256_set_ps(0.0, 0.05, 0.05, 0.05, 0.05, 0.1, 0.2, 0.5) };

        Mixer {
            weights,
            learning_rate: 0.01,
            ar_coeffs: [0.7, -0.2], 
            history: [128.0, 128.0],
            ar_learning_rate: 0.05,
            running_mean: 128.0,
            running_var: 1000.0, 
            count: 0,
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    pub fn new() -> Self {
        // Initial weights matching SIMD layout
        // _mm256_set_ps args are e7, e6, e5, e4, e3, e2, e1, e0 (Little Endian? Careful with order)
        // set_ps(e7...e0) -> [e0, e1, ..., e7] in memory usually
        // Actually set_ps puts first arg in high bits.
        // Let's just use explicit array: [0.5, 0.2, 0.1, 0.05, 0.05, 0.05, 0.05, 0.0] 
        // Index 0..4 are models.
        // Model 0 (Linear) gets 0.5? No, let's look at previous _mm256_set_ps(0.0... 0.5).
        // The previous code had 0.5 at the end (e0). So index 0.
        // models: 5. 
        
        Mixer {
            weights: [0.5, 0.2, 0.1, 0.05, 0.05, 0.05, 0.05, 0.0], 
            learning_rate: 0.01,
            ar_coeffs: [0.7, -0.2], 
            history: [128.0, 128.0],
            ar_learning_rate: 0.05,
            running_mean: 128.0,
            running_var: 1000.0, 
            count: 0,
        }
    }

    pub fn mix(&self, preds: &[u8; NUM_MODELS]) -> u8 {
        // 1. Calculate Ensemble Prediction
        let ensemble_sum = self.compute_ensemble_score(preds);

        // 2. Calculate AR(2) Prediction (Scalar)
        let ar_pred = self.ar_coeffs[0] * self.history[0] + self.ar_coeffs[1] * self.history[1];

        // 3. Dynamic Selection
        let std = (self.running_var / (self.count.max(1) as f32)).sqrt();
        
        let prediction = if std < 45.0 {
            0.8 * ar_pred + 0.2 * ensemble_sum
        } else {
            ensemble_sum
        };

        if prediction > 255.0 { 255 } else if prediction < 0.0 { 0 } else { prediction.round() as u8 }
    }

    #[cfg(target_arch = "x86_64")]
    fn compute_ensemble_score(&self, preds: &[u8; NUM_MODELS]) -> f32 {
        let mut p_arr = [0.0f32; 8];
        for i in 0..NUM_MODELS {
            p_arr[i] = preds[i] as f32;
        }
        let p_simd = unsafe { _mm256_loadu_ps(p_arr.as_ptr()) };

        let weighted = unsafe { _mm256_mul_ps(self.weights, p_simd) };
        let h1 = unsafe { _mm256_hadd_ps(weighted, weighted) };
        let h2 = unsafe { _mm256_hadd_ps(h1, h1) };
        unsafe { _mm256_cvtss_f32(h2) }
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn compute_ensemble_score(&self, preds: &[u8; NUM_MODELS]) -> f32 {
        let mut sum = 0.0;
        for i in 0..NUM_MODELS {
            sum += self.weights[i] * (preds[i] as f32);
        }
        sum
    }

    // Batch and Update logic follow...
    pub fn update(&mut self, actual: u8, preds: &[u8; NUM_MODELS]) {
        let y = actual as f32;

        // A. Update Statistics
        self.count += 1;
        let delta = y - self.running_mean;
        self.running_mean += delta / self.count as f32;
        let delta2 = y - self.running_mean;
        self.running_var = self.running_var * 0.95 + (delta * delta2) * 0.05;

        // B. Update Ensemble Weights
        self.update_weights(y, preds);

        // C. Update AR(2)
        let ar_est = self.ar_coeffs[0] * self.history[0] + self.ar_coeffs[1] * self.history[1];
        let ar_error = y - ar_est;
        const NORM: f32 = 1.0 / 10000.0; 
        
        self.ar_coeffs[0] += self.ar_learning_rate * ar_error * self.history[0] * NORM;
        self.ar_coeffs[1] += self.ar_learning_rate * ar_error * self.history[1] * NORM;
        
        self.ar_coeffs[0] = self.ar_coeffs[0].clamp(-1.9, 1.9);
        self.ar_coeffs[1] = self.ar_coeffs[1].clamp(-0.99, 0.99);

        self.history[1] = self.history[0];
        self.history[0] = y;
    }

    #[cfg(target_arch = "x86_64")]
    fn update_weights(&mut self, y: f32, preds: &[u8; NUM_MODELS]) {
        let mut p_arr = [0.0f32; 8];
        for i in 0..NUM_MODELS {
            p_arr[i] = preds[i] as f32;
        }
        let p_simd = unsafe { _mm256_loadu_ps(p_arr.as_ptr()) };
        let y_simd = unsafe { _mm256_set1_ps(y) };

        let diff = unsafe { _mm256_sub_ps(p_simd, y_simd) };
        let error = unsafe { _mm256_max_ps(diff, _mm256_sub_ps(_mm256_setzero_ps(), diff)) };

        let err_norm = unsafe { _mm256_div_ps(error, _mm256_set1_ps(255.0)) };
        let err_norm = unsafe { _mm256_min_ps(_mm256_max_ps(err_norm, _mm256_set1_ps(0.0)), _mm256_set1_ps(1.0)) };
        let factor = unsafe { _mm256_sub_ps(_mm256_set1_ps(1.0), _mm256_mul_ps(_mm256_set1_ps(self.learning_rate), err_norm)) };
        
        self.weights = unsafe { _mm256_mul_ps(self.weights, factor) };
        self.weights = unsafe { _mm256_add_ps(self.weights, _mm256_set1_ps(0.001)) };

        let mask = unsafe { _mm256_set_ps(0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0) };
        self.weights = unsafe { _mm256_mul_ps(self.weights, mask) };

        let h1 = unsafe { _mm256_hadd_ps(self.weights, self.weights) };
        let h2 = unsafe { _mm256_hadd_ps(h1, h1) };
        let sum_w = unsafe { _mm256_cvtss_f32(h2) };
        if sum_w > 0.0001 {
             self.weights = unsafe { _mm256_div_ps(self.weights, _mm256_set1_ps(sum_w)) };
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn update_weights(&mut self, y: f32, preds: &[u8; NUM_MODELS]) {
        let mut sum_w = 0.0;
        for i in 0..NUM_MODELS {
            let p_val = preds[i] as f32;
            let err = (p_val - y).abs();
            let err_norm = (err / 255.0).clamp(0.0, 1.0);
            let factor = 1.0 - (self.learning_rate * err_norm);
            
            self.weights[i] *= factor;
            self.weights[i] += 0.001; // Regen
            sum_w += self.weights[i];
        }

        if sum_w > 0.0001 {
            for w in &mut self.weights {
                *w /= sum_w;
            }
        }
    }
}
