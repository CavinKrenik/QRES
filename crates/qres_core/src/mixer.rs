// QRES v18.0 Mixer: Deterministic Q16.16 Fixed-Point Implementation
// Replaces previous float/SIMD version.
// Strictly NO f32 usage.

pub const NUM_MODELS: usize = 6;
const Q16_ONE: i32 = 1 << 16;
const Q16_HALF: i32 = 1 << 15;

/// Q16.16 Multiplication: (a * b) >> 16
#[inline(always)]
fn mul_q16(a: i32, b: i32) -> i32 {
    ((a as i64 * b as i64) >> 16) as i32
}

/// Q16.16 Division: (a << 16) / b
#[inline(always)]
fn div_q16(a: i32, b: i32) -> i32 {
    if b == 0 {
        return 0;
    }
    (((a as i64) << 16) / (b as i64)) as i32
}

pub struct Mixer {
    // Weights are Q16.16
    pub weights: [i32; 8],
    learning_rate: i32,

    // AR(2) Components (Q16.16)
    ar_coeffs: [i32; 2],
    history: [i32; 2],
    ar_learning_rate: i32,
    ar_velocities: [i32; 2],

    // Variance Tracking (Q16.16)
    running_mean: i32,
    running_var: i32,
    count: i32,

    // Lock-on Detection
    current_winner: usize,
    win_streak: usize,

    // Phase 2: FedProx
    global_weights: Option<[i32; 8]>,
}

impl Mixer {
    /// Create a new Mixer with deterministic weights.
    pub fn new(init: Option<&[i32]>, global: Option<&[i32]>) -> Self {
        // Defaults: 0.4, 0.2, 0.1, 0.1, 0.1, 0.1 converted to Q16.16
        // 0.1 * 65536 = 6553.6 -> 6554
        // 0.2 * 65536 = 13107
        // 0.4 * 65536 = 26214
        let default_w = [26214, 13107, 6554, 6554, 6554, 6554, 0, 0];

        let weights = if let Some(w) = init {
            let mut arr = [0; 8];
            for (i, &val) in w.iter().take(8).enumerate() {
                arr[i] = val;
            }
            arr
        } else {
            default_w
        };

        let global_weights = global.map(|g| {
            let mut arr = [0; 8];
            for (i, &val) in g.iter().take(8).enumerate() {
                arr[i] = val;
            }
            arr
        });

        // AR Constants: 0.05 -> 3277
        // Mean: 128.0 -> 128 * 65536 = 8388608
        // Var: 1000.0 -> 1000 * 65536 = 65536000
        Mixer {
            weights,
            learning_rate: 655, // ~0.01
            ar_coeffs: [Q16_ONE, 0],
            history: [0, 0],
            ar_learning_rate: 3277, // ~0.05
            ar_velocities: [0, 0],
            running_mean: 8388608,
            running_var: 65536000,
            count: 0,
            current_winner: 0,
            win_streak: 0,
            global_weights,
        }
    }

    pub fn mix(&self, preds: &[u8; NUM_MODELS]) -> u8 {
        // 1. Calculate Ensemble Prediction
        // Inputs are u8, convert to Q16 (x << 16) before multiply
        // But weights are Q16. So weight * (pred << 16) >> 16 == weight * pred.
        // We can just accumulate weight * pred then result is Q16.
        let mut ensemble_sum: i32 = 0;
        for (i, &pred) in preds.iter().enumerate().take(NUM_MODELS) {
            ensemble_sum += mul_q16(self.weights[i], (pred as i32) << 16);
        }

        // 2. Calculate AR(2) Prediction
        let term1 = mul_q16(self.ar_coeffs[0], self.history[0]);
        let term2 = mul_q16(self.ar_coeffs[1], self.history[1]);
        let ar_pred = term1 + term2;

        // 3. Dynamic Selection
        // Variance threshold: 45.0^2 = 2025.0
        // Check running_var < 2025.0 * 65536
        // 2025 * 65536 = 132710400
        const VAR_THRESH: i32 = 132710400;

        // Note: Running var is scaled, effectively, so we compare directly.
        // Actually, std = sqrt(var), check std < 45 is same as var < 2025.
        // Logic: if std < 45.0 { 0.6 * ar + 0.4 * ensemble } else { ensemble }
        // 0.6 -> 39322, 0.4 -> 26214

        let prediction = if self.win_streak > 32 {
            // Lock-On
            (preds[self.current_winner] as i32) << 16
        } else if self.running_var < VAR_THRESH {
            let p1 = mul_q16(39322, ar_pred);
            let p2 = mul_q16(26214, ensemble_sum);
            p1 + p2
        } else {
            ensemble_sum
        };

        // Round and clamp
        // Add 0.5 (half Q16) for rounding
        let rounded = prediction + Q16_HALF;
        let byte_val = rounded >> 16;

        if byte_val < 0 {
            0
        } else if byte_val > 255 {
            255
        } else {
            byte_val as u8
        }
    }

    pub fn update_lazy(
        &mut self,
        batch_size: usize,
        sample_actual: u8,
        sample_preds: &[u8; NUM_MODELS],
    ) {
        let y = (sample_actual as i32) << 16; // Q16

        // 1. Update Statistics
        self.count += batch_size as i32;
        let delta = y - self.running_mean;
        // Approximation: self.running_mean += delta / 100.0
        // 1/100 ~ 655 (0.01)
        self.running_mean += mul_q16(delta, 655);

        let delta2 = y - self.running_mean;
        // running_var = var * 0.95 + (delta * delta2) * 0.05
        // 0.95 -> 62259, 0.05 -> 3277
        // delta * delta2 can be large, use i64 for intermediate mul
        let sq_term = mul_q16(delta, delta2);
        self.running_var = mul_q16(self.running_var, 62259) + mul_q16(sq_term, 3277);

        // 2. Lock-On
        let mut best_idx = 0;
        let mut min_err = i32::MAX;
        for (i, &p) in sample_preds.iter().enumerate().take(NUM_MODELS) {
            let p_q16 = (p as i32) << 16;
            let err = (p_q16 - y).abs();
            if err < min_err {
                min_err = err;
                best_idx = i;
            }
        }

        if best_idx == self.current_winner {
            self.win_streak += batch_size;
        } else {
            self.current_winner = best_idx;
            self.win_streak = 0;
        }

        // 3. Learning Rate Logic
        // Threshold check: var > 40.0^2 = 1600.0 -> 104857600
        const LR_THRESH: i32 = 104857600;
        let base_lr = if self.running_var > LR_THRESH {
            3277
        } else {
            328
        }; // 0.05 vs 0.005

        self.learning_rate = if self.win_streak > 32 {
            // 2.5x base_lr
            (base_lr * 5) / 2
        } else {
            base_lr
        };

        // 4. Update Weights (LMS)
        self.update_weights(y, sample_preds);

        // 5. AR(2) Update
        let term1 = mul_q16(self.ar_coeffs[0], self.history[0]);
        let term2 = mul_q16(self.ar_coeffs[1], self.history[1]);
        let ar_est = term1 + term2;
        let ar_error = y - ar_est;

        // NORM = 1/10000 = 0.0001 -> ~ 7 in Q16
        const NORM: i32 = 7;
        // Momentum 0.9 -> 58982
        const MOMENTUM: i32 = 58982;

        let grad0 = mul_q16(mul_q16(ar_error, self.history[0]), NORM);
        let grad1 = mul_q16(mul_q16(ar_error, self.history[1]), NORM);

        self.ar_velocities[0] =
            mul_q16(MOMENTUM, self.ar_velocities[0]) + mul_q16(self.ar_learning_rate, grad0);
        self.ar_velocities[1] =
            mul_q16(MOMENTUM, self.ar_velocities[1]) + mul_q16(self.ar_learning_rate, grad1);

        self.ar_coeffs[0] += self.ar_velocities[0];
        self.ar_coeffs[1] += self.ar_velocities[1];

        // Clamp coefficients: 1.9 -> 124518, 0.99 -> 64880
        self.ar_coeffs[0] = self.ar_coeffs[0].clamp(-124518, 124518);
        self.ar_coeffs[1] = self.ar_coeffs[1].clamp(-64880, 64880);

        self.history[1] = self.history[0];
        self.history[0] = y;
    }

    fn update_weights(&mut self, y: i32, preds: &[u8; NUM_MODELS]) {
        for (i, &pred) in preds.iter().enumerate().take(NUM_MODELS) {
            let p_q16 = (pred as i32) << 16;
            let diff = p_q16 - y;
            let error = diff.abs();

            // Normalize error: err / 255.0.
            // 255.0 in Q16 is 16711680.
            // Better: just divide by 255 using integer div if we want standard norm.
            // Or multiply by 1/255 (approx 257 in Q16).
            let err_norm = mul_q16(error, 257).clamp(0, Q16_ONE);

            // Factor = 1.0 - lr * err_norm
            let penalty = mul_q16(self.learning_rate, err_norm);
            let factor = Q16_ONE - penalty;

            self.weights[i] = mul_q16(self.weights[i], factor);
        }

        // FedProx
        if let Some(global) = self.global_weights {
            // mu = 0.001 -> 66
            const MU: i32 = 66;
            for (i, &g_val) in global.iter().enumerate() {
                let diff_g = g_val - self.weights[i];
                self.weights[i] += mul_q16(diff_g, MU);
            }
        }

        // Regeneration: + 0.001 (66)
        for i in 0..NUM_MODELS {
            self.weights[i] += 66;
        }

        // Normalize
        let mut sum: i32 = 0;
        for i in 0..NUM_MODELS {
            sum += self.weights[i];
        }

        if sum > 10 {
            // Epsilon check
            for i in 0..NUM_MODELS {
                self.weights[i] = div_q16(self.weights[i], sum);
            }
        }
    }
}
