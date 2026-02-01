# QRES Changelog

All notable changes to this project are documented here.

---

## [v19.0.0] - 2026-02-01 "The Immune System II"

### Adversarial Hardening (Phase 2 & 3)
- **Trimmed Mean Aggregator**: Implemented `TrimmedMeanByz` in `qres_core` to neutralize "Inlier Bias" attacks (Drift < 0.05% verified). Replaces Krum as the primary defense against sophisticated drift.
- **BFP-16 Precision**: Introduced `Bfp16Vec` (Block Floating Point) for gradient headers. 
    - Solved "Vanishing Gradient" problem at low learning rates ($10^{-5}$).
    - Maintains `f32` dynamic range with `i16` storage density.
- **Summary Gene Protocol**: 
    - Implemented "Mid-Flight Join" capability using compact 74-byte Summary Genes.
    - Achieves **2,133:1 compression ratio** vs full event log replay.
    - Packet loss resilience verified (recovers state via summary gene).

### Documentation
- Added `docs/adrs/ADR-004-adversarial-hardening.md`.
- Updated `Attack.md` with final golden run results.

### Changed
- Rolled version string to `19.0.0` across all crates and bindings.

## [v18.0.0] - 2026-01-15

### Added
- **Neural Swarm Simulator:** `tools/swarm_sim` with Bloom/HDR visuals.
- **Persistence Layer:** `GeneStorage` trait for Lamarckian evolution.
- **Active Neurons:** Refactored `Predictor` into `SwarmNeuron`.

### Changed
- Pivoted primary architecture from Compression Library to Distributed OS.
- `qres_core` is now strictly `no_std` by default.

## [16.0.0] - 2026-01-14

### Changed
- **Refactor**: Reorganized repository into a standard monorepo structure:
    - `crates/` (formerly `qres_rust/*`) - Rust backend services and libraries.
    - `web/` (formerly `qres-studio`) - Real-time dashboard and frontend.
    - `bindings/` (formerly `python/`) - Python bindings and wrappers.
    - `evaluation/` - Benchmarks and reproducibility scripts.
    - `research/` - Neural experiments and notebooks.
- **Documentation**: Updated README architecture diagrams and removed legacy roadmaps.
- **Paper**: Finalized compilation of `paper/paper.pdf` with corrected citations.

### Security & Performance
- **Safety**: Removed all panic paths (`unwrap`, `expect`) from the `no_std` core.
- **Zero-Copy**: `compress_chunk` now requires a pre-allocated `&mut [u8]` buffer.
- **Determinism**: Replaced floating-point math with `fixed::types::I16F16`.

### Fixed
- **Link Explosion**: P2P sync now uses Deterministic Seed Sync (8 KB/day).
- **Expansion Problem**: Hybrid Gatekeeper falls back to bit-packing on high entropy.

## [15.4.0] - 2026-01-11

### ✨ New Features
- **Weather Replay Engine:** Hardware-in-the-Loop simulation with Jena Climate Dataset.
- **Hive Mind Visualization:** Infinite canvas, node inspector HUD, and gradient packets.
- **Neural Graph:** 5-layer deep network visualization with live spike propagation.

### 🔧 Improvements
- **UI/UX:** Streamlined sidebar, single connect button, and no-scroll layout.
- **Architecture:** simulated compression for browser mode (no WASM required).

### 📝 Documentation
- **README:** Added "Hardware-in-the-Loop Simulation" section.

---

## [15.2.0] - 2026-01-08

### Added
- **Documentation:**
  - `THEORY.md`: Privacy composition, Byzantine tolerance proofs, convergence analysis
  - `RELATED_WORK.md`: 30+ citations, FL framework comparison table
  - `references.bib`: BibTeX for paper submission
  - ADRs: SNN vs ANN, ed25519 vs Dilithium, PRNG Sync

- **Benchmarks:**
  - Baseline comparisons (FedAvg/FedProx vs QRES)
  - Scalability analysis (10-100 nodes)
  - Long-term stability (24hr tests)
  - Energy consumption estimates

- **Reproducibility:**
  - Docker environment (`Dockerfile.qres`, `docker-compose.yml`)
  - Benchmark scripts (`run_all_benchmarks.sh`)
  - Paper figure generation (`generate_paper_plots.py`)

- **Code:**
  - `Aggregator` trait for pluggable aggregation strategies

### Changed
- `PAPER_DRAFT.md`: Full structure for paper submission

---

## [15.0.0] - 2026-01-08 "Privacy"

### Added
- **Phase 3 Security - Privacy:**
  - `privacy.rs`: Differential Privacy with Gaussian mechanism (Box-Muller fallback for no_std)
  - `secure_agg.rs`: Pairwise masking via X25519 ECDH + ChaCha20 RNG
  - `zk_proofs.rs`: Pedersen Commitments + Proof of Norm via EdwardsPoint
  - Config options: `privacy.enabled`, `epsilon`, `delta`, `clipping_threshold`, `secure_aggregation`

### Security
- Model updates now have provable privacy (ε-DP) and aggregation masking
- Outlier rejection + norm proofs defend against poisoning attacks

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

## [10.1.0] - 2026-01-05

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
