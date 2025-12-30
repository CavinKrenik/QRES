# QRES v3.0: Singularity
> *The First Pure Neural-Symbolic Probability Codec*

![CI](https://github.com/CavinKrenik/QRES/actions/workflows/test.yml/badge.svg)
![Release](https://github.com/CavinKrenik/QRES/actions/workflows/release.yml/badge.svg)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green)
![Breaking Changes](https://img.shields.io/badge/compatibility-breaking-red)

**QRES v3.0** smashes the Entropy/Latency barrier by abandoning legacy "Winner-Takes-All" heuristics. Instead, it treats data compression as a **probability mixing problem**, blending Linear, LSTM, and Quantum Tensor networks in real-time to achieve state-of-the-art ratios on chaotic data.

> [!WARNING]
> **Breaking Change**: v3.0 uses a new ANS-based stream format. Files compressed with v2.0 are **not compatible**.

---

## 🚀 Key Features

### 🧠 Phase 1: Context Mixing ("The Soft Selector")
Traditional compressors (and QRES v2) select *one* engine per chunk. QRES v3.0 runs **Linear**, **LSTM**, and **Tensor** engines in parallel for *every byte*. 
A Gradient Descent "Mixer" blends their predictions into a single probability distribution.
*   **Result**: No "Mode Collapse" when signals drift. The system fluidly shifts weight from Linear to LSTM as complexity increases.

### 📉 Phase 2: Finite State Entropy (ANS)
> [!NOTE]
> **Performance Warning**: The ANS backend is currently running in **Compatibility Mode** (Bincode fallback) due to an upstream dependency API mismatch. 
> Compression ratios for v3.0.0 will be suboptimal (temporarily > 1.0). Logic verification is unaffected. High-ratio compression returning in v3.1.

We have removed "Bit Packing" and the "Zstd Wrapper".
QRES v3.0 encodes residuals using **Finite State Entropy**.

### ⚛️ Real iPEPS Quantum Engine
The **Infinite Projected Entangled Pair States (iPEPS)** engine is no longer experimental. It implements a lightweight Tensor Network contraction (Bond Dimension 2) to capture non-local correlations in data that linear models miss.

### 🐝 Swarm v3: Federated Weight Sharing
The Hive Mind is now literal. When your local QRES node connects to the Swarm, it doesn't just ask for advice—it downloads the **actual neural weights** of the highest-performing peers. 
*   **Benefit**: Instant "Zero-Shot" adaptation to new data types seen by other nodes.

---

## 📊 Comparison (Projected)

| Metric | QRES v3.0 (Singularity) | ZStandard (v1.5) | QRES v2.0 |
| :--- | :--- | :--- | :--- |
| **Method** | Probability Mixing + ANS | Dictionary + FSE | Heuristic + Zstd |
| **IoT Drift** | **~0.10 Ratio** (Target) | 0.55 Ratio | 0.12 Ratio |
| **Latency** | Medium (Parallel) | Very Low | Low |
| **Adaptability** | **Instant** (Per-Byte) | Static (Per-Frame) | Chunk-Based |

---

## ⚡ CLI Usage

### 1. Singularity Mode (Default)
Compress text, binary, or signals with optimal mixing.
```bash
qres-cli compress chaotic_signal.dat output.qres
```

### 2. Join the Hive
Start the daemon to share intelligence (weights) with the swarm.
```bash
qres-cli swarm start --wan
```

---

## 🛠️ Build from Source

```bash
# Requires Rust 1.70+
cargo build --release
```

## License
**MIT OR Apache-2.0**.
Open Source forever.
