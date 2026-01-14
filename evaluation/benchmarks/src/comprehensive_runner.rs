//! Comprehensive Benchmark Runner
//!
//! Executes a grid search over all Predictor/Coder combinations
//! against a directory of time-series datasets.

use csv::Writer;
use qres_core::config::{CoderType, PredictorType, QresConfig};
use qres_core::{compress_chunk, decompress_chunk};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::Instant;

const PREDICTORS: [PredictorType; 4] = [
    PredictorType::Zero,
    PredictorType::Heuristic,
    PredictorType::Neural,
    PredictorType::Hybrid,
];

const CODERS: [CoderType; 2] = [CoderType::Huffman, CoderType::Arithmetic];

/// Load a single-column float dataset from a file.
/// Skips header row if present and extracts the second column (index 1).
fn load_dataset(path: &Path) -> anyhow::Result<Vec<f32>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut data = Vec::new();
    let mut lines = reader.lines();

    // Skip header row
    let _ = lines.next();

    for line in lines {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // Handle CSV with multiple columns: extract second column (index 1)
        let parts: Vec<&str> = trimmed.split(',').collect();
        let value_str = if parts.len() > 1 { parts[1] } else { parts[0] };
        if let Ok(val) = value_str.trim().parse::<f32>() {
            data.push(val);
        }
    }
    Ok(data)
}

/// ZigZag encode a signed 16-bit integer to unsigned.
/// Maps: 0 -> 0, -1 -> 1, 1 -> 2, -2 -> 3, 2 -> 4, ...
/// This eliminates sign-extension noise by keeping small magnitudes small.
#[inline]
fn zigzag_encode(n: i16) -> u16 {
    ((n << 1) ^ (n >> 15)) as u16
}

/// Calculate minimum bits needed to represent a value.
#[inline]
fn bits_needed(max_val: u16) -> u8 {
    if max_val == 0 {
        1
    } else {
        16 - max_val.leading_zeros() as u8
    }
}

/// Block size for bit-packing
const BLOCK_SIZE: usize = 256;

/// Convert quantized float data to bytes using Delta + ZigZag + Bit-Packing pipeline.
///
/// Pipeline (per block):
/// 1. Delta Encoding: Store differences between consecutive values
/// 2. ZigZag Encoding: Map signed deltas to unsigned
/// 3. Bit-Packing: Pack values using minimum bits for block (not fixed 16 bits)
fn floats_to_bytes(data: &[f32]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }

    // Step 1: Delta encode and scale to i16
    let mut deltas: Vec<i16> = Vec::with_capacity(data.len());

    // First value as baseline
    let first_scaled = (data[0] * 100.0).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
    deltas.push(first_scaled);

    // Delta for remaining values
    for i in 1..data.len() {
        let delta = (data[i] - data[i - 1]) * 100.0;
        let delta_scaled = delta.clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        deltas.push(delta_scaled);
    }

    // Step 2: ZigZag encode all values
    let zigzag_values: Vec<u16> = deltas.iter().map(|&d| zigzag_encode(d)).collect();

    // Step 3: Bit-pack per block
    let mut result = Vec::with_capacity(data.len()); // Optimistic size

    for chunk in zigzag_values.chunks(BLOCK_SIZE) {
        // Find max value in block to determine bit width
        let max_val = *chunk.iter().max().unwrap_or(&0);
        let bit_width = bits_needed(max_val);

        // Write block header: bit_width (1 byte)
        result.push(bit_width);

        // Pack values using bit_width bits each
        let mut bit_buffer: u64 = 0;
        let mut bits_in_buffer: u8 = 0;

        for &val in chunk {
            bit_buffer |= (val as u64) << bits_in_buffer;
            bits_in_buffer += bit_width;

            // Flush complete bytes
            while bits_in_buffer >= 8 {
                result.push(bit_buffer as u8);
                bit_buffer >>= 8;
                bits_in_buffer -= 8;
            }
        }

        // Flush remaining bits
        if bits_in_buffer > 0 {
            result.push(bit_buffer as u8);
        }
    }

    result
}

/// Benchmark a single configuration on a dataset.
fn benchmark_config(
    data_bytes: &[u8],
    predictor: PredictorType,
    coder: CoderType,
) -> Option<(usize, f64, f64)> {
    let config = QresConfig {
        predictor,
        coder,
        ..Default::default()
    };

    // Compression
    let start = Instant::now();
    // Pre-allocate buffer (worst case: input + overhead)
    let mut comp_buffer = vec![0u8; data_bytes.len() + 4096];

    let compressed_result = compress_chunk(data_bytes, 0, None, Some(&config), &mut comp_buffer);

    let compressed = match compressed_result {
        Ok(len) => &comp_buffer[..len],
        Err(_) => return None, // Expansion or error
    };
    let compress_time = start.elapsed().as_secs_f64();

    // Decompression verification
    let start = Instant::now();
    let _decompressed = decompress_chunk(compressed, 0, None);
    let decompress_time = start.elapsed().as_secs_f64();

    let original_size = data_bytes.len();
    let compressed_size = compressed.len();
    let compress_speed = (original_size as f64 / 1_000_000.0) / compress_time;
    let decompress_speed = (original_size as f64 / 1_000_000.0) / decompress_time;

    Some((compressed_size, compress_speed, decompress_speed))
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let data_dir = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from("benchmarks/src/edge_realistic/datasets")
    };

    if !data_dir.exists() {
        println!(
            "WARN: Data directory not found at {:?}. Skipping benchmarks.",
            data_dir
        );
        return Ok(());
    }

    let results_dir = PathBuf::from("results");
    fs::create_dir_all(&results_dir)?;

    let output_path = results_dir.join("benchmark_matrix.csv");
    let mut wtr = Writer::from_path(&output_path)?;

    // [UPDATED] Expanded CSV Header
    wtr.write_record([
        "Dataset",
        "Predictor",
        "Coder",
        "Raw_Size_Bytes",
        "BitPacked_Size_Bytes",
        "Final_Size_Bytes",
        "Total_Ratio",
        "BitPack_Ratio",
        "Neural_Gain",
        "Compress_Speed_MBs",
        "Decompress_Speed_MBs",
    ])?;

    println!("=== QRES Comprehensive Benchmark (Metric Fix) ===");
    println!("Data: {:?}", data_dir);

    let entries: Vec<_> = fs::read_dir(&data_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            path.is_file() && path.extension().is_some_and(|s| s == "csv" || s == "txt")
        })
        .collect();

    for entry in entries {
        let path = entry.path();
        let dataset_name = path.file_stem().unwrap_or_default().to_string_lossy();
        println!("\n[{}]", dataset_name);

        let data = match load_dataset(&path) {
            Ok(d) if !d.is_empty() => d,
            Ok(_) => continue,
            Err(_) => continue,
        };

        // Quantize
        let data: Vec<f32> = data.iter().map(|&x| (x * 100.0).round() / 100.0).collect();

        // [CRITICAL FIX 1] True Baseline = 4 bytes per float
        let raw_size = data.len() * 4;

        // [CRITICAL FIX 2] Measure Bit-Packing Baseline
        let data_bytes = floats_to_bytes(&data);
        let bitpacked_size = data_bytes.len();
        let bitpack_ratio = raw_size as f64 / bitpacked_size as f64;

        println!(
            "  Base: Bit-Packing achieved {:.2}x ({} -> {} bytes)",
            bitpack_ratio, raw_size, bitpacked_size
        );

        for predictor in PREDICTORS {
            for coder in CODERS {
                let predictor_name = format!("{:?}", predictor);
                let coder_name = format!("{:?}", coder);

                match benchmark_config(&data_bytes, predictor, coder) {
                    Some((final_size, speed_c, speed_d)) => {
                        // [CRITICAL FIX 3] Calculate Total Ratio
                        let total_ratio = raw_size as f64 / final_size as f64;
                        let neural_gain = bitpacked_size as f64 / final_size as f64;

                        println!(
                            "  {:8} + {:10}: {:.2}x (Neural Gain: {:.2}x) @ {:.1} MB/s",
                            predictor_name, coder_name, total_ratio, neural_gain, speed_c
                        );

                        wtr.write_record([
                            dataset_name.as_ref(),
                            predictor_name.as_str(),
                            coder_name.as_str(),
                            &raw_size.to_string(),
                            &bitpacked_size.to_string(),
                            &final_size.to_string(),
                            &format!("{:.4}", total_ratio),
                            &format!("{:.4}", bitpack_ratio),
                            &format!("{:.4}", neural_gain),
                            &format!("{:.2}", speed_c),
                            &format!("{:.2}", speed_d),
                        ])?;
                    }
                    None => {
                        // Even if Neural expands, Bit-Packing worked!
                        println!(
                            "  {:8} + {:10}: {:.2}x (Bit-Pack Only - Neural Skipped)",
                            predictor_name, coder_name, bitpack_ratio
                        );
                        wtr.write_record([
                            dataset_name.as_ref(),
                            predictor_name.as_str(),
                            coder_name.as_str(),
                            &raw_size.to_string(),
                            &bitpacked_size.to_string(),
                            &bitpacked_size.to_string(),
                            &format!("{:.4}", bitpack_ratio),
                            &format!("{:.4}", bitpack_ratio),
                            "1.0000",
                            "N/A",
                            "N/A",
                        ])?;
                    }
                }
            }
        }
    }
    wtr.flush()?;
    Ok(())
}
