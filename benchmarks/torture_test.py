import time
import io
import sys
import zlib
import numpy as np
import qres
import requests

def generate_sine_wave(size_mb=2):
    """Scenario A: Pure Sine Wave (Should use QRES Mode)"""
    size = size_mb * 1024 * 1024
    x = np.linspace(0, 500 * np.pi, size)
    y = np.sin(x) * 100 + 128
    return y.astype(np.uint8).tobytes()

def download_shakespeare():
    """Scenario B: Shakespeare/Text (Should use Raw Mode)"""
    url = "https://www.gutenberg.org/files/100/100-0.txt" # Complete Works
    try:
        print("Downloading Shakespeare...")
        r = requests.get(url)
        return r.content[:2*1024*1024] # Clip to 2MB
    except:
        print("Download failed, using synthetic random text.")
        # Synthetic fallback: Random ASCII
        return np.random.randint(65, 122, size=2*1024*1024, dtype=np.uint8).tobytes()

def measure(name, data):
    print(f"\n--- {name} ({len(data)/1024/1024:.2f} MB) ---")
    
    # 1. QRES
    start = time.time()
    compressed_qres = qres.compress(data)
    qres_time = time.time() - start
    qres_ratio = len(compressed_qres) / len(data)
    
    # 2. Zlib Raw (Level 6)
    start = time.time()
    compressed_zlib = zlib.compress(data, level=6)
    zlib_time = time.time() - start
    zlib_ratio = len(compressed_zlib) / len(data)

    print(f"{'Metric':<15} | {'QRES (v2 Smart)':<15} | {'Zlib (Raw)':<15}")
    print("-" * 55)
    print(f"{'Ratio':<15} | {qres_ratio*100:6.2f}%          | {zlib_ratio*100:6.2f}%")
    print(f"{'Size':<15} | {len(compressed_qres):<15} | {len(compressed_zlib):<15}")
    print(f"{'Speed':<15} | {len(data)/1024/1024/qres_time:6.2f} MB/s    | {len(data)/1024/1024/zlib_time:6.2f} MB/s")

    # Verification Logic
    if "Sine" in name:
        if qres_ratio > 0.05: # Sine should be < 1% usually, but headers add minimal
             print("⚠️  WARNING: QRES ratio seems high for Sine Wave. Is bit-packing working?")
        else:
             print("✅ QRES crushed the Sine Wave (Expected).")
             
    if "Shakespeare" in name:
        if qres_ratio > zlib_ratio * 1.5:
             print("❌ FAILURE: QRES expanded the text significantly! Adaptive mode failed.")
        elif qres_ratio > zlib_ratio:
             print("⚠️  Acceptable: QRES slightly larger due to header overhead, but within range.")
        else:
             print("✅ QRES matches or beats Zlib on text (Adaptive Mode success).")

    # Round Trip Check
    restored = qres.decompress(compressed_qres)
    if restored != data:
        print("❌ FATAL: Decompression mismatch!")
        sys.exit(1)
    else:
        print("✅ Integrity Check Passed.")

def main():
    sine = generate_sine_wave()
    measure("Scenario A: Sine Wave", sine)
    
    shakespeare = download_shakespeare()
    measure("Scenario B: Shakespeare Text", shakespeare)

if __name__ == "__main__":
    main()
