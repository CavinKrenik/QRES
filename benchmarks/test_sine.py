
import numpy as np
import sys
import os

# Add local python path if needed, though usually installed in venv
try:
    import qres
except ImportError:
    print("QRES module not found. Please run 'maturin develop --release' first.")
    # Attempt to add target/wheels or similar? No, rely on user environment.
    sys.exit(1)

def test_sine():
    print("[Test] Generating 64KB Sine Wave...")
    # Generate pure sine wave
    # f = 1.0, sampling at decent rate
    t = np.linspace(0, 100, 65536)
    # Amplitude 127, Offset 128 -> 1..255 range (uint8)
    data = ((np.sin(2 * np.pi * t) * 127) + 128).astype(np.uint8)
    data_bytes = data.tobytes()
    
    # Compress
    # encode_bytes(data, level, window_log) - None uses default
    compressed = qres.encode_bytes(data_bytes, 0, None)
    
    orig_len = len(data_bytes)
    comp_len = len(compressed)
    ratio = comp_len / orig_len
    
    print(f"Original: {orig_len} bytes")
    print(f"Compressed: {comp_len} bytes")
    print(f"Ratio: {ratio:.2%} (Target: <60%)")
    
    if ratio < 0.60:
        print("[SUCCESS] Spectral Predictor is active.")
    else:
        print("[FAILURE] Compression ratio too high. Spectral Predictor might be failing.")
        sys.exit(1)

if __name__ == "__main__":
    test_sine()
