# QRES: Quantum-Relational Encoding System 🧠⚛️

> **Revolutionary compression for the Singularity Era**: Brain-like spiking neural networks with quantum-inspired ML and distributed P2P swarms.

[![License](https://img.shields.io/badge/license-Apache%202.0-blue)](LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/CavinKrenik/QRES/test.yml?style=flat)](https://github.com/CavinKrenik/QRES/actions)
[![Version](https://img.shields.io/badge/version-v9.0-brightgreen)](https://github.com/CavinKrenik/QRES/releases)
[![Python](https://img.shields.io/badge/python-3.10+-blue)](https://python.org)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange)](https://rust-lang.org)

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
| ⚛️ **Quantum VQC** | Variational circuits for entanglement-based correlation |
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

# Build Rust core (optional, for native performance)
cd qres_rust && cargo build --release
```

### Basic Usage

```python
from qres import qres_rust

# Compress
data = open("input.dat", "rb").read()
compressed = qres_rust.encode_bytes(data, 0, b'')

# Decompress
restored = qres_rust.decode_bytes(compressed, b'')
```

### Try the Demo

Explore the interactive notebook: **[examples/brain_demo.ipynb](examples/brain_demo.ipynb)**

---

## 📁 Project Structure

```
QRES/
├── ai/                    # Neural networks (SNN, QNN, Hive Mind)
├── python/qres/           # Python API & utilities
├── qres_rust/             # Rust core engine
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
