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

// --- Predictor Logic (v0.5.0) ---
#[derive(Clone, Copy)]
enum PredictorMode { Previous = 0, Linear = 1 }
impl From<u8> for PredictorMode {
    fn from(v: u8) -> Self { if v == 1 { PredictorMode::Linear } else { PredictorMode::Previous } }
}

struct PredictorEngine { mode: PredictorMode, p1: u8, p2: u8 }
impl PredictorEngine {
    fn new(mode: PredictorMode) -> Self { PredictorEngine { mode, p1: 0, p2: 0 } }
    #[inline(always)]
    fn predict(&self) -> u8 {
        match self.mode {
            PredictorMode::Previous => self.p1,
            PredictorMode::Linear => self.p1.wrapping_add(self.p1.wrapping_sub(self.p2)),
        }
    }
    #[inline(always)]
    fn update(&mut self, actual: u8) { self.p2 = self.p1; self.p1 = actual; }
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
fn predictive_encode(data: &[u8], mode: PredictorMode) -> Vec<i8> {
    let mut predictor = PredictorEngine::new(mode);
    let mut deltas = Vec::with_capacity(data.len());
    for &actual in data {
        let predicted = predictor.predict();
        deltas.push(actual.wrapping_sub(predicted) as i8);
        predictor.update(actual);
    }
    deltas
}

fn predictive_decode(deltas: &[i8], mode: PredictorMode) -> Vec<u8> {
    let mut predictor = PredictorEngine::new(mode);
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

pub fn compress_chunk(chunk: &[u8], predictor_id: u8) -> io::Result<Vec<u8>> {
    // Adaptive Pass-through Check (Simulated for Streaming)
    // For now, we assume Predictor is forced or optimal.
    let mode = PredictorMode::from(predictor_id);
    let deltas = predictive_encode(chunk, mode);
    let packed = bit_pack_encode(&deltas);
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    e.write_all(&packed)?;
    e.finish()
}

pub fn decompress_chunk(compressed: &[u8], predictor_id: u8) -> io::Result<Vec<u8>> {
    let mut d = ZlibDecoder::new(compressed);
    let mut dec = Vec::new();
    d.read_to_end(&mut dec)?;
    let deltas = bit_pack_decode(&dec);
    Ok(predictive_decode(&deltas, PredictorMode::from(predictor_id)))
}

// --- Streaming Architecture ---

pub struct QresWriter<W: Write> {
    writer: W,
    buffer: Vec<u8>,
    predictor_id: u8,
    header_written: bool,
}

impl<W: Write> QresWriter<W> {
    pub fn new(writer: W, predictor_id: u8) -> Self {
        QresWriter {
            writer,
            buffer: Vec::with_capacity(CHUNK_SIZE),
            predictor_id,
            header_written: false,
        }
    }

    fn write_header(&mut self) -> io::Result<()> {
        if self.header_written { return Ok(()); }
        
        // V3 Header (Streaming)
        let header = QresHeader {
            version: 5, // v0.5.0
            flags: 0x01, // Streaming Mode
            predictor_id: self.predictor_id,
            timestamp: Utc::now().timestamp(),
            original_size: 0, // Unknown
            compressed_size: 0, // Unknown
            file_name: "stream".to_string(),
            chunk_compressed_sizes: vec![],
        };

        self.writer.write_all(QRES_MAGIC)?;
        let hb = bincode::serialize(&header).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        self.writer.write_all(&(hb.len() as u32).to_le_bytes())?; // LE for header length
        self.writer.write_all(&hb)?;
        self.header_written = true;
        Ok(())
    }

    fn compress_and_flush_buffer(&mut self) -> io::Result<()> {
        if self.buffer.is_empty() { return Ok(()); }
        
        let compressed = compress_chunk(&self.buffer, self.predictor_id)?;
        
        // Write Framing: [Size u32] [Body]
        self.writer.write_all(&(compressed.len() as u32).to_le_bytes())?;
        self.writer.write_all(&compressed)?;
        
        self.buffer.clear();
        Ok(())
    }
}

impl<W: Write> Write for QresWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if !self.header_written { self.write_header()?; }
        
        let mut bytes_written = 0;
        while bytes_written < buf.len() {
            let space = CHUNK_SIZE - self.buffer.len();
            let to_copy = min(space, buf.len() - bytes_written);
            
            self.buffer.extend_from_slice(&buf[bytes_written..bytes_written+to_copy]);
            bytes_written += to_copy;
            
            if self.buffer.len() == CHUNK_SIZE {
                self.compress_and_flush_buffer()?;
            }
        }
        Ok(bytes_written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.compress_and_flush_buffer()?;
        self.writer.flush()
    }
}

pub struct QresReader<R: Read> {
    reader: R,
    buffer: Cursor<Vec<u8>>, // Decoded bytes ready to read
    header: Option<QresHeader>,
}

impl<R: Read> QresReader<R> {
    pub fn new(reader: R) -> Self {
        QresReader {
            reader,
            buffer: Cursor::new(Vec::new()),
            header: None,
        }
    }

    fn read_header_internal(&mut self) -> io::Result<()> {
        if self.header.is_some() { return Ok(()); }
        
        let mut magic = [0u8; 4];
        self.reader.read_exact(&mut magic)?;
        if &magic != QRES_MAGIC { return Err(io::Error::new(io::ErrorKind::InvalidData, "Not QRES")); }
        
        let mut len_b = [0u8; 4];
        self.reader.read_exact(&mut len_b)?;
        let h_len = u32::from_le_bytes(len_b) as usize; // V3 uses LE
        
        let mut h_buf = vec![0u8; h_len];
        self.reader.read_exact(&mut h_buf)?;
        
        let header: QresHeader = bincode::deserialize(&h_buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        self.header = Some(header);
        Ok(())
    }

    fn fill_buffer(&mut self) -> io::Result<bool> {
        // Read Framing: [Size u32]
        let mut size_b = [0u8; 4];
        match self.reader.read_exact(&mut size_b) {
            Ok(_) => {},
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(false), // EOF
            Err(e) => return Err(e),
        }
        
        let chunk_size = u32::from_le_bytes(size_b) as usize;
        if chunk_size == 0 { return Ok(false); } // Explicit EOF frame
        
        let mut compressed = vec![0u8; chunk_size];
        self.reader.read_exact(&mut compressed)?;
        
        let header = self.header.as_ref().ok_or(io::Error::new(io::ErrorKind::Other, "No Header"))?;
        let decoded = decompress_chunk(&compressed, header.predictor_id)?;
        
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
fn encode_bytes<'a>(py: Python<'a>, data: &[u8], predictor_id: u8) -> PyResult<&'a PyBytes> {
    let compressed = compress_chunk(data, predictor_id).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    Ok(PyBytes::new(py, &compressed))
}

#[pyfunction]
fn decode_bytes<'a>(py: Python<'a>, data: &[u8], predictor_id: u8) -> PyResult<&'a PyBytes> {
    let decompressed = decompress_chunk(data, predictor_id).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
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
            let mut writer = QresWriter::new(&mut encoded_buffer, 0); // Previous Predictor
            writer.write_all(&original_data).unwrap();
            writer.finish().unwrap(); // Force flush
        }

        let mut reader = QresReader::new(io::Cursor::new(&encoded_buffer));
        let mut decoded_data = Vec::new();
        reader.read_to_end(&mut decoded_data).unwrap();

        assert_eq!(original_data, decoded_data);
    }
}
