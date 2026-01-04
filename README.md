# QRES: The Singularity Engine

[![Latest Release](https://img.shields.io/github/v/tag/CavinKrenik/QRES?include_prereleases&style=flat-square&color=blue&label=release)](https://github.com/CavinKrenik/QRES/releases)
[![Build Status](https://img.shields.io/github/actions/workflow/status/CavinKrenik/QRES/test.yml?style=flat-square)](https://github.com/CavinKrenik/QRES/actions)
[![License](https://img.shields.io/badge/license-Apache--2.0-green?style=flat-square)](LICENSE)

**QRES (Quantum-Relational Encoding System)** is a next-generation compression platform designed for the "Singularity Era" of data. It moves beyond static algorithms by using **Neural-Symbolic Telepathy**—a hybrid approach where an autonomous "Living Brain" dynamically selects the optimal compression strategy for every micro-chunk of data.

> "Data is not static; your compressor shouldn't be either."

**Topics:** ai-compression, neural-symbolic, quantum-inspired, rust, python, p2p-swarm, reinforcement-learning

---

## 🚀 Key Features

### 🧠 Cognitive Context & The "Living Brain"
- **Neural-Symbolic Hybrid:** Combines raw speed (Linear) with deep pattern recognition (Graph, Spectral, LSTM).
- **LLM Semantic Prediction (v6):** Uses Transformer-based models (CodeLlama/GPT) to predict text and code streams with unprecedented accuracy.
- **Reinforcement Learning (v7):** A PPO Agent (Gymnasium) autonomously learns the optimal compression strategy for your specific data type.

### ⚛️ Quantum-Inspired Tensors (v7/v8)
- **Tensor Networks:** Simulates quantum states to represent complex data relationships in high-dimensional space.
- **Noise Simulation:** Validated against QuTiP noise models for robustness.
- **Ethical Pruning:** Automatically detects and mitigates bias in the learned representation.

### 🐝 Swarm Intelligence (P2P)
- **Hive Mind:** Nodes form a Kademlia DHT network to share learned models ("Epiphanies") via GossipSub.
- **Persistent World State:** Synchronize your compressed "World State" across devices with >0.98 fidelity.
- **Distributed Learning:** Train on edge devices and broadcast the wisdom to the swarm.

### 📦 QRAR Archives
- **Deduplication:** Content-Defined Chunking (CDC) eliminates redundancy across petabytes of data.
- **Solid Compression:** Archive-aware optimization for maximum density.

---

## 📊 Current Status

| Version | Status | Focus |
| :--- | :--- | :--- |
| **v6.0.0-alpha** | **Stable** | AI Foundation (LLM, GPU, Starship GUI) |
| **v7.5** | **Preview** | Quantum Foundations |
| **v8.0** | **Released** | **The AEON Update:** Living Brain, Swarm Persistence, PPO Agent |

---

## ⚡ Performance

See [BENCHMARKS.md](docs/BENCHMARKS.md) for full analysis.

| Engine | Ratio (IoT) | Ratio (Text) | Speed (MB/s) |
| :--- | :---: | :---: | :---: |
| **QRES v7.0** | **~0.048*** | **~0.19*** | **~150** |
| **QRES v6.0α** | **0.07** | **0.29** | **180** |
| Zstd (L19) | 0.12 | 0.35 | 25 |

---

## 🛠️ Installation & Usage

### 1. Python API
The easiest way to integrate QRES into your AI workflows. *PyPI package coming soon.*

```bash
pip install qres
```

```python
import qres
# Use the Semantic Predictor (LLM)
from qres.llm_predictor import SemanticPredictor

predictor = SemanticPredictor(model="codellama-7b")
data = b"def fibonacci(n): ..."
compressed = qres.encode_bytes(data, predictor=predictor)
```

### 2. Quantum CLI (v8.2 Preview)
Interact with the research-grade Quantum core.

```bash
# Compress with Quantum Tensor Networks (QuTiP Backend)
python qres_quantum_cli.py data.txt --mode quantum --save-state

# Optimize Neural Weights via and AQC Simulation
python qres_quantum_cli.py --optimize

# Start Quantum Swarm Receiver
python qres_quantum_receiver.py --dir ./inbox
```

### 3. Archive Studio (GUI)
A premium, "AEON" branded interface. Built with **Svelte 5** and **Tauri v2**.
Visualizes the **Living Brain** decision graph and Swarm topology in real-time.

![QRES Studio AEON](docs/screenshots/aeon_studio_preview.png)
*(Note: Screenshot is a placeholder for the v6 Starship revamp)*

- [Download Latest Release](https://github.com/CavinKrenik/QRES/releases)

---

## 📚 Documentation
Detailed documentation is located in the `docs/` directory:

- [**Roadmap**](docs/ROADMAP.md): Tracking v6, v7, and v8 progress.
- [**Whitepaper**](docs/WHITEPAPER.md): The theory (Telepathy, Swarm, Deduplication).
- [**Benchmarks**](docs/BENCHMARKS.md): Performance data.
- [**P2P Implementation**](docs/guides/P2P_IMPLEMENTATION.md): How the Swarm works.
- [**Research Notes**](docs/RESEARCH_NOTES.md): Academic citations.
- [**Contributing**](docs/CONTRIBUTING.md): Join the Hive.

---

## 🔬 Research & Citations
QRES evolves by implementing cutting-edge compression theory.
See [RESEARCH_NOTES.md](docs/RESEARCH_NOTES.md) for details.

- **LLM Compression:** *Delétang et al. (2024)*.
- **Linear Attention:** *Katharopoulos et al. (2020)*.
- **Swarm FedProx:** *Li et al. (2018)*.

---

**License:** Apache 2.0 (see [LICENSE](LICENSE))

*Designed by Cavin Krenik & The QRES Team.*
