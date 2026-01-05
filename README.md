# QRES: Quantum-Relational Encoding System 🧠⚛️

> **Revolutionary compression for the Singularity Era**: Brain-like spiking neural networks with quantum-inspired ML and distributed P2P swarms.

[![License](https://img.shields.io/badge/license-Apache%202.0-blue)](LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/CavinKrenik/QRES/test.yml?style=flat)](https://github.com/CavinKrenik/QRES/actions)
[![Version](https://img.shields.io/badge/version-v10.0-brightgreen)](https://github.com/CavinKrenik/QRES/releases)
[![Python](https://img.shields.io/badge/python-3.10+-blue)](https://python.org)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange)](https://rust-lang.org)

---

## Non-Goals & Limitations
QRES is optimized for *structured, time-series, and predictable* data (e.g., telemetry, logs, sensor streams).
*   **Encrypted/Random Data:** QRES will *expand* high-entropy data (like ZIP files or randomness). Use Zstd for these.
*   **Cold Storage:** QRES requires a small "learning curve" (warmup) to build its model. It is less effective on tiny files (<1KB).

---

## 🌟 Overview

QRES transcends traditional compression with a **"Living Brain"** agent that predicts, adapts, and evolves data storage like human memory. Unlike static algorithms (Zstd, gzip), QRES learns from your data and gets smarter over time.

```
┌─────────────────────────────────────────────────────────────┐
│                    QRES Architecture                        │
├─────────────────────────────────────────────────────────────┤
│  Input Data  →  [SNN Predictor]  →  [QNN Fusion]  →  Output │
│       ↑              ↓                   ↓             ↓    │
│       └── [MetaBrain RL Agent] ←─ [Mixer Weights] ←────┘    │
│                      ↓                                      │
│              [P2P Hive Mind]  ←→  [Other Nodes]             │
└─────────────────────────────────────────────────────────────┘
```

---

## ✨ Key Features

| Feature | Description |
|---------|-------------|
| 🧠 **Spiking Neural Networks** | GIF neurons with 97% sparsity via OSBC pruning |
| ⚛️ **Tensor Network Correlator** | Variational circuits for non-linear correlation |
| 🐝 **Hive Mind** | Federated learning with KL-FedDis divergence filtering |
| 🔄 **Auto-Tuning** | Fine-tune on your data with federated sharing |
| 📦 **Multimodal** | Text, IoT, images, audio, PDFs |
| 🖥️ **GUI** | QRES Studio with D3 visualizations |

---

## 📊 Performance

| Dataset | Ratio | Fidelity | Speed |
|---------|-------|----------|-------|
| **IoT Telemetry** | 0.537 | >0.99 | 150 MB/s |
| **Text/Code** | ~0.19 | >0.99 | 200 MB/s |
| **Multimodal** | ~0.6-0.9 | >0.99 | 150 MB/s |

*Benchmarks on Intel Ice Lake. See [BENCHMARKS.md](docs/BENCHMARKS.md) for details.*

---

## 🚀 Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/CavinKrenik/QRES.git
cd QRES

# Install Python dependencies
pip install -e .

# Build Rust core (optimized native binary)
cd qres_rust && cargo build --release --workspace
```

### Basic Usage

```python
import qres

# Compress
data = open("input.dat", "rb").read()
compressed = qres.compress(data)

# Decompress
restored = qres.decompress(compressed)
```

### Try the Demo

Explore the interactive notebook: **[examples/brain_demo.ipynb](examples/brain_demo.ipynb)**

---

## 📁 Project Structure

```
QRES/
├── ai/                    # Neural networks (SNN, QNN, Hive Mind)
├── python/qres/           # Python API & utilities
├── qres_rust/             # Rust Workspace
│   ├── qres_core/         # Pure compression library (Codecs)
│   └── qres_daemon/       # P2P Swarm & Training Node
├── qres-studio/           # Svelte/Tauri GUI
├── docs/                  # Documentation
├── examples/              # Demo notebooks
├── tests/                 # Test suite
└── benchmarks/            # Performance data
```

---

## 📖 Documentation

| Document | Description |
|----------|-------------|
| [WHITEPAPER.md](docs/WHITEPAPER.md) | Technical deep-dive |
| [ROADMAP.md](docs/ROADMAP.md) | Development phases |
| [BENCHMARKS.md](docs/BENCHMARKS.md) | Performance metrics |
| [RESEARCH_NOTES.md](docs/RESEARCH_NOTES.md) | Academic citations |

---

## 🗺️ Roadmap

- ✅ **v8.0** – MetaBrain v4, Multimodal, World State Persistence
- ✅ **v8.1** – SNN Integration, QNN Fusion, Hive Mind
- ✅ **v9.0** – GIF Neurons, OSBC Pruning, Auto-Tuning
- 🔮 **v10.0** – True Quantum Hardware (AWS Braket), Ratio <0.30

---

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines.

**Areas of interest:**
- SNN optimization for breakthrough ratios
- Quantum circuit improvements
- P2P swarm scalability

---

## 📜 License

[Apache 2.0](LICENSE) – Free for commercial and personal use.

---

<p align="center">
  <strong>Built with 🧠 for the Singularity Era</strong><br>
  <a href="https://github.com/CavinKrenik/QRES">GitHub</a> •
  <a href="docs/WHITEPAPER.md">Whitepaper</a> •
  <a href="examples/brain_demo.ipynb">Demo</a>
</p>
