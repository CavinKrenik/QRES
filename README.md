# QRES: Quantum-Relational Encoding System 🧠⚛️

> **Revolutionary compression for the Singularity Era**: Brain-like spiking neural networks with quantum-inspired ML and distributed P2P swarms.

[![License](https://img.shields.io/badge/license-Apache%202.0-blue)](LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/CavinKrenik/QRES/release.yml?style=flat)](https://github.com/CavinKrenik/QRES/actions)
[![Version](https://img.shields.io/badge/version-v10.1.0-brightgreen)](https://github.com/CavinKrenik/QRES/releases)
[![Python](https://img.shields.io/badge/python-3.10+-blue)](https://python.org)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange)](https://rust-lang.org)
[![WASM Ready](https://img.shields.io/badge/target-wasm32-blueviolet)](https://github.com/CavinKrenik/QRES)
[![no_std](https://img.shields.io/badge/std-optional-blue)](https://docs.rust-embedded.org/book/intro/no-std.html)

## 🔧 Build Targets
QRES v10.5+ supports diverse build targets:
* **x86_64 / ARM64:** Full `std` support (Daemon + Core)
* **wasm32-unknown-unknown:** Client-side browser compression (Core only)
* **thumbv7em-none-eabihf:** Embedded/Bare-metal ready (Core only)

### 🌍 WebAssembly Build (Client-Side)
To compile the `qres_wasm` bridge for browser usage:

```bash
# 1. Install wasm-pack
cargo install wasm-pack

# 2. Build the WASM package
cd qres_rust/qres_wasm
wasm-pack build --target web
```
*Artifacts will be output to `qres_rust/qres_wasm/pkg` (includes .wasm binary and .js glue code).*

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
│  Input Data  →  [SNN Predictor]  →  [TNC Fusion]  →  Output │
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

## 🎯 Real-World Use Cases

### 📡 Ultra-Low Bandwidth IoT
Sensors often produce repetitive signals (sine waves, timestamps). QRES's **Lock-On Mixer** achieves **~19% compression ratio** on these streams, allowing edge devices to transmit **5x more data** on the same bandwidth budget.

### 🏛️ Bit-Perfect Archival
Unlike floating-point based compressors which can drift across architectures, QRES uses **Q16.16 Fixed-Point Arithmetic**. A medical record or scientific dataset compressed today is guaranteed to decompress **bit-perfectly** on any future hardware (RISC-V, ARM, x86).

### 🔒 Privacy-Preserving Analytics
QRES allows systems to share "compression intelligence" (models) without sharing data. This enables **Zero-Knowledge Federated Learning**, where secure institutions can collaborate on data efficiency models without ever exposing sensitive records.

---

## 📊 Performance

| Dataset | Ratio | Fidelity | Speed |
|---------|-------|----------|-------|
| **Binary Telemetry** | **~0.15** | 1.00 | 300+ MB/s |
| **IoT (Correlated CSV)** | ~0.76 | >0.99 | 150 MB/s |
| **Text/Code** | ~0.19 | 1.00 | 200 MB/s |
| **Sine Wave** | **~0.19** | 1.00 | 250 MB/s |

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

**Start the Daemon (Secure by Default):**
```bash
# Binds to 127.0.0.1
qres-daemon start
```

**Start for External Access (e.g., in Docker/P2P):**
```bash
# Binds to 0.0.0.0 (Warning: Publicly accessible)
QRES_PUBLIC=1 qres-daemon start
```

**Python Client:**

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
├── .github/               # CI/CD Workflows
├── ai/                    # Neural networks (SNN, QNN, Hive Mind)
├── assets/                # Design assets & diagrams
├── benchmarks/            # Performance evaluation scripts
├── data/                  # Sample datasets & telemetry
├── docs/                  # Documentation (Technical & Vision)
├── examples/              # Usage examples & notebooks
├── python/qres/           # Python API & core bindings
├── qres_rust/             # Rust Workspace
│   ├── qres_core/         # High-performance compression library
│   └── qres_daemon/       # P2P Node & background service
├── qres-studio/           # Svelte/Tauri Desktop Application
├── tests/                 # Integration & unit tests
└── utils/                 # Development utilities
```

---

## 📖 Documentation

| Document | Description |
|----------|-------------|
| [VISION.md](docs/VISION.md) | **Product Strategy & Vision** |
| [WHITEPAPER.md](docs/WHITEPAPER.md) | Technical deep-dive |
| [ROADMAP.md](docs/ROADMAP.md) | Development phases |
| [BENCHMARKS.md](docs/BENCHMARKS.md) | Performance metrics |
| [RESEARCH_NOTES.md](docs/RESEARCH_NOTES.md) | Academic citations |

---

## 🗺️ Roadmap & Status

- ✅ **v8.0 - v10.0** – (Legacy milestones completed)
- ✅ **v10.1** – Security Hardening & Workspace Split
- 🚀 **v10.5 (Current)** – The Hybrid Era
    - [x] **FPGA Prep:** `no_std` Refactor for `qres_core`.
    - [x] **WebAssembly:** Client-side compression in QRES Studio.
    - [x] **Hybrid Runtime:** Native/WASM toggling in GUI.

---

## 🏗️ Engineering Roadmap (v10.5 - Hardware Era)

The focus shifts from software architecture to hardware acceleration and edge deployment.

### 🏎️ Phase 1: FPGA Acceleration (Active)
> **Goal:** Offload the `SNN Predictor` and `Mixer` to FPGA logic for microsecond latency.

- [x] **`no_std` Refactor:** Decouple `qres_core` from standard library for embedded/FPGA usage.
- [ ] **Hardware Description:** Port `Mixer` logic to Verilog/HLS.
- [ ] **Driver Layer:** Create DMA bridge between Rust Daemon and FPGA Core.

### 🌐 Phase 2: WebAssembly Core (COMPLETED)
> **Goal:** Run QRES entirely in the browser for client-side compression.

- [x] **WASM Target:** Ensure `qres_core` compiles to `wasm32-unknown-unknown`.
- [x] **JS Bindings:** `wasm-bindgen` interface for TypeScript Studio.
- [ ] **Browser Persistence:** Adapt `WorldStateManager` to use `IndexedDB` instead of file system.
