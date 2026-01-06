# QRES: Quantum-Relational Encoding System

> **A neural, deterministic compression engine for the IoT and Edge Computing era.**

[![License](https://img.shields.io/badge/license-Apache%202.0-blue)](LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/CavinKrenik/QRES/release.yml?style=flat)](https://github.com/CavinKrenik/QRES/actions)
[![Version](https://img.shields.io/badge/version-v10.5.0-brightgreen)](https://github.com/CavinKrenik/QRES/releases)
[![Docs](https://img.shields.io/badge/docs-vision-orange)](docs/VISION.md)

QRES is a compression framework built for structured time-series data, leveraging Spiking Neural Networks (SNNs) and a federated P2P architecture to learn data patterns. Unlike general-purpose compressors (like Zstd or Gzip), QRES allows nodes to share "compression intelligence" (models) without sharing raw data, making it ideal for bandwidth-constrained IoT swarms and privacy-preserving analytics.

It features a **Rust core** (`no_std` capable), **Python bindings** for ML integration, and a **WebAssembly** build for browser-based decompression.

---

## 🚀 Quickstart

### 1. Encode your first file
The fastest way to test QRES is via the Python CLI.

```bash
# Clone the repository
git clone https://github.com/CavinKrenik/QRES.git
cd QRES

# Install Python package (requires Rust toolchain)
pip install .

# Compress a file
python3 -c "import qres; print(f'Compressed size: {len(qres.compress(open(\"README.md\", \"rb\").read()))} bytes')"
```

### 2. Build the Daemon (Rust)
For production or P2P usage, build the native daemon:

```bash
cd qres_rust
cargo build --release --bin qres_daemon
./target/release/qres_daemon start
```

---

## ✨ Core Features

*   **Neural Prediction Engine**: Utilizes SNNs to predict and compress repetitive data streams (telemetry, logs) more efficiently than static dictionaries.
*   **Deterministic Architecture**: Built on **Q16.16 fixed-point arithmetic**, ensuring bit-perfect reproducibility across x86, ARM, RISC-V, and WASM targets.
*   **Federated Learning ("Hive Mind")**: Nodes can exchange model weights to improve compression ratios on similar data types without transmitting the data itself.
*   **Edge-Native**: Core logic is `no_std` compatible, capable of running on bare-metal embedded devices.
*   **Hybrid Runtime**: Seamlessly switch between native performance and portable WebAssembly execution.

For the full technical vision, including our "Singularity Engine" concepts, read [VISION.md](docs/VISION.md).

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    QRES Architecture                        │
├─────────────────────────────────────────────────────────────┤
│  Input Data  →  [SNN Predictor]  →  [TNC Fusion]  →  Output │
│       ↑              ↓                   ↓             ↓    │
│       └── [MetaBrain RL Agent] ←─ [Mixer Weights] ←────┘    │
│                      ↓                                      │
│              [P2P Hive Mind]  ←→  [Other Nodes]             │
└─────────────────────────────────────────────────────────────┘
```

---

## 🎯 When to Use QRES

| ✅ Use QRES For... | ❌ Do NOT Use QRES For... |
|--------------------|---------------------------|
| **IoT Telemetry**: Highly repetitive sensor logs. | **High-Entropy Data**: Encrypted files, random noise. |
| **Structured Logs**: Server logs with consistent timestamps/headers. | **Existing Archives**: .zip, .jpg, .mp4 files. |
| **Edge Networks**: Where bandwidth is more expensive than compute. | **Tiny Files**: Files < 1KB (header overhead is too high). |
| **Archival**: Long-term storage requiring deterministic restoration. | **General Purpose**: If strictly fastest speed is needed (use LZ4). |

---

## � Mini Benchmarks

Performance on structured data sets (Intel Ice Lake). See full details in [BENCHMARKS.md](docs/BENCHMARKS.md).

| Dataset | Type | Ratio | Speed |
|---------|------|-------|-------|
| **Sensor Stream** | IoT Telemetry | **~0.15** (6.6x) | 300+ MB/s |
| **Server Logs** | Text/Time-series | **~0.19** (5.2x) | 200 MB/s |
| **CSV Data** | Correlated Numerics | ~0.76 (1.3x) | 150 MB/s |

---

## � Project Structure

*   `qres_rust/`: The Rust workspace containing the core engine and daemon.
    *   `qres_core`: High-performance, `no_std` compression library.
    *   `qres_daemon`: P2P node and API service.
*   `python/`: Python bindings and experimental ML models.
*   `qres-studio/`: Cross-platform GUI (Tauri/Svelte) for visualization.
*   `docs/`: Documentation hub.
    *   [VISION.md](docs/VISION.md): Project philosophy and long-term goals.
    *   [WHITEPAPER.md](docs/WHITEPAPER.md): Detailed technical specification.
    *   [RESEARCH_NOTES.md](docs/RESEARCH_NOTES.md): Academic context and citations.
*   `benchmarks/`: Data generation and performance testing scripts.

---

## 🗺️ Status & Roadmap

**Current Version:** v10.5 (Hybrid Era)

*   ✅ **Production Ready**: `qres_core`, Python bindings, WASM decoder.
*   � **Beta**: P2P Swarm APIs, Advanced Federation.
*   📅 **Planned**: Hardware description language (HDL) implementation.

See [ROADMAP.md](docs/ROADMAP.md) for the detailed timeline.

---

## 📄 License & Acknowledgments

This project is licensed under the Apache 2.0 License.

**QRES** – *Building the neural pathways for a self-optimizing internet.*
