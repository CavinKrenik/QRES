import qres as qres_rust
import math
import random

print("=" * 70)
print("QRES v3.0 - Adaptive ANS + Zstd Fallback - Compression Tests")
print("=" * 70)

tests = []

# Test 1: Repetitive text
data1 = b'Hello World! ' * 100
c1 = qres_rust.encode_bytes(data1, 0, None)
d1 = qres_rust.decode_bytes(c1, 0, None)
r1 = len(c1) / len(data1)
tests.append(("Repetitive Text", len(data1), len(c1), r1, data1 == d1))

# Test 2: Sine wave
sine = bytes([(int(math.sin(i * 0.1) * 127) + 128) for i in range(1024)])
c2 = qres_rust.encode_bytes(sine, 0, None)
d2 = qres_rust.decode_bytes(c2, 0, None)
r2 = len(c2) / len(sine)
tests.append(("Sine Wave", len(sine), len(c2), r2, sine == d2))

# Test 3: All zeros
zeros = b'\x00' * 1024
c3 = qres_rust.encode_bytes(zeros, 0, None)
d3 = qres_rust.decode_bytes(c3, 0, None)
r3 = len(c3) / len(zeros)
tests.append(("All Zeros", len(zeros), len(c3), r3, zeros == d3))

# Test 4: Random data (zstd fallback)
random.seed(42)
rand = bytes([random.randint(0, 255) for _ in range(1024)])
c4 = qres_rust.encode_bytes(rand, 0, None)
d4 = qres_rust.decode_bytes(c4, 0, None)
r4 = len(c4) / len(rand)
tests.append(("Random (Zstd)", len(rand), len(c4), r4, rand == d4))

# Test 5: Varied text
text = b"The quick brown fox jumps over the lazy dog. " * 20
c5 = qres_rust.encode_bytes(text, 0, None)
d5 = qres_rust.decode_bytes(c5, 0, None)
r5 = len(c5) / len(text)
tests.append(("Varied Text", len(text), len(c5), r5, text == d5))

# Print results
print("\nTest Results:")
print("-" * 70)
for i, (name, orig, comp, ratio, passed) in enumerate(tests, 1):
    status = "PASS" if passed else "FAIL"
    print(f"{i}. {name:20s} | {orig:5d}B -> {comp:5d}B | {ratio:6.2%} | {status}")

print("-" * 70)
avg_ratio = sum(r for _, _, _, r, _ in tests) / len(tests)
best_ratio = min(r for _, _, _, r, _ in tests)
worst_ratio = max(r for _, _, _, r, _ in tests)
all_passed = all(p for _, _, _, _, p in tests)

print(f"\nSummary:")
print(f"  Average Ratio: {avg_ratio:.2%}")
print(f"  Best Ratio:    {best_ratio:.2%}")
print(f"  Worst Ratio:   {worst_ratio:.2%}")
print(f"  All Tests:     {'PASSED' if all_passed else 'FAILED'}")
print("=" * 70)
