use qres_rust::ans_coder::{AnsReader, AnsWriter};
use rand::prelude::*;

fn main() {
    let mut rng = StdRng::seed_from_u64(42);

    let mut data = Vec::with_capacity(1_000_000);
    for _ in 0..1_000_000 {
        // Skewed distribution: 80% zeros, rest small ints
        let val = if rng.gen_bool(0.8) {
            0
        } else if rng.gen_bool(0.5) {
            rng.gen_range(-5..5)
        } else {
            rng.gen_range(-20..20)
        };
        data.push(val as i8);
    }

    println!("Generated 1MB data. Encoding...");

    let start = std::time::Instant::now();
    let mut writer = AnsWriter::new();
    for &byte in &data {
        writer.write_residual(byte);
    }
    let compressed = writer.finish();
    let duration = start.elapsed();

    let ratio = compressed.len() as f64 / data.len() as f64;
    println!("Compressed Size: {} bytes", compressed.len());
    println!("Compression Ratio: {:.4}", ratio);
    println!("Speed: {:.2} MB/s", 1.0 / duration.as_secs_f64());

    assert!(ratio < 0.50, "Ratio should be good for sparse data");

    // Decompression
    println!("Decoding...");
    let mut reader = AnsReader::new(&compressed, data.len());
    for (i, &expected) in data.iter().enumerate() {
        let actual = reader.read_residual();
        if actual != expected {
            panic!(
                "Mismatch at index {}: expected {}, got {}",
                i, expected, actual
            );
        }
    }
    println!("Round trip successful!");
}
