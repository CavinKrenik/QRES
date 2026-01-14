"""
Generate Noisy IoT Dataset for QRES Benchmarking
Creates a challenging dataset with reduced repetition and added noise.
"""

import numpy as np
import os

def generate_noisy_iot(
    output_path: str,
    size_mb: int = 20,
    noise_level: float = 0.3,
    num_sensors: int = 50
):
    """
    Generate a less-compressible IoT telemetry dataset.
    
    Args:
        output_path: Output file path.
        size_mb: Target size in megabytes.
        noise_level: 0.0 = pure signal, 1.0 = pure noise.
        num_sensors: Number of simulated sensors.
    """
    size_bytes = size_mb * 1024 * 1024
    samples_per_sensor = size_bytes // num_sensors
    
    data = bytearray()
    
    for sensor_id in range(num_sensors):
        # Base signal: sine wave with varying frequency
        freq = 0.01 + (sensor_id * 0.001)
        t = np.arange(samples_per_sensor)
        signal = 128 + 100 * np.sin(2 * np.pi * freq * t)
        
        # Add noise
        noise = np.random.randn(samples_per_sensor) * (127 * noise_level)
        noisy_signal = np.clip(signal + noise, 0, 255).astype(np.uint8)
        
        data.extend(noisy_signal.tobytes())
    
    # Truncate/pad to exact size
    data = bytes(data[:size_bytes])
    
    with open(output_path, 'wb') as f:
        f.write(data)
    
    print(f"Generated {len(data)} bytes to {output_path}")
    print(f"  - Noise Level: {noise_level}")
    print(f"  - Sensors: {num_sensors}")

if __name__ == "__main__":
    output = "data/iot/iot_noisy_challenging.dat"
    os.makedirs(os.path.dirname(output), exist_ok=True)
    generate_noisy_iot(output, size_mb=20, noise_level=0.4)
