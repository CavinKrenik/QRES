use std::fs::File;
use std::io::{self, Read, Write, BufReader, BufWriter};
use std::path::Path;
use chrono::Utc;
use serde::{Serialize, Deserialize};
use std::env;
use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use flate2::Compression;
use rayon::prelude::*;

const CHUNK_SIZE: usize = 4 * 1024 * 1024; // 4MB chunks
const QRES_MAGIC: &[u8] = b"QRES";

#[derive(Serialize, Deserialize, Debug)]
struct QresHeader {
    version: u8,
    timestamp: i64,
    original_size: u64,
    compressed_size: u64,
    file_name: String,
    chunk_compressed_sizes: Vec<u64>,
}

// --- Safety Infrastructure: BitWriter & BitReader ---

struct BitWriter {
    buffer: Vec<u8>,
    current_byte: u8,
    bit_count: u8,
}

impl BitWriter {
    fn new() -> Self {
        BitWriter {
            buffer: Vec::new(),
            current_byte: 0,
            bit_count: 0,
        }
    }

    // Write up to 8 bits
    fn write_bits(&mut self, value: u8, bits: u8) {
        if bits == 0 { return; }
        
        let mut bits_left = bits;
        let mut value_shift = value;

        while bits_left > 0 {
            let space_in_byte = 8 - self.bit_count;
            let bits_to_write = std::cmp::min(bits_left, space_in_byte);

            // Mask the value to keep only the bits we want to write
            // (Assumes value is right-aligned for the chunk we want)
            // Actually, usually we write MSB or LSB. Let's stick to LSB filling.
            // But for 2-bit codes, order matters.
            // Let's write bits such that later we read them in same order.
            // If we write 01 (Value 1) with 2 bits.
            // We put it at the "bottom" of the current stream?
            // Standard idiom: Fill byte standardly.
            
            // Implementation: Fill MSB to LSB or LSB to MSB? 
            // Let's use LSB to MSB packing for simplicity or MSB to LSB?
            // MSB to LSB is easier to read in hex dump.
            // Example: Write 01 (0x1) -> Byte: 01xxxxxx
            
            // Let's do MSB-first packing.
            // current_byte: [xxxx....]
            
            let shift = space_in_byte - bits_to_write;
            // Get the top 'bits_to_write' from 'value_shift' (which assumes value passed is lower-bit aligned? 
            // No, the caller usually passes 'value' as a u8 where the relevant bits are at the bottom).
            
            // To make it simple: iterate bits.
            for _ in 0..bits_to_write {
                // Take top bit of remaining value? No, let's take BIT BY BIT for safety if we are unsure,
                // BUT for speed, let's just mask.
                
                // Let's assume 'value' has 'bits' significant bits at the LSB.
                // We want to write the most significant of those first? 
                // E.g. write_bits(0b10, 2). We want 1 then 0.
                
                let bit_val = (value_shift >> (bits_left - 1)) & 1;
                self.current_byte |= bit_val << (7 - self.bit_count);
                self.bit_count += 1;
                bits_left -= 1;
                
                if self.bit_count == 8 {
                    self.buffer.push(self.current_byte);
                    self.current_byte = 0;
                    self.bit_count = 0;
                }
            }
        }
    }
    
    // Explicit 2-bit write helper for speed/clarity
    fn write_2bits(&mut self, value: u8) {
        // Optimizes for the common case
        // value must be 0..3
        self.write_bits(value, 2);
    }

    fn flush(&mut self) -> Vec<u8> {
        if self.bit_count > 0 {
            self.buffer.push(self.current_byte);
            self.current_byte = 0;
            self.bit_count = 0;
        }
        std::mem::take(&mut self.buffer)
    }
}

struct BitReader<'a> {
    buffer: &'a [u8],
    byte_index: usize,
    bit_offset: u8, // 0..7, MSB first
}

impl<'a> BitReader<'a> {
    fn new(buffer: &'a [u8]) -> Self {
        BitReader {
            buffer,
            byte_index: 0,
            bit_offset: 0,
        }
    }

    fn read_bits(&mut self, bits: u8) -> Option<u8> {
        if bits == 0 { return Some(0); }
        let mut result = 0u8;
        for _ in 0..bits {
            if self.byte_index >= self.buffer.len() {
                return None;
            }
            
            let bit = (self.buffer[self.byte_index] >> (7 - self.bit_offset)) & 1;
            result = (result << 1) | bit;
            
            self.bit_offset += 1;
            if self.bit_offset == 8 {
                self.bit_offset = 0;
                self.byte_index += 1;
            }
        }
        Some(result)
    }
}

// --- v2 Protocol Logic ---

// 1. Delta Encoding: Calculate difference between bytes
fn delta_encode(data: &[u8]) -> Vec<i8> {
    let mut result = Vec::with_capacity(data.len());
    let mut prev = 0u8;
    for &byte in data {
        result.push(byte.wrapping_sub(prev) as i8);
        prev = byte;
    }
    result
}

fn delta_decode(data: &[i8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len());
    let mut prev = 0u8;
    for &delta in data {
        let byte = prev.wrapping_add(delta as u8);
        result.push(byte);
        prev = byte;
    }
    result
}

// 2. Bit Packed Encoding (v2)
// 00: 0
// 01: +1
// 10: -1
// 11: Escape -> next 8 bits = literal
fn bit_pack_encode(deltas: &[i8]) -> Vec<u8> {
    let mut writer = BitWriter::new();
    
    // Header: Store number of deltas strictly (u32 LE)
    // This ensures we stop reading exactly when done, ignoring padding.
    let count = deltas.len() as u32;
    writer.buffer.extend_from_slice(&count.to_le_bytes()); 
    // Note: BitWriter writes to buffer. We just pre-filled the buffer.
    // This is safe because bit state (current_byte) is empty.
    
    for &d in deltas {
        match d {
            0 => writer.write_2bits(0b00),
            1 => writer.write_2bits(0b01),
            -1 => writer.write_2bits(0b10),
            _ => {
                writer.write_2bits(0b11); // Escape
                writer.write_bits(d as u8, 8); // Write full byte
            }
        }
    }
    
    writer.flush()
}

fn bit_pack_decode(encoded: &[u8]) -> Vec<i8> {
    if encoded.len() < 4 { return Vec::new(); }
    
    // Read count
    let count_bytes: [u8; 4] = encoded[0..4].try_into().unwrap();
    let count = u32::from_le_bytes(count_bytes) as usize;
    
    // Start bit reader after slice
    let mut reader = BitReader::new(&encoded[4..]);
    let mut result = Vec::with_capacity(count);
    
    for _ in 0..count {
        // Read 2-bit code
        let code = match reader.read_bits(2) {
            Some(c) => c,
            None => break, // Should not happen if count is correct
        };
        
        let delta = match code {
            0b00 => 0,
            0b01 => 1,
            0b10 => -1,
            0b11 => {
                // Escape
                match reader.read_bits(8) {
                    Some(lit) => lit as i8,
                    None => break,
                }
            },
            _ => unreachable!(),
        };
        result.push(delta);
    }
    
    result
}

fn compress_chunk(chunk: &[u8]) -> io::Result<Vec<u8>> {
    let deltas = delta_encode(chunk);
    let packed_data = bit_pack_encode(&deltas);
    // Zlib Stage
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&packed_data)?;
    encoder.finish()
}

fn decompress_chunk(compressed: &[u8]) -> io::Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(compressed);
    let mut decoded = Vec::new();
    decoder.read_to_end(&mut decoded)?;
    
    // In v2, the decoded data is the bit-packed stream
    let deltas = bit_pack_decode(&decoded);
    Ok(delta_decode(&deltas))
}

fn compress_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let input_file = File::open(input_path)?;
    let mut reader = BufReader::new(input_file);

    let mut raw_chunks: Vec<Vec<u8>> = Vec::new();
    let mut original_size: u64 = 0;
    
    // Read file in chunks
    loop {
        let mut chunk = vec![0u8; CHUNK_SIZE];
        let bytes_read = reader.read(&mut chunk)?;
        if bytes_read == 0 { break; }
        chunk.truncate(bytes_read);
        original_size += bytes_read as u64;
        raw_chunks.push(chunk);
    }

    println!("Encoding {} chunks (v2 Bit-Packed)...", raw_chunks.len());

    // Parallel processing
    let compressed_chunks: Vec<Vec<u8>> = raw_chunks.par_iter()
        .map(|chunk| compress_chunk(chunk).unwrap()) 
        .collect();

    let compressed_size: u64 = compressed_chunks.iter().map(|c| c.len() as u64).sum();
    let chunk_sizes: Vec<u64> = compressed_chunks.iter().map(|c| c.len() as u64).collect();

    let header = QresHeader {
        version: 2, // v2 Alpha
        timestamp: Utc::now().timestamp(),
        original_size,
        compressed_size,
        file_name: Path::new(input_path).file_name().unwrap().to_string_lossy().to_string(),
        chunk_compressed_sizes: chunk_sizes,
    };

    let header_bytes = bincode::serialize(&header).unwrap();
    let mut writer = BufWriter::new(File::create(output_path)?);
    
    // Write Binary Format
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

fn decompress_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let input_file = File::open(input_path)?;
    let mut reader = BufReader::new(input_file);

    // Validate Magic
    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    if &magic != QRES_MAGIC {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Not a QRES file"));
    }

    // Read Header
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf)?;
    let header_len = u32::from_be_bytes(len_buf) as usize;
    let mut header_buf = vec![0u8; header_len];
    reader.read_exact(&mut header_buf)?;
    let header: QresHeader = bincode::deserialize(&header_buf).unwrap();

    let mut output = BufWriter::new(File::create(output_path)?);
    
    println!("Decompressing: {} (v{})", header.file_name, header.version);

    for (i, size) in header.chunk_compressed_sizes.iter().enumerate() {
        println!("Reading chunk {} of size {}", i, size);
        let mut compressed_chunk = vec![0u8; *size as usize];
        reader.read_exact(&mut compressed_chunk)?;
        let decompressed_chunk = decompress_chunk(&compressed_chunk)?;
        output.write_all(&decompressed_chunk)?;
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage:\n  qres_rust compress <input> <output>\n  qres_rust decompress <input> <output>");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "compress" => compress_file(&args[2], &args[3]).expect("Compression failed"),
        "decompress" => decompress_file(&args[2], &args[3]).expect("Decompression failed"),
        _ => eprintln!("Unknown command: {}", args[1]),
    }
}

// --- Unit Tests ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_writer_reader() {
        let mut writer = BitWriter::new();
        // Write 00, 01, 10, 11
        writer.write_2bits(0b00);
        writer.write_2bits(0b01);
        writer.write_2bits(0b10);
        writer.write_2bits(0b11);
        
        let bytes = writer.flush();
        // Should be 1 byte: 00 01 10 11 -> 00011011 -> 0x1B (27)
        assert_eq!(bytes.len(), 1);
        assert_eq!(bytes[0], 0x1B);
        
        let mut reader = BitReader::new(&bytes);
        assert_eq!(reader.read_bits(2), Some(0b00));
        assert_eq!(reader.read_bits(2), Some(0b01));
        assert_eq!(reader.read_bits(2), Some(0b10));
        assert_eq!(reader.read_bits(2), Some(0b11));
        assert_eq!(reader.read_bits(2), None);
    }

    #[test]
    fn test_bit_pack_round_trip() {
        // Test patterns: 0, +1, -1, +10 (escape)
        let deltas: Vec<i8> = vec![0, 1, -1, 10, 0, -128];
        let encoded = bit_pack_encode(&deltas);
        let decoded = bit_pack_decode(&encoded);
        
        assert_eq!(deltas, decoded);
    }

    #[test]
    fn test_full_round_trip() {
        let original_data = vec![10, 11, 12, 11, 11, 50, 51]; 
        // Deltas: 10, 1, 1, -1, 0, 39, 1
        // encoded should handle the escapes.
        
        let compressed = compress_chunk(&original_data).unwrap();
        let restored = decompress_chunk(&compressed).unwrap();
        
        assert_eq!(original_data, restored);
    }
}
