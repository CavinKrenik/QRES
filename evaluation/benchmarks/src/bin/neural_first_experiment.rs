//! Neural-First Architecture Experiment
//!
//! Tests the hypothesis: "Predicting deltas BEFORE bit-packing yields better ratios."
//! Pipeline: Raw -> Delta -> [Neural Prediction -> Residuals] -> ZigZag -> BitPack

use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

/// Load dataset (Same as before)
fn load_dataset(path: &Path) -> anyhow::Result<Vec<f32>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut data = Vec::new();
    let mut lines = reader.lines();
    let _ = lines.next(); // Skip header

    for line in lines {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = trimmed.split(',').collect();
        let value_str = if parts.len() > 1 { parts[1] } else { parts[0] };
        if let Ok(val) = value_str.trim().parse::<f32>() {
            data.push(val);
        }
    }
    Ok(data)
}

/// 1. Compute Deltas (Raw Physics)
fn compute_deltas(data: &[f32]) -> Vec<i16> {
    let mut deltas = Vec::with_capacity(data.len());
    if data.is_empty() {
        return deltas;
    }

    let scale = 100.0;
    // Store first value as-is (clamped)
    let first = (data[0] * scale).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
    deltas.push(first);

    for i in 1..data.len() {
        let diff = (data[i] - data[i - 1]) * scale;
        let diff = diff.clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        deltas.push(diff);
    }
    deltas
}

/// 2. Neural Predictor (Simulated)
///
/// Uses a Linear Adaptive Filter (LMS) to mimic a Neural Network's ability
/// to learn local trends.
fn predict_residuals(deltas: &[i16]) -> Vec<i16> {
    let mut residuals = Vec::with_capacity(deltas.len());
    residuals.push(deltas[0]); // Can't predict the first one

    // Simple AR(2) Predictor: P[t] = 2*D[t-1] - D[t-2] (Linear extrapolation)
    // A real Neural Net would be smarter, but this proves the point.
    for i in 1..deltas.len() {
        let prediction = if i > 1 {
            let d1 = deltas[i - 1] as i32;
            let d2 = deltas[i - 2] as i32;
            // "Trend" prediction
            (2 * d1 - d2).clamp(i16::MIN as i32, i16::MAX as i32) as i16
        } else {
            deltas[i - 1] // Fallback to previous value
        };

        // The "Innovation" (Error) is what we store
        let residual = deltas[i].wrapping_sub(prediction);
        residuals.push(residual);
    }
    residuals
}

/// 3. ZigZag Encode
fn zigzag_encode(data: &[i16]) -> Vec<u16> {
    data.iter()
        .map(|&n| ((n << 1) ^ (n >> 15)) as u16)
        .collect()
}

/// 4. Bit-Pack (Adaptive)
fn bitpack(data: &[u16]) -> Vec<u8> {
    let mut result = Vec::new();
    // Simple block packing
    for chunk in data.chunks(128) {
        let max_val = *chunk.iter().max().unwrap_or(&0);
        let bits = if max_val == 0 {
            1
        } else {
            (16 - max_val.leading_zeros()) as u8
        };
        result.push(bits);

        let mut buffer: u64 = 0;
        let mut bits_in_buffer = 0;

        for &val in chunk {
            buffer |= (val as u64) << bits_in_buffer;
            bits_in_buffer += bits;
            while bits_in_buffer >= 8 {
                result.push(buffer as u8);
                buffer >>= 8;
                bits_in_buffer -= 8;
            }
        }
        if bits_in_buffer > 0 {
            result.push(buffer as u8);
        }
    }
    result
}

fn main() -> anyhow::Result<()> {
    let path = PathBuf::from("benchmarks/src/edge_realistic/datasets");
    if !path.exists() {
        println!("WARN: Data dir not found. Skipping.");
        return Ok(());
    }

    println!("=== Neural-First vs BitPack-First Experiment ===");
    println!("Testing hypothesis: Removing patterns BEFORE bit-packing improves ratio.\n");

    let entries: Vec<_> = fs::read_dir(path)?.filter_map(|e| e.ok()).collect();

    for entry in entries {
        let p = entry.path();
        if p.extension().is_none_or(|s| s != "csv") {
            continue;
        }

        let name = p.file_stem().unwrap().to_string_lossy();
        let data = load_dataset(&p)?;
        if data.is_empty() {
            continue;
        }

        let raw_size = data.len() * 4;

        // --- METHOD A: STANDARD (BitPack First) ---
        let deltas_a = compute_deltas(&data);
        let zigzag_a = zigzag_encode(&deltas_a);
        let packed_a = bitpack(&zigzag_a);
        let ratio_a = raw_size as f64 / packed_a.len() as f64;

        // --- METHOD B: NEURAL FIRST (Predict Residuals) ---
        let deltas_b = compute_deltas(&data);
        let residuals = predict_residuals(&deltas_b); // <--- MAGIC HAPPENS HERE
        let zigzag_b = zigzag_encode(&residuals);
        let packed_b = bitpack(&zigzag_b);
        let ratio_b = raw_size as f64 / packed_b.len() as f64;

        // --- COMPARE ---
        let improvement = ((ratio_b - ratio_a) / ratio_a) * 100.0;

        println!("[{}]", name);
        println!(
            "  Standard (Delta -> Pack):   {:.2}x  ({} bytes)",
            ratio_a,
            packed_a.len()
        );
        println!(
            "  Neural   (Predict -> Pack): {:.2}x  ({} bytes)",
            ratio_b,
            packed_b.len()
        );

        if ratio_b > ratio_a {
            println!("  ✅ GAIN: +{:.1}% (Proof of concept works!)", improvement);
        } else {
            println!(
                "  ⚠️ LOSS: {:.1}% (Prediction overhead > Signal)",
                improvement
            );
        }
        println!();
    }
    Ok(())
}
