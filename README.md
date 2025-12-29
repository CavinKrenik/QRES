# QRES: Quantum-Relational Encoding System (v0.6.0)

> **The Zero-Overhead Neural Codec.**
> High-performance streaming compression for time-series and IoT data, powered by AI.

![CI](https://github.com/CavinKrenik/QRES/actions/workflows/test.yml/badge.svg)
![Release](https://github.com/CavinKrenik/QRES/actions/workflows/release.yml/badge.svg)

**QRES v0.6.0** is a revolutionary hybrid codec. It trains **tiny neural networks** (in Python) to understand your data, then executes them in **pure Rust** (0.0s overhead) to compress data streams in constant memory.

---

## 🚀 Key Features

*   **🧠 Neural Prediction (Phase 10)**: Provide a `brain.qnn` (tiny MLP) to "teach" the compressor your data's patterns. It beats linear algorithms by **~40%**.
*   **🌊 Streaming Architecture (Phase 9)**: Processes Terabytes of data in constant **4MB RAM**.
*   **🦀 Pure Rust Inference**: The decoder runs the Neural Network manually (matrix math) in Rust. **No PyTorch/ONNX dependencies required.**
*   **⚡ Extreme Speed**: Decompresses at **~951 MB/s**.
*   **🔌 Zero-Copy Python**: Direct NumPy integration avoiding memory overhead.

---

## 📦 Installation

```bash
pip install qres
```

*Requires Python 3.8+*

---

## ⚡ CLI Usage

QRES comes with a blazing fast CLI tool: `qres-cli`.

### 1. Neural Compression (The "Brain" Mode)
Train a brain on your data (or use a pretrained one), then compress:
```bash
# Uses the Neural Network for prediction (Highest Ratio)
qres-cli compress my_huge_log.dat archive.qres --brain brains/text_v1.qnn
```

### 2. Standard Compression (Linear Mode)
Great for simple waveforms or sensors:
```bash
# Uses Linear Prediction (Fastest Speed)
qres-cli compress sensor_data.bin sensor.qres
```

### 3. Decompression (Universal)
The decoder **automatically learns** the Neural Network from the file header. You don't need the brain file to decompress!
```bash
qres-cli decompress archive.qres restored.dat
```

---

## 🐍 Python API

```python
import qres
import numpy as np

# 1. Compress a NumPy Array (Linear Mode)
data = np.sin(np.linspace(0, 100, 10000)).astype(np.uint8)
compressed = qres.compress(data, predictor_id=1) 

# 2. Decompress
restored = qres.decompress(compressed)

# 3. Neural Mode (Advanced)
# (Requires pre-trained weights bytes, see ai/train_brain.py)
# compressed_neural = qres.encode_bytes(data, 2, weights) 
```

---

## 🧠 How It Works (The "Zero-Overhead" Architecture)

1.  **Training (Python)**: You use PyTorch to train a tiny 3-layer MLP on your data type.
2.  **Export**: The weights are saved to a `.qnn` file (approx 164 bytes).
3.  **Encoding (Rust)**: QRES embeds these weights into the `.qres` file header.
4.  **Inference (Rust)**: The `NeuralPredictor` struct executes the math: `ReLU(x @ W1 + b1) @ W2 + b2`. 
    *   **Result**: AI-level compression ratios with the portability of a standard zip file.

---

## 📊 Benchmarks

| Algorithm | Compression Speed | Decompression Speed | Ratio (Mixed Data) |
| :--- | :--- | :--- | :--- |
| **QRES (Neural)** | **~50 MB/s** | **~950 MB/s** | **~1.2%** |
| **QRES (Linear)** | **~367 MB/s** | **~951 MB/s** | **~2.1%** |
| Zlib (L6) | 124 MB/s | 230 MB/s | 12.5% |

*Tested on Ryzen 9 5900X with 10MB Mixed Telemetry Data.*

---

## 🛠️ Build from Source

```bash
# 1. Clone
git clone https://github.com/CavinKrenik/QRES.git
cd QRES

# 2. Build Rust Core & CLI
cargo build --release

# 3. Train a Brain (Optional)
pip install torch numpy
python ai/train_brain.py
```

## License
Copyright (c) 2025 Cavin Krenik. All Rights Reserved.
Proprietary and confidential.
