# QRES v16 Zero-Cost Home Lab Roadmap (2026)

**Vision:** Transform QRES into a biologically-inspired edge intelligence platform using zero-cost resources, simulation, and rigorous benchmarking.

**Current Status:** Q2 Complete / Entering Q3

---

## Completed Milestones
*(See [COMPLETED_MILESTONES.md](COMPLETED_MILESTONES.md) for details)*

*   ✅ **Q1 2026: Pure Software Foundation** (Benchmarks, Virtual IoT, Adaptive Lossy)
*   ✅ **Q2 2026: Predictor Advancement** (Multivariate, Arithmetic Coding, Neural Predictor)

---

## CURRENT FOCUS: Q3 2026 (Jul-Sep) - Novel Contributions

**Goal:** Implement closed-loop feedback and Spiking Neural Networks (SNNs).

### 1. Feedback Loop Implementation
*Real-time model adaptation based on reconstruction error.*
*   [ ] `src/adaptive/feedback_loop.rs`: Online learning trigger
*   [ ] `regime_detector.rs`: Detect signal drift/shifts

### 2. SNN Predictor Simulation
*Spike-based prediction for theoretical energy savings.*
*   [ ] `python/qres_experiments/snn/`: `snnTorch` implementation
*   [ ] `docs/SNN_ENERGY_ANALYSIS.md`: Joules/Spike vs Joules/FLOP calculations

### 3. Azure-Based "Edge" Simulation
*Validate on actual constrained cloud hardware.*
*   [ ] `cloud_benchmarks/azure_deploy.sh`: Auto-scale testing script
*   [ ] Results comparison: B1ls vs B1s vs B2s VMs.

---

## UPCOMING: Q4 2026 (Oct-Dec) - Integration & Validation

**Goal:** Polish, document, and release QRES v16.0.0.

*   [ ] **Feature Flags:** Toggle Lossless/Lossy, Neural/SNN, Arithmetic/Huffman.
*   [ ] **Comprehensive Benchmarking:** Run full suite on 100+ UCR datasets.
*   [ ] **Paper Submission:** Finalize LaTeX submission.