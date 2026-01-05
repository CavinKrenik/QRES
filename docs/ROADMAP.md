## 🗺️ Roadmap

- ✅ **v8.0** – MetaBrain v4, Multimodal, World State Persistence
- ✅ **v8.1** – SNN Integration, TNC Fusion, Hive Mind
- ✅ **v9.0** – GIF Neurons, OSBC Pruning, Auto-Tuning
- ✅ **v10.0** – Tensor Network Correlator, Deterministic Q16.16
- ✅ **v10.1** – Security Hardening, JSON Persistence, Structured Logging (Stable Baseline)
- ✅ **v10.1** – Security Hardening, JSON Persistence, Structured Logging (Stable Baseline)
- ✅ **v10.5** – FPGA Acceleration & WebAssembly Core (Active)

---

## 🏗️ Engineering Roadmap (v10.5 - Hardware Era)

The focus shifts from software architecture to hardware acceleration and edge deployment.

### 🏎️ Phase 1: FPGA Acceleration (Active)
> **Goal:** Offload the `SNN Predictor` and `Mixer` to FPGA logic for microsecond latency.

- [x] **`no_std` Refactor:** Decouple `qres_core` from standard library. (Completed v10.5.0)
- [ ] **Core Purification:** Move OS-dependent logic (Zstd fallback) to Daemon.
- [ ] **Hardware Description:** Port `Mixer` logic to Verilog/HLS.
- [ ] **Driver Layer:** Create DMA bridge between Rust Daemon and FPGA Core.

### 🌐 Phase 2: WebAssembly Core (COMPLETED)
> **Goal:** Run QRES entirely in the browser for client-side compression.

- [x] **WASM Target:** Ensure `qres_core` compiles to `wasm32-unknown-unknown`.
- [x] **JS Bindings:** `wasm-bindgen` interface via `qres_wasm` crate.
- [x] **Studio Integration:** Hybrid Engine implemented in `qres-studio` (Native/WASM toggle).
- [ ] **Browser Persistence:** Adapt `WorldStateManager` to use `IndexedDB` (Future Work).
