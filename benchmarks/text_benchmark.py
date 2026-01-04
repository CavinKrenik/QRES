import os
import time
import subprocess
import shutil

# QRES v7.0 Text Benchmark
# Purpose: Validate ratio < 0.20 on Natural Language (English)

def run_zstd(filename):
    out_file = filename + ".zst"
    start = time.time()
    try:
        import zstandard as zstd
        cctx = zstd.ZstdCompressor(level=3)
        with open(filename, 'rb') as ifh, open(out_file, 'wb') as ofh:
            cctx.copy_stream(ifh, ofh)
    except ImportError:
        print("[Warn] 'zstandard' python lib not found.")
        return 0, 0
        
    elapsed = time.time() - start
    size = os.path.getsize(out_file)
    return size, elapsed

def run_qres(filename):
    out_file = filename + ".qres"
    start = time.time()
    try:
        base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        cli = os.path.join(base_dir, "qres_rust", "target", "release", "qres-cli.exe")
        
        subprocess.run([cli, "compress", filename, out_file], 
                       check=True, stdout=subprocess.DEVNULL)
    except Exception as e:
        print(f"[Error] QRES failed: {e}")
        return 0, 0

    elapsed = time.time() - start
    size = os.path.getsize(out_file)
    return size, elapsed

def main():
    base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    test_file = os.path.join(base_dir, "benchmarks", "datasets", "text_1mb.txt")
    
    if not os.path.exists(test_file):
        print(f"Error: {test_file} not found.")
        return

    orig_size = os.path.getsize(test_file)
    print(f"\n--- Benchmarking Text ({orig_size} bytes) ---")
    
    # Zstd
    z_size, z_time = run_zstd(test_file)
    if z_size > 0:
        z_ratio = z_size / orig_size
        z_speed = (orig_size / 1024 / 1024) / z_time
        print(f"Zstd:  {z_ratio:.2%} ratio, {z_speed:.2f} MB/s")
    
    # QRES
    q_size, q_time = run_qres(test_file)
    if q_size > 0:
        q_ratio = q_size / orig_size
        q_speed = (orig_size / 1024 / 1024) / q_time
        print(f"QRES:  {q_ratio:.2%} ratio, {q_speed:.2f} MB/s")
        
        if z_size > 0:
            imp = (z_ratio - q_ratio) / z_ratio
            if imp > 0.40: # Aim explicitly high for NLP
                print("✅ Goal Met: >40% improvement (Quantum/LLM Level)")
            elif imp > 0.0:
                print("⚠️ Beats Zstd, but Short of Target")
            else:
                print("❌ Failed to beat Zstd")

if __name__ == "__main__":
    main()
