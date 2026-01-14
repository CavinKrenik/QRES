#!/bin/bash
# QRES Benchmark Runner
# Generates CSV outputs for paper figures

set -e

OUTPUT_DIR="reproducibility/results"
mkdir -p $OUTPUT_DIR

echo "=== QRES Benchmark Suite ==="
echo "Output directory: $OUTPUT_DIR"

# Build release if needed
echo "[1/5] Building release..."
cd qres_rust
cargo build --release

# Run unit tests
echo "[2/5] Running tests..."
cargo test --workspace

# Run compression benchmarks
echo "[3/5] Running compression benchmarks..."
cargo bench -- --save-baseline paper 2>&1 | tee $OUTPUT_DIR/compression_bench.txt

# Run aggregation benchmarks
echo "[4/5] Running aggregation benchmarks..."
cargo test --release -- --nocapture test_krum 2>&1 | tee $OUTPUT_DIR/aggregation_bench.txt

# Generate summary
echo "[5/5] Generating summary..."
echo "Benchmark run completed at $(date)" > $OUTPUT_DIR/summary.txt
echo "Git commit: $(git rev-parse HEAD)" >> $OUTPUT_DIR/summary.txt

echo "=== Complete ==="
echo "Results saved to $OUTPUT_DIR"
