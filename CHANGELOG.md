# QRES v8.1 Changelog

## [8.1.0] - 2026-01-04 (Brain-Quantum ML Release)

### Added
- **Spiking Neural Networks (SNN)**: Implemented `ai/snn_predictor.py` with Leaky Integrate-and-Fire neurons for temporal, sparse data encoding (Breakthrough 1).
- **Quantum VQC Fusion**: Implemented `ai/qnn_vqc.py` with Variational Quantum Circuits for detecting entangled correlations (Breakthrough 2).
- **Hive Mind**: Implemented `ai/hive_mind.py` with Federated Averaging (FedProx) for P2P continual learning (Breakthrough 3).
- **MetaBrain v5**: Trained new hybrid SNN+QNN agent with 261-dimensional observation space.
- **Deep Dive Audit**: Added `docs/DEEP_DIVE_AUDIT.md` for v8.0 analysis and v8.1 planning.
- **Brain-Quantum Architecture Doc**: Added `docs/BRAIN_QUANTUM_ARCH.md` with full technical specifications.
- **Ratio Target Tests**: Added `tests/test_ratio_targets.py` to validate breakthrough KPIs.

### Changed
- **Observation Space**: Expanded from 257 to 261 dimensions (Histogram + Entropy + QNN Features).
- **Training Script**: Updated to train v5 models from scratch due to architecture change.
- **Roadmap**: Added Phase 5 (Brain-Like Quantum ML Breakthroughs).
- **Whitepaper**: Updated with SNN and Quantum Entanglement theory.
- **P2P Guide**: Updated with Hive Mind (FedProx) documentation.

### Fixed
- **Rust Lint Errors**: Enabled Spectral/Transformer predictors in encoder to sync with decoder.

## [8.0.1-alpha] - 2026-01-04

### Added
- Tagged release for v8.0.1 planning cycle.

## [8.0.0] - 2026-01-04 (Multimodal MetaBrain Release)

### Added
- MetaBrain v4 with multimodal training (PDF, WAV, IoT).
- WorldStateManager for graph + tensor persistence.
- Fidelity verification (`verify_fidelity.py`).
- CLIP embeddings for multimodal search.

### Performance
- IoT Ratio: 0.537 (stable baseline).
- Text Ratio: ~0.19.
- Swarm: PPO trained at ~500 FPS.

---

*For older releases, see `docs/releases/`.*
