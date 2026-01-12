#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
// use alloc::boxed::Box;
// Use #[allow(unused_imports)] or just remove.
// Removing it.
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

// --- v3.0/v4.0 Modules ---
pub mod aggregation;
pub mod adaptive;
pub mod ans_coder;
#[cfg(feature = "std")]
pub mod archive;
#[cfg(feature = "std")]
pub mod compression;
#[cfg(feature = "std")]
pub mod dedup;
#[cfg(feature = "std")]
pub mod encoding;
#[cfg(feature = "gpu")]
pub mod gpu;
#[cfg(feature = "std")]
pub mod inference;
pub mod meta_brain; // Inference Engine (moved to Core)
pub mod mixer;
#[cfg(feature = "std")]
pub mod multivariate;
pub mod predictors;
pub mod privacy;
#[cfg(feature = "std")]
pub mod resource_management;
pub mod secure_agg;
pub mod spectral;
pub mod tensor;
pub mod transformer;
pub mod zk_proofs; // Phase 2 Security: Byzantine-tolerant aggregation
use crate::ans_coder::{AnsReader, AnsWriter};
use crate::mixer::{Mixer, NUM_MODELS};
use crate::predictors::{GraphPredictor, LzMatchPredictor, Predictor, SimplePredictor};
use crate::spectral::SpectralPredictor;
use transformer::TransformerPredictor;

// --- Living Brain (Adaptive Learning) ---
// Note: LivingBrain struct moved to qres_daemon. Core only handles inference via meta_brain.rs.

#[allow(dead_code)]
const CHUNK_SIZE: usize = 1024 * 1024;
#[allow(dead_code)]
const QRES_MAGIC: &[u8] = b"QRES";
const QRES_PROTOCOL_VERSION: u8 = 10; // v10.0 Engineering

// Known Predictor IDs
#[allow(dead_code)]
const PREDICTOR_ID_DEFAULT: u8 = 0;
#[allow(dead_code)]
const PREDICTOR_ID_NEURAL: u8 = 1;
const PREDICTOR_ID_SPLIT: u8 = 2; // Reserved for Interleaved

const NUM_PREDICTORS: usize = 6;
const WEIGHTS_LEN: usize = NUM_PREDICTORS * 4;

// --- Header Architecture (V3) ---
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

// --- V4 Encoding Logic ---

#[cfg(feature = "std")]
#[allow(dead_code)]
fn calculate_sample_entropy(data: &[u8]) -> f32 {
    let mut counts = [0usize; 256];
    let step = if data.len() > 4096 { 4 } else { 1 }; // Stride for speed
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

fn predictive_encode_v4(data: &[u8], lossy: Option<u8>, weights: Option<&[u8]>) -> Vec<u8> {
    #[cfg(feature = "std")]
    println!("DEBUG: Running Optimized Encoder");
    // Lazy Mixer Update batch size - weights update every N bytes
    const UPDATE_BATCH_SIZE: usize = 32;

    // 1. Initialize Engines
    let mut linear = 0u8;
    let mut simple = SimplePredictor::new();
    let mut graph = GraphPredictor::new();
    let mut spectral = SpectralPredictor::new(2048);

    let mut lz_match = LzMatchPredictor::new();
    let mut transformer = TransformerPredictor::new();

    // 2. Initialize V4 Mixer (Hybrid AR2 + Ensemble + FedProx)
    let (init_w, global_w) = if let Some(w_bytes) = weights {
        // Cast bytes to f32 slice assuming native endianness
        // Safety: We assume the caller passes a valid byte representation of [f32; N]
        let f32_count = w_bytes.len() / 4;
        if f32_count > 0 {
            let ptr = w_bytes.as_ptr() as *const f32;
            let slice = unsafe { core::slice::from_raw_parts(ptr, f32_count) };

            if f32_count >= 2 * NUM_MODELS {
                (
                    Some(&slice[0..NUM_MODELS]),
                    Some(&slice[NUM_MODELS..2 * NUM_MODELS]),
                )
            } else if f32_count >= NUM_MODELS {
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

    // 3. Initialize Range Encoder (Lazy ANS)
    let mut ans = AnsWriter::new();

    // Prepare quantization factor
    let q_factor = lossy.unwrap_or(1).max(1) as i8;

    let mut preds = [0u8; 6];
    let mut batch_counter = 0usize;

    for &actual in data {
        // A. Predict
        preds[0] = linear;
        preds[1] = simple.predict_next();
        preds[2] = graph.predict_next();
        preds[3] = spectral.predict();
        preds[4] = lz_match.predict_next();
        preds[5] = transformer.predict_next();

        // B. Mix (V4: Dynamic AR2 Switching happens inside mix())
        let mixed_prediction = mixer.mix(&preds);

        // C. Calculate Residual
        let base_residual = actual.wrapping_sub(mixed_prediction) as i8;

        // --- Rate-Distortion Optimization (RDO) ---
        // If lossy, quantize residual to reduce entropy.
        let residual = if q_factor > 1 {
            (base_residual / q_factor) * q_factor
        } else {
            base_residual
        };

        // D. Encode Residual
        ans.write_residual(residual);

        // E. Reconstruct value for predictor updates
        let reconstructed = mixed_prediction.wrapping_add(residual as u8);

        // F. Lazy Mixer Update - only update mixer weights every N bytes
        batch_counter += 1;
        if batch_counter >= UPDATE_BATCH_SIZE {
            // Use current sample as representative for the batch
            mixer.update_lazy(UPDATE_BATCH_SIZE, reconstructed, &preds);
            batch_counter = 0;
        }

        // G. Update individual predictors EVERY byte (they're cheap and need context)
        linear = reconstructed;
        simple.update(reconstructed);
        graph.update(reconstructed);
        spectral.update(reconstructed);
        lz_match.update(reconstructed);
        transformer.update(reconstructed);
    }

    // H. Final batch update for remaining bytes
    if batch_counter > 0 {
        mixer.update_lazy(batch_counter, linear, &preds);
    }

    // I. Finish (Seal the stream)
    ans.finish()
}

fn predictive_decode_v4(
    compressed_words: &[u8],
    decoded_len: usize,
    weights: Option<&[u8]>,
) -> Vec<u8> {
    // Lazy Mixer Update batch size - MUST match encoder
    const UPDATE_BATCH_SIZE: usize = 32;

    // 1. Initialize Engines
    let mut linear = 0u8;
    let mut simple = SimplePredictor::new();
    let mut graph = GraphPredictor::new();
    let mut spectral = SpectralPredictor::new(2048);

    let mut lz_match = LzMatchPredictor::new();
    let mut transformer = TransformerPredictor::new();

    // Setup Mixer weights
    let (init_w, global_w) = if let Some(w_bytes) = weights {
        let f32_count = w_bytes.len() / 4;
        if f32_count > 0 {
            let ptr = w_bytes.as_ptr() as *const f32;
            let slice = unsafe { core::slice::from_raw_parts(ptr, f32_count) };

            if f32_count >= 2 * NUM_MODELS {
                (
                    Some(&slice[0..NUM_MODELS]),
                    Some(&slice[NUM_MODELS..2 * NUM_MODELS]),
                )
            } else if f32_count >= NUM_MODELS {
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

    // 2. Initialize Range Decoder
    let mut ans = AnsReader::new(compressed_words);

    let mut out = Vec::with_capacity(decoded_len);
    let mut preds = [0u8; 6];
    let mut batch_counter = 0usize;

    for _ in 0..decoded_len {
        // A. Predict
        preds[0] = linear;
        preds[1] = simple.predict_next();
        preds[2] = graph.predict_next();
        preds[3] = spectral.predict();
        preds[4] = lz_match.predict_next();
        preds[5] = transformer.predict_next();

        // B. Mix
        let mixed_prediction = mixer.mix(&preds);

        // C. Read Residual
        let residual = ans.read_residual();

        // D. Reconstruct
        let actual = mixed_prediction.wrapping_add(residual as u8);
        out.push(actual);

        // E. Lazy Mixer Update - must match encoder exactly
        batch_counter += 1;
        if batch_counter >= UPDATE_BATCH_SIZE {
            mixer.update_lazy(UPDATE_BATCH_SIZE, actual, &preds);
            batch_counter = 0;
        }

        // F. Update individual predictors EVERY byte
        linear = actual;
        simple.update(actual);
        graph.update(actual);
        spectral.update(actual);
        lz_match.update(actual);
        transformer.update(actual);
    }

    // G. Final batch update for remaining bytes (must match encoder)
    if batch_counter > 0 {
        mixer.update_lazy(batch_counter, linear, &preds);
    }

    out
}

pub fn compress_chunk(
    chunk: &[u8],
    _predictor_id: u8,
    _weights: Option<&[u8]>,
    _lossy: Option<u8>,
) -> Result<Vec<u8>> {
    // 1. SAFETY CHECK: Validate Predictor ID
    if _predictor_id > PREDICTOR_ID_SPLIT {
        return Err(QresError::InvalidInput(format!(
            "Unsupported Predictor ID: {}",
            _predictor_id
        )));
    }

    // 0. Interleave Detection (Smart Pre-Pass)
    if chunk.len() > 1024 {
        let n = chunk.len().min(4096);
        let mut diff1 = 0i64;
        let mut diff2 = 0i64;

        for i in 2..n {
            diff1 += (chunk[i] as i64 - chunk[i - 1] as i64).abs();
            diff2 += (chunk[i] as i64 - chunk[i - 2] as i64).abs();
        }

        // If Lag-2 variation is significantly lower than Lag-1, it's interleaved
        if diff2 < (diff1 as f64 * 0.7) as i64 {
            let mut even = Vec::with_capacity(chunk.len() / 2 + 1);
            let mut odd = Vec::with_capacity(chunk.len() / 2 + 1);
            for (i, &b) in chunk.iter().enumerate() {
                if i % 2 == 0 {
                    even.push(b);
                } else {
                    odd.push(b);
                }
            }

            // Recursive compression
            let c_even = compress_chunk(&even, 0, _weights, _lossy)?;
            let c_odd = compress_chunk(&odd, 0, _weights, _lossy)?;

            // Flag 0x03: Interleaved Split
            // Structure: [Flag] [TotalLen: 4] [EvenLen: 4] [EvenData] [OddData]
            let total_len = chunk.len() as u32;
            let even_compressed_len = c_even.len() as u32;

            // Heuristic: Only use split if it actually compresses better than original
            if (c_even.len() + c_odd.len() + 9) < chunk.len() {
                // Wrap with VERSIONED Header
                // Mode 0x03: Interleaved
                let ver = QRES_PROTOCOL_VERSION & 0x0F;
                let flag_byte = (ver << 4) | 0x03;

                let mut out = Vec::with_capacity(9 + c_even.len() + c_odd.len());
                out.push(flag_byte);
                out.extend_from_slice(&total_len.to_le_bytes());
                out.extend_from_slice(&even_compressed_len.to_le_bytes());
                out.extend_from_slice(&c_even);
                out.extend_from_slice(&c_odd);
                return Ok(out);
            }
        }
    }

    // 1. Smart Fallback Pre-scan (ZSTD)
    // 1. Pre-scan Check
    // If entropy is extremely high (random), we might skip or warn.
    // For no_std/Core purification, we do NOT fallback to Zstd.
    // We let the caller decide if the result is larger than input.

    // 2. Prepare Weights (Neural vs Static)
    // (Assuming standard logic for effective_weights...)
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
            let b = 0.0f32.to_le_bytes(); // NOTE: These serve as placeholders, actual math is now i32
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

    // [Handling Global weights...]
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

    // 3. Encode
    let compressed_body = predictive_encode_v4(chunk, _lossy, w_arg);

    // 4. Wrap with VERSIONED Header
    if compressed_body.len() < chunk.len() {
        // Flag Layout: [7-4: Version] [3-0: Codec Mode]
        // Mode 0x00: Standard
        // Mode 0x02: Neural (includes weights)
        let mode = if is_neural { 0x02 } else { 0x00 };

        // Safety: Ensure version fits in 4 bits
        let ver = QRES_PROTOCOL_VERSION & 0x0F;
        let flag_byte = (ver << 4) | mode;

        let mut out = Vec::with_capacity(1 + 4 + stored_init_weights.len() + compressed_body.len());

        out.push(flag_byte);
        out.extend_from_slice(&(chunk.len() as u32).to_le_bytes());

        if is_neural {
            out.extend_from_slice(&stored_init_weights);
        }

        out.extend_from_slice(&compressed_body);
        Ok(out)
    } else {
        // Core Purification: If we expand, return specific error so Daemon can handle fallback.
        // Or return Expanded variant if we had an Enum return type.
        // For now, Err(CompressionError("Expansion detected")) allows Daemon to catch and fallback.
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

    // 1. SAFETY CHECK: Protocol Version
    // We can allow backward compatibility (e.g., allow v9 if we are v10),
    // but for now strict matching ensures safety during dev.
    if version != (QRES_PROTOCOL_VERSION & 0x0F) {
        // Graceful fallback for legacy files (pre-handshake) could go here
        // But for Engineering Phase 1, we fail fast.
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
            // ANS codec (V4)
            Ok(predictive_decode_v4(&compressed[5..], decomp_len, _weights))
        }
        0x01 => {
            // Zstd fallback - Not supported in Core Purification
            Err(QresError::CompressionError(String::from(
                "Zstd decompression not supported in pure Core",
            )))
        }
        0x02 => {
            // ANS codec with Neural Init
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
            // Interleaved Split
            if compressed.len() < 9 {
                return Err(QresError::InvalidData(String::from(
                    "Split chunk too short",
                )));
            }
            let even_len = u32::from_le_bytes(compressed[5..9].try_into().unwrap()) as usize;
            let even_data = &compressed[9..9 + even_len];
            let odd_data = &compressed[9 + even_len..];

            // Recursive calls must handle the header too!
            let even_decomp = decompress_chunk(even_data, 0, _weights)?;
            let odd_decomp = decompress_chunk(odd_data, 0, _weights)?;

            let mut out = Vec::with_capacity(decomp_len);
            let mut e_iter = even_decomp.iter();
            let mut o_iter = odd_decomp.iter();

            for _ in 0..decomp_len / 2 {
                if let Some(b) = e_iter.next() {
                    out.push(*b);
                }
                if let Some(b) = o_iter.next() {
                    out.push(*b);
                }
            }
            if let Some(b) = e_iter.next() {
                out.push(*b);
            }

            Ok(out)
        }
        _ => Err(QresError::InvalidData(format!(
            "Unknown codec mode: {:#x}",
            codec_mode
        ))),
    }
}

// --- File-Level Compression with Callback (for GUI) ---

#[cfg(feature = "std")]
use std::fs::File;
#[cfg(feature = "std")]
use std::io;
#[cfg(feature = "std")]
use std::io::{BufReader, BufWriter, Read, Write};
#[cfg(feature = "std")]
use std::path::Path;

#[cfg(feature = "std")]
const FILE_CHUNK_SIZE: usize = 64 * 1024; // 64KB chunks

/// Compress a file with progress callback
/// callback receives: (progress_percent, current_ratio, engine_name)
#[cfg(feature = "std")]
pub fn compress_with_callback<P, C>(src: P, dest: P, mut callback: C) -> io::Result<()>
where
    P: AsRef<Path>,
    C: FnMut(u32, f64, &str),
{
    let src_file = File::open(src.as_ref())?;
    let src_size = src_file.metadata()?.len();
    let mut reader = BufReader::new(src_file);

    let dest_file = File::create(dest.as_ref())?;
    let mut writer = BufWriter::new(dest_file);

    // Write header magic
    writer.write_all(b"QRES")?;

    // Write static header placeholder (will be updated at end)
    let _header_position = 4;
    let header = [0u8; 27]; // Version, flags, pred_id, timestamps, sizes
    writer.write_all(&header)?;

    // Write filename
    let filename = src
        .as_ref()
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("data")
        .as_bytes();
    writer.write_all(&(filename.len() as u32).to_le_bytes())?;
    writer.write_all(filename)?;

    let mut buffer = vec![0u8; FILE_CHUNK_SIZE];
    let mut total_read = 0u64;
    let mut total_written = 0u64;

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let chunk = &buffer[..bytes_read];
        let compressed = compress_chunk(chunk, 0, None, None)?;

        // Write chunk length + data
        writer.write_all(&(compressed.len() as u32).to_le_bytes())?;
        writer.write_all(&compressed)?;

        total_read += bytes_read as u64;
        total_written += compressed.len() as u64 + 4;

        // Calculate progress and ratio
        let progress = if src_size > 0 {
            ((total_read as f64 / src_size as f64) * 100.0) as u32
        } else {
            50
        };
        let ratio = if total_read > 0 {
            total_written as f64 / total_read as f64
        } else {
            1.0
        };

        callback(progress, ratio, "Neural");
    }

    writer.flush()?;

    // Final callback
    callback(
        100,
        total_written as f64 / total_read.max(1) as f64,
        "Neural",
    );

    Ok(())
}

// --- Python Wrapper (PyO3 0.22+) ---

#[cfg(feature = "python")]
use pyo3::types::PyModule;

#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(signature = (data, predictor_id=0, weights=None))]
fn encode_bytes(
    py: Python<'_>,
    data: &[u8],
    predictor_id: u8,
    weights: Option<&[u8]>,
) -> PyResult<Py<pyo3::types::PyBytes>> {
    let compressed = compress_chunk(data, predictor_id, weights, None)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    Ok(pyo3::types::PyBytes::new_bound(py, &compressed).unbind())
}

#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(signature = (data, predictor_id=0, weights=None))]
fn decode_bytes(
    py: Python<'_>,
    data: &[u8],
    predictor_id: u8,
    weights: Option<&[u8]>,
) -> PyResult<Py<pyo3::types::PyBytes>> {
    let decompressed = decompress_chunk(data, predictor_id, weights)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    Ok(pyo3::types::PyBytes::new_bound(py, &decompressed).unbind())
}

#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(signature = (_data, _predictor_id=0, _weights=None))]
fn get_residuals_py(_data: &[u8], _predictor_id: u8, _weights: Option<&[u8]>) -> PyResult<Vec<i8>> {
    Ok(Vec::new())
}

#[cfg(feature = "python")]
#[pyfunction]
fn compress_matrix_v1(
    data: Vec<f64>,
    rows: usize,
    cols: usize,
    threshold: f64,
) -> PyResult<Vec<f64>> {
    let compressor = tensor::MpsCompressor::new(10, threshold);
    let cores = compressor.compress_matrix(&data, rows, cols);
    if let Some(first_core) = cores.first() {
        Ok(first_core.clone())
    } else {
        Ok(vec![])
    }
}

#[cfg(feature = "python")]
#[pymodule]
fn qres_rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(encode_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(decode_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(get_residuals_py, m)?)?;
    m.add_function(wrap_pyfunction!(compress_matrix_v1, m)?)?;
    Ok(())
}

// --- Tauri Interface ---
// Removed from Core. Use qres_core::compress_chunk directly.

// --- WASM Interface ---
// #[cfg(target_arch = "wasm32")]
// pub mod wasm {
//     use crate::{compress_chunk, decompress_chunk};
//     use wasm_bindgen::prelude::*;
//
//     #[wasm_bindgen]
//     pub fn compress(data: &[u8]) -> Result<Vec<u8>, JsValue> {
//         compress_chunk(data, 0, None, None).map_err(|e| JsValue::from_str(&e.to_string()))
//     }
//
//     #[wasm_bindgen]
//     pub fn decompress(data: &[u8]) -> Result<Vec<u8>, JsValue> {
//         decompress_chunk(data, 0, None).map_err(|e| JsValue::from_str(&e.to_string()))
//     }
// }
