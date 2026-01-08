# QRES Changelog

All notable changes to this project are documented here.

---

## [13.0.0] - 2026-01-08 "Security Hardening"

### Added
- **Phase 1 Security - Authentication:**
  - `security.rs`: ed25519 signing/verification with replay prevention (nonces + timestamps)
  - `peer_keys.rs`: PeerKeyStore with libp2p Identify protocol integration
  - Signed brain broadcasts and verified receives in `swarm_p2p.rs`
  - Config options: `require_signatures`, `key_path`, `trusted_peers`, `trusted_pubkeys`

- **Phase 2 Security - Robust Aggregation:**
  - `aggregation.rs`: Krum, Multi-Krum, Trimmed Mean, Median algorithms
  - `brain_aggregator.rs`: Buffered updates with Byzantine-tolerant aggregation
  - Config options: `aggregation.mode`, `expected_byzantines_fraction`, `buffer_size`

### Changed
- Brain updates now buffer before aggregation (configurable via `buffer_size`)
- Deltas apply immediately; Full brains wait for Krum/Median aggregation

### Security
- Unsigned messages rejected when `require_signatures = true`
- Outlier updates rejected by Krum algorithm (defends against poisoning attacks)

---

## [10.1.0] - 2026-01-05 "The Hybrid Era"

### Added
- **WebAssembly Core**: `qres_core` now compiles to WASM for browser-side compression.
- **Hybrid Studio**: QRES Studio now features a runtime toggle (Native Daemon vs WASM).
- **Security Hardening**: CI/CD pipeline now includes secret-signed releases for Tauri v2.

### Changed
- **Architecture**: Strict separation between `qres_core` (no_std) and `qres_daemon` (Tokio).
- **Docs**: Comprehensive update to `ROADMAP.md` and component READMEs.

---

## [10.0.0] - 2026-01-04 "Engineering Hardening"

### Critical Architecture Changes
- **Workspace Split**: `qres_rust` is now a workspace with two crates: `qres_core` (pure codec) and `qres_daemon` (brain/swarm node).
- **Delta Gossip**: Swarm P2P now uses delta encoding for efficient model updates.
- **Fixed-Point Arithmetic**: Predictor weights now use Q16.16 i32 format for bit-perfect cross-arch deterministic compression.

### Added
- **Cross-Arch CI**: "Battle Royale" workflow verifies Linux/x86 compression matches macOS/ARM decompression.
- **Python Bindings**: Updated to PyO3 0.22 with `abi3` support for Python 3.8+ compatibility.

### Changed
- **CLI**: The binary is now `qres_daemon` (or use `qres` Python wrapper).
- **Python-Rust Bridge**: Refactored to support safe threading and clearer API mapping.

### Removed
- **Bloat**: Removed direct dependencies on `libp2p` from the core compression path.

---

## [9.0.0] - 2026-01-04 "Singularity Brain"

### Added
- **GIF Neurons**: Generalized Integrate-and-Fire from SpikeLLM (ICLR 2025)
- **OSBC Pruning**: Second-order pruning achieving 97% sparsity
- **Equivariant Tensor Network**: Symmetry-preserving lattice compression
- **Auto-Tuning**: Fine-tune MetaBrain on user data (`auto_tune.py`)
- **Research Citations**: 2025 papers for SNN/QML advances

### Changed
- Upgraded SNN predictor from LIF to GIF neurons
- Enhanced tensor predictor with equivariant lattice method

---

## [8.1.0] - 2026-01-04 "Brain-Neural ML"

### Added
- **Spiking Neural Networks**: `snn_predictor.py` with LIF neurons
- **Tensor Predictor**: `tensor_predictor.py` with correlation detection
- **Hive Mind**: `hive_mind.py` with FedProx and KL-FedDis
- **MetaBrain v5**: SNN + Tensor hybrid (261-dim observations)
- **Swarm CLI**: `swarm_cli.py` with Fed2Com delta compression
- **Demo Notebook**: `examples/brain_demo.ipynb`

---

## [8.0.0] - 2026-01-02 "AEON Update"

### Added
- **MetaBrain v4**: Multimodal training (IoT, text, images, PDFs, audio)
- **WorldStateManager**: Graph + tensor + neural persistence
- **Fidelity Verification**: `verify_fidelity.py` (>0.98 threshold)
- **CLIP Embeddings**: Multimodal search support

### Performance
- IoT Ratio: 0.537
- Text Ratio: ~0.19
- Swarm: ~500 FPS training

---

## [7.5.0] - 2025-12-30 "Tensor Foundations"

### Added
- Tensor Network Compression (`TensorEncoder`)
- Haar Wavelet Transform for MPS
- Tensor Mode CLI (`qres_tensor_cli.py`)

---

## [7.0.0] - 2025-12-28 "Strategic Enhancements"

### Added
- MultiModal Memory (NetworkX + CLIP)
- PPO Agent (Gymnasium)
- QRES Studio GUI with D3 Knowledge Graph

---

*For full history, see git log.*
