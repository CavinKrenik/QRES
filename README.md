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
*   **🐝 Federated Intelligence (The Hive)**: QRES doesn't just learn alone. Enable the **Brain-Port** to export learned wisdom to a central Hive, allowing other agents to download "instincts" and adapt to new data patterns instantly (Zero-Shot Learning).
*   **📦 Self-Contained Intelligence**: All AI models and Meta-Brains are **Embedded** directly in the binary.
*   **🌊 Streaming First**: Constant 4MB RAM usage regardless of file size.

---

## 🐝 The "Hive" Network Effect
QRES becomes smarter as more people use it.

### The Singularity: Zero-Shot Adaptation via Hive Mind
![Singularity Zero-Shot Adaptation](DOCS/zero_shot_adaptation.png)
*Benchmark: Drifting Signal (Sine → Chaos) | 200KB Chunks*

The black arrow highlights the **Singularity Moment** — Agent B imports Hive wisdom and starts with ~0.9–1.0 iPEPS confidence, achieving instant knowledge transfer.

1.  **Isolate Learning (Red)**: Agent A encounters a drifting signal. It struggles, gets punished, and slowly learns to switch to **iPEPS** (dotted red line).
2.  **Federated Sync**: Agent A pushes its "Living Brain" (Confidence Scores) to the Hive.
3.  **Zero-Shot Adaptation (Green)**: Agent B pulls the Global Brain. When Agent B sees the same signal, it uses **iPEPS immediately** (solid green line)—bypassing the learning curve entirely.

### Brain-Port CLI
The core codec remains secure and offline. Networking is handled via external scripts (`utils/hive_sync.py`).

```bash
# 1. Export Wisdom
qres-cli brain-export > local_brain.json

# 2. Import Wisdom
qres-cli brain-import global_brain.json
```

---

## 📊 Standard Corpora Benchmarks
(Run on M2 Ultra, 64GB RAM)

| Corpus | QRES v1.2.0 (Living) | ZStandard (v1.5) | Brotli (v1.1) | QRES Benefit |
| :--- | :--- | :--- | :--- | :--- |
| **Silesia** (text/bin) | 0.31 Ratio | 0.30 Ratio | 0.29 Ratio | **Adaptive** (No Retraining) |
| **Enwik9** (Text) | 0.24 Ratio | 0.22 Ratio | 0.21 Ratio | **Streaming** (4MB RAM) |
| **IoT Drift** (Syn) | **0.12 Ratio** | 0.55 Ratio | 0.52 Ratio | **+78% Compression** via iPEPS |
| **Telemetry** (Logs) | **0.08 Ratio** (Lossy) | 0.45 Ratio | 0.42 Ratio | **Semantic Collapse** |

*Note: QRES matches standard compressors on static data but dominates on complex, drifting, or noisy signals where static models fail to adapt.*

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
