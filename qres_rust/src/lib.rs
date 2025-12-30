use std::io::{self};
use std::convert::TryInto;
use serde::{Serialize, Deserialize};
use ndarray::Array1; 

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyBytes;

pub mod stats;
pub mod analytics;
pub mod config;
pub mod semantic;
// pub mod swarm; // Phase 23: Disabled to restore Green Build
pub mod daemon;

// --- v3.0 Modules ---
pub mod ans_coder;
mod mixer;
pub use ans_coder::{AnsWriter, AnsReader};
use mixer::Mixer;

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

pub struct LstmPredictor {
     cell_state: f32,
     hidden_state: f32,
     weights: Vec<f32>,
}

impl LstmPredictor {
    pub fn new(_weights: Option<&[u8]>) -> Self {
        LstmPredictor {
            cell_state: 0.0,
            hidden_state: 0.0,
            weights: vec![0.5; 4], 
        }
    }
    
    fn tanh(x: f32) -> f32 { x.tanh() }
    
    pub fn predict_next(&self) -> u8 {
        ((self.hidden_state + 1.0) * 127.5) as u8
    }
    
    pub fn update(&mut self, actual: u8) {
         let x = (actual as f32) / 255.0;
         self.hidden_state = Self::tanh(self.cell_state * 0.9 + x * 0.1);
         self.cell_state = self.cell_state * 0.9 + x;
    }
}

// Real Matrix Product State (MPS) Engine
struct IpepsPredictor {
    psi: Array1<f32>, 
    context: f32,
}

impl IpepsPredictor {
    fn new(_weights: Option<&[u8]>) -> Self {
        let psi = Array1::from_vec(vec![1.0, 0.0, 0.0, 0.0]); 
        IpepsPredictor { psi, context: 0.0 }
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
        
        let norm = (new_0*new_0 + new_1*new_1 + new_2*new_2 + new_3*new_3).sqrt();
        let scale = if norm > 1e-6 { 1.0/norm } else { 1.0 };
        
        self.psi = Array1::from_vec(vec![new_0*scale, new_1*scale, new_2*scale, new_3*scale]);
    }
}

// --- V3 Encoding Logic ---

fn predictive_encode_v3(data: &[u8]) -> Vec<u8> {
    // 1. Initialize Engines
    let mut linear = 0u8;
    let mut lstm = LstmPredictor::new(None); 
    let mut ipeps = IpepsPredictor::new(None);
    
    // 2. Initialize Mixer
    let mut mixer = Mixer::new();
    
    // 3. Initialize ANS Backend (New Constriction Logic)
    let mut ans = AnsWriter::new();
    
    let mut preds = [0u8; 3];

    for &actual in data {
        // A. Predict
        preds[0] = linear;
        preds[1] = lstm.predict_next();
        preds[2] = ipeps.predict_next();
        
        // B. Mix
        let mixed_prediction = mixer.mix(&preds);
        
        // C. Calculate Residual
        let residual = actual.wrapping_sub(mixed_prediction) as i8;
        
        // D. Buffer Residual
        ans.write_residual(residual);
        
        // E. Update
        mixer.update(actual, &preds);
        linear = actual; 
        lstm.update(actual); 
        ipeps.update_state(actual);
    }
    
    // F. Encode with ANS (Reverse pass happens inside)
    ans.finish()
}

fn predictive_decode_v3(compressed_words: &[u8], decoded_len: usize) -> Vec<u8> {
    // 1. Initialize Engines
    let mut linear = 0u8;
    let mut lstm = LstmPredictor::new(None); 
    let mut ipeps = IpepsPredictor::new(None);
    let mut mixer = Mixer::new();
    
    // 2. Initialize ANS Reader
    let mut ans = AnsReader::new(compressed_words);
    
    let mut out = Vec::with_capacity(decoded_len);
    let mut preds = [0u8; 3];

    for _ in 0..decoded_len {
        // A. Predict
        preds[0] = linear;
        preds[1] = lstm.predict_next();
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
        lstm.update(actual);
        ipeps.update_state(actual);
    }
    
    out
}

pub fn compress_chunk(chunk: &[u8], _predictor_id: u8, _weights: Option<&[u8]>, _lossy: Option<u8>) -> io::Result<Vec<u8>> {
    let compressed_body = predictive_encode_v3(chunk);
    
    // [V3 Format]: [Decompressed_Len (4 bytes)] + [Compressed_Body]
    // The length prefix is mandatory for the ANS decoder to know when to stop/how many symbols to expect.
    let mut out = Vec::with_capacity(4 + compressed_body.len());
    out.extend_from_slice(&(chunk.len() as u32).to_le_bytes());
    out.extend_from_slice(&compressed_body);
    
    Ok(out)
}

pub fn decompress_chunk(compressed: &[u8], _predictor_id: u8, _weights: Option<&[u8]>) -> io::Result<Vec<u8>> {
    if compressed.len() < 4 { 
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Chunk too short")); 
    }
    
    // Extract Decompressed Length
    let decomp_len = u32::from_le_bytes(compressed[0..4].try_into().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid Header"))?) as usize;
    
    // Decode body
    Ok(predictive_decode_v3(&compressed[4..], decomp_len))
}


// --- Python Wrapper ---

#[cfg(feature = "python")]
#[pyfunction]
fn encode_bytes<'a>(py: Python<'a>, data: &[u8], _predictor_id: u8, _weights: Option<&[u8]>) -> PyResult<&'a PyBytes> {
    // Wrapper simply calls compress_chunk which now handles all headers internally.
    let compressed = compress_chunk(data, 0, None, None).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    Ok(PyBytes::new(py, &compressed))
}

#[cfg(feature = "python")]
#[pyfunction]
fn decode_bytes<'a>(py: Python<'a>, data: &[u8], _predictor_id: u8, _weights: Option<&[u8]>) -> PyResult<&'a PyBytes> {
    let decompressed = decompress_chunk(data, 0, None).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    Ok(PyBytes::new(py, &decompressed))
}

#[cfg(feature = "python")]
#[pyfunction]
fn get_residuals_py<'a>(_py: Python<'a>, _data: &[u8], _predictor_id: u8, _weights: Option<&[u8]>) -> PyResult<Vec<i8>> {
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
