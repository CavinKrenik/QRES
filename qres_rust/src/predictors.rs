use std::collections::VecDeque;

#[cfg(target_arch = "x86_64")]
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

impl Default for SimplePredictor {
    fn default() -> Self {
        Self::new()
    }
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

#[cfg(target_arch = "x86_64")]
type GraphWeights = __m256;
#[cfg(not(target_arch = "x86_64"))]
type GraphWeights = [f32; 8];

pub struct GraphPredictor {
    // Weights are aligned for SIMD (__m256 for x86_64)
    weights: GraphWeights,
    edges: [usize; 8], // Fixed edges [1, 2, 3, 4, 8, 16, 32, 0]
    history: VecDeque<u8>,
    learning_rate: f32,
}

impl Default for GraphPredictor {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphPredictor {
    #[cfg(target_arch = "x86_64")]
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

    #[cfg(not(target_arch = "x86_64"))]
    pub fn new() -> Self {
        let edges = [1, 2, 3, 4, 8, 16, 32, 0];
        // Scalar equivalent of _mm256_set_ps(0.0, 0.05, ... , 0.5)
        // Note: set_ps(e7, e6... e0). e7 is highest index 7. e0 is index 0.
        // So index 0 is 0.5.
        let weights = [0.5, 0.2, 0.1, 0.05, 0.05, 0.05, 0.05, 0.0];

        GraphPredictor {
            weights,
            edges,
            history: VecDeque::from(vec![0; 40]),
            learning_rate: 0.015,
        }
    }
}

impl Predictor for GraphPredictor {
    #[cfg(target_arch = "x86_64")]
    fn predict_next(&self) -> u8 {
        // 1. Gather inputs
        let mut inputs = [0.0f32; 8];
        let hist_len = self.history.len();

        for (i, input) in inputs.iter_mut().enumerate().take(7) {
            let lag = self.edges[i];
            if lag <= hist_len {
                *input = self.history[hist_len - lag] as f32;
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

    #[cfg(not(target_arch = "x86_64"))]
    fn predict_next(&self) -> u8 {
        let mut sum = 0.0;
        let hist_len = self.history.len();

        for i in 0..7 {
            let lag = self.edges[i];
            if lag <= hist_len {
                let input = self.history[hist_len - lag] as f32;
                sum += self.weights[i] * input;
            }
        }

        sum.clamp(0.0, 255.0) as u8
    }

    #[cfg(target_arch = "x86_64")]
    fn update(&mut self, actual: u8) {
        let pred = self.predict_next() as f32;
        let err = actual as f32 - pred;

        // 1. Gather inputs again
        let mut inputs = [0.0f32; 8];
        let hist_len = self.history.len();
        for (i, input) in inputs.iter_mut().enumerate().take(7) {
            let lag = self.edges[i];
            if lag <= hist_len {
                *input = self.history[hist_len - lag] as f32;
            }
        }
        let input_simd = unsafe { _mm256_loadu_ps(inputs.as_ptr()) };

        // 2. SIMD Weight Update (LMS)
        let lr_simd = unsafe { _mm256_set1_ps(self.learning_rate) };
        let err_simd = unsafe { _mm256_set1_ps(err) };
        let norm_factor = unsafe { _mm256_set1_ps(1.0 / 255.0) };

        let delta = unsafe {
            _mm256_mul_ps(
                lr_simd,
                _mm256_mul_ps(err_simd, _mm256_mul_ps(input_simd, norm_factor)),
            )
        };
        self.weights = unsafe { _mm256_add_ps(self.weights, delta) };

        // Stability Clamp
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

    #[cfg(not(target_arch = "x86_64"))]
    fn update(&mut self, actual: u8) {
        let pred = self.predict_next() as f32;
        let err = actual as f32 - pred;

        let hist_len = self.history.len();

        for i in 0..7 {
            let lag = self.edges[i];
            if lag <= hist_len {
                let input = self.history[hist_len - lag] as f32;
                let delta = self.learning_rate * err * (input / 255.0);
                self.weights[i] += delta;
                self.weights[i] = self.weights[i].clamp(-5.0, 5.0);
            }
        }

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

impl Default for LzMatchPredictor {
    fn default() -> Self {
        Self::new()
    }
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
        if data.len() < 4 {
            return 0;
        }
        ((data[0] as usize) << 12)
            ^ ((data[1] as usize) << 8)
            ^ ((data[2] as usize) << 4)
            ^ (data[3] as usize)
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
