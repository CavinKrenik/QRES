use crate::predictors::Predictor;

/// TransformerPredictor: A lightweight Self-Attention mechanism for byte streams.
/// Unlike LZ (hard match), this calculates a weighted average of "Values" (next bytes)
/// based on the similarity of "Keys" (context patches) to the current "Query" (current context).
///
/// Architecture:
/// - Window Size: 512 bytes
/// - Patch Size (Key/Query dim): 4 bytes
/// - Attention: Softmax(DotProduct(Q, K)) or InverseDistance
pub struct TransformerPredictor {
    history: Vec<u8>,
    buffer_mask: usize,
    pos: usize,
}

impl TransformerPredictor {
    pub fn new() -> Self {
        TransformerPredictor {
            history: vec![0u8; 1024], // Power of 2 for easy masking
            buffer_mask: 1023,
            pos: 0,
        }
    }

    fn similarity(&self, p1: usize, p2: usize) -> u32 {
        // L1 distance between patch at p1 and patch at p2 (length 4)
        let mut dist = 0;
        // Manual unroll for speed
        dist += (self.history[p1 & self.buffer_mask] as i32 - self.history[p2 & self.buffer_mask] as i32).abs();
        dist += (self.history[(p1 + 1) & self.buffer_mask] as i32 - self.history[(p2 + 1) & self.buffer_mask] as i32).abs();
        dist += (self.history[(p1 + 2) & self.buffer_mask] as i32 - self.history[(p2 + 2) & self.buffer_mask] as i32).abs();
        dist += (self.history[(p1 + 3) & self.buffer_mask] as i32 - self.history[(p2 + 3) & self.buffer_mask] as i32).abs();
        
        dist as u32
    }
}

impl Predictor for TransformerPredictor {
    fn predict_next(&self) -> u8 {
        if self.pos < 32 {
            return 128; // Warmup
        }

        // Query: Recent context (ending at pos-1)
        // Key starts at: pos - 4
        let query_start = self.pos.wrapping_sub(4);

        let mut sum_weights = 0.0;
        let mut sum_values = 0.0;

        // Sparse Attention: Check 16 positions in history with stride
        // We look back up to 256 bytes
        for i in 1..64 {
            let key_pos_end = self.pos.wrapping_sub(i * 4); // Stride 4
            let key_start = key_pos_end.wrapping_sub(4);

            let dist = self.similarity(query_start, key_start);
            
            // "Temp" determines sharpness of attention.
            // dist=0 -> weight=1.0
            // dist=10 -> weight ~ 0.1
            // We use a simplified inverse kernel for speed + integer math friendly logic later
            let weight = 1.0 / (1.0 + dist as f32);

            // Value is the byte *following* the key patch
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
