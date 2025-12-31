import qres as qres_rust
import math

print("=" * 60)
print("QRES v3.0 Adaptive ANS Compression Tests")
print("=" * 60)

# Test 1: Repetitive text
data1 = b'Hello World! ' * 100
compressed1 = qres_rust.encode_bytes(data1, 0, None)
decompressed1 = qres_rust.decode_bytes(compressed1, 0, None)
ratio1 = len(compressed1) / len(data1)
print(f"\n1. Repetitive Text ('Hello World! ' x100)")
print(f"   Original: {len(data1)} bytes")
print(f"   Compressed: {len(compressed1)} bytes")
print(f"   Ratio: {ratio1:.2%}")
print(f"   Status: {'PASS' if data1 == decompressed1 else 'FAIL'}")

# Test 2: Sine wave (smooth, predictable)
sine_data = bytes([(int(math.sin(i * 0.1) * 127) + 128) for i in range(1024)])
compressed2 = qres_rust.encode_bytes(sine_data, 0, None)
decompressed2 = qres_rust.decode_bytes(compressed2, 0, None)
ratio2 = len(compressed2) / len(sine_data)
print(f"\n2. Sine Wave (1024 samples)")
print(f"   Original: {len(sine_data)} bytes")
print(f"   Compressed: {len(compressed2)} bytes")
print(f"   Ratio: {ratio2:.2%}")
print(f"   Status: {'PASS' if sine_data == decompressed2 else 'FAIL'}")

# Test 3: Random data (incompressible)
import random
random.seed(42)
random_data = bytes([random.randint(0, 255) for _ in range(1024)])
compressed3 = qres_rust.encode_bytes(random_data, 0, None)
decompressed3 = qres_rust.decode_bytes(compressed3, 0, None)
ratio3 = len(compressed3) / len(random_data)
print(f"\n3. Random Data (1024 bytes)")
print(f"   Original: {len(random_data)} bytes")
print(f"   Compressed: {len(compressed3)} bytes")
print(f"   Ratio: {ratio3:.2%}")
print(f"   Status: {'PASS' if random_data == decompressed3 else 'FAIL'}")

# Test 4: Zeros (highly compressible)
zeros_data = b'\x00' * 1024
compressed4 = qres_rust.encode_bytes(zeros_data, 0, None)
decompressed4 = qres_rust.decode_bytes(compressed4, 0, None)
ratio4 = len(compressed4) / len(zeros_data)
print(f"\n4. All Zeros (1024 bytes)")
print(f"   Original: {len(zeros_data)} bytes")
print(f"   Compressed: {len(compressed4)} bytes")
print(f"   Ratio: {ratio4:.2%}")
print(f"   Status: {'PASS' if zeros_data == decompressed4 else 'FAIL'}")

# Test 5: Text with variation
text_data = b"The quick brown fox jumps over the lazy dog. " * 20
compressed5 = qres_rust.encode_bytes(text_data, 0, None)
decompressed5 = qres_rust.decode_bytes(compressed5, 0, None)
ratio5 = len(compressed5) / len(text_data)
print(f"\n5. Varied Text (pangram x20)")
print(f"   Original: {len(text_data)} bytes")
print(f"   Compressed: {len(compressed5)} bytes")
print(f"   Ratio: {ratio5:.2%}")
print(f"   Status: {'PASS' if text_data == decompressed5 else 'FAIL'}")

print("\n" + "=" * 60)
print("Summary:")
print(f"  Average Ratio: {(ratio1 + ratio2 + ratio3 + ratio4 + ratio5) / 5:.2%}")
print(f"  Best: {min(ratio1, ratio2, ratio3, ratio4, ratio5):.2%}")
print(f"  Worst: {max(ratio1, ratio2, ratio3, ratio4, ratio5):.2%}")
print("=" * 60)
