# QRES Changelog

All notable changes to this project are documented here.

---

## [9.0.0] - 2026-01-04 "Singularity Brain"

### Added
- **GIF Neurons**: Generalized Integrate-and-Fire from SpikeLLM (ICLR 2025)
- **OSBC Pruning**: Second-order pruning achieving 97% sparsity
- **Equivariant QNN**: Symmetry-preserving lattice compression
- **Auto-Tuning**: Fine-tune MetaBrain on user data (`auto_tune.py`)
- **Research Citations**: 2025 papers for SNN/QML advances

### Changed
- Upgraded SNN predictor from LIF to GIF neurons
- Enhanced QNN with equivariant lattice method

---

## [8.1.0] - 2026-01-04 "Brain-Quantum ML"

### Added
- **Spiking Neural Networks**: `snn_predictor.py` with LIF neurons
- **Quantum VQC**: `qnn_vqc.py` with variational circuits
- **Hive Mind**: `hive_mind.py` with FedProx and KL-FedDis
- **MetaBrain v5**: SNN+QNN hybrid (261-dim observations)
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

## [7.5.0] - 2025-12-30 "Quantum Foundations"

### Added
- Tensor Network Compression (`QuantumEncoder`)
- Haar Wavelet Transform for MPS
- Quantum Mode CLI (`qres_quantum_cli.py`)

---

## [7.0.0] - 2025-12-28 "Strategic Enhancements"

### Added
- MultiModal Memory (NetworkX + CLIP)
- PPO Agent (Gymnasium)
- QRES Studio GUI with D3 Knowledge Graph

---

*For full history, see git log.*
