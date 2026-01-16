use crate::predictors::Predictor;
use alloc::vec;
use alloc::vec::Vec;

const WINDOW_SIZE: usize = 4096; // bytes retained in the circular buffer
const BUFFER_MASK: usize = WINDOW_SIZE - 1; // assumes WINDOW_SIZE is power-of-two
const SEARCH_DEPTH: usize = 32; // how many prior patches to scan
const FIXED_ONE: i64 = 1 << 16; // Q16.16 unity

/// TransformerPredictor: deterministic fixed-point (Q16.16) self-attention over a byte stream.
pub struct TransformerPredictor {
    history: Vec<u8>,
    buffer_mask: usize,
    pos: usize,
}

impl TransformerPredictor {
    pub fn new() -> Self {
        TransformerPredictor {
            history: vec![0u8; WINDOW_SIZE],
            buffer_mask: BUFFER_MASK,
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

        let q0 = self.history[q_idx] as i32;
        let q1 = self.history[(q_idx + 1) & self.buffer_mask] as i32;
        let q2 = self.history[(q_idx + 2) & self.buffer_mask] as i32;
        let q3 = self.history[(q_idx + 3) & self.buffer_mask] as i32;

        let mut sum_weights: i64 = 0;
        let mut sum_values: i64 = 0;

        for i in 1..SEARCH_DEPTH {
            let key_pos_end = self.pos.wrapping_sub(i * 4);
            let k_idx = key_pos_end.wrapping_sub(4) & self.buffer_mask;

            let d0 = (q0 - self.history[k_idx] as i32).abs();
            let d1 = (q1 - self.history[(k_idx + 1) & self.buffer_mask] as i32).abs();
            let d2 = (q2 - self.history[(k_idx + 2) & self.buffer_mask] as i32).abs();
            let d3 = (q3 - self.history[(k_idx + 3) & self.buffer_mask] as i32).abs();

            let dist = (d0 + d1 + d2 + d3) as i64;

            if dist == 0 {
                return self.history[key_pos_end & self.buffer_mask];
            }

            let weight = (FIXED_ONE * 2) / (2 + dist);
            let value = self.history[key_pos_end & self.buffer_mask] as i64;

            sum_values += weight * value;
            sum_weights += weight;
        }

        if sum_weights == 0 {
            return 128;
        }

        let prediction = (sum_values + (sum_weights / 2)) / sum_weights;

        if prediction < 0 {
            0
        } else if prediction > 255 {
            255
        } else {
            prediction as u8
        }
    }

    fn update(&mut self, byte: u8) {
        self.history[self.pos & self.buffer_mask] = byte;
        self.pos += 1;
    }

    fn reset(&mut self) {
        self.history.fill(0);
        self.pos = 0;
    }
}

impl Default for TransformerPredictor {
    fn default() -> Self {
        Self::new()
    }
}
