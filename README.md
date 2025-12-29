# QRES: Quantum-Relational Encoding System (v1.0.0)

> **The Autonomic Neural-Symbolic-Quantum Codec.**
> A Hybrid Compression Engine that automatically selects the best brain for your data.

![CI](https://github.com/CavinKrenik/QRES/actions/workflows/test.yml/badge.svg)
![Release](https://github.com/CavinKrenik/QRES/actions/workflows/release.yml/badge.svg)

**QRES v1.0.0** represents the convergence of **Classical**, **Neural**, and **Quantum-Inspired** compression. It features a built-in **Autonomic Selector** that races three distinct engines against each other in real-time to optimize for either Speed or Compression Ratio.

---

## 🚀 Key Features

*   **🤖 Autonomic Selection (Phase 14)**: The "Qualifier" engine samples your data stream, races all available models (Linear, Tensor, LSTM), and behaves like a smart compiler—picking the best tool for the job.
*   **⚛️ Hybrid Engine Architecture**:
    *   **Linear (Native)**: Blazing fast delta-encoding for simple data.
    *   **Tensor (MPS)**: Quantum-Inspired Linear Networks for high-speed adaptable streams.
    *   **LSTM (Neural)**: Deep Recurrent Networks for complex, non-linear signals (Bio-data, Audio).
*   **📦 Self-Contained Intelligence**: All AI models are **Embedded** directly in the binary. No external `.qnn` files or Python dependencies required for inference.
*   **🌊 Streaming First**: Constant 4MB RAM usage regardless of file size (TB+).

---

## ⚡ CLI Usage

Simplicity is the ultimate sophistication.

### 1. Auto Mode (Default)
Let the codec decide. It balances Speed vs. Size.
```bash
# Automatically races Linear vs Tensor vs LSTM
qres-cli compress bio_sensor.dat output.qres
```

### 2. Max Compression
Force the Neural Engine (LSTM) to squeeze every bit, regardless of CPU cost.
```bash
# Force LSTM (Predictor ID 3)
qres-cli compress vital_signs.log archive.qres --mode max
```

### 3. Fast Mode
Force the Linear Engine for maximum throughput (IGB/s scenarios).
```bash
# Force Linear (Predictor ID 1)
qres-cli compress high_speed_log.csv fast.qres --mode fast
```

---

## 📊 The "Race" (Benchmarks)

How the Autonomic Selector makes decisions:

| Candidate | Tech | Speed | Ratio (Wave) | Selection Logic |
| :--- | :--- | :--- | :--- | :--- |
| **Linear** | Delta Math | **0.04s** ⚡ | 3.5% | **Selected** if gains < 5% vs CPU cost. |
| **Tensor** | Quantum MPS | 0.04s ⚡ | 15.0% | **Selected** for linear but varying streams. |
| **LSTM** | Deep RNN | 0.20s 🐢 | **3.4%** | **Selected** only if high-compression justifies slow speed. |

*In v1.0.0, QRES prevents "over-engineering" by defaulting to Linear/Tensor when the LSTM's heavy compute isn't justified by significant space savings.*

---

## 🛠️ Build from Source

```bash
# Build the self-contained binary (Embeds all Brains)
cargo build --release
```

## License
Copyright (c) 2025 Cavin Krenik. All Rights Reserved.
Proprietary and confidential.
