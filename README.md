# QRES: Quantum-Relational Encoding System (v0.7.0)

> **The Generative AI Codec.**
> Embeds "Digital Twin" neural models (LSTM/MLP) into data streams for high-performance compression.

![CI](https://github.com/CavinKrenik/QRES/actions/workflows/test.yml/badge.svg)
![Release](https://github.com/CavinKrenik/QRES/actions/workflows/release.yml/badge.svg)

**QRES v0.7.0** introduces **Deep Temporal Compression**. It uses **Long Short-Term Memory (LSTM)** networks to learn complex, long-range dependencies in your data (like biological signals or audio waveforms), executing them in **pure Rust** with zero overhead.

---

## 🚀 Key Features

*   **🧠 Deep Temporal Compression (LSTM)**: New in v0.7.0. Captures periodic signals and complex waves that linear predictors miss.
*   **🕸️ Neural Prediction (MLP)**: Great for non-linear but stateless patterns.
*   **🦀 Zero-Dependency Inference**: Both MLP and LSTM models run in **pure Rust** (manual matrix math). No heavy AI runtimes.
*   **🌊 Streaming Architecture**: Processes unlimited data size in constant **4MB RAM**.
*   **� Self-Contained Archives**: The neural model weights are **embedded in the file header**. Decompression needs only the file itself.

---

## ⚡ CLI Usage

### 1. Deep Temporal Compression (LSTM)
Train an LSTM on your data type, then compress:
```bash
# Uses LSTM (Predictor ID 3) - Best for complex waves
qres-cli compress bio_signal.dat Signal.qres --brain models/lstm_bio.qnn
```

### 2. Neural Compression (MLP)
```bash
# Uses MLP (Predictor ID 2) - Good for general non-linear data
qres-cli compress sensor_log.dat Sensor.qres --brain models/mlp_sensor.qnn
```

### 3. Decompression (Universal)
The decoder **automatically detects** the model (LSTM or MLP) from the file header and configures the inference engine instantly.
```bash
qres-cli decompress Signal.qres restored.dat
```

---

## 🐍 Python API

```python
import qres
import numpy as np

# 1. Compress a NumPy Array (Linear Mode)
data = np.sin(np.linspace(0, 100, 10000)).astype(np.uint8)
compressed = qres.compress(data, predictor_id=1) 

# 2. Advanced: Neural/LSTM Encoding
# (Requires pre-trained weights bytes)
# compressed = qres.encode_bytes(data, 3, lstm_weights_bytes)
```

---

## 📊 Benchmarks

| Algorithm | Model | Ratio (Complex Wave) | Notes |
| :--- | :--- | :--- | :--- |
| **QRES (LSTM)** | **MicroLSTM (H=8)** | **3.4%** | **Winner 🏆** - Captured the wave period. |
| **QRES (Linear)** | Delta | 3.5% | Good baseline, but missed nuances. |
| **QRES (MLP)** | MLP (3x8) | 13.4% | Failed to capture temporal dependencies. |
| Zlib (L6) | DEFLATE | 12.5% | General purpose, not tuned for signals. |

*Tested on 1MB Modulated Sine Wave (`sin(t) * cos(t/3)`).*

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
python ai/train_lstm.py
```

## License
Copyright (c) 2025 Cavin Krenik. All Rights Reserved.
Proprietary and confidential.
