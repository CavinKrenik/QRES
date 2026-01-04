# QRES Release Notes

---

# v8.0.0 (AEON)
*Release Date: January 3, 2026*

**Summary**
This release consolidates all experimental streams into a single, unified release. It introduces the "Aeon" quantum-simulation layer while stabilizing the core neural-symbolic engine.

## 🚀 New Features

### 🌌 Quantum & Neural Enhancements
*   **Quantum Tensor Compression:** `QuantumEncoder` maps multi-modal graphs to density matrices for exponential compression (Simulated).
*   **AQC Pruning:** `NeuralOptimizer` uses Hamiltonian evolution to discover optimal sparsity masks for neural weights.
*   **Unified API & CLI:** `python/qres/api.py` and `qres_quantum_cli.py` enable seamless switching between standard and quantum modes.

### 🧠 Core Intelligence
*   **Multi-Modal Memory:** Implemented NetworkX + CLIP graph memory for relational understanding.
*   **Adaptive RL Mixer:** PPO Agent (Gymnasium) dynamically adjusts compression strategies based on data entropy.
*   **Neural-Symbolic "Telepathy":** The "Living Brain" autonomously selects between 4 distinct predictor models (Linear, Simple, Graph, Spectral) per byte.
*   **LLM Semantic Predictor:** Production-ready integration of local Transformers (GPT-2, Phi, TinyLlama) for semantic text prediction.

### 📦 Architecture & Performance
*   **Deduplication Engine (CDC):** Gear-based rolling hash for cross-file deduplication and solid compression.
*   **Native Swarm P2P:** High-performance `libp2p` (Rust) implementation for decentralized model weight synchronization (GossipSub).
*   **GPU Compute Pipeline:** WebGPU (`wgpu`) acceleration for batched mixing operations (~10x throughput).

### 🖥️ QRES Studio (GUI)
*   **Visual Interface:** New Tauri + Svelte frontend with a futuristic "SpaceX-inspired" design.
*   **Archive Browser:** Inspect `.qrar` contents and compression ratios without extraction.
*   **Explainable AI (XAI):** D3.js Knowledge Graph visualization and real-time neural feedback.

## 🛡️ Ethics & Safety
*   **Ethical Pruning:** Gini coefficient bias detection to prevent data misrepresentation in graph edges.

## ⚠️ Breaking Changes
*   **Archive Extension:** Default for solid archives is `.qrar`. `.qres` is reserved for single-file streams.
*   **CLI Renames:** `qres create` -> `qres archive`.
*   **Manifest:** Updated to v5.1 (Flag `0x05`) with `xxhash64` checksums.

## � Benchmarks
*   **Semantic Bench:** `benchmarks/semantic_bench.py` validates LLM predictor.
*   **Research:** See `docs/RESEARCH_NOTES.md` for citations (Delétang et al., Katharopoulos et al.).

## 📜 License
QRES is now officially licensed under **Apache 2.0**.
