# QRES: Quantum-Relational Encoding System

> **High-Performance Adaptive Compression for Time-Series & IoT Data**

![CI](https://github.com/CavinKrenik/QRES/actions/workflows/test.yml/badge.svg)
![Release](https://github.com/CavinKrenik/QRES/actions/workflows/release.yml/badge.svg)

**QRES v2** is a next-generation hybrid compression codec (Rust Core + Python Bindings) designed for data that moves in predictable patterns (sensors, stocks, physics simulations). It uses **Bit-Packed Delta Encoding** with **Adaptive Logic** to achieve massive compression ratios speedups over Zlib/Gzip.

---

## 🚀 Key Features

*   **Extreme Speed**: Compresses at **~367 MB/s** and decompresses at **~951 MB/s** (4x faster than Zlib).
*   **Adaptive Intelligence**: Automatically selects between **QRES Mode** (for waveforms) and **Raw Mode** (for text), ensuring you never lose efficiency.
*   **Zero-Copy Python**: Direct NumPy integration avoids memory overhead.
*   **SIMD Optimized**: Uses SWAR (SIMD Within A Register) for parallel bit processing.

## 📦 Installation

```bash
pip install qres
```

*Requires Python 3.8+*

## ⚡ Usage

```python
import qres
import numpy as np

# 1. Compress a NumPy Array (Zero-Copy)
data = np.sin(np.linspace(0, 100, 10000)).astype(np.uint8)
compressed = qres.compress(data)

# 2. Decompress
restored = qres.decompress(compressed)

# 3. File Context Manager
with qres.open("sensor_log.qres", "wb") as f:
    f.write(data)
```

## 📊 Benchmarks

| Algorithm | Compression Speed | Decompression Speed | Ratio (Sine Wave) |
| :--- | :--- | :--- | :--- |
| **QRES v2** | **367 MB/s** | **951 MB/s** | **0.14%** |
| Zlib (L6) | 124 MB/s | 230 MB/s | 12.5% |

*Tested on Ryzen 9 5900X with 10MB Synthetic Telemetry Data.*

## 🛠️ Build from Source

```bash
# Prerequisites: Rust (stable) & Python 3.8+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 1. Clone
git clone https://github.com/CavinKrenik/QRES.git
cd QRES

# 2. Build & Install (using maturin)
pip install maturin
maturin develop --release

# 3. Run Verification
python benchmarks/torture_test.py
```

## 📜 License

MIT License. Copyright (c) 2025 Cavin Krenik.
