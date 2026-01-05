import time
import io
import sys
import random
import zlib
import numpy as np
import pandas as pd
import qres
try:
    import zstandard as zstd
except ImportError:
    zstd = None
    print("WARNING: zstandard not installed. Skipping zstd benchmark.")

def generate_synthetic_data(size_mb=10):
    """Generate a sine wave (predictable) converted to bytes."""
    size = size_mb * 1024 * 1024
    x = np.linspace(0, 100 * np.pi, size)
    y = np.sin(x) * 100 + 128 # Center around 128
    data = y.astype(np.uint8).tobytes()
    return data

def generate_real_world_simulation(rows=100000):
    """Simulate a telemetry DataFrame export."""
    df = pd.DataFrame({
        'timestamp': pd.date_range(start='2024-01-01', periods=rows, freq='S').asi8,
        'sensor_a': np.cumsum(np.random.choice([-1, 0, 1], size=rows)), # Random Walk (Perfect for QRES)
        'sensor_b': np.random.randint(0, 100, size=rows), # Noise
        'status': np.repeat([b'OK'], rows)
    })
    
    # Export to CSV (bytes)
    buffer = io.BytesIO()
    df.to_csv(buffer, index=False)
    return buffer.getvalue()

def run_benchmark(name, data):
    print(f"\n--- Benchmarking: {name} ({len(data) / 1024 / 1024:.2f} MB) ---")
    print(f"{'Algorithm':<10} | {'Ratio':<10} | {'Comp Speed':<15} | {'Decomp Speed':<15} | {'Speedup (vs Zlib)'}")
    print("-" * 85)

    methods = [
        ("Zlib (L6)", lambda d: zlib.compress(d, level=6), lambda d: zlib.decompress(d)),
        ("QRES (v2)", lambda d: qres.compress(d), lambda d: qres.decompress(d)),
    ]
    
    if zstd:
        methods.append(("Zstd (Def)", lambda d: zstd.compress(d), lambda d: zstd.decompress(d)))

    results = {}

    for algo_name, comp_func, decomp_func in methods:
        # Warmup
        try:
            _ = comp_func(data[:1000])
        except:
            pass

        # Compress
        start = time.time()
        compressed = comp_func(data)
        end = time.time()
        comp_time = end - start
        comp_speed = (len(data) / 1024 / 1024) / comp_time
        ratio = len(compressed) / len(data)

        # Decompress
        start = time.time()
        restored = decomp_func(compressed)
        end = time.time()
        decomp_time = end - start
        decomp_speed = (len(data) / 1024 / 1024) / decomp_time
        
        results[algo_name] = {"comp_speed": comp_speed, "ratio": ratio}

        # Calculate Speedup vs Zlib (using previous Zlib result if available, else 1.0)
        speedup = 1.0
        if algo_name != "Zlib (L6)" and "Zlib (L6)" in results:
             speedup = comp_speed / results["Zlib (L6)"]["comp_speed"]

        print(f"{algo_name:<10} | {ratio*100:6.2f}%    | {comp_speed:6.2f} MB/s    | {decomp_speed:6.2f} MB/s    | {speedup:6.2f}x")

def main():
    import argparse
    parser = argparse.ArgumentParser(description="QRES Battle Royale Benchmark")
    parser.add_argument("--quick", action="store_true", help="Run with smaller data for faster CI")
    args = parser.parse_args()
    
    if args.quick:
        print("Running in QUICK mode (reduced data size for CI)")
        size_mb = 1
        rows = 10000
    else:
        size_mb = 5
        rows = 50000
    
    print("generating synthetic data...")
    synthetic = generate_synthetic_data(size_mb=size_mb)
    
    print("generating telemetry data...")
    telemetry = generate_real_world_simulation(rows=rows)
    
    run_benchmark("Sine Wave (Predictable)", synthetic)
    run_benchmark("Telemetry CSV (Mixed)", telemetry)

if __name__ == "__main__":
    main()
