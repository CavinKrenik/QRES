//! Baseline Comparison Benchmarks
//!
//! Compares QRES aggregation strategies (FedAvg, Krum, TrimmedMean)
//! for FLICS 2026 paper evaluation.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use qres_core::aggregation::{
    aggregate_updates, AggregationMode, Aggregator, FedAvgAggregator, KrumAggregator,
    TrimmedMeanAggregator,
};
use std::time::Duration;

/// Generate synthetic model updates simulating federated learning scenario
/// Each update is a flattened weight vector with slight variations
fn generate_model_updates(
    n_clients: usize,
    model_dim: usize,
    byzantine_frac: f32,
) -> Vec<Vec<f32>> {
    let n_byzantine = (n_clients as f32 * byzantine_frac) as usize;
    let mut updates = Vec::with_capacity(n_clients);

    // Honest clients: small variations around a "true" model
    let base_model: Vec<f32> = (0..model_dim).map(|i| (i as f32 * 0.01).sin()).collect();

    for i in 0..n_clients {
        if i < n_byzantine {
            // Byzantine: completely wrong values (poisoning attack)
            updates.push(vec![100.0; model_dim]);
        } else {
            // Honest: base model + small noise
            let noise_scale = 0.1;
            updates.push(
                base_model
                    .iter()
                    .enumerate()
                    .map(|(j, &v)| v + noise_scale * ((i * j) as f32).sin())
                    .collect(),
            );
        }
    }
    updates
}

/// Benchmark aggregation throughput (ops/sec) for different methods
fn bench_aggregation_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregation_throughput");
    group.measurement_time(Duration::from_secs(5));

    let model_dim = 1000; // Typical small SNN model

    for n_clients in [10, 20, 50].iter() {
        let updates = generate_model_updates(*n_clients, model_dim, 0.0);

        // FedAvg (baseline)
        group.bench_with_input(
            BenchmarkId::new("FedAvg", n_clients),
            &updates,
            |b, updates| {
                b.iter(|| aggregate_updates(black_box(updates), &AggregationMode::SimpleMean));
            },
        );

        // Krum (Byzantine-tolerant)
        group.bench_with_input(
            BenchmarkId::new("Krum", n_clients),
            &updates,
            |b, updates| {
                b.iter(|| {
                    aggregate_updates(
                        black_box(updates),
                        &AggregationMode::Krum { expected_byz: 1 },
                    )
                });
            },
        );

        // Multi-Krum
        group.bench_with_input(
            BenchmarkId::new("MultiKrum", n_clients),
            &updates,
            |b, updates| {
                b.iter(|| {
                    aggregate_updates(
                        black_box(updates),
                        &AggregationMode::MultiKrum {
                            expected_byz: 1,
                            k: 3,
                        },
                    )
                });
            },
        );

        // Trimmed Mean
        group.bench_with_input(
            BenchmarkId::new("TrimmedMean", n_clients),
            &updates,
            |b, updates| {
                b.iter(|| {
                    aggregate_updates(
                        black_box(updates),
                        &AggregationMode::TrimmedMean { trim_fraction: 0.1 },
                    )
                });
            },
        );
    }

    group.finish();
}

/// Benchmark Byzantine resilience: measure accuracy under attack
fn bench_byzantine_resilience(c: &mut Criterion) {
    let mut group = c.benchmark_group("byzantine_resilience");
    group.measurement_time(Duration::from_secs(3));

    let n_clients = 20;
    let model_dim = 500;

    // Test with different Byzantine fractions
    for byz_frac in [0.0, 0.1, 0.2, 0.3].iter() {
        let updates = generate_model_updates(n_clients, model_dim, *byz_frac);
        let label = format!("{}%_byz", (*byz_frac * 100.0) as i32);

        // Krum should handle Byzantine nodes better
        group.bench_with_input(BenchmarkId::new("Krum", &label), &updates, |b, updates| {
            b.iter(|| {
                aggregate_updates(
                    black_box(updates),
                    &AggregationMode::Krum {
                        expected_byz: (n_clients as f32 * byz_frac) as usize,
                    },
                )
            });
        });

        // FedAvg for comparison (will fail under Byzantine)
        group.bench_with_input(
            BenchmarkId::new("FedAvg", &label),
            &updates,
            |b, updates| {
                b.iter(|| aggregate_updates(black_box(updates), &AggregationMode::SimpleMean));
            },
        );
    }

    group.finish();
}

/// Test pluggable aggregator trait
fn bench_pluggable_aggregators(c: &mut Criterion) {
    let mut group = c.benchmark_group("pluggable_aggregators");

    let updates = generate_model_updates(10, 500, 0.0);

    // Use trait objects
    let aggregators: Vec<Box<dyn Aggregator>> = vec![
        Box::new(FedAvgAggregator),
        Box::new(KrumAggregator::default()),
        Box::new(TrimmedMeanAggregator::default()),
    ];

    for (i, agg) in aggregators.iter().enumerate() {
        let name = agg.name();
        group.bench_with_input(BenchmarkId::new("trait", name), &updates, |b, updates| {
            b.iter(|| aggregators[i].aggregate(black_box(updates)));
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_aggregation_throughput,
    bench_byzantine_resilience,
    bench_pluggable_aggregators,
);
criterion_main!(benches);
