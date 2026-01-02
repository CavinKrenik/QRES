# QRES: The Singularity Engine

[![Release](https://img.shields.io/github/v/release/CavinKrenik/QRES?style=flat-square&color=blue)](https://github.com/CavinKrenik/QRES/releases)
[![Build Status](https://img.shields.io/github/actions/workflow/status/CavinKrenik/QRES/test.yml?style=flat-square)](https://github.com/CavinKrenik/QRES/actions)
[![License](https://img.shields.io/github/license/CavinKrenik/QRES?style=flat-square)](LICENSE)

**QRES (Quantum-Relational Encoding System)** is a next-generation compression platform designed for the "Singularity Era" of data. It moves beyond static algorithms like Deflate or Zstd by using **Neural-Symbolic Telepathy**—a hybrid approach where an autonomous "Living Brain" dynamically selects the optimal compression strategy for every micro-chunk of data.

> "Data is not static; your compressor shouldn't be either."

## 🚀 Key Features

### 🧠 Cognitive Context & The "Living Brain"
QRES does not just "compress"; it **understands**. 
- **Neural-Symbolic Hybrid**: Combines the raw speed of linear prediction with the pattern-recognition of distinct neural models (Simple, Graph, Spectral, LSTM).
- **Zero-Shot Adaptation**: The engine learns the data structure in real-time, adapting its internal weights to match the entropy of the stream instantly.

### 🐝 Swarm Intelligence (P2P)
Compression is no longer an isolated task.
- **Hive Mind**: Instances of QRES can synchronize their learned models across a private peer-to-peer swarm.
- **Distributed Learning**: When one node learns how to compress a specific log format effectively, the entire swarm gets smarter.

### 📦 Solid Archives (QRAR)
The new **QRAR** format brings full archival capabilities.
- **Deduplication Engine**: Integrated Content-Defined Chunking (CDC) eliminates duplicate data across terabytes of files.
- **Solid Compression**: Concatenates similar files to exploit cross-file redundancy.
- **"Telepathy" Browsing**: Browse, search, and extract individual files from massive solid archives without decompressing the whole stream.

## ⚡ Performance

Comparisons performed on the `IoT-Drift` and `Shakespeare` corpora.

| Engine | Ratio (IoT) | Ratio (Text) | Speed (MB/s) |
| :--- | :---: | :---: | :---: |
| **QRES v5.1** | **0.08** | **0.32** | **180** |
| Zstd (L19) | 0.12 | 0.35 | 25 |
| Gzip (L9) | 0.28 | 0.42 | 40 |
| LZ4 | 0.55 | 0.60 | 800 |

*QRES achieves higher compression density by sacrificing raw throughput for cognitive adaptation.*

## 💻 Requirements

- **CPU:** x86_64 with **AVX2** support, or ARM64 with **NEON**. The neural mixer relies heavily on SIMD vectorization.
- **OS:** Linux (Kernel 5.4+), macOS (12+), Windows 10+.
- **Software:** Rust 1.75+ (for building from source), Python 3.8+ (for API).

## 🛠️ Quick Start

### 1. The CLI (Rust)
Ideal for servers and pipelines.

```bash
# Compress a file
qres compress data.log --out data.qres

# Decompress
qres decompress data.qres

# Create an archive
qres archive --dir ./logs --out logs.qrar
```

### 2. Python API
Integrate into your AI workflows.

```bash
pip install qres
```

```python
import qres

data = b"..." * 1000
compressed = qres.encode_bytes(data)
original = qres.decode_bytes(compressed)
```

### 3. Archive Studio (GUI)
A beautiful, GPU-accelerated interface for desktop users.
*   **Drag & Drop**: Drop any file, folder, or archive.
*   **Visualizations**: Watch the "Living Brain" think in real-time.
*   [Download Latest Release](https://github.com/CavinKrenik/QRES/releases)

## 📚 Documentation

Detailed documentation has been consolidated into the `docs/` directory:
- **Python API:** `pip install qres` (Coming to PyPI). See [API Reference](docs/API_REFERENCE.md).
- [**Whitepaper**](docs/WHITEPAPER.md): The theory behind Neural-Symbolic Compression.
- [**Benchmarks**](docs/benchmarks/BENCHMARK_v5.md): Extensive performance analysis.
- [**P2P Guides**](docs/guides/P2P_IMPLEMENTATION.md): Setting up your own Swarm.
- [**Roadmap**](docs/ROADMAP.md): Future plans for Version 6.0 and beyond.
- [**Contributing**](docs/CONTRIBUTING.md): Join the Hive.

## 🔬 Research & Citations
QRES evolves by implementing cutting-edge compression theory.
- **LLM Compression:** Inspired by *Delétang et al. (2024)*.
- **Linear Attention:** Adapted from *Katharopoulos et al. (2020)*.
- **Swarm FedProx:** Based on *Li et al. (2018)*.

See [RESEARCH_NOTES.md](docs/RESEARCH_NOTES.md) for detailed bibliography and implementation strategies.

---

*Designed by Cavin Krenik & The QRES Team.*
