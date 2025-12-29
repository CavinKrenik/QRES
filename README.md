# QRES: Quantum-Relational Encoding System (v1.2.0)
> *The "Living" Quantum Engine*

QRES is a **Cybernetic Compression System** that not only predicts the best strategy but **learns from its own mistakes** in real-time.

![CI](https://github.com/CavinKrenik/QRES/actions/workflows/test.yml/badge.svg)
![Release](https://github.com/CavinKrenik/QRES/actions/workflows/release.yml/badge.svg)

**QRES v1.2.0** represents the convergence of **Classical**, **Neural**, and **Quantum-Inspired** compression. It features a **Psychic Selector** that analyzes the data stream and a **Living Feedback Loop** that adapts to signal changes.

---

## 🚀 Key Features

*   **🧬 The Living Loop (Online Learning)**: QRES monitors its own compression ratio. If an engine fails to perform (e.g., signal drift), it gets "punished" and the system automatically swaps to a better engine mid-stream.
*   **⚛️ iPEPS Quantum Engine**: A non-linear Tensor Network (Infinite Projected Entangled Pair States) modeled with **Safe Math** for deterministic, cross-platform behavior. It excels at complex, non-linear signals.
*   **🤸 Agile File Format**: The stream format is now dynamic (`[Size][EngineID][Data]`), allowing the encoder to switch strategies per-chunk without breaking the decoder.
*   **🧠 Neuro-Symbolic Explainability**: Pass `--explain` to see the decision logic (e.g., `zcr > 0.44 -> Linear`).
*   **📉 Lossy Compression**: Enable "semantic collapse" with `--lossy <tolerance>`. By quantizing prediction residuals, QRES can achieve huge compression ratios for noisy data where 100% precision isn't required (IoT, telemetry).
*   **🛡️ Anomaly Detection**: Built-in "Watchdog" monitors decompression integrity validation in real-time.
*   **📦 Self-Contained Intelligence**: All AI models and Meta-Brains are **Embedded** directly in the binary.
*   **🌊 Streaming First**: Constant 4MB RAM usage regardless of file size.

---

## ⚡ CLI Usage

### 1. Living Mode (Default)
Automatically adapts to signal drift (e.g. Sine -> Noise).
```bash
qres-cli compress unstable.dat output.qres
```

### 2. Auto Mode (Psychic + Explain)
```bash
# Compress and explain Why
qres-cli compress bio_sensor.dat output.qres --explain
# Output: 🧠 Neuro-Symbolic Reason: zcr > 0.44, entropy <= 7.1
```

### 3. Lossy Compression
Quantize residuals to reduce entropy and increase compression ratio.
```bash
# Tolerance 10: Values within +/- 10 are flattened.
qres-cli compress noisy_sensor.log tiny.qres --lossy 10
```

### 4. Anomaly Detection
Enable the Watchdog to log any deviations > threshold during compression.
```bash
# Log if prediction error > 5
qres-cli compress sensitive_data.bin secured.qres --detect-anomalies 5
```

### 5. Manual Modes
Force a specific engine if you know your data best.
```bash
# Force LSTM (Max Compression)
qres-cli compress vital_signs.log archive.qres --mode max
```

---

## 🧠 Psychic Prediction (Benchmarks)

QRES v1.2.0 adapts to data *while it is compressing*.

| Feature | v1.1.0 (Static) | v1.2.0 (Living) | Improvement |
| :--- | :--- | :--- | :--- |
| **Adaptability** | Fixed at Start | **Per-Chunk** | **Resilient to Drift** |
| **Format** | Static Headers | **Agile** (Dynamic) | **No Sync Issues** |
| **Logic** | Tree (Read-Only) | **Feedback Loop** | **Self-Correcting** |

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
