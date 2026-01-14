"""
Generate Diverse IoT Datasets for QRES v11.1 Benchmarking
Creates datasets with various patterns: trends, anomalies, correlations.
"""

import numpy as np
import os

def generate_trending_data(size_bytes: int) -> bytes:
    """
    Generate data with sine wave + linear trend (simulates temperature sensors).
    """
    samples = size_bytes
    t = np.arange(samples)
    
    # sine wave (daily pattern) + linear trend (seasonal drift)
    signal = 128 + 50 * np.sin(2 * np.pi * t / 1000) + (t / samples) * 30
    signal = np.clip(signal, 0, 255).astype(np.uint8)
    
    return signal.tobytes()


def generate_anomaly_data(size_bytes: int, anomaly_rate: float = 0.01) -> bytes:
    """
    Generate mostly stable data with random spikes (anomalies).
    """
    samples = size_bytes
    
    # stable baseline
    signal = np.full(samples, 100, dtype=np.float32)
    
    # add random anomalies
    num_anomalies = int(samples * anomaly_rate)
    anomaly_indices = np.random.choice(samples, num_anomalies, replace=False)
    signal[anomaly_indices] = np.random.randint(200, 255, num_anomalies)
    
    return signal.astype(np.uint8).tobytes()


def generate_correlated_sensors(size_bytes: int, num_sensors: int = 5) -> bytes:
    """
    Generate correlated multi-sensor data (simulates sensor array with shared noise).
    """
    samples_per_sensor = size_bytes // num_sensors
    
    # shared noise component
    shared_noise = np.random.randn(samples_per_sensor) * 20
    
    data = bytearray()
    for sensor_id in range(num_sensors):
        # each sensor = base value + shared noise + individual noise
        base = 100 + sensor_id * 10
        individual_noise = np.random.randn(samples_per_sensor) * 5
        signal = base + shared_noise + individual_noise
        signal = np.clip(signal, 0, 255).astype(np.uint8)
        data.extend(signal.tobytes())
    
    return bytes(data[:size_bytes])


def generate_mixed_patterns(size_bytes: int) -> bytes:
    """
    Generate data with alternating patterns (trend -> stable -> anomaly).
    """
    chunk_size = size_bytes // 3
    
    # trending segment
    t = np.arange(chunk_size)
    trend = 50 + (t / chunk_size) * 150
    trend = np.clip(trend, 0, 255).astype(np.uint8)
    
    # stable segment
    stable = np.full(chunk_size, 128, dtype=np.uint8)
    
    # anomaly segment
    anomaly = np.random.randint(0, 256, chunk_size, dtype=np.uint8)
    
    combined = np.concatenate([trend, stable, anomaly])
    return combined[:size_bytes].tobytes()


def main():
    output_dir = "data/iot"
    os.makedirs(output_dir, exist_ok=True)
    
    size_mb = 15
    size_bytes = size_mb * 1024 * 1024
    
    datasets = {
        "iot_trending.dat": generate_trending_data,
        "iot_anomaly.dat": generate_anomaly_data,
        "iot_correlated.dat": generate_correlated_sensors,
        "iot_mixed.dat": generate_mixed_patterns,
    }
    
    for filename, generator in datasets.items():
        path = os.path.join(output_dir, filename)
        data = generator(size_bytes)
        
        with open(path, 'wb') as f:
            f.write(data)
        
        print(f"Generated {filename}: {len(data):,} bytes")
    
    print(f"\nAll datasets saved to {output_dir}/")


if __name__ == "__main__":
    main()
