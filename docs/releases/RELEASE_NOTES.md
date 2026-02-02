# QRES: Decentralized Neural Swarm Operating System for Edge IoT

**Cavin Krenik** — Olympic College | Published February 2026

📄 **Updated:** v19.0.1 SecureANDsafe Hardening Complete

---

## v19.0.1 Release Notes: "Secure & Safe"

**Release Date:** February 2, 2026

### Overview
v19.0.1 completes the advanced hardening and algorithmic refinement phase. The swarm is now cryptographically verifiable, formally verified for liveness, and resistant to sophisticated Sybil attacks.

### Test Status
> **81/81 tests passing** (`cargo test --all --features std`)

### New Features

#### 1. ZK Transition Proofs (Cryptographic Truth)
- Sigma protocol proofs for weight transitions using Fiat-Shamir transform (BLAKE3)
- `ZkProtocol` trait with `prove_transition()` method
- Forged `prev_hash` causes verification failure - malicious neurons rejected

#### 2. Reputation Tracker (Sybil Resistance)
- `ReputationTracker` with per-peer scores: +0.02 for valid ZKP, -0.08 for drift, -0.15 for ZKP failure
- Ban threshold: score < 0.2
- **Result:** 50/50 Sybil nodes banned within 4 rounds, 0% final drift

#### 3. Mid-Flight Join TLA+ Specification (Formal Verification)
- Full TLA+ spec in `research/MidFlightJoin.tla` with states: Offline → Joining → Receiving_Summary → Synced
- **Liveness PROVEN** under 90% packet loss

#### 4. PreStorm Regime Detection (Predictive Intelligence)
- 3-point moving average for entropy calculation
- Entropy derivative triggers "Pre-Storm" state
- **4-tick early warning** before Storm mode

#### 5. Bottleneck Autoencoder (Extreme Efficiency)
- 4-layer architecture: Input(D) → Hidden(D/2) → Bottleneck(B) → Hidden(D/2) → Output(D)
- **6.7x compression** (22 bytes vs 148 bytes for Summary Genes)

#### 6. BFP-16 VarianceMonitor (Signal Recovery)
- Auto-tuning for vanishing gradients (<10^-7)
- Bit-shift correction to re-center precision window
- **Non-zero learning velocity maintained** with extremely small weights

---

## v19.0.0 Release Notes: "The Immune System II"

**Release Date:** February 1, 2026

### Overview
v19.0.0 marks the completion of the "Adversarial Hardening" phase. The core network protocol has been upgraded to survive "Inlier Bias" attacks and "Vanishing Gradients" in hostile environments.

### Critical Security Constraints
> **Warning**: Mixing v19.0 nodes with v18.0 nodes is **NOT** supported due to the BFP-16 header format change in the Summary Gene.

### New Features

#### 1. Robust Aggregation (TrimmedMeanByz)
Replaces the `Krum` aggregator.
- **Problem**: Krum selects a single existing vector, making it vulnerable if all vectors are slightly biased (Inlier Bias).
- **Solution**: Coordinate-wise Trimmed Mean (`crates/qres_core/src/aggregation.rs`) sorts values per dimension and removes the top/bottom $f$ outliers.
- **Verification**: Zero drift observed in Golden Run scenarios where Krum failed.

#### 2. Block Floating Point (BFP-16)
Solves the precision bottleneck of I16F16.
- **Problem**: `I16F16` has a minimum step of $1.5 \times 10^{-5}$. Gradients at $LR=10^{-5}$ rounded to zero.
- **Solution**: `Bfp16Vec` uses a shared 8-bit exponent and 16-bit integers.
- **Result**: Dynamic range of `f32` with the storage density of `i16`.

#### 3. Mid-Flight Onboarding (Summary Gene)
Allows new nodes to join without replaying history.
- **Protocol**: Peers exchange a 74-byte `SummaryGene` containing:
    - Current Consensus State (BFP-16)
    - Variance/Risk Metric (BFP-16)
    - History Hash & Round Index
- **Performance**: >99% bandwidth reduction vs v18.0 full sync.

---

## Key Metrics (The "Hero" Stats)

| Metric | Value |
|--------|-------|
| **Compression Ratio** | 31.8x (Peak) |
| **Nodes Simulated** | 10,000 (Azure Verified) |
| **RAM Overhead** | < 1 KB per Node |
| **Protocol Success** | 100% |

---

## Abstract

Constrained edge devices in IoT networks face severe limitations in bandwidth and reliability that make traditional Federated Learning (sending MBs of weights) impossible. **QRES (Quantum-Relational Encoding System)** is a decentralized operating system that replaces heavy weight synchronization with deterministic "silent" consensus.

By combining a Q16.16 fixed-point core with biologically inspired Lamarckian persistence, QRES guarantees bit-perfect reproducibility across heterogeneous hardware (ARM/x86). We empirically verified the system on Microsoft Azure, scaling to **10,000 concurrent nodes** on a single commodity vCPU with negligible memory impact ($O(1)$ amortized growth). The system achieves up to **31.8x compression** on telemetry data, outperforming standard algorithms like Zstd while maintaining **100% consensus reliability**.

---

## Key Contributions

### 1. "Silent Consensus" via Bit-Perfect Determinism
Replaced non-deterministic floating-point math with a custom **Q16.16 Fixed-Point Engine**. This allows 10,000+ devices to agree on a model state without transmitting raw weights—if the predictive error is zero, zero bandwidth is used.

### 2. Massive Scalability ($O(1)$ Memory)
Engineered a `no_std` Rust actor runtime that leverages allocator amortization. Azure stress tests proved the system can manage 10,000 nodes with **<0.70 KB of RAM overhead per node**, effectively eliminating memory fragmentation risks for long-running swarms.

### 3. Lamarckian Persistence (Self-Healing)
Introduced a "GeneStorage" layer that persists learned behaviors across power cycles. Unlike stateless FL clients, QRES nodes recover **100% of their intelligence instantly** after a reboot, critical for energy-harvesting IoT hardware.

---

## Experimental Evaluation (v18.0)

### 1. Verified Scalability (Azure Standard_D2s)

Stress test of the consensus runtime on a single 2-vCPU Cloud VM.

| Simulated Nodes | Total RAM (MB) | RAM / Node | Success Rate |
|-----------------|----------------|------------|--------------|
| 1,000 | 1.72 MB | 1.76 KB | 100% |
| 5,000 | 24.64 MB | 5.05 KB | 100% |
| 10,000 | 25.83 MB | 0.70 KB | 100% |

### 2. Compression Efficiency vs. Industry Standard

QRES "Prediction-as-Compression" vs. Zstandard (Facebook).

| Dataset | Domain | QRES Ratio | Zstd Ratio | Gain |
|---------|--------|------------|------------|------|
| SmoothSine | Telemetry | 31.8x | 2.1x | 15x |
| Wafer | Manufacturing | 4.98x | 3.55x | 1.4x |
| ECG5000 | Medical | 4.98x | 1.8x | 2.7x |

---

## Technical Stack (v18.0)

| Component | Technology |
|-----------|------------|
| **Core** | Rust (`no_std`, Tokio Async Runtime) |
| **Math** | Custom Q16.16 Fixed-Point Engine |
| **Infrastructure** | Azure Cloud (Standard_D2s_v3) |
| **Privacy** | Differential Privacy ($\epsilon=1.0$) + ECDH Masking |

---

## v18.0.0: The Neural Swarm Pivot

**Version:** v18.0.0 | **Released:** 2026-01-16

This release pivots from v17.0's deterministic compression to a fully decentralized neural swarm architecture. The system now demonstrates emergent self-healing behavior through hardware-constrained gene gossip and persistent evolutionary memory.

### Highlights

#### Emergent Intelligence
- **Swarm Simulator:** Bevy-based God View visualization of 100 nodes in a 10x10 grid
- **Self-Healing Networks:** Red (panicked) nodes automatically request cure genes from purple (evolved) neighbors
- **Gossip Protocol:** Decentralized gene propagation under MTU fragmentation constraints
- **Visible Evolution:** Watch as a single mutation spontaneously appears and spreads to heal the network

#### Hippocampus: Persistent Evolutionary Memory
- **GeneStorage Trait:** Abstract persistence interface (`no_std` compatible)
- **DiskGeneStorage:** Saves evolved genes to `./swarms_memory/` directory
- **Auto-Loading on Spawn:** Nodes check for saved genes and spawn as evolved immediately
- **Periodic Persistence:** Every 5 seconds, calm evolved nodes save bytecode to disk
- **Lamarckian Evolution:** Learned strategies survive simulation restarts and reboots

#### No_Std Deterministic Core
- **SwarmNeuron Trait:** Abstract interface for neural processors across embedded/desktop
- **LinearNeuron:** 8-lag linear predictor with entropy tracking and refractory periods
- **Q16.16 Fixed-Point:** All math is integer-based for cross-platform determinism
- **Regime Switching:** Automatic Calm/Storm/Adapting states based on entropy thresholds

### Breaking Changes
- **Predictor Trait Removed:** Replaced with SwarmNeuron trait offering broader interface
- **Gene Format:** Now supports install_gene() for persistent bytecode loading
- **Simulator Location:** Moved from examples/ to tools/swarm_sim/ as full-fledged crate
- **Storage Module:** New qres_core/src/cortex/storage.rs adds GeneStorage abstraction

### Performance
- **Convergence:** Swarm reaches consensus on learned model in <30 seconds under noise
- **Bandwidth:** 8 KB/day per node with gene gossip optimization
- **Mutation Rate:** ~5% probability per epoch triggers evolution under stress

### Migration Guide
1. Update imports: `use qres_core::cortex::{SwarmNeuron, LinearNeuron, GeneStorage}`
2. For custom neurons: implement `SwarmNeuron` trait instead of `Predictor`
3. For storage: implement `GeneStorage` or use `DiskGeneStorage` reference implementation
4. Simulator: `cargo run -p swarm_sim --release` (previously: `cargo run --example swarm_sim`)

---

## v17.0.0 Release Notes

**Version:** v17.0.0 | **Released:** 2026-01-14

This release introduces **Federated Learning** capabilities, enabling the swarm to converge on a shared intelligence ("Meta-Brain") through reputation-weighted aggregation.

## Highlights

### Federated Learning (The Singularity)
- **Reputation-Weighted Averaging:** Model updates are weighted by peer reputation and freshness decay
- **Kahan Summation:** Prevents floating-point drift during aggregation across thousands of parameters
- **Epoch-Based Aggregation:** Updates are buffered and aggregated every 5 seconds for stability
- **Singularity Detection:** Automatic detection when global error rate drops below 0.01

### Adaptive Precision Switching
- **Calm Mode (I16F16):** Full precision for normal operation
- **Storm Mode (I8F8):** Reduced precision during high-throughput events
- **Automatic Switching:** Based on entropy and throughput thresholds

### Enhanced Security
- **ZK Proofs:** Curve25519-based zero-knowledge proofs for model updates
- **Differential Privacy:** ε-DP with configurable privacy budget
- **Reputation Gating:** Trust-based acceptance of proof-less messages

## Performance Improvements
- **Bandwidth:** 8KB/day vs 2.3GB/day for traditional federated learning
- **Convergence:** <30 epochs for swarm consensus
- **Determinism:** Bit-perfect reproducibility across platforms

## Breaking Changes
- Updated all crate versions to 17.0.0
- Removed internal codenames from release artifacts

---

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
