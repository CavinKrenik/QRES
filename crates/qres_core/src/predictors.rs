pub trait Predictor {
    fn predict_next(&self) -> u8;
    fn update(&mut self, actual: u8);
    /// Reset internal state to initial values without reallocating memory.
    /// This allows reusing predictors across chunks to avoid allocation overhead.
    fn reset(&mut self);
}

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

// --- Constants for Fixed-Point Arithmetic (Q16.16) ---
// 1.0 in fixed point = 1 << 16 = 65536
const FIXED_SCALE: i32 = 1 << 16;
const FIXED_ROUND: i32 = 1 << 15; // 0.5 for rounding

fn float_to_fixed(f: f32) -> i32 {
    (f * FIXED_SCALE as f32) as i32
}

// --- Simple Predictor (Text/Code) ---
// Order-2 Markov (Context = last 2 bytes)
pub struct SimplePredictor {
    prev1: u8,
    prev2: u8,
    prev3: u8,
    context: Box<[u8]>, // Order-3 (256^3) = 16MB
}

impl Default for SimplePredictor {
    fn default() -> Self {
        Self::new()
    }
}

impl SimplePredictor {
    pub fn new() -> Self {
        SimplePredictor {
            prev1: 0,
            prev2: 0,
            prev3: 0,
            context: vec![0u8; 16777216].into_boxed_slice(),
        }
    }
}

impl Predictor for SimplePredictor {
    fn predict_next(&self) -> u8 {
        let idx =
            ((self.prev3 as usize) << 16) | ((self.prev2 as usize) << 8) | (self.prev1 as usize);
        self.context[idx]
    }

    fn update(&mut self, actual: u8) {
        let idx =
            ((self.prev3 as usize) << 16) | ((self.prev2 as usize) << 8) | (self.prev1 as usize);
        self.context[idx] = actual;
        self.prev3 = self.prev2;
        self.prev2 = self.prev1;
        self.prev1 = actual;
    }

    fn reset(&mut self) {
        self.prev1 = 0;
        self.prev2 = 0;
        self.prev3 = 0;
        // Zero the context table in-place - NO reallocation
        self.context.fill(0);
    }
}

// --- Graph Predictor (Telemetry/Complex Patterns) ---
// REFACTORED: Uses i32 Fixed-Point (Q16.16) for cross-platform determinism.
pub struct GraphPredictor {
    weights: [i32; 8], // Q16.16 Fixed Point
    edges: [usize; 8],
    history: [u8; 64],
    cursor: usize,
    learning_rate: i32, // Q16.16 Fixed Point
}

impl Default for GraphPredictor {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphPredictor {
    pub fn new() -> Self {
        // Lag intervals
        let edges = [1, 2, 3, 4, 8, 16, 32, 0];

        // Initial weights converted to Q16.16
        // 0.0, 0.05, 0.05, 0.05, 0.05, 0.1, 0.2, 0.5
        let weights = [
            0,
            float_to_fixed(0.05),
            float_to_fixed(0.05),
            float_to_fixed(0.05),
            float_to_fixed(0.05),
            float_to_fixed(0.1),
            float_to_fixed(0.2),
            float_to_fixed(0.5),
        ];

        GraphPredictor {
            weights,
            edges,
            history: [0; 64],
            cursor: 0,
            learning_rate: float_to_fixed(0.015),
        }
    }
}

impl Predictor for GraphPredictor {
    fn predict_next(&self) -> u8 {
        let mut sum: i32 = 0;

        for i in 0..7 {
            let lag = self.edges[i];
            let idx = (self.cursor + 64 - lag) % 64;
            let input = self.history[idx] as i32; // 0..255 integer

            // Multiply: Q16.16 * Integer = Q16.16
            // e.g. 0.5 (32768) * 200 = 6,553,600 (100.0 in Q16.16)
            sum += self.weights[i].wrapping_mul(input);
        }

        // Convert back to integer: (sum + 0.5) >> 16
        let result = (sum + FIXED_ROUND) >> 16;
        result.clamp(0, 255) as u8
    }

    fn update(&mut self, actual: u8) {
        // 1. Calculate Prediction again to get error (in pure int space)
        let pred = self.predict_next() as i32;
        let err = actual as i32 - pred; // Integer error

        // 2. Update Weights
        // Delta = LR * Err * Input
        // We want Delta in Q16.16.
        // LR is Q16.16. Err is Int. Input is Int.
        // If we do LR * Err * Input, we get Q16.16.
        // BUT: We need to normalize input by 255.0 like the original f32 code did.
        // Original: delta = lr * err * (input / 255.0)
        // Fixed: delta = (lr * err * input) / 255

        for i in 0..7 {
            let lag = self.edges[i];
            let idx = (self.cursor + 64 - lag) % 64;
            let input = self.history[idx] as i32;

            // Calculation:
            // numerator = (LR * err) * input  <-- Result is Q16.16 * int * int
            // With LR=0.015 (983), Err=255, Input=255 -> 983*255*255 = 63,919,575.
            // i32 max is 2 billion. This is safe from overflow.

            let numerator = self.learning_rate * err * input;
            let delta = numerator / 255;

            self.weights[i] += delta;

            // Clamp weights to [-5.0, 5.0] in Q16.16
            // 5.0 * 65536 = 327680
            const MAX_WEIGHT: i32 = 5 * FIXED_SCALE;
            const MIN_WEIGHT: i32 = -5 * FIXED_SCALE;
            self.weights[i] = self.weights[i].clamp(MIN_WEIGHT, MAX_WEIGHT);
        }

        // 3. Update History
        self.history[self.cursor] = actual;
        self.cursor = (self.cursor + 1) % 64;
    }

    fn reset(&mut self) {
        // Reset history and cursor
        self.history = [0; 64];
        self.cursor = 0;
        // CRITICAL: Reset weights to exact initial Q16.16 values for v18 bit-perfect compatibility
        // Initial values: 0.0, 0.05, 0.05, 0.05, 0.05, 0.1, 0.2, 0.5
        self.weights = [
            0,
            float_to_fixed(0.05),
            float_to_fixed(0.05),
            float_to_fixed(0.05),
            float_to_fixed(0.05),
            float_to_fixed(0.1),
            float_to_fixed(0.2),
            float_to_fixed(0.5),
        ];
        self.learning_rate = float_to_fixed(0.015);
    }
}

// --- Task A: LzMatchPredictor (LZ77 Simulation) ---
// Uses a fixed-size circular buffer to avoid O(n²) reallocations.
// Buffer size matches CHUNK_SIZE (1MB) to ensure bit-perfect predictions.

const LZ_BUFFER_SIZE: usize = 1024 * 1024; // 1MB - matches CHUNK_SIZE
const LZ_BUFFER_MASK: usize = LZ_BUFFER_SIZE - 1;

pub struct LzMatchPredictor {
    table: Vec<usize>,
    history: Vec<u8>,
    pos: usize,
    hash_mask: usize,
}

impl Default for LzMatchPredictor {
    fn default() -> Self {
        Self::new()
    }
}

impl LzMatchPredictor {
    pub fn new() -> Self {
        const HASH_BITS: usize = 20; // 1M entries (4MB RAM)
        let hash_size = 1 << HASH_BITS;
        LzMatchPredictor {
            table: vec![0; hash_size],
            history: vec![0u8; LZ_BUFFER_SIZE], // Pre-allocate fixed buffer
            pos: 0,
            hash_mask: hash_size - 1,
        }
    }

    #[inline(always)]
    fn get(&self, idx: usize) -> u8 {
        self.history[idx & LZ_BUFFER_MASK]
    }

    #[inline(always)]
    fn hash_ctx(&self, start: usize) -> usize {
        let b0 = self.get(start) as u32;
        let b1 = self.get(start + 1) as u32;
        let b2 = self.get(start + 2) as u32;
        let b3 = self.get(start + 3) as u32;
        let key = b0 | (b1 << 8) | (b2 << 16) | (b3 << 24);
        (key.wrapping_mul(0x9E3779B9)) as usize
    }

    #[inline(always)]
    fn ctx_matches(&self, pos1: usize, pos2: usize) -> bool {
        self.get(pos1) == self.get(pos2)
            && self.get(pos1 + 1) == self.get(pos2 + 1)
            && self.get(pos1 + 2) == self.get(pos2 + 2)
            && self.get(pos1 + 3) == self.get(pos2 + 3)
    }
}

impl Predictor for LzMatchPredictor {
    fn predict_next(&self) -> u8 {
        if self.pos < 4 {
            return 0;
        }
        let start = self.pos - 4;
        let h = self.hash_ctx(start) & self.hash_mask;
        let match_pos = self.table[h];

        // Check if match is valid and within current chunk
        if match_pos > 0 && match_pos + 4 < self.pos && self.ctx_matches(match_pos, start) {
            return self.get(match_pos + 4);
        }
        self.get(self.pos - 1)
    }

    fn update(&mut self, actual: u8) {
        // Write to circular buffer - O(1), no allocation
        self.history[self.pos & LZ_BUFFER_MASK] = actual;
        self.pos += 1;

        if self.pos > 4 {
            let start = self.pos - 5;
            let h = self.hash_ctx(start) & self.hash_mask;
            self.table[h] = start;
        }
    }

    fn reset(&mut self) {
        self.pos = 0;
        // Zero tables in-place - NO reallocation (saves ~5MB per chunk)
        self.table.fill(0);
        self.history.fill(0);
    }
}

// --- Task B: Zero Predictor (Baseline) ---
pub struct ZeroPredictor;

impl ZeroPredictor {
    pub fn new() -> Self {
        ZeroPredictor
    }
}

impl Default for ZeroPredictor {
    fn default() -> Self {
        Self::new()
    }
}

impl Predictor for ZeroPredictor {
    fn predict_next(&self) -> u8 {
        0
    }
    fn update(&mut self, _actual: u8) {}
    fn reset(&mut self) {
        // No state to reset
    }
}
