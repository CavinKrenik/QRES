# QRES v16.0.0 Release Notes

**Date:** January 12, 2026
**Title:** QRES: Adapter Hybrid Compression System

## 🚀 Major Features

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

## 🐛 Fixes
- Fixed "Metric Fallacy" in benchmarks (now measuring against raw 4-byte `f32`).
- Fixed CI/CD failures related to missing data directories.
- Resolved `cargo fmt` and `clippy` lints.

---

# QRES v15.4.0 Release Notes

**Release Date:** January 11, 2026  

---

## 🎉 Overview

v15.4.0 introduces **Hardware-in-the-Loop Simulation** using real-world climate data, along with major visualization upgrades to the Hive Mind and Neural Graph pages.

---

## ✨ New Features

### 🌡️ Weather Replay Engine
* **Real-World Data:** Integrates the [Jena Climate Dataset](https://www.bgc-jena.mpg.de/wetter/) (Max Planck Institute) for high-fidelity sensor simulation
* **Storm Detection:** Maps atmospheric pressure drops to vibration spikes, triggering `LEARNING` mode
* **Debug Panel:** Real-time display of Frame index, Pressure (mbar), and Compression ratio

### 🕸️ Hive Mind: Interactive Neural Swarm
* **Infinite Canvas:** Zoom (0.1x-8x) and pan controls for exploring large networks
* **Node Inspector HUD:** Click any node to view IP, CPU load, Memory, and Status
* **Gradient Packets:** Animated particles flow between nodes when streaming is active

### 🧠 Neural Graph: Deep Learning Visualization
* **Layered Architecture:** 5-layer deep network (Input → Hidden A/B → Attention → Output)
* **Live Spike Propagation:** Visual pulses travel from input sensors to output nodes
* **Reactive to Data:** Input nodes flash based on real telemetry intensity

---

## 🔧 Improvements

### UI/UX Enhancements
* **Single Connect Button:** Removed duplicate header button; swarm toggle in Edge Swarm panel only
* **Clean Sidebar:** Text-only navigation labels (no icons)
* **No-Scroll Layout:** Dashboard now fits entirely in viewport

### Architecture
* **Simulated Compression:** Browser mode uses realistic compression ratios (~4-6:1) without requiring WASM
* **ResizeObserver:** Charts properly resize and fill available space
* **Vite Config:** Updated `server.fs.allow` for WASM file access

---

## 📝 Documentation

* **README:** Added "Hardware-in-the-Loop Simulation" section
* **Release Notes:** Updated v15.3.0 notes with simulation features

---

## 📦 Upgrade Instructions

```bash
# 1. Pull latest
git pull origin main

# 2. Install dependencies
cd qres-studio && npm install

# 3. (Optional) Fetch weather data
python3 scripts/fetch_weather_replay.py

# 4. Launch dashboard
npm run dev
```

---

## 📊 Metrics

| Metric | v15.3.0 | v15.4.0 |
|--------|---------|---------|
| Startup Time | ~1.6s | ~1.5s |
| Bundle Size | 1.4MB | 1.5MB |
| Visualization FPS | 30 | 60 |

---

**Full Changelog:** [v15.3.0...v15.4.0](https://github.com/CavinKrenik/QRES/compare/v15.3.0...v15.4.0)
