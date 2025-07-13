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

#[derive(Serialize, Deserialize, Debug)]
struct QresHeader {
    timestamp: i64,
    original_size: u64,
    compressed_size: u64,
    file_name: String,
    chunk_compressed_sizes: Vec<u64>, // New: sizes for each compressed chunk
}

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

fn rle_encode(deltas: &[i8]) -> Vec<u8> {
    let mut result = Vec::new();
    if deltas.is_empty() {
        return result;
    }
    let mut current = deltas[0];
    let mut count: u16 = 1;
    for &val in &deltas[1..] {
        if val == current && count < u16::MAX {
            count += 1;
        } else {
            result.push(0xFF); // Marker for run
            result.extend_from_slice(&current.to_le_bytes());
            result.extend_from_slice(&count.to_le_bytes());
            current = val;
            count = 1;
        }
    }
    result.push(0xFF);
    result.extend_from_slice(&current.to_le_bytes());
    result.extend_from_slice(&count.to_le_bytes());
    result
}

fn rle_decode(encoded: &[u8]) -> Vec<i8> {
    let mut result = Vec::new();
    let mut i = 0;
    while i < encoded.len() {
        if encoded[i] == 0xFF {
            i += 1;
            if i + 3 > encoded.len() {
                break; // Error handling simplified
            }
            let val = i8::from_le_bytes([encoded[i]]);
            let count = u16::from_le_bytes([encoded[i+1], encoded[i+2]]);
            for _ in 0..count {
                result.push(val);
            }
            i += 3;
        } else {
            result.push(encoded[i] as i8);
            i += 1;
        }
    }
    result
}

fn compress_chunk(chunk: &[u8]) -> io::Result<Vec<u8>> {
    let deltas = delta_encode(chunk);
    let rle_data = rle_encode(&deltas);
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
    loop {
        let mut chunk = vec![0u8; CHUNK_SIZE];
        let bytes_read = reader.read(&mut chunk)?;
        if bytes_read == 0 {
            break;
        }
        chunk.truncate(bytes_read);
        original_size += bytes_read as u64;
        raw_chunks.push(chunk);
    }

    // Parallel compress chunks with rayon
    let compressed_chunks: Vec<Vec<u8>> = raw_chunks.par_iter()
        .map(|chunk| compress_chunk(chunk).unwrap()) // Unwrap for simplicity; handle properly in prod
        .collect();

    let compressed_size: u64 = compressed_chunks.iter().map(|c| c.len() as u64).sum();
    let chunk_sizes: Vec<u64> = compressed_chunks.iter().map(|c| c.len() as u64).collect();

    let header = QresHeader {
        timestamp: Utc::now().timestamp(),
        original_size,
        compressed_size,
        file_name: Path::new(input_path).file_name().unwrap().to_string_lossy().to_string(),
        chunk_compressed_sizes: chunk_sizes,
    };

    let header_bytes = bincode::serialize(&header)?;
    let mut writer = BufWriter::new(File::create(output_path)?);
    writer.write_all(&(header_bytes.len() as u32).to_be_bytes())?;
    writer.write_all(&header_bytes)?;

    for chunk in compressed_chunks {
        writer.write_all(&chunk)?;
    }
    Ok(())
}

fn decompress_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let input_file = File::open(input_path)?;
    let mut reader = BufReader::new(input_file);

    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf)?;
    let header_len = u32::from_be_bytes(len_buf) as usize;
    let mut header_buf = vec![0u8; header_len];
    reader.read_exact(&mut header_buf)?;
    let header: QresHeader = bincode::deserialize(&header_buf)?;

    let mut output = BufWriter::new(File::create(output_path)?);
    let mut restored_size: u64 = 0;

    for size in header.chunk_compressed_sizes {
        let mut compressed_chunk = vec![0u8; size as usize];
        reader.read_exact(&mut compressed_chunk)?;
        let decompressed_chunk = decompress_chunk(&compressed_chunk)?;
        output.write_all(&decompressed_chunk)?;
        restored_size += decompressed_chunk.len() as u64;
    }

    if restored_size != header.original_size {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Size mismatch after decompression"));
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        println!("Usage:\n  qres_rust compress <input> <output>\n  qres_rust decompress <input> <output>");
        return;
    }

    match args[1].as_str() {
        "compress" => compress_file(&args[2], &args[3]).expect("Compression failed"),
        "decompress" => decompress_file(&args[2], &args[3]).expect("Decompression failed"),
        _ => println!("Unknown command: {}", args[1]),
    }
}