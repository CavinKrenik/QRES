// QRES v4.0 Mixer: Hybrid Neural-Statistical Ensemble
// Features:
// 1. Weighted Ensemble of Predictors (Linear, Simple, iPEPS)
// 2. Dynamic AR(2) Auto-regressor for Waveforms
// 3. Variance-based Algorithm Switching (Stable -> AR2, Chaotic -> Ensemble)

const NUM_MODELS: usize = 4;

pub struct Mixer {
    // Ensemble Weights
    pub weights: [f32; NUM_MODELS],
    learning_rate: f32,

    // AR(2) Components
    ar_coeffs: [f32; 2],  // [phi1, phi2]
    history: [f32; 2],     // [x_{t-1}, x_{t-2}]
    ar_learning_rate: f32,
    
    // Variance Tracking (Signal Stability)
    running_mean: f32,
    running_var: f32,
    count: usize,
}

impl Mixer {
    pub fn new() -> Self {
        Mixer {
            weights: [0.25, 0.25, 0.25, 0.25],
            learning_rate: 0.01,
            // Initialize AR(2) to generic approximate sine/smooth values
            ar_coeffs: [0.7, -0.2], 
            history: [128.0, 128.0],
            ar_learning_rate: 0.05, // Faster adaptation for AR
            
            running_mean: 128.0,
            running_var: 1000.0, // Start high to prefer Ensemble initially
            count: 0,
        }
    }

    pub fn mix(&self, preds: &[u8; NUM_MODELS]) -> u8 {
        // 1. Calculate Ensemble Prediction (Stable)
        let mut ensemble_sum = 0.0;
        for (i, &pred) in preds.iter().enumerate() {
            ensemble_sum += (pred as f32) * self.weights[i];
        }

        // 2. Calculate AR(2) Prediction (Aggressive)
        // x_t = phi1*x_{t-1} + phi2*x_{t-2}
        let ar_pred = self.ar_coeffs[0] * self.history[0] + self.ar_coeffs[1] * self.history[1];

        // 3. Dynamic Selection based on Signal Stability
        let std = (self.running_var / (self.count.max(1) as f32)).sqrt();
        
        let prediction = if std < 45.0 {
            // Low Variance: Signal is predictable/clean (e.g., Sine Wave) -> Trust AR(2)
            // Blend: mostly AR, some Ensemble for correction
             0.8 * ar_pred + 0.2 * ensemble_sum
        } else {
            // High Variance: Chaotic or complex -> Trust Ensemble
            ensemble_sum
        };

        // Clamp
        if prediction > 255.0 { 255 } else if prediction < 0.0 { 0 } else { prediction.round() as u8 }
    }

    pub fn update(&mut self, actual: u8, preds: &[u8; NUM_MODELS]) {
        let y = actual as f32;

        // --- A. Update Statistics (Welford) ---
        self.count += 1;
        let delta = y - self.running_mean;
        self.running_mean += delta / self.count as f32;
        let delta2 = y - self.running_mean;
        self.running_var = self.running_var * 0.95 + (delta * delta2) * 0.05; // Decay old variance

        // --- B. Update Ensemble Weights ---
        let mut total_w = 0.0;
        for (i, &pred) in preds.iter().enumerate() {
            let error = (pred as f32 - y).abs();
            // Weight update rule: Penalize errors
            // w_i = w_i * (1 - rate * error_norm)
            // Normalized error [0, 1] approach
            let err_norm = (error / 255.0).clamp(0.0, 1.0);
            self.weights[i] = self.weights[i] * (1.0 - self.learning_rate * err_norm);
            
            // Add slight regeneration to prevent death
            self.weights[i] += 0.001; 
            total_w += self.weights[i];
        }
        // Normalize
        for i in 0..NUM_MODELS {
            self.weights[i] /= total_w;
        }

        // --- C. Update AR(2) Coefficients (LMS Filter) ---
        // Prediction using CURRENT history
        let ar_est = self.ar_coeffs[0] * self.history[0] + self.ar_coeffs[1] * self.history[1];
        let ar_error = y - ar_est;

        // LMS Update: w = w + mu * error * input
        // Normalize input power approx (128^2)
        const NORM: f32 = 1.0 / 10000.0; 
        
        // Update phi1
        self.ar_coeffs[0] += self.ar_learning_rate * ar_error * self.history[0] * NORM;
        // Update phi2
        self.ar_coeffs[1] += self.ar_learning_rate * ar_error * self.history[1] * NORM;
        
        // Stability Clamp (Prevent AR blowup)
        // For stable AR(2), coeffs usually in specific triangle, roughly |phi| < 2
        self.ar_coeffs[0] = self.ar_coeffs[0].clamp(-1.9, 1.9);
        self.ar_coeffs[1] = self.ar_coeffs[1].clamp(-0.99, 0.99);

        // Shift History
        self.history[1] = self.history[0];
        self.history[0] = y;
    }
}
