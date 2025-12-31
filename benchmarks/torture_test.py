import pytest
import time
import io
import sys
import zlib
import numpy as np
import qres_rust

def generate_repetitive_text(size_mb=2):
    """Dataset A: Repetitive Text (should use adaptive ANS)"""
    size = size_mb * 1024 * 1024
    pattern = b"The quick brown fox jumps over the lazy dog. " * 10
    data = (pattern * (size // len(pattern) + 1))[:size]
    return data

def generate_sine_wave(size_mb=2):
    """Dataset B: Sine Wave (should use neural predictors)"""
    size = size_mb * 1024 * 1024
    x = np.linspace(0, 500 * np.pi, size)
    y = np.sin(x) * 100 + 128
    return y.astype(np.uint8).tobytes()

def generate_zeros(size_mb=2):
    """Dataset C: Constant Zeros (optimal case)"""
    return b'\x00' * (size_mb * 1024 * 1024)

def generate_random(size_mb=2):
    """Dataset D: Random Data (fallback to zstd)"""
    return np.random.randint(0, 256, size_mb * 1024 * 1024, dtype=np.uint8).tobytes()

@pytest.mark.parametrize("data_func,expected_max_ratio", [
    (generate_repetitive_text, 95.0),
    (generate_sine_wave, 90.0),
    (generate_zeros, 80.0),
    (generate_random, 105.0),  # Allow slight expansion
])
def test_compression_ratio(data_func, expected_max_ratio):
    """Test compression ratios are within expected bounds"""
    data = data_func()
    
    # Compress
    start = time.time()
    compressed = qres_rust.encode_bytes(data, 0, None)  # Predictor 0 = SimplePredictor
    comp_time = time.time() - start
    
    # Decompress
    start = time.time()
    restored = qres_rust.decode_bytes(compressed, 0, None)
    decomp_time = time.time() - start
    
    # Verify round-trip
    assert restored == data, "Round-trip integrity failed"
    
    # Check ratio
    ratio = (len(compressed) / len(data)) * 100
    assert ratio <= expected_max_ratio, f"Ratio {ratio:.2f}% exceeds {expected_max_ratio}%"
    
    # Performance check
    comp_mbps = len(data) / 1024 / 1024 / comp_time
    decomp_mbps = len(data) / 1024 / 1024 / decomp_time
    assert comp_mbps > 1.0, f"Compression too slow: {comp_mbps:.2f} MB/s"
    assert decomp_mbps > 1.0, f"Decompression too slow: {decomp_mbps:.2f} MB/s"
    
    print(f"✓ {data_func.__name__}: {ratio:.2f}% ratio, {comp_mbps:.1f}/{decomp_mbps:.1f} MB/s")

if __name__ == "__main__":
    pytest.main([__file__, "-v"])
