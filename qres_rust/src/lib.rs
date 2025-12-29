use std::io::{self, Read, Write, Cursor};
use std::cmp::min;
use chrono::Utc;
use serde::{Serialize, Deserialize};
use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use flate2::Compression;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

const CHUNK_SIZE: usize = 4 * 1024 * 1024; // 4MB
const QRES_MAGIC: &[u8] = b"QRES";

// --- Header Architecture (V3) ---
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QresHeader {
    pub version: u8,
    pub flags: u8,       // 0x00 = Block, 0x01 = Streaming
    pub predictor_id: u8,
    pub timestamp: i64,
    pub original_size: u64, // 0 if streaming
    pub compressed_size: u64, // 0 if streaming
    pub file_name: String,
    pub chunk_compressed_sizes: Vec<u64>, // Empty if streaming
}

// --- Predictor Logic ---
#[derive(Clone, Copy)]
enum PredictorMode { Previous = 0, Linear = 1, Neural = 2, Lstm = 3, Tensor = 4 }
impl From<u8> for PredictorMode {
    fn from(v: u8) -> Self { 
        match v {
            1 => PredictorMode::Linear,
            2 => PredictorMode::Neural,
            3 => PredictorMode::Lstm,
            4 => PredictorMode::Tensor,
            _ => PredictorMode::Previous,
        }
    }
}

// Fixed-sized Neural Network (3 -> 8 -> 1)
struct NeuralPredictor {
    w1: [f32; 24], b1: [f32; 8], w2: [f32; 8], b2: [f32; 1],
    context: [f32; 3],
}
impl NeuralPredictor {
    fn new(weights: Option<&[u8]>) -> Self {
        let mut n = NeuralPredictor {
            w1: [0.0; 24], b1: [0.0; 8], w2: [0.0; 8], b2: [0.0; 1],
            context: [0.0; 3],
        };
        if let Some(w_bytes) = weights {
             if w_bytes.len() == 164 {
                unsafe {
                    std::ptr::copy_nonoverlapping(w_bytes.as_ptr(), n.w1.as_mut_ptr() as *mut u8, 96);
                    std::ptr::copy_nonoverlapping(w_bytes.as_ptr().add(96), n.b1.as_mut_ptr() as *mut u8, 32);
                    std::ptr::copy_nonoverlapping(w_bytes.as_ptr().add(128), n.w2.as_mut_ptr() as *mut u8, 32);
                    std::ptr::copy_nonoverlapping(w_bytes.as_ptr().add(160), n.b2.as_mut_ptr() as *mut u8, 4);
                }
            }
        }
        n
    }
}

struct LstmPredictor {
    w_ih: [f32; 32], w_hh: [f32; 256], b_ih: [f32; 32], b_hh: [f32; 32], w_fc: [f32; 8], b_fc: [f32; 1],
    h: [f32; 8], c: [f32; 8],
}
impl LstmPredictor {
    fn new(weights: Option<&[u8]>) -> Self {
        let mut n = LstmPredictor {
            w_ih: [0.0; 32], w_hh: [0.0; 256], b_ih: [0.0; 32], b_hh: [0.0; 32], 
            w_fc: [0.0; 8], b_fc: [0.0; 1],
            h: [0.0; 8], c: [0.0; 8],
        };
        if let Some(w) = weights {
            if w.len() >= 1444 {
                unsafe {
                    let mut ptr = w.as_ptr();
                    std::ptr::copy_nonoverlapping(ptr, n.w_ih.as_mut_ptr() as *mut u8, 128); ptr = ptr.add(128);
                    std::ptr::copy_nonoverlapping(ptr, n.w_hh.as_mut_ptr() as *mut u8, 1024); ptr = ptr.add(1024);
                    std::ptr::copy_nonoverlapping(ptr, n.b_ih.as_mut_ptr() as *mut u8, 128); ptr = ptr.add(128);
                    std::ptr::copy_nonoverlapping(ptr, n.b_hh.as_mut_ptr() as *mut u8, 128); ptr = ptr.add(128);
                    std::ptr::copy_nonoverlapping(ptr, n.w_fc.as_mut_ptr() as *mut u8, 32); ptr = ptr.add(32);
                    std::ptr::copy_nonoverlapping(ptr, n.b_fc.as_mut_ptr() as *mut u8, 4);
                }
            }
        }
        n
    }
    #[inline(always)] fn sigmoid(x: f32) -> f32 { 1.0 / (1.0 + (-x).exp()) }
    #[inline(always)] fn tanh(x: f32) -> f32 { x.tanh() }
}

struct TensorPredictor {
    a0: [f32; 16], // 4x4
    a1: [f32; 16], // 4x4
    c:  [f32; 4],  // 4x1
    b:  [f32; 1],  // 1
    psi: [f32; 4], // State
}
impl TensorPredictor {
    fn new(weights: Option<&[u8]>) -> Self {
        let mut n = TensorPredictor {
            a0: [0.0; 16], a1: [0.0; 16], c: [0.0; 4], b: [0.0; 1],
            psi: [1.0, 0.0, 0.0, 0.0], // Initial State [1,0,0,0]
        };
        if let Some(w) = weights {
            // Expected size: 16*4 + 16*4 + 4*4 + 4 = 148 bytes
             if w.len() >= 148 {
                unsafe {
                    let mut ptr = w.as_ptr();
                    std::ptr::copy_nonoverlapping(ptr, n.a0.as_mut_ptr() as *mut u8, 64); ptr = ptr.add(64);
                    std::ptr::copy_nonoverlapping(ptr, n.a1.as_mut_ptr() as *mut u8, 64); ptr = ptr.add(64);
                    std::ptr::copy_nonoverlapping(ptr, n.c.as_mut_ptr() as *mut u8, 16); ptr = ptr.add(16);
                    std::ptr::copy_nonoverlapping(ptr, n.b.as_mut_ptr() as *mut u8, 4);
                }
             }
        }
        n
    }
}

struct PredictorEngine { 
    mode: PredictorMode, 
    p1: u8, p2: u8,
    neural: NeuralPredictor,
    lstm: LstmPredictor,
    tensor: TensorPredictor,
}

impl PredictorEngine {
    fn new(mode: PredictorMode, weights: Option<&[u8]>) -> Self { 
        PredictorEngine { 
            mode, p1: 0, p2: 0,
            neural: if matches!(mode, PredictorMode::Neural) { NeuralPredictor::new(weights) } else { NeuralPredictor::new(None) },
            lstm: if matches!(mode, PredictorMode::Lstm) { LstmPredictor::new(weights) } else { LstmPredictor::new(None) },
             tensor: if matches!(mode, PredictorMode::Tensor) { TensorPredictor::new(weights) } else { TensorPredictor::new(None) },
        } 
    }
    
    #[inline(always)]
    fn predict(&self) -> u8 {
        match self.mode {
            PredictorMode::Previous => self.p1,
            PredictorMode::Linear => self.p1.wrapping_add(self.p1.wrapping_sub(self.p2)),
            PredictorMode::Neural => {
                let mut hidden = [0.0f32; 8];
                for h in 0..8 {
                    let mut sum = self.neural.b1[h];
                    for i in 0..3 { sum += self.neural.context[i] * self.neural.w1[i * 8 + h]; }
                    hidden[h] = if sum > 0.0 { sum } else { 0.0 };
                }
                let mut sum = self.neural.b2[0];
                for h in 0..8 { sum += hidden[h] * self.neural.w2[h]; }
                let out = (sum * 255.0).round();
                if out > 255.0 { 255 } else if out < 0.0 { 0 } else { out as u8 }
            },
            PredictorMode::Lstm => {
                let mut sum = self.lstm.b_fc[0];
                for i in 0..8 { sum += self.lstm.h[i] * self.lstm.w_fc[i]; }
                let out = (sum * 255.0).round();
                if out > 255.0 { 255 } else if out < 0.0 { 0 } else { out as u8 }
            },
            PredictorMode::Tensor => {
                // y = psi @ C + b
                let mut sum = self.tensor.b[0];
                for i in 0..4 { sum += self.tensor.psi[i] * self.tensor.c[i]; }
                let out = (sum * 255.0).round();
                if out > 255.0 { 255 } else if out < 0.0 { 0 } else { out as u8 }
            }
        }
    }
    
    #[inline(always)]
    fn update(&mut self, actual: u8) { 
        self.p2 = self.p1; 
        self.p1 = actual; 
        
        match self.mode {
            PredictorMode::Neural => {
                self.neural.context[0] = self.neural.context[1];
                self.neural.context[1] = self.neural.context[2];
                self.neural.context[2] = (actual as f32) / 255.0;
            },
            PredictorMode::Lstm => {
                let x = (actual as f32) / 255.0;
                let mut gates = [0.0f32; 32];
                for i in 0..32 { gates[i] = self.lstm.w_ih[i] * x + self.lstm.b_ih[i] + self.lstm.b_hh[i]; }
                for row in 0..32 {
                    let mut sum = 0.0;
                    for col in 0..8 { sum += self.lstm.w_hh[row * 8 + col] * self.lstm.h[col]; }
                    gates[row] += sum;
                }
                for i in 0..8 {
                    let it = LstmPredictor::sigmoid(gates[i]);
                    let ft = LstmPredictor::sigmoid(gates[i+8]);
                    let gt = LstmPredictor::tanh(gates[i+16]);
                    let ot = LstmPredictor::sigmoid(gates[i+24]);
                    self.lstm.c[i] = ft * self.lstm.c[i] + it * gt;
                    self.lstm.h[i] = ot * LstmPredictor::tanh(self.lstm.c[i]);
                }
            },
            PredictorMode::Tensor => {
                // Tensor State Update
                // psi_new = psi @ A_eff
                // A_eff = A0 + x * A1
                
                let x = (actual as f32) / 255.0;
                let mut psi_next = [0.0f32; 4];
                
                // psi is Row Vector [1, 4]. A_eff is [4, 4].
                // psi_next[j] = sum(psi[i] * A_eff[i][j])
                
                for j in 0..4 {
                    let mut sum = 0.0;
                    for i in 0..4 {
                         // A0 and A1 are flattened row-major. Index = i*4 + j
                         let idx = i * 4 + j;
                         let a_eff_val = self.tensor.a0[idx] + x * self.tensor.a1[idx];
                         sum += self.tensor.psi[i] * a_eff_val;
                    }
                    psi_next[j] = sum;
                }
                
                self.tensor.psi = psi_next;
            },
            _ => {}
        }
    }
}

// --- Bit Packing (v2 Optimized) ---
struct BitWriter { buffer: Vec<u8>, current_byte: u8, bit_count: u8 }
impl BitWriter {
    fn new() -> Self { BitWriter { buffer: Vec::with_capacity(4096), current_byte: 0, bit_count: 0 } }
    fn write_2bits(&mut self, val: u8) {
        self.current_byte |= (val & 0b11) << (6 - self.bit_count);
        self.bit_count += 2;
        if self.bit_count == 8 { self.buffer.push(self.current_byte); self.current_byte = 0; self.bit_count = 0; }
    }
    fn write_byte(&mut self, byte: u8) {
        if self.bit_count == 0 { self.buffer.push(byte); }
        else {
            let top = byte >> self.bit_count;
            let bot = byte << (8 - self.bit_count);
            self.current_byte |= top;
            self.buffer.push(self.current_byte);
            self.current_byte = bot;
        }
    }
    fn flush(&mut self) -> Vec<u8> {
        if self.bit_count > 0 { self.buffer.push(self.current_byte); }
        std::mem::take(&mut self.buffer)
    }
}

struct BitReader<'a> { buffer: &'a [u8], byte_index: usize, bit_offset: u8 }
impl<'a> BitReader<'a> {
    fn new(buffer: &'a [u8]) -> Self { BitReader { buffer, byte_index: 0, bit_offset: 0 } }
    fn read_2bits(&mut self) -> Option<u8> {
        if self.byte_index >= self.buffer.len() { return None; }
        let val = (self.buffer[self.byte_index] >> (6 - self.bit_offset)) & 0b11;
        self.bit_offset += 2;
        if self.bit_offset == 8 { self.bit_offset = 0; self.byte_index += 1; }
        Some(val)
    }
    fn read_byte(&mut self) -> Option<u8> {
        if self.bit_offset == 0 {
            if self.byte_index >= self.buffer.len() { return None; }
            let b = self.buffer[self.byte_index];
            self.byte_index += 1;
            Some(b)
        } else {
            if self.byte_index + 1 >= self.buffer.len() { return None; }
            let top = self.buffer[self.byte_index] << self.bit_offset;
            let bot = self.buffer[self.byte_index + 1] >> (8 - self.bit_offset);
            self.byte_index += 1;
            Some(top | bot)
        }
    }
}

// --- Encoding Logic ---
// --- Encoding Logic ---
fn predictive_encode(data: &[u8], mode: PredictorMode, weights: Option<&[u8]>) -> Vec<i8> {
    let mut predictor = PredictorEngine::new(mode, weights);
    let mut deltas = Vec::with_capacity(data.len());
    for &actual in data {
        let predicted = predictor.predict();
        deltas.push(actual.wrapping_sub(predicted) as i8);
        predictor.update(actual);
    }
    deltas
}

fn predictive_decode(deltas: &[i8], mode: PredictorMode, weights: Option<&[u8]>) -> Vec<u8> {
    let mut predictor = PredictorEngine::new(mode, weights);
    let mut data = Vec::with_capacity(deltas.len());
    for &delta in deltas {
        let predicted = predictor.predict();
        let actual = predicted.wrapping_add(delta as u8);
        data.push(actual);
        predictor.update(actual);
    }
    data
}

fn bit_pack_encode(deltas: &[i8]) -> Vec<u8> {
    let mut w = BitWriter::new();
    w.buffer.extend_from_slice(&(deltas.len() as u32).to_le_bytes()); 
    for &d in deltas {
        match d {
            0 => w.write_2bits(0b00),
            1 => w.write_2bits(0b01),
            -1 => w.write_2bits(0b10),
            _ => { w.write_2bits(0b11); w.write_byte(d as u8); }
        }
    }
    w.flush()
}

fn bit_pack_decode(encoded: &[u8]) -> Vec<i8> {
    if encoded.len() < 4 { return Vec::new(); }
    let count = u32::from_le_bytes(encoded[0..4].try_into().unwrap()) as usize;
    let mut r = BitReader::new(&encoded[4..]);
    let mut res = Vec::with_capacity(count);
    for _ in 0..count {
        match r.read_2bits() {
            Some(0b00) => res.push(0),
            Some(0b01) => res.push(1),
            Some(0b10) => res.push(-1),
            Some(0b11) => if let Some(lit) = r.read_byte() { res.push(lit as i8); } else { break; },
            _ => break,
        }
    }
    res
}

pub fn compress_chunk(chunk: &[u8], predictor_id: u8, weights: Option<&[u8]>) -> io::Result<Vec<u8>> {
    let mode = PredictorMode::from(predictor_id);
    let deltas = predictive_encode(chunk, mode, weights);
    let packed = bit_pack_encode(&deltas);
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    e.write_all(&packed)?;
    e.finish()
}

pub fn decompress_chunk(compressed: &[u8], predictor_id: u8, weights: Option<&[u8]>) -> io::Result<Vec<u8>> {
    let mut d = ZlibDecoder::new(compressed);
    let mut dec = Vec::new();
    d.read_to_end(&mut dec)?;
    let deltas = bit_pack_decode(&dec);
    Ok(predictive_decode(&deltas, PredictorMode::from(predictor_id), weights))
}

// --- Streaming Architecture ---

// --- Streaming Architecture ---

use rayon::prelude::*;

// --- Embedded Brains ---
const LSTM_WEIGHTS: &[u8] = include_bytes!("models/lstm.qnn");
const TENSOR_WEIGHTS: &[u8] = include_bytes!("models/tensor.qnn");

// --- Autonomic Selector ---
fn qualify_stream(sample: &[u8]) -> (u8, Vec<u8>) {
    // Race Candidates: Linear (1), LSTM (3), Tensor (4)
    // Structure: (id, weights, name)
    let candidates = vec![
        (1, vec![], "Linear"),
        (3, LSTM_WEIGHTS.to_vec(), "LSTM"),
        (4, TENSOR_WEIGHTS.to_vec(), "Tensor"),
    ];
    
    // Parallel Race
    let results: Vec<(u8, f64)> = candidates.par_iter().map(|(id, weights, _name)| {
        let start = std::time::Instant::now();
        // Compress sample
        let weights_ref = if weights.is_empty() { None } else { Some(weights.as_slice()) };
        let compressed = match compress_chunk(sample, *id, weights_ref) {
            Ok(c) => c.len(),
            Err(_) => usize::MAX, // Fail
        };
        let duration = start.elapsed().as_micros() as f64;
        
        // Score: Size (lower better) + Duration Penalty
        // Adjust bias: We want compression, but avoid extreme slowness if ratio is similar.
        // Let's say 1 byte saved is worth 10 microseconds.
        // Score = Size + (Duration / 10.0)
        let score = (compressed as f64) + (duration / 20.0); 
        
        (*id, score)
    }).collect();
    
    // Pick Winner (Lowest Score)
    let mut best_id = 1;
    let mut best_score = f64::MAX;
    
    for (id, score) in results {
        if score < best_score {
            best_score = score;
            best_id = id;
        }
    }
    
    // Return ID and Weights
    match best_id {
        3 => (3, LSTM_WEIGHTS.to_vec()),
        4 => (4, TENSOR_WEIGHTS.to_vec()),
        _ => (1, vec![]),
    }
}

// --- Streaming Architecture ---

enum WriterState {
    Buffering,
    Streaming,
}

pub struct QresWriter<W: Write> {
    writer: W,
    buffer: Vec<u8>,
    
    // Config
    mode_hint: u8, // 0=Auto, 1=Fast(Linear), 3=Max(LSTM)
    
    // State
    state: WriterState,
    predictor_id: u8,
    weights: Vec<u8>,
    header_written: bool,
}

impl<W: Write> QresWriter<W> {
    pub fn new(writer: W, mode_hint: u8) -> Self {
        QresWriter {
            writer,
            buffer: Vec::with_capacity(64 * 1024), // 64KB Buffer for race
            mode_hint, // 0=Auto
            state: WriterState::Buffering,
            predictor_id: 1, // Default
            weights: Vec::new(),
            header_written: false,
        }
    }

    fn write_header(&mut self) -> io::Result<()> {
        if self.header_written { return Ok(()); }
        
        // If we are forcing a mode (Fast/Max), set it now if not already set by race
        // Actually race sets it. If skip race, set default.
        
        let header = QresHeader {
            version: 8, // v0.8.0 / v0.9.0 / v1.0.0
            flags: 0x01, 
            predictor_id: self.predictor_id,
            timestamp: Utc::now().timestamp(),
            original_size: 0,
            compressed_size: 0,
            file_name: "stream".to_string(),
            chunk_compressed_sizes: vec![],
        };

        self.writer.write_all(QRES_MAGIC)?;
        let hb = bincode::serialize(&header).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        self.writer.write_all(&(hb.len() as u32).to_le_bytes())?; 
        self.writer.write_all(&hb)?;

        // Embed Weights
        if self.predictor_id >= 2 {
            self.writer.write_all(&(self.weights.len() as u32).to_le_bytes())?;
            self.writer.write_all(&self.weights)?;
        }

        self.header_written = true;
        Ok(())
    }

    fn flush_chunk(&mut self, chunk: &[u8]) -> io::Result<()> {
        if chunk.is_empty() { return Ok(()); }
        
        let weights_ref = if self.predictor_id >= 2 { Some(self.weights.as_slice()) } else { None };
        let compressed = compress_chunk(chunk, self.predictor_id, weights_ref)?;
        
        self.writer.write_all(&(compressed.len() as u32).to_le_bytes())?;
        self.writer.write_all(&compressed)?;
        Ok(())
    }

    fn perform_race_and_flush(&mut self) -> io::Result<()> {
        // Race Logic
        if self.mode_hint == 3 {
             // Force LSTM
             self.predictor_id = 3;
             self.weights = LSTM_WEIGHTS.to_vec();
        } else if self.mode_hint == 1 {
             // Force Linear
             self.predictor_id = 1;
             self.weights = Vec::new();
        } else {
             // Auto (0) - RACE!
             if self.buffer.len() > 1024 { // Only race if we have decent sample
                 let (winner, w) = qualify_stream(&self.buffer);
                 self.predictor_id = winner;
                 self.weights = w;
             } else {
                 // Too small, default Linear
                 self.predictor_id = 1; 
             }
        }
        
        self.write_header()?;
        
        // Flush buffer
        // Note: buffer might be larger than chunk size? 
        // Our buffer is 64KB, chunk size 4MB. Safe to flush as one chunk.
        // Or strictly follow CHUNK_SIZE.
        
        let mut offset = 0;
        while offset < self.buffer.len() {
             let end = min(offset + CHUNK_SIZE, self.buffer.len());
             let chunk = self.buffer[offset..end].to_vec();
             self.flush_chunk(&chunk)?;
             offset = end;
        }
        
        self.buffer.clear();
        self.state = WriterState::Streaming;
        Ok(())
    }
}

impl<W: Write> Write for QresWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut bytes_processed = 0;
        
        match self.state {
            WriterState::Buffering => {
                // Fill buffer until 64KB
                let needed = 64 * 1024 - self.buffer.len();
                let to_copy = min(needed, buf.len());
                self.buffer.extend_from_slice(&buf[0..to_copy]);
                bytes_processed += to_copy;
                
                if self.buffer.len() >= 64 * 1024 {
                    self.perform_race_and_flush()?;
                    
                    // Process remaining bytes in this call via Streaming logic
                    if bytes_processed < buf.len() {
                        self.writer.write_all(&[])?; // Just to sync trait? No-op.
                        // Recurse or Loop? Better loop structure.
                        // Let's just handle rest here.
                        return self.write(&buf[bytes_processed..]).map(|n| n + bytes_processed);
                    }
                }
            },
            WriterState::Streaming => {
                // Standard Streaming
                 while bytes_processed < buf.len() {
                    let space = CHUNK_SIZE - self.buffer.len();
                    let to_copy = min(space, buf.len() - bytes_processed);
                    
                    self.buffer.extend_from_slice(&buf[bytes_processed..bytes_processed+to_copy]);
                    bytes_processed += to_copy;
                    
                    if self.buffer.len() == CHUNK_SIZE {
                        self.flush_chunk(&clone_buffer(&self.buffer))?; // Helper to avoid borrow issues? 
                        // Actually flush_chunk takes &self, so we can't drain buffer easily.
                        // Let's modify flush_chunk to NOT take &self buffer but just bytes. Done.
                        // Wait, previous implem used state buffer.
                        // Here we use self.buffer for chunking.
                        
                        // We need to pass slice to flush_chunk.
                        // But flush_chunk writes to self.writer.
                        
                        // Correct pattern:
                        let c = self.buffer.clone(); // Copy 4MB? Meh.
                        self.flush_chunk(&c)?;
                        self.buffer.clear();
                    }
                }
            }
        }
        Ok(bytes_processed)
    }

    fn flush(&mut self) -> io::Result<()> {
        if let WriterState::Buffering = self.state {
            // Flush triggered before 64KB. Race on what we have.
            self.perform_race_and_flush()?;
        }
        
        if !self.buffer.is_empty() {
             let c = self.buffer.clone();
             self.flush_chunk(&c)?;
             self.buffer.clear();
        }
        self.writer.flush()
    }
}

fn clone_buffer(b: &Vec<u8>) -> Vec<u8> { b.clone() }

pub struct QresReader<R: Read> {
    reader: R,
    buffer: Cursor<Vec<u8>>, 
    header: Option<QresHeader>,
    weights: Vec<u8>, // Weights loaded from stream
}

impl<R: Read> QresReader<R> {
    pub fn new(reader: R) -> Self {
        QresReader {
            reader,
            buffer: Cursor::new(Vec::new()),
            header: None,
            weights: Vec::new(),
        }
    }

    fn read_header_internal(&mut self) -> io::Result<()> {
        if self.header.is_some() { return Ok(()); }
        
        let mut magic = [0u8; 4];
        self.reader.read_exact(&mut magic)?;
        if &magic != QRES_MAGIC { return Err(io::Error::new(io::ErrorKind::InvalidData, "Not QRES")); }
        
        let mut len_b = [0u8; 4];
        self.reader.read_exact(&mut len_b)?;
        let h_len = u32::from_le_bytes(len_b) as usize;
        
        let mut h_buf = vec![0u8; h_len];
        self.reader.read_exact(&mut h_buf)?;
        
        let header: QresHeader = bincode::deserialize(&h_buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        // V3.1: Read Weights if Neural/LSTM
        if header.predictor_id >= 2 {
            let mut w_len_b = [0u8; 4];
            self.reader.read_exact(&mut w_len_b)?;
            let w_len = u32::from_le_bytes(w_len_b) as usize;
            
            let mut w_buf = vec![0u8; w_len];
            self.reader.read_exact(&mut w_buf)?;
            self.weights = w_buf;
        }

        self.header = Some(header);
        Ok(())
    }

    fn fill_buffer(&mut self) -> io::Result<bool> {
        let mut size_b = [0u8; 4];
        match self.reader.read_exact(&mut size_b) {
            Ok(_) => {},
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(false), 
            Err(e) => return Err(e),
        }
        
        let chunk_size = u32::from_le_bytes(size_b) as usize;
        if chunk_size == 0 { return Ok(false); } 
        
        let mut compressed = vec![0u8; chunk_size];
        self.reader.read_exact(&mut compressed)?;
        
        let header = self.header.as_ref().ok_or(io::Error::new(io::ErrorKind::Other, "No Header"))?;
        let weights_ref = if header.predictor_id >= 2 { Some(self.weights.as_slice()) } else { None };
        
        let decoded = decompress_chunk(&compressed, header.predictor_id, weights_ref)?;
        
        self.buffer = Cursor::new(decoded);
        Ok(true)
    }
}

impl<R: Read> Read for QresReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.header.is_none() { self.read_header_internal()?; }
        
        if self.buffer.position() == self.buffer.get_ref().len() as u64 {
            if !self.fill_buffer()? {
                return Ok(0); // EOF
            }
        }
        
        self.buffer.read(buf)
    }
}

// --- Python Bindings ---
#[pyfunction]
fn encode_bytes<'a>(py: Python<'a>, data: &[u8], predictor_id: u8, weights: Option<&[u8]>) -> PyResult<&'a PyBytes> {
    let compressed = compress_chunk(data, predictor_id, weights).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    Ok(PyBytes::new(py, &compressed))
}

#[pyfunction]
fn decode_bytes<'a>(py: Python<'a>, data: &[u8], predictor_id: u8, weights: Option<&[u8]>) -> PyResult<&'a PyBytes> {
    let decompressed = decompress_chunk(data, predictor_id, weights).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    Ok(PyBytes::new(py, &decompressed))
}

#[pymodule]
fn qres_rust(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(encode_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(decode_bytes, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_roundtrip() {
        let original_data: Vec<u8> = (0..10_000).map(|i| (i % 255) as u8).collect();
        let mut encoded_buffer = Vec::new();

        {
            let mut writer = QresWriter::new(&mut encoded_buffer, 1); // Mode 1 = Linear (Fast)
            writer.write_all(&original_data).unwrap();
            writer.flush().unwrap(); // Force flush
        }

        let mut reader = QresReader::new(io::Cursor::new(&encoded_buffer));
        let mut decoded_data = Vec::new();
        reader.read_to_end(&mut decoded_data).unwrap();

        assert_eq!(original_data, decoded_data);
    }
}
