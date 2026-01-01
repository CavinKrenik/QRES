use ndarray::Array1;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;
use std::io;

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyBytes;

pub mod analytics;
pub mod config;
pub mod daemon;
pub mod semantic;
pub mod stats;
pub mod swarm;

// --- v3.0/v4.0 Modules ---
pub mod ans_coder;
mod mixer;
pub mod spectral;
pub mod predictors; // Task 6
pub use ans_coder::{AnsReader, AnsWriter};
use mixer::Mixer;
use spectral::SpectralPredictor;
use predictors::{SimplePredictor, GraphPredictor}; // Replaces IpepsPredictor

// --- Living Brain (Adaptive Learning) ---

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LivingBrain {
    pub version: u8,
    pub predictors: Vec<String>,
    pub stats: serde_json::Value,
    pub confidence: Vec<f32>,
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
            predictors: vec!["lstm".to_string(), "graph".to_string()], // Updated ipeps->graph
            stats: serde_json::json!({"compressions": 0}),
            confidence: vec![0.5; 4],
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
    }
}

const CHUNK_SIZE: usize = 64 * 1024; // 64KB
const QRES_MAGIC: &[u8] = b"QRES";

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

fn predictive_encode_v4(data: &[u8], lossy: Option<u8>) -> Vec<u8> {
    // 1. Initialize Engines
    let mut linear = 0u8;
    let mut simple = SimplePredictor::new();
    let mut graph = GraphPredictor::new();
    let mut spectral = SpectralPredictor::new(2048);

    // 2. Initialize V4 Mixer (Hybrid AR2 + Ensemble)
    let mut mixer = Mixer::new();

    // 3. Initialize Range Encoder (Lazy ANS)
    let mut ans = AnsWriter::new();
    
    // Prepare quantization factor
    let q_factor = lossy.unwrap_or(1).max(1) as i8;

    let mut preds = [0u8; 4];

    for &actual in data {
        // A. Predict
        preds[0] = linear;
        preds[1] = simple.predict_next();
        preds[2] = graph.predict_next();
        preds[3] = spectral.predict();

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

        // D. Encode Residual (V4: Lazy Batched Updates)
        ans.write_residual(residual);

        // E. Update
        // Crucial: Update models with the *reconstructed* value, not the original 'actual',
        // to prevent drift (encoder/decoder desync).
        // Recalculate actual from prediction + quantized residual.
        let reconstructed = mixed_prediction.wrapping_add(residual as u8);
        
        mixer.update(reconstructed, &preds);
        linear = reconstructed;
        simple.update(reconstructed);
        graph.update(reconstructed);
        spectral.update(reconstructed);
    }

    // F. Finish (Seal the stream)
    ans.finish()
}

fn predictive_decode_v4(compressed_words: &[u8], decoded_len: usize) -> Vec<u8> {
    // 1. Initialize Engines
    let mut linear = 0u8;
    let mut simple = SimplePredictor::new();
    let mut graph = GraphPredictor::new();
    let mut spectral = SpectralPredictor::new(2048);
    let mut mixer = Mixer::new();

    // 2. Initialize Range Decoder
    let mut ans = AnsReader::new(compressed_words);

    let mut out = Vec::with_capacity(decoded_len);
    let mut preds = [0u8; 4];

    for _ in 0..decoded_len {
        // A. Predict
        preds[0] = linear;
        preds[1] = simple.predict_next();
        preds[2] = graph.predict_next();
        preds[3] = spectral.predict();

        // B. Mix
        let mixed_prediction = mixer.mix(&preds);

        // C. Read Residual
        let residual = ans.read_residual();

        // D. Reconstruct
        let actual = mixed_prediction.wrapping_add(residual as u8);
        out.push(actual);

        // E. Update
        mixer.update(actual, &preds);
        linear = actual;
        simple.update(actual);
        graph.update(actual);
        spectral.update(actual);
    }

    out
}

pub fn compress_chunk(
    chunk: &[u8],
    _predictor_id: u8,
    _weights: Option<&[u8]>,
    _lossy: Option<u8>,
) -> io::Result<Vec<u8>> {
    const HIGH_ENTROPY_THRESHOLD: f32 = 7.5; // Max is 8.0.

    // 1. Smart Fallback Pre-scan
    if chunk.len() > 512 {
         let entropy = calculate_sample_entropy(chunk);
         if entropy > HIGH_ENTROPY_THRESHOLD {
             let zstd_compressed = zstd::bulk::compress(chunk, 3).map_err(io::Error::other)?;
             
             if zstd_compressed.len() < chunk.len() {
                 let mut out = Vec::with_capacity(1 + 4 + zstd_compressed.len());
                 out.push(0x01); // Flag: Zstd
                 out.extend_from_slice(&(chunk.len() as u32).to_le_bytes());
                 out.extend_from_slice(&zstd_compressed);
                 return Ok(out);
             } else {
                 let mut out = Vec::with_capacity(1 + 4 + zstd_compressed.len());
                 out.push(0x01); 
                 out.extend_from_slice(&(chunk.len() as u32).to_le_bytes());
                 out.extend_from_slice(&zstd_compressed);
                 return Ok(out);
             }
         }
    }

    // 2. Try ANS (Neural/Predictive) Compression
    let compressed_body = predictive_encode_v4(chunk, _lossy);

    if compressed_body.len() < chunk.len() {
        let mut out = Vec::with_capacity(1 + 4 + compressed_body.len());
        out.push(0x00); // Flag: ANS codec
        out.extend_from_slice(&(chunk.len() as u32).to_le_bytes());
        out.extend_from_slice(&compressed_body);
        Ok(out)
    } else {
        let zstd_compressed = zstd::bulk::compress(chunk, 3).map_err(io::Error::other)?;
        let mut out = Vec::with_capacity(1 + 4 + zstd_compressed.len());
        out.push(0x01); // Flag: Zstd fallback
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
            Ok(predictive_decode_v4(&compressed[5..], decomp_len))
        }
        0x01 => {
            // Zstd fallback
            zstd::bulk::decompress(&compressed[5..], decomp_len).map_err(io::Error::other)
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
    let compressed = compress_chunk(data, 0, None, None)
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
    let decompressed = decompress_chunk(data, 0, None)
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
#[pymodule]
fn qres_rust(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(encode_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(decode_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(get_residuals_py, m)?)?;
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
