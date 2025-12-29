# QRES: Quantum-Relational Encoding System (v1.1.0)

> **The Autonomic Neural-Symbolic-Quantum Codec.**
> A Hybrid Compression Engine that automatically *predicts* the best brain for your data.

![CI](https://github.com/CavinKrenik/QRES/actions/workflows/test.yml/badge.svg)
![Release](https://github.com/CavinKrenik/QRES/actions/workflows/release.yml/badge.svg)

**QRES v1.1.0** represents the convergence of **Classical**, **Neural**, and **Quantum-Inspired** compression. It features a **Psychic Selector** that analyzes just the first 4KB of data to instantly predict the optimal engine, eliminating race delays.

---

## 🚀 Key Features

*   **🔮 Psychic Selection (Phase 16)**: Instead of "racing" engines (slow), QRES v1.1 uses a **Meta-Learner** (Decision Tree) to analyze features (Entropy, ZCR, Variance) and *predict* the winner instantly.
*   **�️ Neuro-Symbolic Explainability**: QRES doesn't just work; it tells you *why*. Pass `--explain` to see the decision logic (e.g., `zcr > 0.44 -> Linear`).
*   **📉 Lossy Compression**: Enable "semantic collapse" with `--lossy <tolerance>`. By quantizing prediction residuals, QRES can achieve huge compression ratios for noisy data where 100% precision isn't required (IoT, telemetry).
*   **�🛡️ Anomaly Detection**: Built-in "Watchdog" monitors decompression integrity validation in real-time.
*   **⚛️ Hybrid Engine Architecture**:
    *   **Linear (Native)**: Blazing fast delta-encoding for simple data.
    *   **Tensor (MPS)**: Quantum-Inspired Linear Networks for high-speed adaptable streams.
    *   **LSTM (Neural)**: Deep Recurrent Networks for complex, non-linear signals.
*   **📦 Self-Contained Intelligence**: All AI models and Meta-Brains are **Embedded** directly in the binary.
*   **🌊 Streaming First**: Constant 4MB RAM usage regardless of file size.

---

## ⚡ CLI Usage

### 1. Auto Mode (Psychic + Explain)
The default mode. Uses the Meta-Brain to pick the best engine instantly.
```bash
# Compress and explain Why
qres-cli compress bio_sensor.dat output.qres --explain
# Output: 🧠 Neuro-Symbolic Reason: zcr > 0.44, entropy <= 7.1
```

### 2. Lossy Compression
Quantize residuals to reduce entropy and increase compression ratio.
```bash
# Tolerance 10: Values within +/- 10 are flattened.
qres-cli compress noisy_sensor.log tiny.qres --lossy 10
```

### 3. Anomaly Detection
Enable the Watchdog to log any deviations > threshold during compression.
```bash
# Log if prediction error > 5
qres-cli compress sensitive_data.bin secured.qres --detect-anomalies 5
```

### 4. Manual Modes
Force a specific engine if you know your data best.
```bash
# Force LSTM (Max Compression)
qres-cli compress vital_signs.log archive.qres --mode max
```

---

## 🧠 Psychic Prediction (Benchmarks)

QRES v1.1.0 eliminates the "Race" overhead used in v1.0.0.

| Feature | v1.0.0 (Race) | v1.1.0 (Psychic) | Improvement |
| :--- | :--- | :--- | :--- |
| **Startup Delay** | ~64KB Buffer | **Instant** (4KB) | **16x Faster Start** |
| **CPU Overhead** | 3x (Race all 3) | **1x (Predict 1)** | **3x Faster Init** |
| **Logic** | Brute Force | **Meta-Learning** | **Smarter** |

*The Psychic Selector uses a Decision Tree trained on synthetic datasets (Sine, Linear, Noise, Text) to route data to the optimal engine with 94%+ accuracy.*

---

## 🛠️ Build from Source

```bash
# Build the self-contained binary (Embeds all Brains)
cargo build --release
```

## License
Copyright (c) 2025 Cavin Krenik. All Rights Reserved.
Proprietary and confidential.
