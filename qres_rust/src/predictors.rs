use std::collections::{HashMap, VecDeque};
use std::arch::x86_64::*; // SIMD


// QRES v5.0 Predictor Trait
pub trait Predictor {
    fn predict_next(&self) -> u8;
    fn update(&mut self, actual: u8);
}

// QRES v4.0 Predictors
// 1. SimplePredictor: Order-1 Markov Context (Fast Text)
// 2. GraphPredictor: DAG-based Long-Range Dependency Model (Telemetry/Logs)

// --- Simple Predictor (Text/Code) ---
pub struct SimplePredictor {
    prev: u8,
    context: [u8; 256], 
}

impl SimplePredictor {
    pub fn new() -> Self {
        SimplePredictor {
            prev: 0,
            context: [0; 256],
        }
    }
}

impl Predictor for SimplePredictor {
    fn predict_next(&self) -> u8 {
        self.context[self.prev as usize]
    }

    fn update(&mut self, actual: u8) {
        self.context[self.prev as usize] = actual;
        self.prev = actual;
    }
}

// --- Graph Predictor (Telemetry/Complex Patterns) ---
// Replaces the experimental iPEPS model with a concrete DAG-based learner.
// Captures dependencies at specific lag intervals (edges).

// --- Graph Predictor (SIMD Optimized) ---
pub struct GraphPredictor {
    // Weights are aligned for SIMD (__m256 for x86_64)
    weights: __m256, 
    edges: [usize; 8],        // Fixed edges [1, 2, 3, 4, 8, 16, 32, 0]
    history: VecDeque<u8>,
    learning_rate: f32,
}

impl GraphPredictor {
    pub fn new() -> Self {
        // Tuned lags for telemetry/logs
        let edges = [1, 2, 3, 4, 8, 16, 32, 0]; // 0 is dummy padding
        let weights = unsafe { _mm256_set_ps(0.0, 0.05, 0.05, 0.05, 0.05, 0.1, 0.2, 0.5) };

        GraphPredictor {
            weights,
            edges,
            history: VecDeque::from(vec![0; 40]), // Sufficient buffer
            learning_rate: 0.015,
        }
    }
}

impl Predictor for GraphPredictor {
    fn predict_next(&self) -> u8 {
        // 1. Gather inputs
        let mut inputs = [0.0f32; 8];
        let hist_len = self.history.len();

        for i in 0..7 { // predictable branch (const 7)
            let lag = self.edges[i];
            // history back is newest. index 0 is oldest.
            // history[len - 1] is t-1
            if lag <= hist_len {
                inputs[i] = self.history[hist_len - lag] as f32;
            }
        }
        
        let input_simd = unsafe { _mm256_loadu_ps(inputs.as_ptr()) };

        // 2. SIMD Dot Product
        let product = unsafe { _mm256_mul_ps(self.weights, input_simd) };
        let h1 = unsafe { _mm256_hadd_ps(product, product) };
        let h2 = unsafe { _mm256_hadd_ps(h1, h1) };
        let sum = unsafe { _mm256_cvtss_f32(h2) };

        sum.clamp(0.0, 255.0) as u8
    }

    fn update(&mut self, actual: u8) {
        let pred = self.predict_next() as f32;
        let err = actual as f32 - pred;
        
        // 1. Gather inputs again (redundant but clean for ownership)
        let mut inputs = [0.0f32; 8];
        let hist_len = self.history.len();
        for i in 0..7 {
            let lag = self.edges[i];
            if lag <= hist_len {
                inputs[i] = self.history[hist_len - lag] as f32;
            }
        }
        let input_simd = unsafe { _mm256_loadu_ps(inputs.as_ptr()) };

        // 2. SIMD Weight Update (LMS)
        // w_new = w + lr * err * input_norm
        // input_norm = input / 255.0
        let lr_simd = unsafe { _mm256_set1_ps(self.learning_rate) };
        let err_simd = unsafe { _mm256_set1_ps(err) };
        let norm_factor = unsafe { _mm256_set1_ps(1.0 / 255.0) };

        let delta = unsafe { _mm256_mul_ps(lr_simd, _mm256_mul_ps(err_simd, _mm256_mul_ps(input_simd, norm_factor))) };
        self.weights = unsafe { _mm256_add_ps(self.weights, delta) };

        // Stability Clamp (Prevent Exploding Gradients in Graph)
        let mut w_arr = [0.0f32; 8];
        unsafe { _mm256_storeu_ps(w_arr.as_mut_ptr(), self.weights) };
        for w in &mut w_arr {
            *w = w.clamp(-5.0, 5.0);
        }
        self.weights = unsafe { _mm256_loadu_ps(w_arr.as_ptr()) };

        // Update history
        self.history.push_back(actual);
        if self.history.len() > 40 {
            self.history.pop_front();
        }
    }
}

// --- Task A: LzMatchPredictor (LZ77 Simulation) ---
pub struct LzMatchPredictor {
    // Hash table: maps 4-byte hash -> absolute position in stream
    table: Vec<usize>, 
    history: Vec<u8>,
    pos: usize,
    hash_mask: usize,
}

impl LzMatchPredictor {
    pub fn new() -> Self {
        // 16-bit hash (64K entries) fits in L2 cache
        let hash_bits = 16; 
        let hash_size = 1 << hash_bits;
        
        LzMatchPredictor {
            table: vec![0; hash_size],
            history: Vec::with_capacity(65536), // Window size
            pos: 0,
            hash_mask: hash_size - 1,
        }
    }

    #[inline(always)]
    fn hash(data: &[u8]) -> usize {
        // Simple hash: (b1 << 12 ^ b2 << 8 ^ b3 << 4 ^ b4)
        // Or multiplicative. Let's use a standard fast rolling-like hash.
        // FXZ hash style:
        if data.len() < 4 { return 0; }
        ((data[0] as usize) << 12) ^ 
        ((data[1] as usize) << 8) ^ 
        ((data[2] as usize) << 4) ^ 
        (data[3] as usize)
        // Note: Using a better hash helps collisions, but this is fast.
        // Let's use a slightly better mixer:
        // let mut h = 0xcf1bbcdcb7a56463u64; // Approx
        // We'll stick to the shift-xor for speed in inner loop.
    }
}

impl Predictor for LzMatchPredictor {
    fn predict_next(&self) -> u8 {
        if self.pos < 4 { return 0; }
        
        // 1. Hash last 3 bytes + 'current implied' (LZ lookahead usually needs current byte... wait)
        // Prediction happens BEFORE we see x_t.
        // We predict x_t based on x_{t-1}, x_{t-2}, ...
        // So we look up the context `data[t-4..t]` (length 4 bytes prior to current).
        // Let's use context length 4.
        
        let start = self.pos - 4;
        let ctx = &self.history[start..self.pos];
        let h = Self::hash(ctx) & self.hash_mask;
        
        // 2. Lookup match
        let match_pos = self.table[h];
        
        // 3. Verify match (optional, but good for accuracy)
        // We need `history[match_pos..match_pos+4] == ctx`
        // If match_pos is valid and not too close to current (avoid self-ref overlap mess for now, though LZ allowed)
        if match_pos > 0 && match_pos + 4 < self.history.len() {
             // Check if context actually matches (avoids hash collisions)
             if &self.history[match_pos..match_pos+4] == ctx {
                 // Predict the byte *after* the match
                 return self.history[match_pos + 4];
             }
        }
        
        // Fallback: No match found, use simple 0 or let mixer handle it (Mixer will weight this down if it's constant)
        // But usually LZ returns 'literal' if no match. Here we must return a byte prediction.
        // We return the last byte (RLE) as a safe fallback? 
        // Or 0? Let's return self.history[self.pos-1] (order-1)
        self.history[self.pos - 1]
    }

    fn update(&mut self, actual: u8) {
        // 1. Add to history
        self.history.push(actual);
        self.pos += 1;
        
        // 2. Update Hash Table with PREVIOUS sequence
        // We want to record the occurrence of `history[t-4..t]` so future lookups find it.
        // We update the hash for the sequence ending at `pos - 1`.
        // Sequence: history[pos-5 .. pos-1] (length 4) -> maps to start index (pos-5)
        if self.pos > 4 {
            let start = self.pos - 5; // The sequence that just finished
            let ctx = &self.history[start..self.pos-1];
            let h = Self::hash(ctx) & self.hash_mask;
            self.table[h] = start; // Overwrite with most recent
        }
    }
}
