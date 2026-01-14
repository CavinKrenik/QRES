import qres
import numpy as np
import time
import os
import zlib

def run_benchmark():
    print("--- QRES Quantum/Structured Benchmark (v7.5) ---")
    
    # 1. Generate Structured Data (Correlated Matrix)
    # Rank-5 Matrix: Sum of 5 outer products
    rows, cols = 1024, 1024
    rank = 5
    print(f"Generating {rows}x{cols} Rank-{rank} Matrix (Float64)...")
    
    np.random.seed(42)
    matrix = np.zeros((rows, cols))
    for _ in range(rank):
        vec_a = np.random.randn(rows)
        vec_a = np.cumsum(vec_a) # Make it smooth (random walk)
        vec_b = np.random.randn(cols)
        vec_b = np.cumsum(vec_b)
        matrix += np.outer(vec_a, vec_b)
        
    # Flatten for transmission/compression
    data_flat = matrix.flatten().tolist()
    raw_bytes = matrix.tobytes()
    raw_size = len(raw_bytes)
    print(f"Raw Size: {raw_size / 1024 / 1024:.2f} MB")
    
    # 2. Zstd (Baseline)
    start = time.time()
    # Zstd on floats is usually bad unless using bit-shuffle, but let's try raw
    try:
        import zstandard as zstd
        cctx = zstd.ZstdCompressor(level=3)
        z_compressed = cctx.compress(raw_bytes)
        z_size = len(z_compressed)
        z_time = time.time() - start
        print(f"Zstd: {z_size/1024/1024:.2f} MB ({z_size/raw_size:.2%}), {raw_size/1024/1024/z_time:.1f} MB/s")
    except ImportError:
        print("Zstd not installed, skipping.")

    # 3. QRES Quantum (MPS/Haar)
    # Threshold for lossy compression (simulating low rank approx)
    # Signal variance is high (random walk). 
    # Let's pick a threshold relative to std dev?
    # Haar Wavelet: many coeffs will be small.
    threshold = 1.0 # Tune this
    
    print(f"Compressing with QRES Quantum (Threshold={threshold})...")
    start = time.time()
    try:
        # qres.compress_matrix_v1(data, rows, cols, threshold)
        # Returns Vec<f64> (Sparse Coeffs)
        compressed_floats = qres.compress_matrix_v1(data_flat, rows, cols, threshold)
        
        # In a real codec, we would bit-pack these floats and run entropy coding.
        # For now, we count the number of non-zero floats * 8 bytes (worst case) 
        # or just the size of the returned vector (if sparse format is assumed).
        # My implementation returned "flattened_sparse" which included Zeros if not thresholded?
        # Let's check quantum.rs implementation logic.
        
        # Ah, quantum.rs logic: 
        # if val.abs() > self.threshold { push(val) } else { push(0.0) }
        # It pushes 0.0! So it returns SAME size vector.
        # This is strictly "Denoising", not "Compression" in size yet.
        # UNLESS the python wrapper or next stage handles RLE/Zero-Skipping.
        
        # To measure "Potential Compression", we count non-zeros.
        non_zeros = sum(1 for x in compressed_floats if x != 0.0)
        
        # Assume CSR or RLE overhead is small (e.g. 10%).
        # Compressed Size approx = NonZeros * 8 bytes.
        q_size_approx = non_zeros * 8
        q_time = time.time() - start
        
        print(f"QRES (Sparse): {q_size_approx/1024/1024:.2f} MB (Approx {q_size_approx/raw_size:.2%})")
        print(f"  Speed: {raw_size/1024/1024/q_time:.1f} MB/s")
        print(f"  Sparsity: {1.0 - (non_zeros / len(compressed_floats)):.2%}")
        
    except Exception as e:
        print(f"QRES Failed: {e}")

if __name__ == "__main__":
    run_benchmark()
