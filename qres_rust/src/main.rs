use std::fs::File;
use std::io::{Read, Write, BufReader, BufWriter};
use std::path::Path;
use chrono::Utc;
use serde::{Serialize, Deserialize};
use std::env;
use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use flate2::Compression;

#[derive(Serialize, Deserialize, Debug)]
struct QresHeader {
    timestamp: i64,
    original_size: u64,
    compressed_size: u64,
    file_name: String,
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

fn compress_file(input_path: &str, output_path: &str) -> std::io::Result<()> {
    let input_file = File::open(input_path)?;
    let mut reader = BufReader::new(input_file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    let encoded = delta_encode(&buffer);
    let encoded_bytes: Vec<u8> = encoded.iter().map(|b| *b as u8).collect();

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&encoded_bytes)?;
    let compressed_data = encoder.finish()?;

    let header = QresHeader {
        timestamp: Utc::now().timestamp(),
        original_size: buffer.len() as u64,
        compressed_size: compressed_data.len() as u64,
        file_name: Path::new(input_path).file_name().unwrap().to_string_lossy().to_string(),
    };

    let mut writer = BufWriter::new(File::create(output_path)?);
    writer.write_all(&bincode::serialize(&header).unwrap())?;
    writer.write_all(&compressed_data)?;
    Ok(())
}

fn decompress_file(input_path: &str, output_path: &str) -> std::io::Result<()> {
    let input_file = File::open(input_path)?;
    let mut reader = BufReader::new(input_file);

    let mut header_buf = vec![0u8; 1024];
    reader.read_exact(&mut header_buf)?;
    let header: QresHeader = bincode::deserialize(&header_buf).unwrap();

    let mut compressed_data = Vec::new();
    reader.read_to_end(&mut compressed_data)?;

    let mut decoder = ZlibDecoder::new(&compressed_data[..]);
    let mut decoded_bytes = Vec::new();
    decoder.read_to_end(&mut decoded_bytes)?;

    let decoded_deltas: Vec<i8> = decoded_bytes.iter().map(|b| *b as i8).collect();
    let restored_data = delta_decode(&decoded_deltas);

    let mut output = BufWriter::new(File::create(output_path)?);
    output.write_all(&restored_data)?;
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        println!("Usage:\n  qres_rust compress <input> <output>\n  qres_rust decompress <input> <output>");
        return;
    }

    match args[1].as_str() {
        "compress" => {
            compress_file(&args[2], &args[3]).expect("Compression failed");
            println!("Compressed successfully to {}", &args[3]);
        }
        "decompress" => {
            decompress_file(&args[2], &args[3]).expect("Decompression failed");
            println!("Decompressed successfully to {}", &args[3]);
        }
        _ => println!("Unknown command: {}", args[1]),
    }
}
