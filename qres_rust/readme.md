# QRES Rust Workspace (v10.1)

> **The Singularity Engine**: A bit-perfect, deterministic compression architecture split into a high-performance core and a biological-inspired swarm daemon.

[![Cross-Arch Battle](https://github.com/CavinKrenik/QRES/actions/workflows/cross_arch_battle.yml/badge.svg)](https://github.com/CavinKrenik/QRES/actions)
[![Crates.io](https://img.shields.io/crates/v/qres_core.svg)](https://crates.io/crates/qres_core)

## 🏗️ Architecture

This workspace contains the two pillars of QRES:

### 1. `qres_core` (The Codec)
* **Role:** Deterministic compression library.
* **Tech:** Pure Rust, `no_std` compatible, Q16.16 Fixed-Point Arithmetic.
* **Guarantee:** Compressing `file.dat` on Linux x86 produces the *exact same bitstream* as on macOS ARM64.
* **Usage:** Embeddable in C++, Python, WASM, and Embedded systems.

### 2. `qres_daemon` (The Brain)
* **Role:** P2P Swarm Node & CLI.
* **Tech:** Tokio (Async), Libp2p (GossipSub), SNN (Spiking Neural Networks).
* **Function:** Manages the "Hive Mind," distributing weight epiphanies across the network without blocking the hot compression path.

## 🚀 Quick Start

### Build Everything
```bash
# Build Daemon and Core (Release Optimized)
cargo build --release --workspace
```

### Run the Daemon
```bash
# Start the node (Binds to 127.0.0.1 for security)
./target/release/qres_daemon swarm --port 8080
```

### Use the CLI
```bash
# Compress using the Living Brain
./target/release/qres_daemon compress input.dat output.qres

# Decompress (Bit-Perfect)
./target/release/qres_daemon decompress output.qres restored.dat
```

## 🧪 Testing & Verification
We use a "Battle Royale" CI pipeline to ensure cross-architecture determinism.

```bash
# Run local test suite
cargo test --workspace

# Verify WASM compilation (for Web Clients)
cd qres_wasm && wasm-pack build --target web
```