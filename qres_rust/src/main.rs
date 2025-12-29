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

// 2. Hybrid RLE: The "Senior Dev" Fix
// Does not compress runs shorter than 4 bytes to avoid expansion.
// Format:
// [0xFF] [Value] [Count (u16)] -> Run of 'Value' for 'Count' times
// [Literal Byte] -> Single literal (if not 0xFF)
// [0xFF] [0xFF] [0x01] [0x00] -> Escaped 0xFF literal (edge case)
fn rle_encode(deltas: &[i8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(deltas.len());
    if deltas.is_empty() { return result; }

    let mut i = 0;
    while i < deltas.len() {
        let current = deltas[i];
        let mut run_len = 1u16;

        // Look ahead for a run
        while i + (run_len as usize) < deltas.len() 
            && deltas[i + run_len as usize] == current 
            && run_len < u16::MAX 
        {
            run_len += 1;
        }

        // HEURISTIC: Only compress if run saves space (run > 3)
        // Or if the value is the special marker 0xFF (which must be escaped)
        let is_marker = current as u8 == 0xFF;

        if run_len > 3 || (is_marker && run_len > 1) {
            // Write Run
            result.push(0xFF);
            result.push(current as u8);
            result.extend_from_slice(&run_len.to_le_bytes());
            i += run_len as usize;
        } else {
            // Write Literals (Packet of 1)
            // If it happens to be 0xFF, we MUST encode it as a run of 1 to escape it
            if is_marker {
                result.push(0xFF);
                result.push(0xFF);
                result.extend_from_slice(&1u16.to_le_bytes());
                i += 1;
            } else {
                result.push(current as u8);
                i += 1;
            }
        }
    }
    result
}

fn rle_decode(encoded: &[u8]) -> Vec<i8> {
    let mut result = Vec::new();
    let mut i = 0;
    while i < encoded.len() {
        if encoded[i] == 0xFF {
            // It's a run (or escaped literal)
            if i + 3 >= encoded.len() { break; } // Safety check
            let val = encoded[i+1] as i8;
            let count = u16::from_le_bytes([encoded[i+2], encoded[i+3]]);
            for _ in 0..count {
                result.push(val);
            }
            i += 4;
        } else {
            // It's a literal
            result.push(encoded[i] as i8);
            i += 1;
        }
    }
    result
}

fn compress_chunk(chunk: &[u8]) -> io::Result<Vec<u8>> {
    let deltas = delta_encode(chunk);
    let rle_data = rle_encode(&deltas);
    // Zlib Stage (Level 6 default is good balance)
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&rle_data)?;
    encoder.finish()
}

fn decompress_chunk(compressed: &[u8]) -> io::Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(compressed);
    let mut decoded = Vec::new();
    decoder.read_to_end(&mut decoded)?;
    let deltas = rle_decode(&decoded);
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

    println!("Encoding {} chunks...", raw_chunks.len());

    // Parallel processing
    let compressed_chunks: Vec<Vec<u8>> = raw_chunks.par_iter()
        .map(|chunk| compress_chunk(chunk).unwrap()) 
        .collect();

    let compressed_size: u64 = compressed_chunks.iter().map(|c| c.len() as u64).sum();
    let chunk_sizes: Vec<u64> = compressed_chunks.iter().map(|c| c.len() as u64).collect();

    let header = QresHeader {
        version: 1,
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
    
    println!("Decompressing: {}", header.file_name);

    for size in header.chunk_compressed_sizes {
        let mut compressed_chunk = vec![0u8; size as usize];
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