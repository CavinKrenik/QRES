# QRES: Quantum-Relational Encoding System (v0.8.0)

> **The Neural-Symbolic-Quantum Codec.**
> High-performance compression using Learnable Tensor Networks and Recurrent Models.

![CI](https://github.com/CavinKrenik/QRES/actions/workflows/test.yml/badge.svg)
![Release](https://github.com/CavinKrenik/QRES/actions/workflows/release.yml/badge.svg)

**QRES v0.8.0** is the world's first codec to integrate **Quantum-Inspired Tensor Networks** (MPS) alongside **Deep Learning** (LSTM/MLP). It offers a spectrum of "Digital Twin" predictors that run in **pure Rust** with zero dependencies.

---

## 🚀 Key Features

*   **⚛️ Tensor Prediction (Phase 13)**: Uses **Matrix Product States (MPS)** to engage a linear, learnable model that runs at **native speed** (5x faster than LSTM).
*   **🧠 Deep Temporal Compression (Phase 12)**: Uses **MicroLSTMs** to capture complex, non-linear periodic signals (Waveforms, Bio-data).
*   **🦀 Zero-Dependency Inference**: No PyTorch. No ONNX. Just manual, SIMD-optimized matrix math in Rust.
*   **🌊 Streaming Architecture**: Constant 4MB RAM usage for any file size.
*   **📦 Smart Headers**: Weights are embedded in the file. The decoder adapts automatically.

---

## ⚡ CLI Usage

QRES offers three "Brains" for your data:

### 1. The Speedster (Tensor/MPS)
Use for high-throughput streams where you need adaptability without the CPU cost of LSTM.
```bash
# Predictor ID 4: Fast, Linear, Learnable
qres-cli compress fast_stream.dat output.qres --brain models/tensor.qnn
```

### 2. The Analyst (LSTM)
Use for complex, non-linear data where compression ratio is paramount.
```bash
# Predictor ID 3: High Precision, captured long-range dependencies
qres-cli compress bio_signal.dat archive.qres --brain models/lstm.qnn
```

### 3. The Classic (Linear)
Use for standard simple sensors or when no model is available.
```bash
# Predictor ID 1: Standard Delta Encoding
qres-cli compress sensor.dat raw.qres
```

---

## 📊 Benchmarks (The Showdown)

| Engine | Type | Speed | Ratio (Complex Wave) | Best For... |
| :--- | :--- | :--- | :--- | :--- |
| **Tensor (MPS)** | Quantum-Linear | **0.04s** (Fast) | 15.0% | **High-Speed Streams** |
| **LSTM** | Non-Linear RNN | 0.20s (Slow) | **3.4%** | **Archival / Complex Data** |
| **Linear** | Static Math | 0.04s (Fast) | 3.5% | Simple Data |

*Tested on 1MB Modulated Sine Wave. Tensor matches Linear speed while offering learnability!*

---

## 🛠️ Build & Train

```bash
# 1. Build Rust Engine
cargo build --release

# 2. Train Your Own Brains
# Tensor (Fast)
python ai/train_tensor.py 
# LSTM (Smart)
python ai/train_lstm.py
```

## License
Copyright (c) 2025 Cavin Krenik. All Rights Reserved.
Proprietary and confidential.
