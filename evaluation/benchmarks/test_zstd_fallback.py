import qres as qres_rust
import random

# Test zstd fallback with random data
random.seed(42)
rand_data = bytes([random.randint(0, 255) for _ in range(1024)])

print("Testing Zstd Fallback with Random Data:")
print(f"Original: {len(rand_data)} bytes")

compressed = qres_rust.encode_bytes(rand_data, 0, None)
print(f"Compressed: {len(compressed)} bytes")
print(f"Ratio: {len(compressed)/len(rand_data):.2%}")

decompressed = qres_rust.decode_bytes(compressed, 0, None)
print(f"Decompressed: {len(decompressed)} bytes")

if rand_data == decompressed:
    print("Round-trip: PASS")
else:
    print("Round-trip: FAIL")
