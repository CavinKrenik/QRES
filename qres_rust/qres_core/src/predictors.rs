pub trait Predictor {
    fn predict_next(&self) -> u8;
    fn update(&mut self, actual: u8);
}

use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use core::convert::TryInto;

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
}

// --- Task A: LzMatchPredictor (LZ77 Simulation) ---
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
            history: Vec::with_capacity(65536),
            pos: 0,
            hash_mask: hash_size - 1,
        }
    }

    #[inline(always)]
    fn hash(data: &[u8]) -> usize {
        if data.len() < 4 {
            return 0;
        }
        let key = u32::from_le_bytes(data[0..4].try_into().unwrap());
        (key.wrapping_mul(0x9E3779B9)) as usize
    }
}

impl Predictor for LzMatchPredictor {
    fn predict_next(&self) -> u8 {
        if self.pos < 4 {
            return 0;
        }
        let start = self.pos - 4;
        let ctx = &self.history[start..self.pos];
        let h = Self::hash(ctx) & self.hash_mask;
        let match_pos = self.table[h];

        if match_pos > 0
            && match_pos + 4 < self.history.len()
            && &self.history[match_pos..match_pos + 4] == ctx
        {
            return self.history[match_pos + 4];
        }
        self.history[self.pos - 1]
    }

    fn update(&mut self, actual: u8) {
        self.history.push(actual);
        self.pos += 1;
        if self.pos > 4 {
            let start = self.pos - 5;
            let ctx = &self.history[start..self.pos - 1];
            let h = Self::hash(ctx) & self.hash_mask;
            self.table[h] = start;
        }
    }
}
