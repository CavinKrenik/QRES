

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*; // SIMD

// QRES v5.0 Predictor Trait
pub trait Predictor {
    fn predict_next(&self) -> u8;
    fn update(&mut self, actual: u8);
}

// QRES v4.0 Predictors

// --- Simple Predictor (Text/Code) ---
// Upgraded to Order-2 Markov (Context = last 2 bytes)
pub struct SimplePredictor {
    prev1: u8,
    prev2: u8,
    context: Box<[u8; 65536]>, // Heap allocate
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
            context: Box::new([0; 65536]),
        }
    }
}

impl Predictor for SimplePredictor {
    fn predict_next(&self) -> u8 {
        let idx = ((self.prev2 as usize) << 8) | (self.prev1 as usize);
        self.context[idx]
    }

    fn update(&mut self, actual: u8) {
        let idx = ((self.prev2 as usize) << 8) | (self.prev1 as usize);
        self.context[idx] = actual;
        self.prev2 = self.prev1;
        self.prev1 = actual;
    }
}

// --- Graph Predictor (Telemetry/Complex Patterns) ---
#[cfg(target_arch = "x86_64")]
type GraphWeights = __m256;
#[cfg(not(target_arch = "x86_64"))]
type GraphWeights = [f32; 8];

pub struct GraphPredictor {
    weights: GraphWeights,
    edges: [usize; 8],
    // OPTIMIZATION: Fixed array instead of VecDeque
    history: [u8; 64],
    cursor: usize,
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
        let edges = [1, 2, 3, 4, 8, 16, 32, 0];
        let weights = unsafe { _mm256_set_ps(0.0, 0.05, 0.05, 0.05, 0.05, 0.1, 0.2, 0.5) };

        GraphPredictor {
            weights,
            edges,
            history: [0; 64],
            cursor: 0,
            learning_rate: 0.015,
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    pub fn new() -> Self {
        let edges = [1, 2, 3, 4, 8, 16, 32, 0];
        let weights = [0.5, 0.2, 0.1, 0.05, 0.05, 0.05, 0.05, 0.0];

        GraphPredictor {
            weights,
            edges,
            history: [0; 64],
            cursor: 0,
            learning_rate: 0.015,
        }
    }
}

impl Predictor for GraphPredictor {
    #[cfg(target_arch = "x86_64")]
    fn predict_next(&self) -> u8 {
        let mut inputs = [0.0f32; 8];
        for (i, input) in inputs.iter_mut().enumerate().take(7) {
            let lag = self.edges[i];
            // Calculate circular index
            let idx = (self.cursor + 64 - lag) % 64;
            *input = self.history[idx] as f32;
        }
        let input_simd = unsafe { _mm256_loadu_ps(inputs.as_ptr()) };
        let product = unsafe { _mm256_mul_ps(self.weights, input_simd) };
        let h1 = unsafe { _mm256_hadd_ps(product, product) };
        let h2 = unsafe { _mm256_hadd_ps(h1, h1) };
        let sum = unsafe { _mm256_cvtss_f32(h2) };
        sum.clamp(0.0, 255.0) as u8
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn predict_next(&self) -> u8 {
        let mut sum = 0.0;
        for i in 0..7 {
            let lag = self.edges[i];
            // Calculate circular index
            let idx = (self.cursor + 64 - lag) % 64;
            let input = self.history[idx] as f32;
            sum += self.weights[i] * input;
        }
        sum.clamp(0.0, 255.0) as u8
    }

    #[cfg(target_arch = "x86_64")]
    fn update(&mut self, actual: u8) {
        let pred = self.predict_next() as f32;
        let err = actual as f32 - pred;
        let mut inputs = [0.0f32; 8];
        for (i, input) in inputs.iter_mut().enumerate().take(7) {
            let lag = self.edges[i];
            // Calculate circular index
            let idx = (self.cursor + 64 - lag) % 64;
            *input = self.history[idx] as f32;
        }
        let input_simd = unsafe { _mm256_loadu_ps(inputs.as_ptr()) };
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

        let mut w_arr = [0.0f32; 8];
        unsafe { _mm256_storeu_ps(w_arr.as_mut_ptr(), self.weights) };
        for w in &mut w_arr {
            *w = w.clamp(-5.0, 5.0);
        }
        self.weights = unsafe { _mm256_loadu_ps(w_arr.as_ptr()) };

        self.history[self.cursor] = actual;
        self.cursor = (self.cursor + 1) % 64;
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn update(&mut self, actual: u8) {
        let pred = self.predict_next() as f32;
        let err = actual as f32 - pred;
        for i in 0..7 {
            let lag = self.edges[i];
            // Calculate circular index
            let idx = (self.cursor + 64 - lag) % 64;
            let input = self.history[idx] as f32;
            let delta = self.learning_rate * err * (input / 255.0);
            self.weights[i] += delta;
            self.weights[i] = self.weights[i].clamp(-5.0, 5.0);
        }
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
            history: Vec::with_capacity(65536), // Buffer grows dynamically anyway
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
        // Multiplicative hash
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
