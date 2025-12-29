use std::io::{self, Read, Write, BufReader, BufWriter};
use std::fs::File; // Required for CLI handlers
use std::path::Path; // Required for CLI handlers
use chrono::Utc;
use serde::{Serialize, Deserialize};
use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use flate2::Compression;
use rayon::prelude::*;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3::buffer::PyBuffer; // Zero-Copy Feature

const CHUNK_SIZE: usize = 4 * 1024 * 1024;
const QRES_MAGIC: &[u8] = b"QRES";

#[derive(Serialize, Deserialize, Debug)]
pub struct QresHeader {
    pub version: u8,
    pub timestamp: i64,
    pub original_size: u64,
    pub compressed_size: u64,
    pub file_name: String,
    pub chunk_compressed_sizes: Vec<u64>,
}

// --- Optimized BitWriter (with SWAR support) ---
struct BitWriter {
    buffer: Vec<u8>,
    current_byte: u8,
    bit_count: u8,
}

impl BitWriter {
    fn new_with_capacity(cap: usize) -> Self {
        BitWriter {
            buffer: Vec::with_capacity(cap),
            current_byte: 0,
            bit_count: 0,
        }
    }

    // Standard slow path (safe for edge cases)
    #[inline(always)]
    fn write_2bits(&mut self, value: u8) {
        // Value is 0..3. 
        // Logic: Write LSB 2 bits of value to next available spots in current_byte (MSB-first filling or LSB-first? Previous was MSB-first)
        // Original: (value & 1) << (7-count) ...
        // Let's stick to standard behavior: Fill MSB to LSB.
        // current_byte starts 0.
        // We write to 7-count, 6-count.
        // E.g. count=0. Write bits at 7, 6.
        // shift = 6 - count.
        self.current_byte |= (value & 0b11) << (6 - self.bit_count);
        self.bit_count += 2;
        if self.bit_count == 8 {
            self.buffer.push(self.current_byte);
            self.current_byte = 0;
            self.bit_count = 0;
        }
    }

    #[inline(always)]
    fn write_byte(&mut self, byte: u8) {
        if self.bit_count == 0 {
            self.buffer.push(byte);
        } else {
            // Expensive misalignment case: write 8 bits manually
            // (Only happens during escapes)
            // Example: count=2 (bits 7,6 used). write 8 bits at 5..0 and next 7,6?
            // current_byte has bits 7,6 set.
            // byte top 6 bits go to 5..0 of current
            // byte bottom 2 bits go to 7,6 of next
            
            let top = byte >> self.bit_count;
            let bot = byte << (8 - self.bit_count);
            
            self.current_byte |= top;
            self.buffer.push(self.current_byte);
            
            self.current_byte = bot;
            // bit_count remains same because we wrote 8 bits (filled remainder + started new same amount)
        }
    }

    fn flush(&mut self) -> Vec<u8> {
        if self.bit_count > 0 {
            self.buffer.push(self.current_byte);
        }
        std::mem::take(&mut self.buffer)
    }
}

// --- Optimized Reader ---
struct BitReader<'a> {
    buffer: &'a [u8],
    byte_index: usize,
    bit_offset: u8,
}

impl<'a> BitReader<'a> {
    fn new(buffer: &'a [u8]) -> Self {
        BitReader { buffer, byte_index: 0, bit_offset: 0 }
    }

    #[inline(always)]
    fn read_2bits(&mut self) -> Option<u8> {
        if self.byte_index >= self.buffer.len() { return None; }
        // Read bits at 7-offset, 6-offset.
        // Shift right by (6 - offset).
        let val = (self.buffer[self.byte_index] >> (6 - self.bit_offset)) & 0b11;
        self.bit_offset += 2;
        if self.bit_offset == 8 {
            self.bit_offset = 0;
            self.byte_index += 1;
        }
        Some(val)
    }

    fn read_byte(&mut self) -> Option<u8> {
        if self.bit_offset == 0 {
            if self.byte_index >= self.buffer.len() { return None; }
            let b = self.buffer[self.byte_index];
            self.byte_index += 1;
            Some(b)
        } else {
            // Reconstruct byte from split parts
            if self.byte_index + 1 >= self.buffer.len() { return None; }
            // Current byte has low bits remaining.
            // We need 8 bits.
            // remaining in current = 8 - offset. (These are high bits of result?)
            // No, we read MSB first.
            // So bits from current byte form the TOP of the result.
            let top = self.buffer[self.byte_index] << self.bit_offset;
            let bot = self.buffer[self.byte_index + 1] >> (8 - self.bit_offset);
            self.byte_index += 1;
            Some(top | bot)
        }
    }
}

// --- The Core Optimizations ---

// 1. Delta Encoding (Vectorizable by LLVM)
fn delta_encode(data: &[u8]) -> Vec<i8> {
    let mut res = Vec::with_capacity(data.len());
    let mut prev = 0u8;
    // The Rust compiler is very good at SIMD-izing this loop automatically
    for &b in data {
        res.push(b.wrapping_sub(prev) as i8);
        prev = b;
    }
    res
}

fn delta_decode(data: &[i8]) -> Vec<u8> {
    let mut res = Vec::with_capacity(data.len());
    let mut prev = 0u8;
    for &d in data {
        let b = prev.wrapping_add(d as u8);
        res.push(b);
        prev = b;
    }
    res
}

// 2. SWAR Bit-Packing (Stable Rust)
fn bit_pack_encode(deltas: &[i8]) -> Vec<u8> {
    // Allocation: 4 deltas fit in 1 byte.
    let mut w = BitWriter::new_with_capacity(deltas.len() / 4 + 64);
    
    // Header: count
    w.buffer.extend_from_slice(&(deltas.len() as u32).to_le_bytes()); 

    for &d in deltas {
        // Hot Path Optimization: 
        // We could process 4 items at once here if we iterate differently,
        // but for now, we rely on `inline(always)` to merge these ops.
        
        match d {
            0 => w.write_2bits(0b00),
            1 => w.write_2bits(0b01),
            -1 => w.write_2bits(0b10),
            _ => {
                w.write_2bits(0b11); // Escape
                w.write_byte(d as u8); // Full byte
            }
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
            Some(0b11) => {
                if let Some(lit) = r.read_byte() { res.push(lit as i8); } 
                else { break; }
            },
            _ => break,
        }
    }
    res
}

// --- Engine ---

// --- Adaptive Engine (Phase 6) ---

const SUB_BLOCK_SIZE: usize = 16 * 1024; // 16KB sub-blocks for adaptive selection

pub fn compress_chunk(chunk: &[u8]) -> io::Result<Vec<u8>> {
    let mut inner_buffer = Vec::with_capacity(chunk.len() + 1024); // Reserve slightly more

    // Iterate over 16KB sub-blocks (or remainder)
    for sub_chunk in chunk.chunks(SUB_BLOCK_SIZE) {
        // Strategy A: QRES v2 (Bit-Packed Deltas)
        let deltas = delta_encode(sub_chunk);
        let packed = bit_pack_encode(&deltas);

        // Strategy B: Raw (Pass-through)
        // Overhead for either is: 1 byte Tag + 2 bytes Length = 3 bytes
        
        // Race: Pick smallest with Bias
        // Zlib prefers Raw data (LZ77 friendly) over Bit-Packed (High Entropy/Misaligned).
        // Only choose QRES if it significantly reduces size (e.g. < 85% of original).
        // If it's borderline (e.g. 99%), keeping it Raw lets Zlib do a better job.
        
        // Threshold: 0.85 * 16384 roughly.
        // We use integer math: packed.len() * 100 < sub_chunk.len() * 85
        
        let threshold = (sub_chunk.len() * 85) / 100;
        
        if packed.len() < threshold {
            // Mode 0x01: QRES
            inner_buffer.push(0x01);
            let len = packed.len() as u16;
            inner_buffer.extend_from_slice(&len.to_le_bytes());
            inner_buffer.extend_from_slice(&packed);
        } else {
            // Mode 0x00: Raw
            inner_buffer.push(0x00);
            let len = sub_chunk.len() as u16;
            inner_buffer.extend_from_slice(&len.to_le_bytes());
            inner_buffer.extend_from_slice(sub_chunk);
        }
    }

    // Zlib Stage (Zlib-ng)
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    e.write_all(&inner_buffer)?;
    e.finish()
}

pub fn decompress_chunk(compressed: &[u8]) -> io::Result<Vec<u8>> {
    // 1. Zlib Decompress
    let mut d = ZlibDecoder::new(compressed);
    let mut inner = Vec::new();
    d.read_to_end(&mut inner)?;
    
    // 2. Parse Adaptive Stream
    let mut result = Vec::new(); // Ideally we'd reserve if we knew total size, usually inner.len() * 2 is safe guess
    let mut cursor = 0;

    while cursor < inner.len() {
        // Check header availability
        if cursor + 3 > inner.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Truncated block header"));
        }

        let tag = inner[cursor];
        let len_bytes: [u8; 2] = inner[cursor+1..cursor+3].try_into().unwrap();
        let len = u16::from_le_bytes(len_bytes) as usize;
        cursor += 3;

        // Check body availability
        if cursor + len > inner.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Truncated block body"));
        }

        let body = &inner[cursor..cursor+len];

        match tag {
            0x01 => {
                // QRES Mode
                let deltas = bit_pack_decode(body);
                let decoded_bytes = delta_decode(&deltas);
                result.extend_from_slice(&decoded_bytes);
            },
            0x00 => {
                // Raw Mode
                result.extend_from_slice(body);
            },
            _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown block tag")),
        }

        cursor += len;
    }

    Ok(result)
}

// --- Zero-Copy Python Bindings ---

/// Accepts bytes, bytearray, or numpy array directly via the Buffer Protocol
#[pyfunction]
fn encode_buffer<'a>(py: Python<'a>, buffer: PyBuffer<u8>) -> PyResult<&'a PyBytes> {
    // 1. Zero-copy read from Python
    // If the buffer is contiguous (C-style), we get a slice.
    // If not, PyBuffer handles the complexity or returns error.
    let slice = buffer.as_slice(py)
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("Buffer must be contiguous (C-style)"))?;
    
    // 2. Compress
    // Note: 'compress_chunk' creates a NEW Vec<u8>. We return this as PyBytes.
    // This part involves one allocation (for the result), which is unavoidable.
    let compressed = compress_chunk(slice)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        
    Ok(PyBytes::new(py, &compressed))
}

#[pyfunction]
fn decode_bytes<'a>(py: Python<'a>, data: &[u8]) -> PyResult<&'a PyBytes> {
    let decompressed = decompress_chunk(data)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    Ok(PyBytes::new(py, &decompressed))
}

#[pymodule]
fn qres_rust(_py: Python, m: &PyModule) -> PyResult<()> {
    // Expose the new buffer-aware function
    m.add_function(wrap_pyfunction!(encode_buffer, m)?)?;
    m.add_function(wrap_pyfunction!(decode_bytes, m)?)?;
    Ok(())
}

// --- CLI Handlers (Preserved for main.rs compatibility) ---

pub fn compress_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let input_file = File::open(input_path)?;
    let mut reader = BufReader::new(input_file);

    let mut raw_chunks: Vec<Vec<u8>> = Vec::new();
    let mut original_size: u64 = 0;
    
    loop {
        let mut chunk = vec![0u8; CHUNK_SIZE];
        let bytes_read = reader.read(&mut chunk)?;
        if bytes_read == 0 { break; }
        chunk.truncate(bytes_read);
        original_size += bytes_read as u64;
        raw_chunks.push(chunk);
    }

    println!("Encoding {} chunks (v2 Optimized)...", raw_chunks.len());

    let compressed_chunks: Vec<Vec<u8>> = raw_chunks.par_iter()
        .map(|chunk| compress_chunk(chunk).unwrap()) 
        .collect();

    let compressed_size: u64 = compressed_chunks.iter().map(|c| c.len() as u64).sum();
    let chunk_sizes: Vec<u64> = compressed_chunks.iter().map(|c| c.len() as u64).collect();

    let header = QresHeader {
        version: 2,
        timestamp: Utc::now().timestamp(),
        original_size,
        compressed_size,
        file_name: Path::new(input_path).file_name().unwrap().to_string_lossy().to_string(),
        chunk_compressed_sizes: chunk_sizes,
    };

    let header_bytes = bincode::serialize(&header).unwrap();
    let mut writer = BufWriter::new(File::create(output_path)?);
    
    writer.write_all(QRES_MAGIC)?;
    writer.write_all(&(header_bytes.len() as u32).to_be_bytes())?;
    writer.write_all(&header_bytes)?;

    for chunk in compressed_chunks {
        writer.write_all(&chunk)?;
    }
    
    println!("Compressed: {} -> {} bytes (Ratio: {:.2}%)", 
        original_size, 
        compressed_size + header_bytes.len() as u64 + 8,
        (compressed_size as f64 / original_size as f64) * 100.0
    );
    Ok(())
}

pub fn decompress_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let input_file = File::open(input_path)?;
    let mut reader = BufReader::new(input_file);

    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    if &magic != QRES_MAGIC {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Not a QRES file"));
    }

    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf)?;
    let header_len = u32::from_be_bytes(len_buf) as usize;
    let mut header_buf = vec![0u8; header_len];
    reader.read_exact(&mut header_buf)?;
    let header: QresHeader = bincode::deserialize(&header_buf).unwrap();

    let mut output = BufWriter::new(File::create(output_path)?);
    
    println!("Decompressing: {} (v{})", header.file_name, header.version);

    for (i, size) in header.chunk_compressed_sizes.iter().enumerate() {
        let mut compressed_chunk = vec![0u8; *size as usize];
        reader.read_exact(&mut compressed_chunk)?;
        let decompressed_chunk = decompress_chunk(&compressed_chunk)?;
        output.write_all(&decompressed_chunk)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_writer_reader() {
        let mut writer = BitWriter::new_with_capacity(10);
        writer.write_2bits(0b00);
        writer.write_2bits(0b01);
        writer.write_2bits(0b10);
        writer.write_2bits(0b11);
        
        let bytes = writer.flush();
        assert_eq!(bytes.len(), 1);
        assert_eq!(bytes[0], 0x1B); // 00 01 10 11 -> 00011011 = 0x1B
        
        let mut reader = BitReader::new(&bytes);
        assert_eq!(reader.read_2bits(), Some(0b00));
        assert_eq!(reader.read_2bits(), Some(0b01));
        assert_eq!(reader.read_2bits(), Some(0b10));
        assert_eq!(reader.read_2bits(), Some(0b11));
        assert_eq!(reader.read_2bits(), None);
    }
}
