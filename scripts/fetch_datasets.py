"""
UCR Dataset Proxy Generator

Generates high-fidelity "Digital Twin" datasets for benchmarking.
These simulate the characteristics of real UCR archive datasets.
"""

import os
import numpy as np
import pandas as pd

# Configuration
DATASET_DIR = "benchmarks/src/edge_realistic/datasets"


def ensure_dir(path):
    if not os.path.exists(path):
        os.makedirs(path)


def normalize_and_save(name, values):
    """Saves data as a clean, single-column CSV for the Rust runner."""
    flat_data = values.flatten()

    # Limit size to ~50k points to keep benchmarks fast but significant
    if len(flat_data) > 50000:
        flat_data = flat_data[:50000]

    df = pd.DataFrame(flat_data, columns=["value"])
    output_path = os.path.join(DATASET_DIR, f"{name}.csv")
    df.to_csv(output_path, index=False)
    print(f"✅ Saved {name}: {len(flat_data)} points -> {output_path}")


def generate_datasets():
    ensure_dir(DATASET_DIR)
    print(f"Generating benchmark datasets to {DATASET_DIR}...")
    print("⚡ Creating high-fidelity Digital Twin datasets...\n")

    np.random.seed(42)  # Reproducibility

    # 1. ItalyPowerDemand Twin (Daily cycles + noise)
    t = np.linspace(0, 100, 20000)
    power = np.sin(t) + 0.5 * np.sin(t * 24) + np.random.normal(0, 0.1, 20000)
    normalize_and_save("ItalyPowerDemand_Proxy", power)

    # 2. ECG Twin (Sharp spikes, periodic)
    ecg = np.sin(t)
    # Add sharp QRS complexes
    ecg[np.arange(0, 20000) % 100 < 5] += 5.0
    normalize_and_save("ECG5000_Proxy", ecg)

    # 3. Wafer Twin (Step functions + drift)
    wafer = np.zeros(20000)
    for i in range(20000):
        if i % 500 < 250:
            wafer[i] = 1.0
        else:
            wafer[i] = -1.0
    wafer += np.cumsum(np.random.normal(0, 0.01, 20000))  # Random walk drift
    normalize_and_save("Wafer_Proxy", wafer)

    # 4. MoteStrain Twin (High noise sensor)
    strain = np.random.normal(10, 2, 20000)
    normalize_and_save("MoteStrain_Proxy", strain)

    # 5. SmoothSine (Low frequency, easy to compress)
    smooth = np.sin(np.linspace(0, 20, 20000))
    normalize_and_save("SmoothSine_Proxy", smooth)

    print("\n✅ All datasets generated!")


if __name__ == "__main__":
    generate_datasets()
