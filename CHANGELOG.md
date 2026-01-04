# Changelog

All notable changes to the QRES project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Fixed
- **CI/CD:** Resolved `qres_rust` import errors in continuous integration pipelines (bcfe3c6).
- **Paths:** Fixed various file path issues for cross-platform compatibility.

## [7.5.0-alpha] - 2026-01-03
### Added
- **Quantum Tensor Compression:** `MpsCompressor` (Haar Wavelet) backend for structured data (`qres_rust::quantum`).
- **GPU Acceleration:** Added `wgpu` dependency and foundational bindings for hardware compute.
- **Improved Performance:**
  - IoT Telemetry: **51% Ratio** (Beats Zstd 57%) using Smart Interleave Detection (`0x03` flag).
  - Structured Data: **64% Ratio** (Beats Zstd 95%) on correlated floating-point matrices.
- **Python Bindings:** New `compress_matrix_v1` for tensor compression.

### Changed
- **Predictors:** Optimizations for `SpectralPredictor` (Linear Detrending) and `TransformerPredictor` (AVX2, 4KB window).
- **Benchmarks:** Validated against Zstd on IoT and Structured data.

## [6.0.0-alpha] - 2026-01-02
### Added
- **GUI Revamp (Starship/AEON):** Complete overhaul of QRES Studio using Tauri v2 and Svelte 5.
  - Reactive state management for real-time telemetry.
  - New "SwarmDashboard" and interactive controls.
  - Futuristic "SpaceX-inspired" aesthetic.
- **Persistent World Compression (Phase 4):**
  - `WorldStateManager` for serializing graph + tensor + neural states.
  - Distributed State Sync logic to broadcast states to the swarm.
  - Fidelity verification (>0.98 threshold).
- **Quantum Integration:**
  - `qres_quantum_cli.py`: CLI for quantum tensor network compression.
  - `qres_quantum_receiver.py`: P2P listener for quantum broadcasting.
  - WAN Bootstrap logic (Kademlia/Server Mode).
- **LLM Semantic Predictor:** Production-ready Transformers integration (`python/qres/llm_predictor.py`).
- **GPU Compute Pipeline:** `wgpu` framework for hardware-accelerated mixing (`qres_rust/src/gpu.rs`).
- **Research Docs:** Added `docs/RESEARCH_NOTES.md` with academic basis.
- **Fuzzing:** Added `qres_rust/fuzz/` infrastructure.

### Changed
- **README:** Updated with v6/v7/v8 features and status.
- **API:** Documented `SemanticPredictor` and `WorldStateManager`.
- **CI:** Added optional LLM predictor tests.

## [5.1.0] - 2026-01-01
### Added
- **Telepathy Engine:** "Living Brain" predictor selection (Linear/Simple/Graph/Spectral).
- **Solid Archives:** `.qrar` format with deduplication.
- **Native Swarm:** Rust `libp2p` implementation (GossipSub).
- **SIMD Optimization:** AVX2/NEON vectorization.

### Changed
- **CLI:** Renamed `qres create` to `qres archive`.
- **Docs:** Consolidated into `docs/` directory.

### Fixed
- **Performance:** Fixed text compression regression.
- **Stability:** Fixed archive doctests.

## [5.0.0] - 2025-12-30
- Initial "Singularity" architecture release.
- Introduction of `MetaBrain` and Neural Predictors.

## [1.0.0] - 2024-01-01
- Initial core engine release.
