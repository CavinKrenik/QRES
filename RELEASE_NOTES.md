# QRES v5.0.5 Release Notes - "Singularity Swarm"
**Date**: January 1, 2026

## 🚀 Major Features

### 1. Decentralized Intelligence Swarm (Phase 3)
The centralized Python Hive server has been replaced with a high-performance **Rust P2P Swarm** powered by `libp2p`.
- **True Peer-to-Peer**: Nodes discover each other via mDNS (local) and GossipSub (mesh).
- **FedProx-Lite Sync**: Brain confidence scores are aggregated using statistical weighting to prevent catastrophic forgetting.
- **Observability API**: Every node exposes a REST API (`/status`, `/brain`) for real-time monitoring.
- **Zero-Config**: Just run `qres-cli swarm` and the swarm auto-assembles.

### 2. Neural Meta-Brain (Phase 2)
The compression engine now features a **Neural Meta-Brain** (MLP) trained to predict optimal initial mixer weights.
- **Zero-Shot Adaptation**: Instantly selects the best weights (e.g. 80% Graph, 20% Linear) for a new file type based on its statistical fingerprint.
- **Flag 0x02**: New `.qres` header flag ensures the decoder initializes with the *exact* neural weights used by the encoder.

### 3. SIMD Mixer & Graph Predictor
- **AVX2/NEON Acceleration**: The mixing layer and graph lookup tables are now vectorized, boosting throughput >40% on supported hardware.
- **Smart Pre-Pass**: A lightweight header analysis step prevents expensive model loading for simple files.

## 🛠️ Technical Improvements
- **Tokio Runtime**: Migrated CLI and Swarm to async Rust for massive concurrency.
- **Header Safety**: Length-prefixing logic moved to `compress_chunk` to prevent OOM panic in Python bindings.
- **Dependencies**: Added `axum`, `libp2p`, `tokio`.

## 📦 Breaking Changes
- **CLI**: `swarm` command arguments changed. Now supports `--brain` and `--port`.
- **.qres Format**: Added flag `0x02` for Neural Initialization. Older decoders may not support files compressed with this flag.

## 🔮 What's Next?
- **Global DHT**: Expanding discovery beyond local networks.
- **WASM Build**: Compiling the P2P node to run in the browser.
