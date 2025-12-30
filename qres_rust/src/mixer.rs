// use rayon::prelude::*;

// We mix 3 signals: Linear(0), LSTM(1), Tensor(2)
const NUM_MODELS: usize = 3;

pub struct Mixer {
    pub weights: [f32; NUM_MODELS],
    learning_rate: f32,
    ar_coeffs: [f32; 2],  // AR(2) coefficients: [phi1, phi2]
    prev_mixed: [f32; 2], // Previous two mixed predictions
}

impl Mixer {
    pub fn new() -> Self {
        Mixer {
            weights: [0.33, 0.33, 0.34], // Start equal
            learning_rate: 0.005,        // Fast adaptation
            ar_coeffs: [0.5, 0.3],       // Initial AR coefficients
            prev_mixed: [128.0, 128.0],  // Start with mid-range
        }
    }

    // Combine predictions into one byte
    pub fn mix(&self, preds: &[u8; NUM_MODELS]) -> u8 {
        let mut sum = 0.0;
        for (i, &pred) in preds.iter().enumerate() {
            sum += (pred as f32) * self.weights[i];
        }

        // Add AR(2) prediction
        let ar_pred =
            self.ar_coeffs[0] * self.prev_mixed[0] + self.ar_coeffs[1] * self.prev_mixed[1];
        sum += ar_pred * 0.1; // Weight the AR component

        // Clamp and Round
        let out = sum.round();
        if out > 255.0 {
            255
        } else if out < 0.0 {
            0
        } else {
            out as u8
        }
    }

    // Online Learning: Gradient Descent on the Stream
    // If Model A was closer to 'actual' than Model B, boost Weight A.
    pub fn update(&mut self, actual: u8, preds: &[u8; NUM_MODELS]) {
        let y = actual as f32;

        // Calculate current prediction again (forward pass)
        let mut y_hat = 0.0;
        for (i, &pred) in preds.iter().enumerate() {
            y_hat += (pred as f32) * self.weights[i];
        }

        // Add AR component
        let ar_pred =
            self.ar_coeffs[0] * self.prev_mixed[0] + self.ar_coeffs[1] * self.prev_mixed[1];
        y_hat += ar_pred * 0.1;

        // Update prev_mixed
        self.prev_mixed[1] = self.prev_mixed[0];
        self.prev_mixed[0] = y_hat;

        // Error
        let _error = y - y_hat;

        // Update weights: w_i = w_i + alpha * error * p_i
        // Normalized update to prevent explosion
        let mut total_w = 0.0;
        for (i, &pred) in preds.iter().enumerate() {
            // Directional update based on who was right
            let pred_error = (pred as f32) - y;
            // If pred_error is small, weight should increase.
            // Simplified rule: Generalized Logistic Weighting
            let accuracy = 1.0 / (1.0 + pred_error.abs());

            self.weights[i] = self.weights[i] * 0.995 + (accuracy * self.learning_rate);
            total_w += self.weights[i];
        }

        // Renormalize
        if total_w > 0.0 {
            for i in 0..NUM_MODELS {
                self.weights[i] /= total_w;
            }
        }
    }
}
