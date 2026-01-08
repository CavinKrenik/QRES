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
- ✅ **v13.0** – Security Hardening: ed25519 Signatures, PKI
- ✅ **v14.0** – Robust Aggregation: Krum, Multi-Krum, Trimmed Mean
- ✅ **v15.0** – Privacy: Differential Privacy, Secure Aggregation, ZK Proofs

---

## 🏗️ Current Focus (v12.x - Swarm Scaling Era)

### Phase 1-2: Security Hardening (v13-v14) ✅ COMPLETE
> **Goal:** Make federated swarms safe for adversarial environments.

- [x] **Authentication:** ed25519 signatures for model updates (`security.rs`)
- [x] **Node Identity:** Public key infrastructure for peer verification (`peer_keys.rs`)
- [x] **Replay Prevention:** Nonces and timestamps for update validation
- [x] **P2P Integration:** Signed broadcasts, verified receives in `swarm_p2p.rs`

### Phase 2: Robust Aggregation (v14 Target) ✅ COMPLETE
> **Goal:** Defend against weight poisoning attacks.

- [x] **Krum Algorithm:** Outlier rejection in federated averaging (`aggregation.rs`)
- [x] **Multi-Krum:** k-best updates averaging
- [x] **Trimmed Mean/Median:** Robust aggregation alternatives
- [x] **Buffered Aggregation:** Brain updates buffer before aggregating (`brain_aggregator.rs`)

### Phase 3: Privacy (v15) ✅ COMPLETE
> **Goal:** Differential privacy and secure aggregation.

- [x] **ε-DP Guarantees:** Differential privacy for shared weights (`privacy.rs`)
- [x] **Secure Aggregation:** Pairwise masking via X25519 (`secure_agg.rs`)
- [x] **ZK Proofs:** Pedersen Commitments + Proof of Norm (`zk_proofs.rs`)

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
