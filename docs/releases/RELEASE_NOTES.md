# QRES v16.5.0 Release Notes

**Codename:** "The Immune System" | **Released:** 2026-01-14

> **"Identity without Exposure. Trust without Centralization."**

This release introduces the **QRES Immune System**—a comprehensive security stack designed to protect the decentralized "Living Brain" from adversarial attacks while preserving the privacy of edge contributors.

## Highlights

### The Ghost Protocol (Privacy Stack)
We have implemented a **Defense-in-Depth** privacy layer that ensures no single peer or component can see the raw model updates:
1.  **Differential Privacy (Noise Layer):** Deterministic Gaussian noise is added to the `I16F16` weights before they leave the device.
2.  **Secure Aggregation (Masking Layer):** Peers establish pairwise shared secrets (X25519) to mask their updates. The Aggregator sees only the global sum, as individual masks cancel out mathematically.
3.  **Zero-Knowledge Proofs (Verification Layer):** Peers attach `NormProofs` (Pedersen Commitments) proving their masked update is bounded (not garbage) without revealing the update itself.

### Trust & Reputation (The Gatekeeper)
The swarm now actively filters participation based on "Mathematical Merit":
*   **Reputation Manager:** A persistent trust score tracks peer behavior.
    *   Accepted Update: `+0.01` Trust
    *   Krum Rejection: `-0.1` Trust
    *   Ban Threshold: Trust `< 0.2`
*   **Identity Binding:** Aggregation results are now cryptographically bound to the sender's Ed25519 identity, enabling long-term accountability.

### Hardened Federated Dreaming
*   **Sanity Checks:** The "Dreaming" process (Generative Replay) now validates synthetic weights against a local buffer of real data before applying them, preventing "hallucinations" or model poisoning via synthesis.

## Changes

### Core (`qres_core`)
*   Added `privacy` module with `add_noise_fixed` for I16F16 support.
*   Added `secure_agg` module with `mask_update_fixed` and strict X25519 key agreement.
*   Added `zk_proofs` module with `ProofBundle` and `verify_batch`.
*   Added `packet` module defining the `GhostUpdate` structure.

### Daemon (`qres_daemon`)
*   Integrated `ReputationManager` into `AppState`.
*   Updated `BrainAggregator` to return accepted/rejected peer lists for scoring.
*   Updated `SwarmP2P` message loop to handle reputation rewards/punishments.

## Breaking Changes
*   **Protocol Update:** The peer-to-peer message format has changed to support `GhostUpdate` packets. v16.5 nodes cannot federate with v16.0 nodes.
*   **Config:** `reputation.json` is now required (automatically created if missing).

## Upgrade Guide
```bash
# Update Rust Toolchain
rustup update stable

# Pull latest
git pull origin main

# Build
cargo build --release
```

---

# QRES v16.0.0 Release Notes

## v16.0.0 - The "Systems" Update
> **Release Date:** January 13, 2026
> **Focus:** Determinism, Safety, and Zero-Copy Performance.

###  Major Changes
- **Breaking:** `compress_chunk` now requires a pre-allocated `&mut [u8]` buffer (Zero-Copy).
- **Feat:** Replaced floating-point math with `fixed::types::I16F16` for bit-perfect cross-arch consensus.
- **Security:** Removed all panic paths (`unwrap`, `expect`) from the `no_std` core.
- **Structure:** Monorepo split into `crates/` (Production) and `research/` (Experiments).

### Bug Fixes
- Fixed "Link Explosion" in P2P sync by implementing Deterministic Seed Sync (8 KB/day).
- Fixed "Expansion Problem" via Hybrid Gatekeeper (fallback to bit-packing on high entropy).

---

# QRES v16.0.0 - Pre-Release Notes

**Date:** January 13, 2026
**Title:** QRES: Adapter Hybrid Compression System

## Major Features

### 1. Hybrid Conditional Pipeline
QRES now dynamically switches between two codec paths based on real-time data entropy (< 7.5 bits/byte threshold):
- **Bit-Packing Path:** High-speed Delta+ZigZag+BitPack algorithm. (Used for Grid/Noise data)
- **Neural-Enhanced Path:** Neural residual prediction for structured data. (Used for Weather/ECG)

### 2. Validated Benchmarks (2.75x - 24.9x)
Comprehensive benchmarking across 7 diverse datasets confirms QRES outperforms standard predictors:
- **SmoothSine:** 24.9x
- **Jena Climate:** 4.9x
- **ItalyPower:** 4.6x
- **Wafer:** 4.2x
- **ECG5000:** 4.0x
- **ETTh1:** 2.8x

### 3. Production-Ready Core
- **`bitpack.rs`:** Integrated validated bit-packing logic directly into `qres_core`.
- **`qres_core` API:** exposed `compress_adaptive` and `decompress_adaptive` for easy integration.
- **Fixed-Point Arithmetic:** `Q16.16` math ensures cross-platform determinism (x86/ARM/WASM).

### 4. Documentation Overhaul
- **New Paper:** "QRES: An Adaptive Hybrid Compression System for Edge IoT" (PDF available in `docs/paper/`)
- **Theory Docs:** "Living Brain" architecture details moved to `docs/THEORY.md`.
- **Roadmap:** v16 milestones marked complete.

## Fixes
- Fixed "Metric Fallacy" in benchmarks (now measuring against raw 4-byte `f32`).
- Fixed CI/CD failures related to missing data directories.
- Resolved `cargo fmt` and `clippy` lints.
- **Hotfix:** Restored `compress_adaptive` Python alias for backward compatibility.
- **Hotfix:** Resolved Tauri plugin version mismatch.

---

# QRES v15.4.0 Release Notes

**Release Date:** January 11, 2026  

---

## Overview

v15.4.0 introduces **Hardware-in-the-Loop Simulation** using real-world climate data, along with major visualization upgrades to the Hive Mind and Neural Graph pages.

---

## New Features

### Weather Replay Engine
* **Real-World Data:** Integrates the [Jena Climate Dataset](https://www.bgc-jena.mpg.de/wetter/) (Max Planck Institute) for high-fidelity sensor simulation
* **Storm Detection:** Maps atmospheric pressure drops to vibration spikes, triggering `LEARNING` mode
* **Debug Panel:** Real-time display of Frame index, Pressure (mbar), and Compression ratio

### Hive Mind: Interactive Neural Swarm
* **Infinite Canvas:** Zoom (0.1x-8x) and pan controls for exploring large networks
* **Node Inspector HUD:** Click any node to view IP, CPU load, Memory, and Status
* **Gradient Packets:** Animated particles flow between nodes when streaming is active

### Neural Graph: Deep Learning Visualization
* **Layered Architecture:** 5-layer deep network (Input → Hidden A/B → Attention → Output)
* **Live Spike Propagation:** Visual pulses travel from input sensors to output nodes
* **Reactive to Data:** Input nodes flash based on real telemetry intensity

---

## Improvements

### UI/UX Enhancements
* **Single Connect Button:** Removed duplicate header button; swarm toggle in Edge Swarm panel only
* **Clean Sidebar:** Text-only navigation labels (no icons)
* **No-Scroll Layout:** Dashboard now fits entirely in viewport

### Architecture
* **Simulated Compression:** Browser mode uses realistic compression ratios (~4-6:1) without requiring WASM
* **ResizeObserver:** Charts properly resize and fill available space
* **Vite Config:** Updated `server.fs.allow` for WASM file access

---

## Documentation

* **README:** Added "Hardware-in-the-Loop Simulation" section
* **Release Notes:** Updated v15.3.0 notes with simulation features

---

## Upgrade Instructions

```bash
# 1. Pull latest
git pull origin main

# 2. Install dependencies
cd web && npm install

# 3. (Optional) Fetch weather data
python3 scripts/fetch_weather_replay.py

# 4. Launch dashboard
npm run dev
```

---

## Metrics

| Metric | v15.3.0 | v15.4.0 |
|--------|---------|---------|
| Startup Time | ~1.6s | ~1.5s |
| Bundle Size | 1.4MB | 1.5MB |
| Visualization FPS | 30 | 60 |

---

**Full Changelog:** [v15.3.0...v15.4.0](https://github.com/CavinKrenik/QRES/compare/v15.3.0...v15.4.0)
