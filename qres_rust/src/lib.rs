use serde::{Deserialize, Serialize};

use std::convert::TryInto;
use std::io;

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyBytes;

#[cfg(feature = "swarm")]
pub mod analytics;
#[cfg(feature = "swarm")]
pub mod config;
#[cfg(feature = "swarm")]
pub mod daemon;
// pub mod semantic; // Disabled: requires tokenizers crate
#[cfg(feature = "swarm")]
pub mod stats;
#[cfg(feature = "swarm")]
pub mod swarm;
#[cfg(feature = "swarm")]
pub mod swarm_p2p;

// --- v3.0/v4.0 Modules ---
pub mod ans_coder;
pub mod archive; // Archive container format
pub mod dedup; // Content-Defined Chunking & Deduplication
pub mod meta_brain;
pub mod mixer;
pub mod predictors;
pub mod spectral; // Task 6
pub use ans_coder::{AnsReader, AnsWriter};
use mixer::{Mixer, NUM_MODELS};
use predictors::{GraphPredictor, LzMatchPredictor, Predictor, SimplePredictor};
use spectral::SpectralPredictor; // Added Predictor
#[cfg(feature = "gpu")]
pub mod gpu;
pub mod quantum;
pub mod transformer;
use transformer::TransformerPredictor;

// --- Living Brain (Adaptive Learning) ---

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LivingBrain {
    pub version: u8,
    pub predictors: Vec<String>,
    pub stats: serde_json::Value,
    pub confidence: Vec<f32>,
    pub global_confidence: Option<Vec<f32>>, // Phase 2: FedProx Anchor
    pub best_engine_weights: Option<Vec<u8>>,
}

impl Default for LivingBrain {
    fn default() -> Self {
        Self::new()
    }
}

impl LivingBrain {
    pub fn new() -> Self {
        LivingBrain {
            version: 1,
            predictors: vec![
                "lstm".to_string(),
                "graph".to_string(),
                "transformer".to_string(),
            ],
            stats: serde_json::json!({"compressions": 0}),
            confidence: vec![0.5; NUM_MODELS.max(4)], // Ensure enough space
            global_confidence: None,
            best_engine_weights: None,
        }
    }

    pub fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or("{}".to_string())
    }

    pub fn merge(&mut self, other: &LivingBrain, alpha: f32) {
        for i in 0..self.confidence.len().min(other.confidence.len()) {
            self.confidence[i] = self.confidence[i] * (1.0 - alpha) + other.confidence[i] * alpha;
        }
        // Always derive global anchor from the imported brain (truth)
        if other.global_confidence.is_some() {
            self.global_confidence = other.global_confidence.clone();
        }
    }
}

const CHUNK_SIZE: usize = 1024 * 1024; // 1MB Chunk for better Text/LZ context
const QRES_MAGIC: &[u8] = b"QRES";

// Define constants to prevent future magic-number errors (Phantom Weight Fix)
const NUM_PREDICTORS: usize = 6;
const WEIGHTS_LEN: usize = NUM_PREDICTORS * 4; // 24 bytes for 6 f32 weights

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
            let slice = unsafe { std::slice::from_raw_parts(ptr, f32_count) };

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
            let slice = unsafe { std::slice::from_raw_parts(ptr, f32_count) };

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
) -> io::Result<Vec<u8>> {
    const HIGH_ENTROPY_THRESHOLD: f32 = 7.8;

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
            // Structure: [0x03] [TotalLen: 4] [EvenLen: 4] [EvenData] [OddData]
            let total_len = chunk.len() as u32;
            let even_compressed_len = c_even.len() as u32;

            // Heuristic: Only use split if it actually compresses better than original
            if (c_even.len() + c_odd.len() + 9) < chunk.len() {
                let mut out = Vec::with_capacity(9 + c_even.len() + c_odd.len());
                out.push(0x03);
                out.extend_from_slice(&total_len.to_le_bytes());
                out.extend_from_slice(&even_compressed_len.to_le_bytes());
                out.extend_from_slice(&c_even);
                out.extend_from_slice(&c_odd);
                return Ok(out);
            }
        }
    }

    // 1. Smart Fallback Pre-scan
    if chunk.len() > 512 {
        let entropy = calculate_sample_entropy(chunk);

        // Low entropy (constant/near-constant data) - zstd is much faster and better
        const LOW_ENTROPY_THRESHOLD: f32 = 0.2;
        if entropy < LOW_ENTROPY_THRESHOLD {
            let zstd_compressed = zstd::bulk::compress(chunk, 3).map_err(io::Error::other)?;
            if zstd_compressed.len() < chunk.len() {
                let mut out = Vec::with_capacity(5 + zstd_compressed.len());
                out.push(0x01); // Flag: Zstd
                out.extend_from_slice(&(chunk.len() as u32).to_le_bytes());
                out.extend_from_slice(&zstd_compressed);
                return Ok(out);
            }
        }

        // High entropy (random data) - also use zstd fallback
        if entropy > HIGH_ENTROPY_THRESHOLD {
            let zstd_compressed = zstd::bulk::compress(chunk, 3).map_err(io::Error::other)?;
            if zstd_compressed.len() < chunk.len() {
                let mut out = Vec::with_capacity(5 + zstd_compressed.len());
                out.push(0x01); // Flag: Zstd
                out.extend_from_slice(&(chunk.len() as u32).to_le_bytes());
                out.extend_from_slice(&zstd_compressed);
                return Ok(out);
            }
        }
    }

    // 2. Prepare Weights (Neural vs Static)
    let mut effective_weights = Vec::new();
    let mut is_neural = false;
    let mut stored_init_weights = Vec::new();

    // A. Init Weights
    if let Some(nw) = crate::meta_brain::predict_init_weights(chunk) {
        is_neural = true;
        // Ensure we serialize exactly WEIGHTS_LEN (24 bytes for 6 predictors)
        for f in nw.iter().take(NUM_PREDICTORS) {
            let b = f.to_le_bytes();
            stored_init_weights.extend_from_slice(&b);
            effective_weights.extend_from_slice(&b);
        }
        // Pad if meta_brain returned fewer weights
        while stored_init_weights.len() < WEIGHTS_LEN {
            let b = 0.0f32.to_le_bytes();
            stored_init_weights.extend_from_slice(&b);
            effective_weights.extend_from_slice(&b);
        }
    } else {
        // Fallback to LivingBrain init (RL Agent provided weights)
        if let Some(w) = _weights {
            // FIX: Take up to WEIGHTS_LEN (24 bytes), not 20
            let take = w.len().min(WEIGHTS_LEN);
            effective_weights.extend_from_slice(&w[0..take]);

            // If explicit weights provided, we treat it as "Neural" for storage
            if take > 0 {
                is_neural = true;
                stored_init_weights.extend_from_slice(&w[0..take]);
                // Pad if necessary
                while stored_init_weights.len() < WEIGHTS_LEN {
                    stored_init_weights.push(0);
                }
            }
        }
    }

    // B. Global Weights (FedProx) - Append if present
    // Assuming input is [Init(24) + Global(24)]
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

    // 4. Wrap
    if compressed_body.len() < chunk.len() {
        let flag = if is_neural { 0x02 } else { 0x00 };
        let mut out = Vec::with_capacity(1 + 4 + stored_init_weights.len() + compressed_body.len());

        out.push(flag);
        out.extend_from_slice(&(chunk.len() as u32).to_le_bytes());

        if is_neural {
            // Flag 0x02 stores WEIGHTS_LEN (24) bytes of init weights
            out.extend_from_slice(&stored_init_weights);
        }

        out.extend_from_slice(&compressed_body);
        Ok(out)
    } else {
        // Zstd Fallback
        let zstd_compressed = zstd::bulk::compress(chunk, 3).map_err(io::Error::other)?;
        let mut out = Vec::with_capacity(1 + 4 + zstd_compressed.len());
        out.push(0x01);
        out.extend_from_slice(&(chunk.len() as u32).to_le_bytes());
        out.extend_from_slice(&zstd_compressed);
        Ok(out)
    }
}

pub fn decompress_chunk(
    compressed: &[u8],
    _predictor_id: u8,
    _weights: Option<&[u8]>,
) -> io::Result<Vec<u8>> {
    if compressed.len() < 5 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Chunk too short",
        ));
    }

    let codec_flag = compressed[0];

    let decomp_len = u32::from_le_bytes(
        compressed[1..5]
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid Header"))?,
    ) as usize;

    match codec_flag {
        0x00 => {
            // ANS codec (V4)
            Ok(predictive_decode_v4(&compressed[5..], decomp_len, _weights))
        }
        0x01 => {
            // Zstd fallback
            zstd::bulk::decompress(&compressed[5..], decomp_len).map_err(io::Error::other)
        }
        0x02 => {
            // ANS codec with Neural Init (V5+)
            // FIX: Header size check must account for WEIGHTS_LEN (24), not 20
            let header_size = 5 + WEIGHTS_LEN; // 5 + 24 = 29 bytes
            if compressed.len() < header_size {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Chunk too short for Neural Header",
                ));
            }
            let init_w_bytes = &compressed[5..header_size]; // 6 floats * 4 bytes = 24

            // Reconstruct effective weights: [Neural Init] + [Global from Args]
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
            // Interleaved Split (V7)
            // Structure: [Flag] [TotalLen:4] [EvenLen:4] [EvenData] [OddData]
            if compressed.len() < 9 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Split chunk too short",
                ));
            }
            let even_len = u32::from_le_bytes(compressed[5..9].try_into().unwrap()) as usize;

            let even_data = &compressed[9..9 + even_len];
            let odd_data = &compressed[9 + even_len..];

            let even_decomp = decompress_chunk(even_data, 0, _weights)?;
            let odd_decomp = decompress_chunk(odd_data, 0, _weights)?;

            // Re-interleave
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
            // Handle residual if odd length (though IoT usually pairs)
            if let Some(b) = e_iter.next() {
                out.push(*b);
            }

            Ok(out)
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unknown codec flag: {:#x}", codec_flag),
        )),
    }
}

// --- Python Wrapper ---

#[cfg(feature = "python")]
#[pyfunction]
fn encode_bytes<'a>(
    py: Python<'a>,
    data: &[u8],
    _predictor_id: u8,
    _weights: Option<&[u8]>,
) -> PyResult<&'a PyBytes> {
    let compressed = compress_chunk(data, _predictor_id, _weights, None)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    Ok(PyBytes::new(py, &compressed))
}

#[cfg(feature = "python")]
#[pyfunction]
fn decode_bytes<'a>(
    py: Python<'a>,
    data: &[u8],
    _predictor_id: u8,
    _weights: Option<&[u8]>,
) -> PyResult<&'a PyBytes> {
    let decompressed = decompress_chunk(data, _predictor_id, _weights)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    Ok(PyBytes::new(py, &decompressed))
}

#[cfg(feature = "python")]
#[pyfunction]
fn get_residuals_py<'a>(
    _py: Python<'a>,
    _data: &[u8],
    _predictor_id: u8,
    _weights: Option<&[u8]>,
) -> PyResult<Vec<i8>> {
    Ok(Vec::new())
}

#[cfg(feature = "python")]
#[pyfunction]
fn compress_matrix_v1(
    _py: Python,
    data: Vec<f64>,
    rows: usize,
    cols: usize,
    threshold: f64,
) -> PyResult<Vec<f64>> {
    let compressor = quantum::MpsCompressor::new(10, threshold);
    // MPS returns Vec<Vec<f64>> (cores). We flatten for v1 prototype.
    let cores = compressor.compress_matrix(&data, rows, cols);
    if let Some(first_core) = cores.first() {
        Ok(first_core.clone())
    } else {
        Ok(vec![])
    }
}

#[cfg(feature = "python")]
#[pymodule]
fn qres_rust(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(encode_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(decode_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(get_residuals_py, m)?)?;
    m.add_function(wrap_pyfunction!(compress_matrix_v1, m)?)?;
    Ok(())
}

// --- Tauri Interface ---

pub fn compress_with_callback<F>(
    src: &str,
    dest: &str,
    mut callback: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut(f32, f32, &str),
{
    use std::fs::File;
    use std::io::{Read, Seek, Write};

    let mut input = File::open(src)?;
    let mut output = File::create(dest)?;

    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut total_read = 0u64;
    let file_size = input.metadata()?.len();

    output.write_all(QRES_MAGIC)?;
    output.write_all(&[4u8])?;
    output.write_all(&[0u8])?;
    output.write_all(&[0u8])?;
    output.write_all(&chrono::Utc::now().timestamp().to_le_bytes())?;
    output.write_all(&file_size.to_le_bytes())?;
    output.write_all(&(0u64.to_le_bytes()))?;
    output.write_all(&(src.len() as u32).to_le_bytes())?;
    output.write_all(src.as_bytes())?;

    let mut compressed_size = 0u64;

    loop {
        let bytes_read = input.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let chunk = &buffer[..bytes_read];
        let compressed = compress_chunk(chunk, 0, None, None)?;

        output.write_all(&(compressed.len() as u32).to_le_bytes())?;
        output.write_all(&compressed)?;

        total_read += bytes_read as u64;
        compressed_size += compressed.len() as u64 + 4;

        let progress = (total_read as f32 / file_size as f32) * 100.0;
        let ratio = compressed_size as f32 / total_read as f32;

        callback(progress, ratio, "predictive");
    }

    output.seek(std::io::SeekFrom::Start(
        (QRES_MAGIC.len() + 1 + 1 + 1 + 8) as u64,
    ))?;
    output.write_all(&compressed_size.to_le_bytes())?;

    Ok(())
}

// --- WASM Interface ---
#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use crate::{compress_chunk, decompress_chunk};
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn compress(data: &[u8]) -> Result<Vec<u8>, JsValue> {
        compress_chunk(data, 0, None, None).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn decompress(data: &[u8]) -> Result<Vec<u8>, JsValue> {
        decompress_chunk(data, 0, None).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
