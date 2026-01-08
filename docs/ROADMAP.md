# QRES Roadmap

## 🗺️ Version History

- ✅ **v8.0** – MetaBrain v4, Multimodal, World State Persistence
- ✅ **v8.1** – SNN Integration, TNC Fusion, Hive Mind
- ✅ **v9.0** – GIF Neurons, OSBC Pruning, Auto-Tuning
- ✅ **v10.0** – Tensor Network Correlator, Deterministic Q16.16
- ✅ **v10.5** – `no_std` Refactor, WebAssembly Core
- ✅ **v11.0** – Fast Sigmoid, Browser Persistence (IndexedDB)
- ✅ **v11.1** – Portable SIMD (`wide` crate), Diverse IoT Benchmarks
- ✅ **v11.2** – Federated Swarms, Federated Dreaming
- ✅ **v12.0** – Swarm Scaling Era, Zero-Bandwidth Sync, Documentation Overhaul

---

## 🏗️ Current Focus (v12.x - Swarm Scaling Era)

### Phase 1: Security Hardening (v13 Target)
> **Goal:** Make federated swarms safe for adversarial environments.

- [ ] **Authentication:** ed25519 signatures for model updates
- [ ] **Node Identity:** Public key infrastructure for peer verification
- [ ] **Replay Prevention:** Nonces and timestamps for update validation

### Phase 2: Robust Aggregation (v14 Target)
> **Goal:** Defend against weight poisoning attacks.

- [ ] **Krum Algorithm:** Outlier rejection in federated averaging
- [ ] **Trimmed Mean:** Robust aggregation alternatives
- [ ] **Local Validation:** Pre-merge testing before accepting updates

### Phase 3: Privacy (v15 Target)
> **Goal:** Differential privacy and secure aggregation.

- [ ] **ε-DP Guarantees:** Differential privacy for shared weights
- [ ] **Secure Aggregation:** Sum weights without revealing individual contributions
- [ ] **ZK Proofs:** Zero-knowledge proofs of model quality

---

## 🔧 Engineering Roadmap

### FPGA Acceleration (Ongoing)
> **Goal:** Offload `SNN Predictor` and `Mixer` to FPGA for microsecond latency.

- [x] **`no_std` Refactor:** Decouple `qres_core` from standard library (Completed v10.5)
- [x] **Core Purification:** Move OS-dependent logic to Daemon (Completed v10.5)
- [ ] **Hardware Description:** Port `Mixer` logic to Verilog/HLS
- [ ] **Driver Layer:** Create DMA bridge between Rust Daemon and FPGA Core

### WebAssembly Core (COMPLETED)
> **Goal:** Run QRES entirely in the browser for client-side compression.

- [x] **WASM Target:** `qres_core` compiles to `wasm32-unknown-unknown`
- [x] **JS Bindings:** `wasm-bindgen` interface via `qres_wasm` crate
- [x] **Studio Integration:** Hybrid Engine in `qres-studio` (Native/WASM toggle)
- [x] **Browser Persistence:** IndexedDB integration (Completed v11.0)

---

## 📚 Documentation

- [Implementation Status](IMPLEMENTATION_STATUS.md) – What's production vs experimental
- [Philosophy](PHILOSOPHY.md) – Origin story and design principles
- [Technical Deep Dives](TECHNICAL_DEEP_DIVES.md) – Architecture details
- [Security Roadmap](SECURITY_ROADMAP.md) – Threat model and defenses
