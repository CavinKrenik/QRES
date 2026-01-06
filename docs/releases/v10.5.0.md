# QRES v10.5.0: The Hybrid Era

> **Native. WebAssembly. FPGA-Ready.**

This milestone release marks the beginning of the **Hybrid Era**, decoupling the QRES Core from the operating system to run anywhere: from high-performance servers to web browsers and embedded silicon.

## 🌟 Major Highlights

### 🌍 WebAssembly (WASM) Support
- **Browser-Native Compression:** QRES now runs entirely client-side in the browser.
- **Zero-Install:** Decompress data instantly in web apps without backend roundtrips.
- **Performance:** Near-native speeds using `wasm32-unknown-unknown` targets.

### 🛡️ `no_std` Architecture
- **Embedded Ready:** The `qres_core` library has been completely refactored to support `no_std` environments.
- **FPGA Prep:** This sets the stage for hardware synthesis (HLS) and bare-metal deployment on ARM Cortex-M and RISC-V.
- **Pure Logic:** All OS-dependent logic (file I/O, networking) has been moved to the Daemon, leaving the Core pure and deterministic.

### 🖥️ Studio Hybrid Runtime
- **Toggle Engine:** The QRES Studio GUI now features a "Hybrid Runtime" toggle.
  - **Native Mode:** Uses the local Rust daemon for maximum performance.
  - **WASM Mode:** Uses the in-memory WASM module for sandboxed, portable execution.

## 📦 Assets
- `qres-daemon` (CLI & Swarm Node)
- `qres-studio` (Cross-platform GUI)
- `qres.js` + `qres_bg.wasm` (Web Artifacts)

## 🛠 Usage

### Rust (Core)
```toml
[dependencies]
qres_core = { version = "10.5", default-features = false }
```

### Web
```javascript
import init, { compress, decompress } from './pkg/qres_wasm.js';

await init();
const compressed = compress(data);
```
