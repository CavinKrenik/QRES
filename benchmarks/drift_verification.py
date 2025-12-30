import numpy as np
import time
import qres
import sys

def generate_drift_signal():
    print("Generating Drifting Signal (Sine -> Chaos)...")
    
    # Phase 1: Pure Sine (Easy for Linear/Tensor) - 2MB
    t = np.linspace(0, 1000, 2000000) 
    msg1 = (np.sin(t) * 100 + 128).astype(np.uint8)
    
    # Phase 2: Interleaved Zero/Random (The "Comb" Pattern) - 8MB
    # Linear (Stride 1): Sees 0, R, 0, R. Predicts garbage. Ratio ~1.0.
    # iPEPS (Stride 2): Sees 0->0 (Perfect) and R->R (Fail). Ratio ~0.5.
    
    n_points = 4000000
    msg2 = np.zeros(n_points * 2, dtype=np.uint8)
    np.random.seed(42)
    msg2[1::2] = np.random.randint(0, 255, n_points, dtype=np.uint8)
    
    data = np.concatenate((msg1, msg2))
    print(f"Created Drift Corpus ({len(data)/1024/1024:.2f} MB)")
    return data

def run_test():
    data = generate_drift_signal()
    
    print("\n--- Compression Test (Python Bindings) ---")
    start = time.time()
    # Using default parameters (predictor_id=0, weights=None)
    compressed = qres.encode_bytes(data.tobytes(), 0, None)
    duration = time.time() - start
    
    ratio = len(compressed) / len(data)
    print(f"Original Size:   {len(data)} bytes")
    print(f"Compressed Size: {len(compressed)} bytes")
    print(f"Ratio:           {ratio:.4f}")
    print(f"Speed:           {len(data)/1024/1024/duration:.2f} MB/s")
    
    print("\n--- Decompression Test ---")
    start = time.time()
    restored = qres.decode_bytes(compressed, 0, None)
    duration = time.time() - start
    print(f"Speed:           {len(data)/1024/1024/duration:.2f} MB/s")
    
    if restored == data.tobytes():
        print("✅ SUCCESS: Data matches perfectly.")
    else:
        print("❌ FAILURE: Data mismatch!")
        # Debugging info
        restored_arr = np.frombuffer(restored, dtype=np.uint8)
        if len(restored_arr) != len(data):
            print(f"Length mismatch: {len(restored_arr)} vs {len(data)}")
        else:
            diff = np.where(restored_arr != data)[0]
            print(f"First mismatch at index {diff[0]}: {restored_arr[diff[0]]} vs {data[diff[0]]}")

if __name__ == "__main__":
    run_test()
