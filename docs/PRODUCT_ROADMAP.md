# QRES v16 Zero-Cost Home Lab Roadmap (2026)

**Vision:** Transform QRES into a biologically-inspired edge intelligence platform using zero-cost resources, simulation, and rigorous benchmarking.

**Current Status:** Q3 (Optimization & Novel Contributions)

---

## ✅ Completed Milestones
*(See [COMPLETED_MILESTONES.md](COMPLETED_MILESTONES.md) for details)*

* ✅ **Q1 2026: Pure Software Foundation** (Benchmarks, Virtual IoT, Adaptive Lossy)
* ✅ **Q2 2026: Predictor Advancement** (Multivariate, Arithmetic Coding, Neural Predictor)

---

## 🚀 CURRENT FOCUS: Q3 2026 (Jul-Sep) - Optimization & Novel Contributions

**Goal:** Solve neural latency, implement closed-loop feedback, and introduce SNNs.

### 1. Hybrid Adaptive Predictor (Latency Optimization)
*Immediate Task: Solve the 1.3ms inference cost by switching strategies dynamically.*
* [ ] `src/inference/hybrid_predictor.rs`: Logic to route easy windows to Heuristic and hard windows to Neural.
* [ ] **Optimization:** Profile ONNX runtime (`cargo flamegraph`) and explore INT8 quantization.
* [ ] **Target:** <100µs average latency per window.

### 2. Feedback Loop Implementation
*Real-time model adaptation based on reconstruction error.*
* [ ] `src/adaptive/feedback_loop.rs`: Online learning trigger
* [ ] `regime_detector.rs`: Detect signal drift/shifts

### 3. SNN Predictor Simulation
*Spike-based prediction for theoretical energy savings.*
* [ ] `python/qres_experiments/snn/`: `snnTorch` implementation
* [ ] `docs/SNN_ENERGY_ANALYSIS.md`: Joules/Spike vs Joules/FLOP calculations

### 4. Azure-Based "Edge" Simulation
*Validate on actual constrained cloud hardware.*
* [ ] `cloud_benchmarks/azure_deploy.sh`: Auto-scale testing script
* [ ] Results comparison: B1ls vs B1s vs B2s VMs.

---

## UPCOMING: Q4 2026 (Oct-Dec) - Integration & Validation

**Goal:** Polish, document, and release QRES v16.0.0.

* [ ] **Feature Flags:** Toggle Lossless/Lossy, Neural/SNN, Arithmetic/Huffman.
* [ ] **Comprehensive Benchmarking:** Run full suite on 100+ UCR datasets.
* [ ] **Paper Update:** Finalize LaTeX tables, figures, and submission (post-validation).