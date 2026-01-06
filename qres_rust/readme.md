# QRES Rust Workspace (v10.1)

> **The Singularity Engine**: A bit-perfect, deterministic compression architecture split into a high-performance core and a biological-inspired swarm daemon.

[![Crates.io](https://img.shields.io/crates/v/qres_core.svg)](https://crates.io/crates/qres_core)
[![WASM Ready](https://img.shields.io/badge/target-wasm32-blueviolet)](https://github.com/CavinKrenik/QRES)

## 🏗️ Architecture

This workspace contains the two pillars of QRES:

### 1. `qres_core` (The Codec)
* **Role:** Deterministic compression library.
* **Tech:** Pure Rust, `no_std` compatible, Q16.16 Fixed-Point Arithmetic.
* **Guarantee:** Compressing `file.dat` on Linux x86 produces the *exact same bitstream* as on macOS ARM64.
* **Targets:** Embedded, WASM (Browser), Desktop, Server.

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

### Build for Web (WASM)
```bash
# Generates the 'pkg' folder for qres-studio
cd qres_wasm && wasm-pack build --target web
```