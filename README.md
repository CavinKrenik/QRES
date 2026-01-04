# QRES: The Singularity Engine

[![Latest Release](https://img.shields.io/github/v/tag/CavinKrenik/QRES?include_prereleases&style=flat-square&color=blue&label=release)](https://github.com/CavinKrenik/QRES/releases)
[![Build Status](https://img.shields.io/github/actions/workflow/status/CavinKrenik/QRES/test.yml?style=flat-square)](https://github.com/CavinKrenik/QRES/actions)
[![License](https://img.shields.io/badge/license-Apache--2.0-green?style=flat-square)](LICENSE)
[![Contributors](https://img.shields.io/github/contributors/CavinKrenik/QRES?style=flat-square)](https://github.com/CavinKrenik/QRES/graphs/contributors)
[![Documentation](https://img.shields.io/badge/docs-complete-blue?style=flat-square)](docs/)

**QRES (Quantum-Relational Encoding System)** is an open-source, adaptive compression framework designed for the "Singularity Era" of massive, dynamic data volumes (e.g., IoT telemetry, code repositories, multimedia archives). Unlike static algorithms like Zstandard or gzip, QRES employs a "Living Brain"—an AI-driven system combining neural-symbolic prediction, reinforcement learning (RL), quantum-inspired tensors, and swarm intelligence—to dynamically optimize compression on a per-chunk basis. It achieves superior ratios, speed, and fidelity (>0.98 reconstruction accuracy) by treating data as a living entity that can be predicted, entangled, and shared across distributed nodes.

**Project Goals**: Build a self-evolving compressor that adapts to diverse data types (text, binary, images, audio, PDFs) through RL training (MetaBrain PPO agent), P2P sharing of learned models, and quantum simulations—targeting 20-62% better ratios than baselines for edge/IoT scenarios, while enabling collaborative, bias-free archival in decentralized systems.

> "Data is not static; your compressor shouldn't be either."

**Topics:** ai-compression, neural-symbolic, quantum-inspired, rust, python, p2p-swarm, reinforcement-learning

---

## 🚀 Key Features

### 🧠 Cognitive Context & The "Living Brain" (MetaBrain v5)
- **Spiking Neural Networks (SNN):** Leaky Integrate-and-Fire neurons for temporal, sparse data encoding (v8.1).
- **Quantum VQC Fusion (QNN):** Variational Quantum Circuits for entanglement-based correlation detection (v8.1).
- **Reinforcement Learning Agent:** PPO-based MetaBrain (v5, SNN+QNN hybrid, 261-dim observations).
- **Multimodal Support:** Handles diverse inputs via CLIP embeddings and binary fallbacks.

### ⚛️ Quantum-Inspired Tensors (v7/v8)
- **Tensor Networks:** QuTiP-simulated states for high-dimensional correlations; ethical pruning for bias mitigation.
- **Binary Fallback:** Spectral graph building for non-UTF8 data in quantum mode.

### 🐝 Swarm Intelligence (P2P) & Hive Mind
- **Hive Mind:** Federated Averaging (FedProx) for continual learning across nodes without raw data sharing (`ai/hive_mind.py`).
- **Kademlia DHT + GossipSub:** For sharing "Epiphanies" (model weights) and quantum states.
- **Persistent World State:** Synchronization with >0.98 fidelity across nodes.

### 📦 QRAR Archives
- **Deduplication:** Content-Defined Chunking (CDC) for redundancy elimination.
- **Optimization:** Widened entropy thresholds (0.2-7.8) for broader predictive engine usage.

---

## 📊 Current Status

| Version | Status | Focus |
| :--- | :--- | :--- |
| **v8.1.0** | **Development** | Brain-Like Quantum ML (SNN, QNN, Hive Mind) |
| **v8.0.0** | **Released** | AEON Update – MetaBrain v4, Swarm Persistence, Multimodal Training |
| **v7.5** | Stable | Quantum Foundations |

Latest release: v8.1.0-dev (Jan 4, 2026). See [CHANGELOG.md](CHANGELOG.md) for details.

---

## ⚡ Performance

See [BENCHMARKS.md](docs/BENCHMARKS.md) for full analysis. Recent v4 training maintains consistency on core data while improving multimodal handling.

| Engine | Ratio (IoT, 20MB) | Ratio (Text) | Speed (MB/s) |
| :--- | :---: | :---: | :---: |
| **QRES v8.1 (MetaBrain v5)** | **0.537** | **~0.19** | **~150** |
| **QRES v8.0 (MetaBrain v4)** | 0.537 | ~0.19 | ~150 |
| Zstd (L19) | 0.12 | 0.35 | 25 |

---

## 🛠️ Installation & Usage

### 1. Python API
```bash
pip install qres  # PyPI package (update to v8.0 for MetaBrain support)
```
```python
import qres
# Load trained MetaBrain for prediction (handled automatically if file present)
compressed = qres.encode_bytes(data, mode="standard", metabrain=qres.load_metabrain("ai/metabrain_ppo_v4.zip"))
```

### 2. Quantum CLI
```bash
python qres_quantum_cli.py data/other/sample.pdf --mode quantum  # Triggers binary fallback
```

### 3. Training the MetaBrain
```bash
python ai/train_compression_ppo.py --data-dir data/ --timesteps 20000  # Use diverse dataset
```

### 4. Archive Studio (GUI)
Download from releases. Visualizes MetaBrain decisions and swarm.

---

## 📚 Documentation

Detailed documentation is located in the `docs/` directory:

- [**Roadmap**](docs/ROADMAP.md): Progress and vision.
- [**Whitepaper**](docs/WHITEPAPER.md): Theory and architecture.
- [**Benchmarks**](docs/BENCHMARKS.md): Performance data.
- [**P2P Implementation**](docs/guides/P2P_IMPLEMENTATION.md): Swarm details.
- [**Research Notes**](docs/RESEARCH_NOTES.md): Citations.
- [**Contributing**](docs/CONTRIBUTING.md): Guidelines.

---

## 🔬 Research & Citations
See [RESEARCH_NOTES.md](docs/RESEARCH_NOTES.md) for details.
- **LLM Compression:** *Delétang et al. (2024)*.
- **Reinforcement Learning:** *Engstrom et al. (2021)* (PPO Implementation).

---

**License:** Apache 2.0 (see [LICENSE](LICENSE))

*Designed by Cavin Krenik & Contributors.*
