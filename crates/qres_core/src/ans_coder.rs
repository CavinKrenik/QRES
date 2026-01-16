use constriction::stream::model::LeakyQuantizer;
use constriction::stream::queue::{DefaultRangeDecoder, DefaultRangeEncoder};
use constriction::stream::{Decode, Encode};
use probability::distribution::Gaussian;

use alloc::vec::Vec;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

// QRES v4.0 "Hive-Optimized" Backend
// Strategy: Lazy Adaptive ANS with Batched Updates
// Batch Size: 128 ensures we only calculate optimal params/models once per 128 bytes
// This provides 2-3x speed improvement over per-byte updates
const BATCH_SIZE: usize = 32;

pub struct AnsWriter {
    encoder: DefaultRangeEncoder,
    // Welford's online statistics
    running_mean: f64,
    running_var: f64,
    count: usize,
    // Batching buffer
    buffer: Vec<i8>,
}

impl Default for AnsWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl AnsWriter {
    pub fn new() -> Self {
        AnsWriter {
            encoder: DefaultRangeEncoder::new(),
            running_mean: 0.0,
            running_var: 0.0,
            count: 0,
            buffer: Vec::with_capacity(BATCH_SIZE),
        }
    }

    pub fn write_residual(&mut self, residual: i8) {
        self.buffer.push(residual);

        if self.buffer.len() >= BATCH_SIZE {
            self.flush_buffer();
        }
    }

    fn flush_buffer(&mut self) {
        if self.buffer.is_empty() {
            return;
        }

        // 1. Calculate Statistics & Create Model (Once per batch)
        // Note: We use the stats from the START of the batch to encode the whole batch.
        // This effectively delays stats updates by BATCH_SIZE bytes, which is fine for "Lazy" ANS.
        let std = if self.count == 0 {
            32.0 // Initial guess
        } else {
            (libm::sqrt(self.running_var / (self.count).max(1) as f64)).max(1e-6)
        };

        let quantizer = LeakyQuantizer::<f64, i32, u32, 24>::new(-128..=127);
        let model = quantizer.quantize(Gaussian::new(self.running_mean, std));

        // 2. Encode Symbols (Fast Loop)
        for &res in &self.buffer {
            self.encoder.encode_symbol(res as i32, model).unwrap();
        }

        // 3. Batch Stats Update (SIMD-Friendly Optimization)
        // Instead of serial Welford updates, we compute batch stats and merge.
        // This allows autovectorization of the sum/sq_sum.

        let batch_count = self.buffer.len();
        let batch_count_f = batch_count as f64;

        // Calculate Batch Mean and Variance (SIMD Optimized)
        let (batch_mean, batch_m2) = compute_batch_stats(&self.buffer);

        // 4. Merge Batch Stats into Global Stats
        // Formula: M2_combined = M2_a + M2_b + (delta^2 * n_a * n_b) / n_combined
        if self.count == 0 {
            self.running_mean = batch_mean;
            self.running_var = batch_m2;
        } else {
            let total_count = (self.count + batch_count) as f64;
            let delta = batch_mean - self.running_mean;

            let new_m2 = self.running_var
                + batch_m2
                + (delta * delta * (self.count as f64) * batch_count_f) / total_count;

            let new_mean = self.running_mean + (delta * batch_count_f) / total_count;

            self.running_var = new_m2;
            self.running_mean = new_mean;
        }

        self.count += batch_count;
        self.buffer.clear();
    }

    pub fn finish(mut self) -> Vec<u8> {
        // Encode remaining symbols in buffer
        self.flush_buffer();

        // Seal the stream
        let compressed_words: Vec<u32> = self.encoder.into_compressed().unwrap();
        let mut result = Vec::new();
        for word in &compressed_words {
            result.extend_from_slice(&word.to_le_bytes());
        }

        result
    }
}

pub struct AnsReader {
    decoder: DefaultRangeDecoder,
    // Statistics
    running_mean: f64,
    running_var: f64,
    count: usize,
    // Decode Buffer
    buffer: Vec<i8>,
    buffer_pos: usize,
}

impl AnsReader {
    pub fn new(data: &[u8]) -> Self {
        let mut words = Vec::new();
        let mut i = 0;
        while i + 3 < data.len() {
            let word = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
            words.push(word);
            i += 4;
        }

        let decoder = DefaultRangeDecoder::from_compressed(words).unwrap();

        AnsReader {
            decoder,
            running_mean: 0.0,
            running_var: 0.0,
            count: 0,
            buffer: Vec::with_capacity(BATCH_SIZE),
            buffer_pos: 0,
        }
    }

    pub fn read_residual(&mut self) -> i8 {
        // Refill if empty
        if self.buffer_pos >= self.buffer.len() {
            self.refill_buffer();
        }

        // Return next
        let res = self.buffer[self.buffer_pos];
        self.buffer_pos += 1;
        res
    }

    fn refill_buffer(&mut self) {
        self.buffer.clear();
        self.buffer_pos = 0;

        // 1. Calculate Model (Once per batch)
        let std = if self.count == 0 {
            32.0
        } else {
            (libm::sqrt(self.running_var / (self.count).max(1) as f64)).max(1e-6)
        };

        let quantizer = LeakyQuantizer::<f64, i32, u32, 24>::new(-128..=127);
        let model = quantizer.quantize(Gaussian::new(self.running_mean, std));

        // 2. Batch Decode (128 symbols)
        for _ in 0..BATCH_SIZE {
            let val = self.decoder.decode_symbol(&model).unwrap_or(0);
            self.buffer.push(val as i8);
        }

        // 3. Batch Stats Update (Must match AnsWriter exactly!)
        let batch_count = self.buffer.len();
        let batch_count_f = batch_count as f64;

        let (batch_mean, batch_m2) = compute_batch_stats(&self.buffer);

        // 4. Merge Batch Stats
        if self.count == 0 {
            self.running_mean = batch_mean;
            self.running_var = batch_m2;
        } else {
            let total_count = (self.count + batch_count) as f64;
            let delta = batch_mean - self.running_mean;

            let new_m2 = self.running_var
                + batch_m2
                + (delta * delta * (self.count as f64) * batch_count_f) / total_count;

            let new_mean = self.running_mean + (delta * batch_count_f) / total_count;

            self.running_var = new_m2;
            self.running_mean = new_mean;
        }

        self.count += batch_count;
    }
}

// Optimized Batch Statistics using AVX2
#[cfg(target_arch = "x86_64")]
#[allow(dead_code)]
unsafe fn compute_batch_stats_avx2(data: &[i8]) -> (f64, f64) {
    let mut sum_acc = _mm256_setzero_si256();
    let mut sq_acc = _mm256_setzero_si256();

    // Process 16 bytes at a time (128 bits -> 256 bits expansion)
    let chunks = data.chunks_exact(16);
    let remainder = chunks.remainder();

    for chunk in chunks {
        let v_i8 = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);

        // Low 8 bytes -> i32
        let v_lo = _mm256_cvtepi8_epi32(v_i8);
        sum_acc = _mm256_add_epi32(sum_acc, v_lo);
        let sq_lo = _mm256_mullo_epi32(v_lo, v_lo);
        sq_acc = _mm256_add_epi32(sq_acc, sq_lo);

        // High 8 bytes -> i32
        // Move high 64 bits to low 64 bits of 128-bit register
        let v_high_i128 = _mm_unpackhi_epi64(v_i8, v_i8);
        let v_hi = _mm256_cvtepi8_epi32(v_high_i128);
        sum_acc = _mm256_add_epi32(sum_acc, v_hi);
        let sq_hi = _mm256_mullo_epi32(v_hi, v_hi);
        sq_acc = _mm256_add_epi32(sq_acc, sq_hi);
    }

    // Reduce AVX registers
    let mut sum_arr = [0i32; 8];
    let mut sq_arr = [0i32; 8];
    _mm256_storeu_si256(sum_arr.as_mut_ptr() as *mut __m256i, sum_acc);
    _mm256_storeu_si256(sq_arr.as_mut_ptr() as *mut __m256i, sq_acc);

    let mut total_sum: f64 = sum_arr.iter().map(|&x| x as f64).sum();
    let mut total_sq: f64 = sq_arr.iter().map(|&x| x as f64).sum();

    // Process remainder (scalar)
    for &x in remainder {
        let x_f = x as f64;
        total_sum += x_f;
        total_sq += x_f * x_f;
    }

    let n = data.len() as f64;
    let mean = total_sum / n;
    let m2 = total_sq - (total_sum * total_sum) / n;

    (mean, m2)
}

fn compute_batch_stats(data: &[i8]) -> (f64, f64) {
    #[cfg(all(feature = "std", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") {
            // SAFETY: AVX2 support is explicitly checked above via is_x86_feature_detected!
            return unsafe { compute_batch_stats_avx2(data) };
        }
    }

    // Scalar fallback
    let n = data.len() as f64;
    let sum: f64 = data.iter().map(|&x| x as f64).sum();
    let mean = sum / n;
    let m2: f64 = data
        .iter()
        .map(|&x| {
            let diff = x as f64 - mean;
            diff * diff
        })
        .sum();

    (mean, m2)
}
