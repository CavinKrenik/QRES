import qres_rust
import time
import sys

# Create dummy data: 1 million integers (0 to 255)
# This mimics sensor data with some repetition
data = bytes([x % 255 for x in range(1000000)])

print(f"Original Size: {len(data)} bytes")

try:
    # 1. Benchmark Compression
    start = time.time()
    # Explicitly ensure it's bytes
    compressed = qres_rust.encode_bytes(data, 1, None) # Predictor 1 = Linear, No Weights
    end = time.time()

    print(f"Compressed Size: {len(compressed)} bytes")
    print(f"Ratio: {len(compressed) / len(data) * 100:.2f}%")
    print(f"Compression Speed: {len(data) / (end - start) / 1024 / 1024:.2f} MB/s")

    # 2. Verify Lossless Round-Trip
    restored = qres_rust.decode_bytes(compressed, 1, None)
    assert data == bytes(restored), "CRITICAL: Data mismatch!"
    print("✅ Integrity Check Passed: Lossless confirmed.")

except Exception as e:
    print(f"ERROR: {e}")
    # Print type of data to debug
    print(f"Input type: {type(data)}")
    sys.exit(1)
