use qres_core::encoding::arithmetic::{compress_residuals, decompress_residuals};
use rand::Rng;

fn main() {
    println!(">> QRES v16 Encoding Benchmark");
    println!("-------------------------------");

    // 1. Generate 1MB of Laplacian-like noise (peaked distribution)
    // This simulates prediction residuals where most values are near zero.
    let size = 1024 * 1024; // 1 MB
    let mut data = Vec::with_capacity(size);
    let mut rng = rand::thread_rng();

    for _ in 0..size {
        // Generate Laplacian-like distribution: many zeros, few large values
        let u: f64 = rng.gen_range(0.0..1.0);
        let sign = if rng.gen_bool(0.5) { 1i16 } else { -1i16 };
        // Inverse CDF of Laplacian: -b * sign(u - 0.5) * ln(1 - 2|u - 0.5|)
        // Simplified: exponential decay from zero
        let value = (-10.0 * u.ln()).min(127.0) as i8 * sign as i8;
        data.push(value as u8);
    }

    println!("Generated {} bytes of Laplacian noise", data.len());
    println!("Sample (first 20): {:?}", &data[..20]);

    // 2. Count zeros (should be very few with this distribution)
    let zeros = data.iter().filter(|&&x| x == 0 || x == 128).count();
    println!(
        "Near-zero bytes: {} ({:.2}%)",
        zeros,
        (zeros as f64 / size as f64) * 100.0
    );

    // 3. Compress with ZSTD (level 3)
    let zstd_start = std::time::Instant::now();
    let zstd_compressed = zstd::encode_all(&data[..], 3).unwrap();
    let zstd_time = zstd_start.elapsed();
    println!(
        "\nZSTD (Level 3): {} -> {} bytes ({:.2}x) in {:?}",
        data.len(),
        zstd_compressed.len(),
        data.len() as f64 / zstd_compressed.len() as f64,
        zstd_time
    );

    // 4. Compress with our Range Coder
    let ac_start = std::time::Instant::now();
    let ac_compressed = compress_residuals(&data);
    let ac_time = ac_start.elapsed();
    println!(
        "Range Coding: {} -> {} bytes ({:.2}x) in {:?}",
        data.len(),
        ac_compressed.len(),
        data.len() as f64 / ac_compressed.len() as f64,
        ac_time
    );

    // 5. Verify roundtrip
    let ac_decompressed = decompress_residuals(&ac_compressed, data.len());
    if ac_decompressed == data {
        println!("\n[OK] Range coding roundtrip verified!");
    } else {
        println!("\n[ERROR] Roundtrip mismatch!");
        println!("  Original len: {}", data.len());
        println!("  Decompressed len: {}", ac_decompressed.len());
        // Find first mismatch
        for (i, (&a, &b)) in data.iter().zip(ac_decompressed.iter()).enumerate() {
            if a != b {
                println!("  First mismatch at {}: {} vs {}", i, a, b);
                break;
            }
        }
    }

    // 6. Summary
    println!("\n>> Summary");
    println!(
        "   ZSTD Ratio:       {:.2}x",
        data.len() as f64 / zstd_compressed.len() as f64
    );
    println!(
        "   Range Coder Ratio: {:.2}x",
        data.len() as f64 / ac_compressed.len() as f64
    );
}
