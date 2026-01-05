# QRES v10.0.0: The Engineering Release

> **Hardened. Modular. Production-Ready.**

This milestone release transforms QRES from an experimental research project into a rigorous engineering standard. It introduces a modular workspace architecture, bit-perfect deterministic compression across architectures (Linux/macOS/Windows), and optimized P2P protocols.

## 🌟 Major Highlights

### 🛡️ Bit-Perfect Determinism (Cross-Arch)
- **The "Butterfly Effect" Fixed:** Replaced floating-point weights (`f32`) with **Q16.16 Fixed-Point Arithmetic** (`i32`).
- **Guarantee:** A file compressed on an Intel Linux server will decompress *byte-for-byte identically* on an Apple Silicon MacBook. Verified by our new "Battle Royale" CI pipeline.

### 🧩 Modular Architecture
- **Workspace Split:** The monolithic `qres_rust` crate has been split:
  - `qres_core`: A pure, lightweight library for compression/decompression (no heavy dependencies).
  - `qres_daemon`: The full "Living Brain" application with P2P networking and AI training.
- **Benefit:** Developers can now embed the QRES codec into other apps without pulling in the entire Swarm stack.

### ⚡ Delta-Gossip Protocol
- **Bandwidth Optimization:** The P2P Swarm now uses **Delta Encoding** for model updates.
- **Efficiency:** Instead of broadcasting the entire 50MB neural brain every few seconds, nodes only transmit the "Epiphanies" (weights that changed significantly), reducing network traffic by >90%.

### � Modern Python Bindings
- **PyO3 0.22 Upgrade:** Bindings now use the latest PyO3 API with `abi3` support.
- **Compatibility:** Native wheels now support Python 3.8 through 3.12+ seamlessly.

## 📦 Assets
- `qres_daemon` (CLI & Swarm Node)
- `qres` (Python Codec Package)
- `qres-studio` (GUI Dashboard)

## 🛠 Usage
```bash
# Codec (Library)
cargo add qres_core

# Full Application
cargo install --path qres_rust/qres_daemon
```
