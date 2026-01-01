use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use qres_rust::{compress_chunk, decompress_chunk};
use std::time::Duration;

// Generate test data
fn generate_sine_wave(size: usize) -> Vec<u8> {
    (0..size)
        .map(|i| {
            let t = i as f64 / 100.0;
            ((t.sin() * 127.0) + 128.0) as u8
        })
        .collect()
}

fn generate_random_data(size: usize) -> Vec<u8> {
    (0..size).map(|i| ((i * 7919) % 256) as u8).collect()
}

fn generate_text_data(size: usize) -> Vec<u8> {
    let text = b"The quick brown fox jumps over the lazy dog. ";
    text.iter().cycle().take(size).copied().collect()
}

// Benchmark compression speed
fn bench_compression_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression_speed");
    group.measurement_time(Duration::from_secs(10));

    for size in [1024, 4096, 16384, 65536].iter() {
        // Sine wave (spectral predictor test)
        let sine_data = generate_sine_wave(*size);
        group.bench_with_input(BenchmarkId::new("sine", size), &sine_data, |b, data| {
            b.iter(|| compress_chunk(black_box(data), 0, None, None).unwrap());
        });

        // Random data (worst case)
        let random_data = generate_random_data(*size);
        group.bench_with_input(BenchmarkId::new("random", size), &random_data, |b, data| {
            b.iter(|| compress_chunk(black_box(data), 0, None, None).unwrap());
        });

        // Text data (typical case)
        let text_data = generate_text_data(*size);
        group.bench_with_input(BenchmarkId::new("text", size), &text_data, |b, data| {
            b.iter(|| compress_chunk(black_box(data), 0, None, None).unwrap());
        });
    }

    group.finish();
}

// Benchmark compression ratio
fn bench_compression_ratio(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression_ratio");

    // Sine wave - target 60%+ compression
    let sine_data = generate_sine_wave(10000);
    let compressed = compress_chunk(&sine_data, 0, None, None).unwrap();
    let ratio = compressed.len() as f64 / sine_data.len() as f64;
    println!(
        "\n📊 Sine Wave Compression Ratio: {:.2}% (Target: <60%)",
        ratio * 100.0
    );

    // Random data - expect ~100% (no compression)
    let random_data = generate_random_data(10000);
    let compressed = compress_chunk(&random_data, 0, None, None).unwrap();
    let ratio = compressed.len() as f64 / random_data.len() as f64;
    println!("📊 Random Data Compression Ratio: {:.2}%", ratio * 100.0);

    // Text data - expect 40-60%
    let text_data = generate_text_data(10000);
    let compressed = compress_chunk(&text_data, 0, None, None).unwrap();
    let ratio = compressed.len() as f64 / text_data.len() as f64;
    println!("📊 Text Data Compression Ratio: {:.2}%\n", ratio * 100.0);

    group.finish();
}

// Benchmark decompression speed
fn bench_decompression_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("decompression_speed");
    group.measurement_time(Duration::from_secs(10));

    for size in [1024, 4096, 16384, 65536].iter() {
        let data = generate_sine_wave(*size);
        let compressed = compress_chunk(&data, 0, None, None).unwrap();

        group.bench_with_input(
            BenchmarkId::new("decompress", size),
            &compressed,
            |b, comp_data| {
                b.iter(|| decompress_chunk(black_box(comp_data), 0, None).unwrap());
            },
        );
    }

    group.finish();
}

// Benchmark ANS coder batch sizes
fn bench_batch_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_optimization");

    let data = generate_sine_wave(65536);

    group.bench_function("compress_64kb", |b| {
        b.iter(|| compress_chunk(black_box(&data), 0, None, None).unwrap());
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_compression_speed,
    bench_compression_ratio,
    bench_decompression_speed,
    bench_batch_sizes
);
criterion_main!(benches);
