//! Privacy Overhead Benchmarks
//!
//! Measures the performance cost of QRES privacy features:
//! - Differential Privacy (Gaussian noise)
//! - L2 Clipping
//! - Full privacy stack

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use qres_core::privacy::DifferentialPrivacy;
use std::time::Duration;

/// Generate a synthetic model update vector
fn generate_update(dim: usize) -> Vec<f32> {
    (0..dim).map(|i| (i as f32 * 0.01).sin() * 2.0).collect()
}

/// Benchmark clipping overhead
fn bench_clipping(c: &mut Criterion) {
    let mut group = c.benchmark_group("privacy_clipping");
    group.measurement_time(Duration::from_secs(3));

    let dp = DifferentialPrivacy::new(1.0, 1e-5, 1.0);

    for dim in [100, 500, 1000, 5000].iter() {
        let update = generate_update(*dim);

        group.bench_with_input(BenchmarkId::new("clip", dim), dim, |b, _| {
            b.iter(|| {
                let mut u = update.clone();
                dp.clip_update(black_box(&mut u))
            });
        });
    }

    group.finish();
}

/// Benchmark noise addition (Gaussian mechanism)
fn bench_noise_addition(c: &mut Criterion) {
    let mut group = c.benchmark_group("privacy_noise");
    group.measurement_time(Duration::from_secs(5));

    // Test different epsilon values (privacy levels)
    for epsilon in [0.1, 1.0, 10.0].iter() {
        let dp = DifferentialPrivacy::new(*epsilon, 1e-5, 1.0);
        let dim = 1000;

        group.bench_with_input(
            BenchmarkId::new("gaussian", format!("eps={}", epsilon)),
            &dim,
            |b, &dim| {
                b.iter(|| {
                    let mut update = generate_update(dim);
                    dp.add_noise(black_box(&mut update))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark full privacy pipeline (clip + noise)
fn bench_full_privacy_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("privacy_full_pipeline");
    group.measurement_time(Duration::from_secs(5));

    let dp = DifferentialPrivacy::new(1.0, 1e-5, 1.0);

    for dim in [500, 1000, 2000].iter() {
        // Baseline: no privacy
        group.bench_with_input(BenchmarkId::new("baseline", dim), dim, |b, &dim| {
            b.iter(|| {
                let update = generate_update(dim);
                black_box(update)
            });
        });

        // With clipping only
        group.bench_with_input(BenchmarkId::new("clip_only", dim), dim, |b, &dim| {
            b.iter(|| {
                let mut update = generate_update(dim);
                dp.clip_update(&mut update);
                black_box(update)
            });
        });

        // Full pipeline: clip + noise
        group.bench_with_input(BenchmarkId::new("full", dim), dim, |b, &dim| {
            b.iter(|| {
                let mut update = generate_update(dim);
                dp.clip_update(&mut update);
                let _ = dp.add_noise(&mut update);
                black_box(update)
            });
        });
    }

    group.finish();
}

/// Benchmark sigma calculation (utility analysis)
fn bench_sigma_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("privacy_sigma");

    for epsilon in [0.1, 0.5, 1.0, 5.0, 10.0].iter() {
        let dp = DifferentialPrivacy::new(*epsilon, 1e-5, 1.0);

        group.bench_with_input(
            BenchmarkId::new("sigma", format!("eps={}", epsilon)),
            epsilon,
            |b, _| {
                b.iter(|| dp.sigma());
            },
        );
    }

    // Print sigma values for reference
    println!("\n📊 Sigma Values (noise scale) for clipping_threshold=1.0, delta=1e-5:");
    for epsilon in [0.1, 0.5, 1.0, 5.0, 10.0].iter() {
        let dp = DifferentialPrivacy::new(*epsilon, 1e-5, 1.0);
        println!("   ε={:.1}: σ={:.4}", epsilon, dp.sigma());
    }
    println!();

    group.finish();
}

criterion_group!(
    benches,
    bench_clipping,
    bench_noise_addition,
    bench_full_privacy_pipeline,
    bench_sigma_calculation,
);
criterion_main!(benches);
