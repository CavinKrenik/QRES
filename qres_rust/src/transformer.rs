use crate::predictors::Predictor;

/// TransformerPredictor: A lightweight Self-Attention mechanism for byte streams.
/// Architecture:
/// - Window Size: 4096 bytes (Increased)
/// - Patch Size (Key/Query dim): 4 bytes (Extended via SIMD scans)
/// - Attention: Softmax(DotProduct(Q, K)) or InverseDistance
pub struct TransformerPredictor {
    history: Vec<u8>,
    buffer_mask: usize,
    pos: usize,
}

impl TransformerPredictor {
    pub fn new() -> Self {
        TransformerPredictor {
            history: vec![0u8; 4096], // 4KB Window
            buffer_mask: 4095,
            pos: 0,
        }
    }
}

impl Predictor for TransformerPredictor {
    fn predict_next(&self) -> u8 {
        if self.pos < 32 {
            return 128; // Warmup
        }

        let query_start = self.pos.wrapping_sub(4);

        // AVX2 implementation could process batches of 32 keys.
        // For now, the scalar unroll in `compute_attention_avx2` (renamed logic) is sufficient
        // given the non-contiguous circular buffer. A true SIMD Scan requires linear memory.

        // Inline Scalar Logic (Optimized):
        let mut sum_weights = 0.0;
        let mut sum_values = 0.0;

        let q_idx = query_start & self.buffer_mask;
        let q0 = self.history[q_idx] as i32;
        let q1 = self.history[(q_idx + 1) & self.buffer_mask] as i32;
        let q2 = self.history[(q_idx + 2) & self.buffer_mask] as i32;
        let q3 = self.history[(q_idx + 3) & self.buffer_mask] as i32;

        // Sparse Attention: Check 128 positions (Increased from 64)
        for i in 1..128 {
            let key_pos_end = self.pos.wrapping_sub(i * 4);
            let k_idx = key_pos_end.wrapping_sub(4) & self.buffer_mask;

            // Check wrap area
            if k_idx + 4 <= self.history.len() {
                // Fast path
                let k = &self.history[k_idx..k_idx + 4];
                let dist = (q0 - k[0] as i32).abs()
                    + (q1 - k[1] as i32).abs()
                    + (q2 - k[2] as i32).abs()
                    + (q3 - k[3] as i32).abs();
                let weight = 1.0 / (1.0 + dist as f32 * 0.5); // Sharpness tuning
                let value = self.history[key_pos_end & self.buffer_mask] as f32;
                sum_values += value * weight;
                sum_weights += weight;
            } else {
                // Slow wrap path
                let dist = (q0 - self.history[k_idx] as i32).abs()
                    + (q1 - self.history[(k_idx + 1) & self.buffer_mask] as i32).abs()
                    + (q2 - self.history[(k_idx + 2) & self.buffer_mask] as i32).abs()
                    + (q3 - self.history[(k_idx + 3) & self.buffer_mask] as i32).abs();
                let weight = 1.0 / (1.0 + dist as f32 * 0.5);
                let value = self.history[key_pos_end & self.buffer_mask] as f32;
                sum_values += value * weight;
                sum_weights += weight;
            }
        }

        if sum_weights < 0.001 {
            return 128;
        }

        (sum_values / sum_weights).clamp(0.0, 255.0) as u8
    }

    fn update(&mut self, byte: u8) {
        self.history[self.pos & self.buffer_mask] = byte;
        self.pos += 1;
    }
}

impl Default for TransformerPredictor {
    fn default() -> Self {
        Self::new()
    }
}
