# QRES Benchmark Guide

## Prerequisites
- **Python 3.10+** with `.venv` active.
- **Rust Toolchain** (for compiling the engine).
- **Corpus:** You need test data. We recommend the comprehensive `Titan` dataset or standard corpora like `Silesia` or `Enwik8`.

## Running the Suite

### 1. Torture Test (Regression Check)
Runs a fast check on edge-case data (Zeroes, Random, Pattern, Text) to ensure no crashes or massive expansion.

```bash
python benchmarks/torture_test.py
```

### 2. Battle Royale (Comparative)
Compares QRES against Zstd, Gzip, and LZMA on your local data.

```bash
# Quick run (small subset)
python benchmarks/battle_royale.py --quick

# Full benchmark on a specific folder
python benchmarks/battle_royale.py --data ./my_corpus/
```

### 3. Swarm Simulation
Simulates a P2P network of N nodes synchronizing weights.

```bash
# Run a 10-node simulation
cargo run --bin swarm_sim --release
```

## Interpreting Results
- **Ratio:** `Compressed Size / Original Size`. Lower is better. (e.g. 0.5 = 50% size).
- **Speed:** MB/s. Higher is better.
- **Score:** A composite metric: `1 / (Ratio * Log(Time))`.

## Adding New Benchmarks
Create a new script in `benchmarks/` and import `utils.timer`.
