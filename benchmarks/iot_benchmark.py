import os
import time
import numpy as np
import subprocess
import shutil

# QRES v4.2 IoT Benchmark
# Purpose: Validate 25% better ratio than Zstd on drifting telemetry

def generate_iot_telemetry(filename, num_samples=1000000):
    """
    Generates synthetic IoT data:
    - Temperature (slow periodic drift)
    - Vibration (fast high-freq noise)
    - Status codes (rare discrete events)
    """
    print(f"[Gen] Generating {num_samples} IoT samples...")
    t = np.linspace(0, 1000, num_samples)
    
    # 1. Temperature: slow sine + trend + noise
    temp = 20 + 10 * np.sin(t * 0.01) + (t * 0.005) + np.random.normal(0, 0.5, num_samples)
    
    # 2. Vibration: harmonics
    vib = 100 * np.sin(t * 0.5) + 50 * np.sin(t * 2.0) + np.random.normal(0, 2, num_samples)
    
    # Combine into bytes
    # Interleave data: [T, V, T, V...] to simulate packet stream
    data = np.zeros(num_samples * 2, dtype=np.uint8)
    
    # Quantize to u8
    data[0::2] = np.clip(temp * 2 + 100, 0, 255).astype(np.uint8)
    data[1::2] = np.clip(vib + 128, 0, 255).astype(np.uint8)
    
    with open(filename, "wb") as f:
        f.write(data.tobytes())
        
    print(f"[Gen] Saved {os.path.getsize(filename)/1024/1024:.2f} MB to {filename}")

def run_zstd(filename):
    out_file = filename + ".zst"
    start = time.time()
    try:
        import zstandard as zstd
        cctx = zstd.ZstdCompressor(level=3) # Level 3 is default
        with open(filename, 'rb') as ifh, open(out_file, 'wb') as ofh:
            cctx.copy_stream(ifh, ofh)
    except ImportError:
        print("[Warn] 'zstandard' python lib not found. Install it for comparison.")
        return 0, 0
        
    elapsed = time.time() - start
    size = os.path.getsize(out_file)
    return size, elapsed

def run_qres(filename):
    out_file = filename + ".qres"
    start = time.time()
    try:
        # Use local qres-cli
        base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        cli = os.path.join(base_dir, "qres_rust", "target", "release", "qres-cli.exe")
        if not os.path.exists(cli):
             # Try building it
             print("[Info] Building qres-cli...")
             subprocess.run(["cargo", "build", "--release"], cwd=os.path.join(base_dir, "qres_rust"), check=True)
        
        subprocess.run([cli, "compress", filename, out_file], 
                       check=True, stdout=subprocess.DEVNULL)
    except Exception as e:
        print(f"[Error] QRES failed: {e}")
        return 0, 0

    elapsed = time.time() - start
    size = os.path.getsize(out_file)
    return size, elapsed

def main():
    test_file = "iot_telemetry.dat"
    # Increase to 10M samples (approx 20MB) to reduce impact of CLI startup overhead
    generate_iot_telemetry(test_file, num_samples=10000000)
    orig_size = os.path.getsize(test_file)
    
    print("\n--- Benchmarking ---")
    
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
            print(f"\nResult: QRES is {imp:.1%} smaller than Zstd")
            if imp > 0.25:
                print("✅ Goal Met: >25% improvement on drift!")
            else:
                print("⚠️ Goal Missed: Keep tuning spectral predictor.")

if __name__ == "__main__":
    main()
