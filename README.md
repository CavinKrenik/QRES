# QRES v5.1: The Singularity Engine

*(Dedicated to the pursuit of the Singularity)*

[![Release](https://img.shields.io/github/v/release/CavinKrenik/QRES)](https://github.com/CavinKrenik/QRES/releases)
[![Build Status](https://github.com/CavinKrenik/QRES/actions/workflows/test.yml/badge.svg)](https://github.com/CavinKrenik/QRES/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)

**QRES (Quantum-Relational Encoding System)** is a cognitive compression framework. It treats compression not as a statistical problem, but as an **intelligence problem**.

**New in v5.1 (Archive Edition):**
* **🗜️ True Archiving**: WinZip/7-Zip-style container format with solid compression
* **🔍 Global Deduplication**: Content-Defined Chunking detects duplicates across the entire archive
* **🧠 Logistic Mixing**: Neural-style probability mixing for better predictions
* **✅ Critical Fixes**: Resolved decompression crashes and memory exhaustion issues
* **📦 Manifest System**: Browse archives without extraction, verify integrity with Blake3

**v5.0 (Singularity Update):**
* **Context Engine**: LzMatchPredictor brings LZ77-style string matching, crushing Zstd on text/code.
* **SIMD Acceleration**: 128-bit Vectorized Mixing for >500MB/s throughput.
* **True P2P**: Decentralized libp2p swarm (removed Python server dependency).
* **Hex Battle View**: Visualizing algorithm dominance in real-time.

---

##  Key Features (v5.0)

###  The "Singularity" Ensemble
QRES v5 uses a **Content-Aware** pre-pass to select the optimal cognitive strategy:

1.  **LzMatch (Context)**: Uses a 64KB sliding window and Hash-Chain matching to instantly predict repeated strings.
2.  **Spectral (FFT)**: 2048-point harmonic detection for waveforms and signals.
3.  **Graph (DAG)**: Learns byte-aligned structures in telemetry/logs.
4.  **Linear**: Baseline predictor for high-entropy streams.

###  SIMD Core
The new Mixer utilizes **AVX2/NEON** intrinsics to process 8 bytes in parallel, performing dot-product mixing and weight updates in a single CPU cycle.

###  Decentralized Swarm
* **LibP2P**: Nodes form a DHT (Distributed Hash Table) to share "Brain Weights."
* **GossipSub**: New file types are learned globally within seconds of detection.
* **Zero-Shot**: No central server. Pure distributed intelligence.

---

##  Benchmarks (v5.1 vs Zstd)
See full report: [BENCHMARK_v5.md](BENCHMARK_v5.md)

| Dataset | QRES Ratio | Zstd Ratio | Advantage |
| :--- | :--- | :--- | :--- |
| **Sine Wave** | **46.0%** | 100.0% | +54.0% |
| **JSON Logs** | **88.6%** | 100.0% | +11.4% |
| **C Source** | **93.6%** | 100.0% | +6.4% |
| **Binary** | 100.0% | 100.0% | Tie |

---

##  Installation

### Python
`ash
pip install qres
`
### Rust CLI
`ash
cargo install --path qres_rust
`
### QRES Studio (GUI)
`ash
cd qres-studio
npm install
npm run tauri dev
`

---

##  Usage

### CLI
```bash
# Compress with visualizer
qres-cli compress -i input.dat -o output.qres --visualize

# Decompress a file
qres-cli decompress -i output.qres -o restored.dat
```

### 🐝 Swarm Mode (P2P)
Run a decentralized node to sync compression intelligence with other peers.
```bash
qres-cli swarm --brain ./my_brain.json --port 8080
```
- **Discovery**: Automatically finds peers on the local network via mDNS.
- **Sync**: Periodically broadcasts and merges brain weights using FedProx-lite.
- **API**: Exposes a REST API for monitoring.
  - `GET /status`: Peer connectivity stats.
  - `GET /brain`: Current neural confidence scores.

### Python API
`python
import qres
# Encode with automatic content detection
compressed = qres.encode_bytes(data, level=5)
```

### 🗃️ Archive Format (.qrar)
QRES v5.1 introduces true archiving with solid compression:

```bash
# Create archive from directory
qres-cli archive create ./my_project -o my_project.qrar

# Browse archive contents (no extraction)
qres-cli archive list my_project.qrar

# Extract specific file
qres-cli archive extract my_project.qrar src/main.rs -o ./extracted/

# Extract all
qres-cli archive extract my_project.qrar -o ./restored_project/
```

**Archive Features:**
- **Solid Compression**: All files concatenated before compression for better ratios
- **Content Deduplication**: Detects duplicate chunks across the entire archive
- **Integrity Verification**: Blake3 hashing for each file
- **Metadata Preservation**: Permissions, timestamps, file paths
- **Partial Extraction**: Extract individual files without decompressing everything

**File Extensions:**
- `.qres` - Individual compressed file
- `.qrar` - QRES Archive (multiple files with manifest)

**When to use archives:**
- Source code projects (20-50% better compression)
- Document collections
- Backups with structure preservation
- Any directory where files share common patterns

### Browser (WASM)
See [docs/WASM_GUIDE.md](docs/WASM_GUIDE.md) for running QRES client-side.

---

##  Contributing

See CONTRIBUTING.md.

QRES v5.0 - Compression through Collective Intelligence 
