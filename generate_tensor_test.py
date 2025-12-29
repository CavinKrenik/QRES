import numpy as np

# Generate the same "Complex Wave" used in training
# Modulated wave: sin(t) * cos(t/3)
t = np.linspace(0, 500 * np.pi, 1024 * 1024) # 1MB
wave = np.sin(t) * np.cos(t / 3.0) 
wave = ((wave + 1.0) / 2.0 * 255.0).astype(np.uint8)

wave.tofile("tensor_test.bin")
print(f"Generated tensor_test.bin ({len(wave)} bytes)")
