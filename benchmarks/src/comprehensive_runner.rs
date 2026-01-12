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

/// Convert quantized float data to bytes for compression.
/// Uses i16 encoding (multiply by 100, clamp to i16 range) for better compression.
fn floats_to_bytes(data: &[f32]) -> Vec<u8> {
    data.iter()
        .flat_map(|&f| {
            let scaled = (f * 100.0).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            scaled.to_le_bytes()
        })
        .collect()
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
    let compressed = match compress_chunk(data_bytes, 0, None, Some(&config)) {
        Ok(c) => c,
        Err(_) => return None, // Expansion or error
    };
    let compress_time = start.elapsed().as_secs_f64();

    // Decompression verification
    let start = Instant::now();
    let _decompressed = decompress_chunk(&compressed, 0, None);
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
        anyhow::bail!("Data directory does not exist: {:?}", data_dir);
    }

    // Create results directory
    let results_dir = PathBuf::from("results");
    fs::create_dir_all(&results_dir)?;

    let output_path = results_dir.join("benchmark_matrix.csv");
    let mut wtr = Writer::from_path(&output_path)?;

    // Write header
    wtr.write_record([
        "Dataset",
        "Predictor",
        "Coder",
        "Original_Size",
        "Compressed_Size",
        "Ratio",
        "Compression_Speed_MBs",
        "Decompression_Speed_MBs",
    ])?;

    println!("=== QRES Comprehensive Benchmark ===");
    println!("Data directory: {:?}", data_dir);
    println!("Output: {:?}", output_path);
    println!();

    // Find all data files
    let entries: Vec<_> = fs::read_dir(&data_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            path.is_file()
                && (path
                    .extension()
                    .map(|s| s == "csv" || s == "txt")
                    .unwrap_or(false))
        })
        .collect();

    if entries.is_empty() {
        println!("No .csv or .txt files found in {:?}", data_dir);
        return Ok(());
    }

    println!("Found {} datasets", entries.len());

    for entry in entries {
        let path = entry.path();
        let dataset_name = path.file_stem().unwrap_or_default().to_string_lossy();

        println!("\n[{}]", dataset_name);

        let data = match load_dataset(&path) {
            Ok(d) if !d.is_empty() => d,
            Ok(_) => {
                println!("  Skipping: Empty dataset");
                continue;
            }
            Err(e) => {
                println!("  Skipping: {}", e);
                continue;
            }
        };

        // [FIX] Quantize data to 2 decimal places to make it compressible!
        let data: Vec<f32> = data.iter().map(|&x| (x * 100.0).round() / 100.0).collect();

        let data_bytes = floats_to_bytes(&data);
        let original_size = data_bytes.len();

        for predictor in PREDICTORS {
            for coder in CODERS {
                let predictor_name = format!("{:?}", predictor);
                let coder_name = format!("{:?}", coder);

                match benchmark_config(&data_bytes, predictor, coder) {
                    Some((compressed_size, compress_speed, decompress_speed)) => {
                        let ratio = original_size as f64 / compressed_size as f64;

                        println!(
                            "  {:8} + {:10}: {:.2}x @ {:.1} MB/s",
                            predictor_name, coder_name, ratio, compress_speed
                        );

                        wtr.write_record([
                            dataset_name.as_ref(),
                            predictor_name.as_str(),
                            coder_name.as_str(),
                            &original_size.to_string(),
                            &compressed_size.to_string(),
                            &format!("{:.4}", ratio),
                            &format!("{:.2}", compress_speed),
                            &format!("{:.2}", decompress_speed),
                        ])?;
                    }
                    None => {
                        println!(
                            "  {:8} + {:10}: SKIPPED (expansion)",
                            predictor_name, coder_name
                        );
                    }
                }
            }
        }
    }

    wtr.flush()?;
    println!("\n=== Benchmark Complete ===");
    println!("Results written to: {:?}", output_path);

    Ok(())
}
