# QRES v5.0 Phase 2 Implementation - Swarm & Neural

## 🎯 Phase 2 Goals
1. **Swarm Robustness (FedProx)**: Implement proximal term to handle heterogeneity and prevent catastrophic forgetting.
2. **Neural Upgrades**: Train a Meta-Brain (MLP/Transformer) to predict optimal compression parameters.
3. **Scale Testing**: Validate on 10-20 nodes.

## 📊 Progress Tracker

### Phase 2.1: Swarm Robustness (FedProx)
- [x] **Client-Side Proximal Term**: Modified `Mixer` to penalize deviation from global weights.
- [x] **Drift Handling**: Implemented client drift detection in `hive_sync.py`.
- [ ] **Heterogeneous Test**: Simulate agents with non-IID data (text vs binary specialists).

### Phase 2.2: Neural Upgrades
- [ ] **Meta-Brain Architecture**: Design simple MLP in Rust (`common/neural.rs`?).
- [ ] **Training Pipeline**: `ai/train_meta_v2.py` to learn from benchmark data.
- [ ] **Inference**: Integrate neural inference into `compress_chunk`.

---

## 🛠️ Implementation Log

### Step 1: FedProx Mixer Support
*Status*: Pending
*Plan*:
1. Modify `Mixer` struct to hold `global_weights`.
2. Update `Mixer::update_weights`: Add `+ μ * (w - w_global)` to error/gradient.
3. Update `qres-cli` to load global weights from brain JSON.

### Step 2: Meta-Brain
*Status*: Pending
