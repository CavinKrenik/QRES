#!/bin/bash
set -e

# Detect VM size for log naming (optional, fallback to generic)
VM_SIZE=$(hostname)
LOG_FILE="results_${VM_SIZE}.log"
REPO_DIR="$HOME/QRES"

echo ">> Starting Benchmark Suite on $VM_SIZE..."
echo ">> Logging to $LOG_FILE"

cd "$REPO_DIR"

# Ensure Environment
source "$HOME/.cargo/env"
export CARGO_NET_GIT_FETCH_WITH_CLI=true

# FIX: Ensure dataset path exists for the runner
# The runner expects "benchmarks/" in the root, but it is in "evaluation/benchmarks/"
if [ ! -d "benchmarks" ] && [ -d "evaluation/benchmarks" ]; then
    echo ">> Linking benchmark datasets..."
    ln -s evaluation/benchmarks benchmarks
fi

# 1. Build (Release Mode)
echo ">> Building Benchmarks (Release Mode)..."
# Using -j1 to prevent OOM on small instances (like B1ls)
cargo build --release --bin comprehensive_runner -j1

# 2. Run Benchmark with Measurements
echo ">> Executing Benchmark..."

# Usage of /usr/bin/time -v:
#   %e: Elapsed real time (seconds)
#   %M: Maximum resident set size (kbytes)
#   %P: Percent of CPU this job got
{
    echo "=== QRES Cloud Benchmark Results ==="
    echo "Date: $(date)"
    echo "VM: $VM_SIZE"
    echo "-----------------------------------"
    
    /usr/bin/time -v ./target/release/comprehensive_runner
    
} 2>&1 | tee "$LOG_FILE"

echo ">> Benchmark Complete. Results saved to $LOG_FILE."
