import numpy as np

# Sine Wave (Predictable by Neural)
x = np.linspace(0, 500 * np.pi, 1024 * 1024) # 1MB Sine
sine = (np.sin(x) * 100 + 128).astype(np.uint8)

# Text (Repeating, predictableish)
text = b"The quick brown fox jumps over the lazy dog. " * 50000 
text = np.frombuffer(text, dtype=np.uint8)

# Combine
data = np.concatenate([sine, text])
data.tofile("neural_test.bin")
print(f"Generated neural_test.bin ({len(data)} bytes)")
