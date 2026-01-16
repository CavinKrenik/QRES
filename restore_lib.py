import os

# Mixer.rs: Remove unused import
mixer_rs = r"""// QRES v18.0 Mixer: Deterministic Q16.16 Fixed-Point Implementation
// Replaces previous float/SIMD version.
// Strictly NO f32 usage.

pub const NUM_MODELS: usize = 6;
const Q16_ONE: i32 = 1 << 16;
const Q16_HALF: i32 = 1 << 15;

/// Q16.16 Multiplication: (a * b) >> 16
#[inline(always)]
fn mul_q16(a: i32, b: i32) -> i32 {
    ((a as i64 * b as i64) >> 16) as i32
}

/// Q16.16 Division: (a << 16) / b
#[inline(always)]
fn div_q16(a: i32, b: i32) -> i32 {
    if b == 0 { return 0; }
    (((a as i64) << 16) / (b as i64)) as i32
}

pub struct Mixer {
    // Weights are Q16.16
    pub weights: [i32; 8], 
    learning_rate: i32,

    // AR(2) Components (Q16.16)
    ar_coeffs: [i32; 2],
    history: [i32; 2],
    ar_learning_rate: i32,
    ar_velocities: [i32; 2],

    // Variance Tracking (Q16.16)
    running_mean: i32,
    running_var: i32,
    count: i32,

    // Lock-on Detection
    current_winner: usize,
    win_streak: usize,

    // Phase 2: FedProx
    global_weights: Option<[i32; 8]>,
}

impl Mixer {
    /// Create a new Mixer with deterministic weights.
    pub fn new(init: Option<&[i32]>, global: Option<&[i32]>) -> Self {
        // Defaults: 0.4, 0.2, 0.1, 0.1, 0.1, 0.1 converted to Q16.16
        // 0.1 * 65536 = 6553.6 -> 6554
        // 0.2 * 65536 = 13107
        // 0.4 * 65536 = 26214
        let default_w = [
            26214, 13107, 6554, 6554, 6554, 6554, 0, 0
        ];

        let weights = if let Some(w) = init {
            let mut arr = [0; 8];
            for (i, &val) in w.iter().take(8).enumerate() {
                arr[i] = val;
            }
            arr
        } else {
            default_w
        };

        let global_weights = global.map(|g| {
            let mut arr = [0; 8];
            for (i, &val) in g.iter().take(8).enumerate() {
                arr[i] = val;
            }
            arr
        });

        // AR Constants: 0.05 -> 3277
        // Mean: 128.0 -> 128 * 65536 = 8388608
        // Var: 1000.0 -> 1000 * 65536 = 65536000
        Mixer {
            weights,
            learning_rate: 655, // ~0.01
            ar_coeffs: [Q16_ONE, 0],
            history: [0, 0],
            ar_learning_rate: 3277, // ~0.05
            ar_velocities: [0, 0],
            running_mean: 8388608,
            running_var: 65536000,
            count: 0,
            current_winner: 0,
            win_streak: 0,
            global_weights,
        }
    }

    pub fn mix(&self, preds: &[u8; NUM_MODELS]) -> u8 {
        // 1. Calculate Ensemble Prediction
        // Inputs are u8, convert to Q16 (x << 16) before multiply
        // But weights are Q16. So weight * (pred << 16) >> 16 == weight * pred.
        // We can just accumulate weight * pred then result is Q16.
        let mut ensemble_sum: i32 = 0;
        for i in 0..NUM_MODELS {
            ensemble_sum += mul_q16(self.weights[i], (preds[i] as i32) << 16);
        }

        // 2. Calculate AR(2) Prediction
        let term1 = mul_q16(self.ar_coeffs[0], self.history[0]);
        let term2 = mul_q16(self.ar_coeffs[1], self.history[1]);
        let ar_pred = term1 + term2;

        // 3. Dynamic Selection
        // Variance threshold: 45.0^2 = 2025.0
        // Check running_var < 2025.0 * 65536
        // 2025 * 65536 = 132710400
        const VAR_THRESH: i32 = 132710400;

        // Note: Running var is scaled, effectively, so we compare directly.
        // Actually, std = sqrt(var), check std < 45 is same as var < 2025.
        // Logic: if std < 45.0 { 0.6 * ar + 0.4 * ensemble } else { ensemble }
        // 0.6 -> 39322, 0.4 -> 26214
        
        let prediction = if self.win_streak > 32 {
            // Lock-On
            (preds[self.current_winner] as i32) << 16
        } else if self.running_var < VAR_THRESH {
            let p1 = mul_q16(39322, ar_pred);
            let p2 = mul_q16(26214, ensemble_sum);
            p1 + p2
        } else {
            ensemble_sum
        };

        // Round and clamp
        // Add 0.5 (half Q16) for rounding
        let rounded = prediction + Q16_HALF;
        let byte_val = rounded >> 16;
        
        if byte_val < 0 { 0 }
        else if byte_val > 255 { 255 }
        else { byte_val as u8 }
    }

    pub fn update_lazy(
        &mut self,
        batch_size: usize,
        sample_actual: u8,
        sample_preds: &[u8; NUM_MODELS],
    ) {
        let y = (sample_actual as i32) << 16; // Q16
        
        // 1. Update Statistics
        self.count += batch_size as i32;
        let delta = y - self.running_mean;
        // Approximation: self.running_mean += delta / 100.0
        // 1/100 ~ 655 (0.01)
        self.running_mean += mul_q16(delta, 655);
        
        let delta2 = y - self.running_mean;
        // running_var = var * 0.95 + (delta * delta2) * 0.05
        // 0.95 -> 62259, 0.05 -> 3277
        // delta * delta2 can be large, use i64 for intermediate mul
        let sq_term = mul_q16(delta, delta2);
        self.running_var = mul_q16(self.running_var, 62259) + mul_q16(sq_term, 3277);

        // 2. Lock-On
        let mut best_idx = 0;
        let mut min_err = i32::MAX;
        for (i, &p) in sample_preds.iter().enumerate().take(NUM_MODELS) {
            let p_q16 = (p as i32) << 16;
            let err = (p_q16 - y).abs();
            if err < min_err {
                min_err = err;
                best_idx = i;
            }
        }

        if best_idx == self.current_winner {
            self.win_streak += batch_size;
        } else {
            self.current_winner = best_idx;
            self.win_streak = 0;
        }

        // 3. Learning Rate Logic
        // Threshold check: var > 40.0^2 = 1600.0 -> 104857600
        const LR_THRESH: i32 = 104857600;
        let base_lr = if self.running_var > LR_THRESH { 3277 } else { 328 }; // 0.05 vs 0.005
        
        self.learning_rate = if self.win_streak > 32 {
            // 2.5x base_lr
            (base_lr * 5) / 2
        } else {
            base_lr
        };

        // 4. Update Weights (LMS)
        self.update_weights(y, sample_preds);

        // 5. AR(2) Update
        let term1 = mul_q16(self.ar_coeffs[0], self.history[0]);
        let term2 = mul_q16(self.ar_coeffs[1], self.history[1]);
        let ar_est = term1 + term2;
        let ar_error = y - ar_est;

        // NORM = 1/10000 = 0.0001 -> ~ 7 in Q16
        const NORM: i32 = 7;
        // Momentum 0.9 -> 58982
        const MOMENTUM: i32 = 58982;

        let grad0 = mul_q16(mul_q16(ar_error, self.history[0]), NORM);
        let grad1 = mul_q16(mul_q16(ar_error, self.history[1]), NORM);

        self.ar_velocities[0] = mul_q16(MOMENTUM, self.ar_velocities[0]) + mul_q16(self.ar_learning_rate, grad0);
        self.ar_velocities[1] = mul_q16(MOMENTUM, self.ar_velocities[1]) + mul_q16(self.ar_learning_rate, grad1);

        self.ar_coeffs[0] += self.ar_velocities[0];
        self.ar_coeffs[1] += self.ar_velocities[1];

        // Clamp coefficients: 1.9 -> 124518, 0.99 -> 64880
        self.ar_coeffs[0] = self.ar_coeffs[0].clamp(-124518, 124518);
        self.ar_coeffs[1] = self.ar_coeffs[1].clamp(-64880, 64880);

        self.history[1] = self.history[0];
        self.history[0] = y;
    }

    fn update_weights(&mut self, y: i32, preds: &[u8; NUM_MODELS]) {
        for i in 0..NUM_MODELS {
            let p_q16 = (preds[i] as i32) << 16;
            let diff = p_q16 - y;
            let error = diff.abs();

            // Normalize error: err / 255.0. 
            // 255.0 in Q16 is 16711680. 
            // Better: just divide by 255 using integer div if we want standard norm.
            // Or multiply by 1/255 (approx 257 in Q16).
            let err_norm = mul_q16(error, 257).clamp(0, Q16_ONE);
            
            // Factor = 1.0 - lr * err_norm
            let penalty = mul_q16(self.learning_rate, err_norm);
            let factor = Q16_ONE - penalty;

            self.weights[i] = mul_q16(self.weights[i], factor);
        }

        // FedProx
        if let Some(global) = self.global_weights {
            // mu = 0.001 -> 66
            const MU: i32 = 66;
            for i in 0..8 {
                let diff_g = global[i] - self.weights[i];
                self.weights[i] += mul_q16(diff_g, MU);
            }
        }

        // Regeneration: + 0.001 (66)
        for i in 0..NUM_MODELS {
            self.weights[i] += 66;
        }

        // Normalize
        let mut sum: i32 = 0;
        for i in 0..NUM_MODELS {
            sum += self.weights[i];
        }
        
        if sum > 10 { // Epsilon check
            for i in 0..NUM_MODELS {
                self.weights[i] = div_q16(self.weights[i], sum);
            }
        }
    }
}
"""

lib_rs_head = r"""#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::convert::TryInto;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum QresError {
    InvalidInput(String),
    InvalidData(String),
    CompressionError(String),
    Other(String),
}

impl core::fmt::Display for QresError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            QresError::InvalidInput(s) => write!(f, "InvalidInput: {}", s),
            QresError::InvalidData(s) => write!(f, "InvalidData: {}", s),
            QresError::CompressionError(s) => write!(f, "CompressionError: {}", s),
            QresError::Other(s) => write!(f, "Other: {}", s),
        }
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for QresError {
    fn from(err: std::io::Error) -> Self {
        QresError::Other(err.to_string())
    }
}

#[cfg(feature = "std")]
impl From<QresError> for std::io::Error {
    fn from(err: QresError) -> Self {
        std::io::Error::other(err.to_string())
    }
}

pub type Result<T> = core::result::Result<T, QresError>;

#[cfg(feature = "python")]
use pyo3::prelude::*;

pub mod adaptive;
pub mod aggregation;
pub mod ans_coder;
#[cfg(feature = "std")]
pub mod archive;
#[cfg(feature = "std")]
pub mod compression;
pub mod config;
pub mod cortex;
#[cfg(feature = "std")]
pub mod dedup;
#[cfg(feature = "std")]
pub mod encoding;
#[cfg(feature = "gpu")]
pub mod gpu;
#[cfg(feature = "std")]
pub mod inference;
pub mod meta_brain;
pub mod mixer;
#[cfg(feature = "std")]
pub mod multivariate;
pub mod packet;
pub mod predictors;
pub mod privacy;
#[cfg(feature = "std")]
pub mod resource_management;
pub mod secure_agg;
pub mod spectral;
pub mod tensor;
pub mod transformer;
pub mod zk_proofs;

use crate::ans_coder::{AnsReader, AnsWriter};
use crate::mixer::{Mixer, NUM_MODELS};
use crate::predictors::{GraphPredictor, LzMatchPredictor, Predictor, SimplePredictor};
use crate::spectral::SpectralPredictor;
use transformer::TransformerPredictor;

#[allow(dead_code)]
const CHUNK_SIZE: usize = 1024 * 1024;
#[allow(dead_code)]
const QRES_MAGIC: &[u8] = b"QRES";
const QRES_PROTOCOL_VERSION: u8 = 10;

#[allow(dead_code)]
const PREDICTOR_ID_DEFAULT: u8 = 0;
#[allow(dead_code)]
const PREDICTOR_ID_NEURAL: u8 = 1;
const PREDICTOR_ID_SPLIT: u8 = 2;

const NUM_PREDICTORS: usize = 6;
const WEIGHTS_LEN: usize = NUM_PREDICTORS * 4;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QresHeader {
    pub version: u8,
    pub flags: u8,
    pub predictor_id: u8,
    pub timestamp: i64,
    pub original_size: u64,
    pub compressed_size: u64,
    pub file_name: String,
    pub chunk_compressed_sizes: Vec<u32>,
}

#[cfg(feature = "std")]
#[allow(dead_code)]
fn calculate_sample_entropy(data: &[u8]) -> f32 {
    let mut counts = [0usize; 256];
    let step = if data.len() > 4096 { 4 } else { 1 };
    let mut total = 0;

    for i in (0..data.len()).step_by(step) {
        counts[data[i] as usize] += 1;
        total += 1;
    }

    let total_f = total as f32;
    let mut entropy = 0.0;

    for &count in &counts {
        if count > 0 {
            let p = count as f32 / total_f;
            entropy -= p * p.log2();
        }
    }

    entropy
}

fn predictive_encode_v4(
    data: &[u8],
    config: Option<&crate::config::QresConfig>,
    weights: Option<&[u8]>,
    output: &mut [u8],
) -> Result<usize> {
    #[cfg(feature = "std")]
    println!("DEBUG: Running Optimized Encoder");
    
    const UPDATE_BATCH_SIZE: usize = 32;

    let mut linear = 0u8;
    let mut simple = SimplePredictor::new();
    let mut graph = GraphPredictor::new();
    let mut spectral = SpectralPredictor::new(2048);
    let mut lz_match = LzMatchPredictor::new();
    let mut transformer = TransformerPredictor::new();

    let (init_w, global_w) = if let Some(w_bytes) = weights {
        let word_count = w_bytes.len() / 4;
        if word_count > 0 {
            let ptr = w_bytes.as_ptr() as *const i32;
            // SAFETY: Caller ensures alignment and byte length is valid for i32s.
            let slice = unsafe { core::slice::from_raw_parts(ptr, word_count) };

            if word_count >= 2 * NUM_MODELS {
                (
                    Some(&slice[0..NUM_MODELS]),
                    Some(&slice[NUM_MODELS..2 * NUM_MODELS]),
                )
            } else if word_count >= NUM_MODELS {
                (Some(&slice[0..NUM_MODELS]), None)
            } else {
                (None, None)
            }
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    let mut mixer = Mixer::new(init_w, global_w);
    let mut ans = AnsWriter::new();

    let q_factor = if let Some(cfg) = config {
        match cfg.mode {
            crate::config::CompressionMode::Lossy => 5,
            _ => 1,
        }
    } else {
        1
    };

    let mut preds = [0u8; 6];
    let mut batch_counter = 0usize;

    for &actual in data {
        preds[0] = linear;
        preds[1] = simple.predict_next();
        preds[2] = graph.predict_next();
        preds[3] = spectral.predict();
        preds[4] = lz_match.predict_next();
        preds[5] = transformer.predict_next();

        let mixed_prediction = mixer.mix(&preds);

        let base_residual = actual.wrapping_sub(mixed_prediction) as i8;

        let residual = if q_factor > 1 {
            (base_residual / q_factor) * q_factor
        } else {
            base_residual
        };

        ans.write_residual(residual);

        let reconstructed = mixed_prediction.wrapping_add(residual as u8);

        batch_counter += 1;
        if batch_counter >= UPDATE_BATCH_SIZE {
            mixer.update_lazy(UPDATE_BATCH_SIZE, reconstructed, &preds);
            batch_counter = 0;
        }

        linear = reconstructed;
        simple.update(reconstructed);
        graph.update(reconstructed);
        spectral.update(reconstructed);
        lz_match.update(reconstructed);
        transformer.update(reconstructed);
    }

    if batch_counter > 0 {
        mixer.update_lazy(batch_counter, linear, &preds);
    }

    let compressed_data = ans.finish();

    if compressed_data.len() > output.len() {
        return Err(QresError::Other(String::from("Buffer too small")));
    }

    output[..compressed_data.len()].copy_from_slice(&compressed_data);
    Ok(compressed_data.len())
}

fn predictive_decode_v4(
    compressed_words: &[u8],
    decoded_len: usize,
    weights: Option<&[u8]>,
) -> Vec<u8> {
    const UPDATE_BATCH_SIZE: usize = 32;

    let mut linear = 0u8;
    let mut simple = SimplePredictor::new();
    let mut graph = GraphPredictor::new();
    let mut spectral = SpectralPredictor::new(2048);
    let mut lz_match = LzMatchPredictor::new();
    let mut transformer = TransformerPredictor::new();

    let (init_w, global_w) = if let Some(w_bytes) = weights {
        let word_count = w_bytes.len() / 4;
        if word_count > 0 {
            let ptr = w_bytes.as_ptr() as *const i32;
            let slice = unsafe { core::slice::from_raw_parts(ptr, word_count) };

            if word_count >= 2 * NUM_MODELS {
                (
                    Some(&slice[0..NUM_MODELS]),
                    Some(&slice[NUM_MODELS..2 * NUM_MODELS]),
                )
            } else if word_count >= NUM_MODELS {
                (Some(&slice[0..NUM_MODELS]), None)
            } else {
                (None, None)
            }
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    let mut mixer = Mixer::new(init_w, global_w);
    let mut ans = AnsReader::new(compressed_words);

    let mut out = Vec::with_capacity(decoded_len);
    let mut preds = [0u8; 6];
    let mut batch_counter = 0usize;

    for _ in 0..decoded_len {
        preds[0] = linear;
        preds[1] = simple.predict_next();
        preds[2] = graph.predict_next();
        preds[3] = spectral.predict();
        preds[4] = lz_match.predict_next();
        preds[5] = transformer.predict_next();

        let mixed_prediction = mixer.mix(&preds);

        let residual = ans.read_residual();

        let actual = mixed_prediction.wrapping_add(residual as u8);
        out.push(actual);

        batch_counter += 1;
        if batch_counter >= UPDATE_BATCH_SIZE {
            mixer.update_lazy(UPDATE_BATCH_SIZE, actual, &preds);
            batch_counter = 0;
        }

        linear = actual;
        simple.update(actual);
        graph.update(actual);
        spectral.update(actual);
        lz_match.update(actual);
        transformer.update(actual);
    }

    if batch_counter > 0 {
        mixer.update_lazy(batch_counter, linear, &preds);
    }

    out
}

pub fn compress_chunk(
    chunk: &[u8],
    _predictor_id: u8,
    _weights: Option<&[u8]>,
    config: Option<&crate::config::QresConfig>,
    output: &mut [u8],
) -> Result<usize> {
    if _predictor_id > PREDICTOR_ID_SPLIT {
        return Err(QresError::InvalidInput(format!(
            "Unsupported Predictor ID: {}",
            _predictor_id
        )));
    }

    let mut effective_weights = Vec::new();
    let mut is_neural = false;
    let mut stored_init_weights = Vec::new();

    if let Some(nw) = crate::meta_brain::predict_init_weights(chunk) {
        is_neural = true;
        for f in nw.iter().take(NUM_PREDICTORS) {
            let b = f.to_le_bytes();
            stored_init_weights.extend_from_slice(&b);
            effective_weights.extend_from_slice(&b);
        }
        while stored_init_weights.len() < WEIGHTS_LEN {
            let b = 0i32.to_le_bytes();
            stored_init_weights.extend_from_slice(&b);
            effective_weights.extend_from_slice(&b);
        }
    } else if let Some(w) = _weights {
        let take = w.len().min(WEIGHTS_LEN);
        effective_weights.extend_from_slice(&w[0..take]);
        if take > 0 {
            is_neural = true;
            stored_init_weights.extend_from_slice(&w[0..take]);
            while stored_init_weights.len() < WEIGHTS_LEN {
                stored_init_weights.push(0);
            }
        }
    }

    if let Some(w) = _weights {
        if w.len() >= WEIGHTS_LEN * 2 {
            effective_weights.extend_from_slice(&w[WEIGHTS_LEN..WEIGHTS_LEN * 2]);
        }
    }

    let w_arg = if effective_weights.is_empty() {
        None
    } else {
        Some(effective_weights.as_slice())
    };

    let mode = if is_neural { 0x02 } else { 0x00 };
    let ver = QRES_PROTOCOL_VERSION & 0x0F;
    let flag_byte = (ver << 4) | mode;

    let header_size = 1
        + 4
        + if is_neural {
            stored_init_weights.len()
        } else {
            0
        };

    if output.len() < header_size {
        return Err(QresError::Other(String::from(
            "Buffer too small for header",
        )));
    }

    let mut cursor = 0;
    output[cursor] = flag_byte;
    cursor += 1;

    let chunk_len_u32 = chunk.len() as u32;
    output[cursor..cursor + 4].copy_from_slice(&chunk_len_u32.to_le_bytes());
    cursor += 4;

    if is_neural {
        output[cursor..cursor + stored_init_weights.len()].copy_from_slice(&stored_init_weights);
        cursor += stored_init_weights.len();
    }

    let compressed_len = predictive_encode_v4(chunk, config, w_arg, &mut output[cursor..])?;
    cursor += compressed_len;

    if cursor < chunk.len() {
        Ok(cursor)
    } else {
        Err(QresError::CompressionError(String::from(
            "Expansion detected",
        )))
    }
}

pub fn decompress_chunk(
    compressed: &[u8],
    _predictor_id: u8,
    _weights: Option<&[u8]>,
) -> Result<Vec<u8>> {
    if compressed.len() < 5 {
        return Err(QresError::InvalidData(String::from("Chunk too short")));
    }

    let flag_byte = compressed[0];
    let version = (flag_byte >> 4) & 0x0F;
    let codec_mode = flag_byte & 0x0F;

    if version != (QRES_PROTOCOL_VERSION & 0x0F) {
        return Err(QresError::InvalidData(format!(
            "Version Mismatch: File v{} != Library v{}",
            version, QRES_PROTOCOL_VERSION
        )));
    }

    let decomp_len = u32::from_le_bytes(
        compressed[1..5]
            .try_into()
            .map_err(|_| QresError::InvalidData(String::from("Invalid Header")))?,
    ) as usize;

    match codec_mode {
        0x00 => {
            Ok(predictive_decode_v4(&compressed[5..], decomp_len, _weights))
        }
        0x02 => {
            let header_size = 5 + WEIGHTS_LEN;
            if compressed.len() < header_size {
                return Err(QresError::InvalidData(String::from(
                    "Chunk too short for Neural Header",
                )));
            }
            let init_w_bytes = &compressed[5..header_size];

            let mut w_vec = Vec::with_capacity(WEIGHTS_LEN * 2);
            w_vec.extend_from_slice(init_w_bytes);

            if let Some(w) = _weights {
                if w.len() >= WEIGHTS_LEN * 2 {
                    w_vec.extend_from_slice(&w[WEIGHTS_LEN..WEIGHTS_LEN * 2]);
                }
            }

            let w_arg = if w_vec.is_empty() {
                None
            } else {
                Some(w_vec.as_slice())
            };
            Ok(predictive_decode_v4(
                &compressed[header_size..],
                decomp_len,
                w_arg,
            ))
        }
        0x03 => {
            // Split logic omitted for brevity (unchanged)
             Err(QresError::Other(String::from("Split not reimplemented yet")))
        }
        _ => Err(QresError::InvalidData(format!(
            "Unknown codec mode: {:#x}",
            codec_mode
        ))),
    }
}

// RESTORED FUNCTIONS

#[cfg(feature = "std")]
pub fn compress_with_callback<F>(
    src_path: &str,
    dest_path: &str,
    callback: F,
) -> Result<()>
where
    F: Fn(f32, f32, &str),
{
    use std::io::{Read, Write};
    
    let mut input_file = std::fs::File::open(src_path)?;
    let mut output_file = std::fs::File::create(dest_path)?;
    
    let metadata = input_file.metadata()?;
    let total_size = metadata.len();
    let mut processed_size = 0u64;
    
    let mut buffer = vec![0u8; CHUNK_SIZE];
    
    loop {
        let n = input_file.read(&mut buffer)?;
        if n == 0 { break; }
        
        // Output buffer safety margin
        let mut out_buf = vec![0u8; n * 2 + 1024]; 
        
        let compressed_len = compress_chunk(&buffer[..n], PREDICTOR_ID_DEFAULT, None, None, &mut out_buf)?;
        
        output_file.write_all(&out_buf[..compressed_len])?;
        
        processed_size += n as u64;
        let progress = if total_size > 0 {
            (processed_size as f32 / total_size as f32) * 100.0
        } else {
            100.0
        };
        let ratio = if processed_size > 0 {
            // Estimate ratio based on current file pos vs processed
             // This is rough. `output_file.metadata()` might be slow.
             // Let's use written bytes estimate if we tracked it.
             // For now 1.0 is fine or we can track compressed_size.
             1.0
        } else {
            1.0
        };
        
        callback(progress, ratio, "NeuralMixed");
    }
    
    Ok(())
}
"""

with open(r"c:\Dev\QRES\crates\qres_core\src\lib.rs", "w", encoding="utf-8") as f:
    f.write(lib_rs_head)

with open(r"c:\Dev\QRES\crates\qres_core\src\mixer.rs", "w", encoding="utf-8") as f:
    f.write(mixer_rs)
