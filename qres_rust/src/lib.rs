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

// --- v3.0 Modules ---
pub mod ans_coder;
mod mixer;
pub use ans_coder::{AnsReader, AnsWriter};
use mixer::Mixer;

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
            predictors: vec!["lstm".to_string(), "ipeps".to_string()],
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

// --- Predictor Logic ---

pub struct SimplePredictor {
    prev: f32,
    prev2: f32,
    context: HashMap<u8, u8>, // Order-1 context: prev -> next
}

impl SimplePredictor {
    pub fn new(_weights: Option<&[u8]>) -> Self {
        SimplePredictor {
            prev: 0.0,
            prev2: 0.0,
            context: HashMap::new(),
        }
    }

    fn predict_next(&self) -> u8 {
        // Use order-1 context if available, else previous value
        self.context
            .get(&(self.prev as u8))
            .copied()
            .unwrap_or(self.prev as u8)
    }

    pub fn update(&mut self, actual: u8) {
        // Learn order-1 context
        self.context.insert(self.prev as u8, actual);
        self.prev2 = self.prev;
        self.prev = actual as f32;
    }
}

// Real Matrix Product State (MPS) Engine
struct IpepsPredictor {
    psi: Array1<f32>,
}

impl IpepsPredictor {
    fn new(_weights: Option<&[u8]>) -> Self {
        let weights: Vec<f32> = bincode::deserialize(include_bytes!("../assets/ipeps.qnn"))
            .unwrap_or(vec![1.0, 0.0, 0.0, 0.0]);
        let psi = Array1::from_vec(weights);
        IpepsPredictor { psi }
    }

    fn predict_next(&self) -> u8 {
        let prob = self.psi[0].abs();
        let val = (prob * 255.0).clamp(0.0, 255.0);
        val as u8
    }

    fn update_state(&mut self, actual: u8) {
        let theta = (actual as f32 / 255.0) * std::f32::consts::PI;
        let (sin, cos) = theta.sin_cos();

        let p = &self.psi;
        let new_0 = p[0] * cos - p[1] * sin;
        let new_1 = p[0] * sin + p[1] * cos;
        let new_2 = p[1];
        let new_3 = p[2];

        let norm = (new_0 * new_0 + new_1 * new_1 + new_2 * new_2 + new_3 * new_3).sqrt();
        let scale = if norm > 1e-6 { 1.0 / norm } else { 1.0 };

        self.psi = Array1::from_vec(vec![
            new_0 * scale,
            new_1 * scale,
            new_2 * scale,
            new_3 * scale,
        ]);
    }
}

// --- V3 Encoding Logic ---

fn predictive_encode_v3(data: &[u8]) -> Vec<u8> {
    // 1. Initialize Engines
    let mut linear = 0u8;
    let mut simple = SimplePredictor::new(None);
    let mut ipeps = IpepsPredictor::new(None);

    // 2. Initialize Mixer
    let mut mixer = Mixer::new();

    // 3. Initialize Range Encoder (Constriction)
    let mut ans = AnsWriter::new();

    let mut preds = [0u8; 3];

    for &actual in data {
        // A. Predict
        preds[0] = linear;
        preds[1] = simple.predict_next();
        preds[2] = ipeps.predict_next();

        // B. Mix
        let mixed_prediction = mixer.mix(&preds);

        // C. Calculate Residual
        let residual = actual.wrapping_sub(mixed_prediction) as i8;

        // D. Encode Residual
        ans.write_residual(residual);

        // E. Update
        mixer.update(actual, &preds);
        linear = actual;
        simple.update(actual);
        ipeps.update_state(actual);
    }

    // F. Finish (Seal the stream)
    ans.finish()
}

fn predictive_decode_v3(compressed_words: &[u8], decoded_len: usize) -> Vec<u8> {
    // 1. Initialize Engines
    let mut linear = 0u8;
    let mut simple = SimplePredictor::new(None);
    let mut ipeps = IpepsPredictor::new(None);
    let mut mixer = Mixer::new();

    // 2. Initialize Range Decoder
    let mut ans = AnsReader::new(compressed_words);

    let mut out = Vec::with_capacity(decoded_len);
    let mut preds = [0u8; 3];

    for _ in 0..decoded_len {
        // A. Predict
        preds[0] = linear;
        preds[1] = simple.predict_next();
        preds[2] = ipeps.predict_next();

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
        ipeps.update_state(actual);
    }

    out
}

pub fn compress_chunk(
    chunk: &[u8],
    _predictor_id: u8,
    _weights: Option<&[u8]>,
    _lossy: Option<u8>,
) -> io::Result<Vec<u8>> {
    // Try ANS compression first
    let compressed_body = predictive_encode_v3(chunk);

    // [V3 Format]: [Flags (1 byte)] + [Decompressed_Len (4 bytes)] + [Compressed_Body]
    // Flags: bit 0 = codec (0=ANS, 1=Zstd fallback)

    // Check if ANS achieved compression
    if compressed_body.len() < chunk.len() {
        // ANS succeeded - use it
        let mut out = Vec::with_capacity(1 + 4 + compressed_body.len());
        out.push(0x00); // Flag: ANS codec
        out.extend_from_slice(&(chunk.len() as u32).to_le_bytes());
        out.extend_from_slice(&compressed_body);
        Ok(out)
    } else {
        // ANS expanded data - fall back to zstd
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

    // Extract codec flag
    let codec_flag = compressed[0];

    // Extract Decompressed Length
    let decomp_len = u32::from_le_bytes(
        compressed[1..5]
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid Header"))?,
    ) as usize;

    // Decode based on codec
    match codec_flag {
        0x00 => {
            // ANS codec
            Ok(predictive_decode_v3(&compressed[5..], decomp_len))
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
    // Wrapper simply calls compress_chunk which now handles all headers internally.
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

    // Write header
    output.write_all(QRES_MAGIC)?;
    output.write_all(&[3u8])?; // version
    output.write_all(&[0u8])?; // flags
    output.write_all(&[0u8])?; // predictor_id
    output.write_all(&chrono::Utc::now().timestamp().to_le_bytes())?;
    output.write_all(&file_size.to_le_bytes())?;
    output.write_all(&(0u64.to_le_bytes()))?; // compressed_size placeholder
    output.write_all(&(src.len() as u32).to_le_bytes())?;
    output.write_all(src.as_bytes())?;

    let mut compressed_size = 0u64;
    let _header_size = QRES_MAGIC.len() + 1 + 1 + 1 + 8 + 8 + 8 + 4 + src.len();

    loop {
        let bytes_read = input.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let chunk = &buffer[..bytes_read];
        let compressed = compress_chunk(chunk, 0, None, None)?;

        // Write chunk size
        output.write_all(&(compressed.len() as u32).to_le_bytes())?;
        output.write_all(&compressed)?;

        total_read += bytes_read as u64;
        compressed_size += compressed.len() as u64 + 4;

        let progress = (total_read as f32 / file_size as f32) * 100.0;
        let ratio = compressed_size as f32 / total_read as f32;

        callback(progress, ratio, "predictive");
    }

    // Update compressed_size in header
    output.seek(std::io::SeekFrom::Start(
        (QRES_MAGIC.len() + 1 + 1 + 1 + 8) as u64,
    ))?;
    output.write_all(&compressed_size.to_le_bytes())?;

    Ok(())
}
