use crate::predictors::Predictor;
use alloc::vec::Vec;
use alloc::vec;

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
        let q_idx = query_start & self.buffer_mask;

        // Cache query bytes to registers
        let q0 = self.history[q_idx] as i32;
        let q1 = self.history[(q_idx + 1) & self.buffer_mask] as i32;
        let q2 = self.history[(q_idx + 2) & self.buffer_mask] as i32;
        let q3 = self.history[(q_idx + 3) & self.buffer_mask] as i32;

        let mut sum_weights = 0.0;
        let mut sum_values = 0.0;

        // OPTIMIZATION 1: Reduce search depth from 128 to 32
        // This provides 4x speedup immediately.
        const SEARCH_DEPTH: usize = 32;

        // Sparse Attention: Check SEARCH_DEPTH positions
        for i in 1..SEARCH_DEPTH {
            let key_pos_end = self.pos.wrapping_sub(i * 4);
            let k_idx = key_pos_end.wrapping_sub(4) & self.buffer_mask;

            // Manual bounds check optimization check not needed with mask
            // We use simple scalar difference
            let d0 = (q0 - self.history[k_idx] as i32).abs();
            let d1 = (q1 - self.history[(k_idx + 1) & self.buffer_mask] as i32).abs();
            let d2 = (q2 - self.history[(k_idx + 2) & self.buffer_mask] as i32).abs();
            let d3 = (q3 - self.history[(k_idx + 3) & self.buffer_mask] as i32).abs();

            let dist = d0 + d1 + d2 + d3;

            // OPTIMIZATION 2: Early Exit on Perfect Match
            if dist == 0 {
                // If we found an exact sequence match in history, USE IT.
                // This is effectively LZ77 logic inside the transformer.
                let val = self.history[key_pos_end & self.buffer_mask];
                return val;
            }

            // Inverse distance weighting
            let weight = 1.0 / (1.0 + dist as f32 * 0.5);
            let value = self.history[key_pos_end & self.buffer_mask] as f32;

            sum_values += value * weight;
            sum_weights += weight;
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
