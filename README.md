# QRES: Neural Swarm Operating System

[![v18.0](https://img.shields.io/badge/version-18.0-blue.svg)](https://github.com/CavinKrenik/QRES/releases)
[![DOI](https://img.shields.io/badge/DOI-10.5281%2Fzenodo.18261441-blue)](https://doi.org/10.5281/zenodo.18261441)
[![no_std](https://img.shields.io/badge/no_std-compatible-green.svg)](https://docs.rust-embedded.org/book/intro/no-std.html)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust 2021](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)

---

## Emergent Intelligence in Action

![Neural Swarm Emergence](docs/images/neural_swarm_emergence.gif)

Visualizing a decentralized neural swarm recovering from a 15% packet loss interference zone. A single mutation propagates its evolved bytecode to heal the network through hardware-constrained gossip. This behavior emerges from network physics constraints, not central orchestration.

---

## Executive Summary

QRES is a decentralized operating system for **Edge AI Swarms**. It solves the "Consensus vs. Bandwidth" dilemma by replacing floating-point math with deterministic fixed-point arithmetic (`Q16.16`) and exchanging small "Evolved Genes" instead of massive gradient models.

**v18.0 Benchmark Highlights:**
- **Compression:** Custom Static Laplace Range Coder achieves **1.40x compression**, beating ZSTD (1.39x) on prediction residuals while maintaining bit-perfect determinism.
- **Efficiency:** Swarms converge **12x faster** (wall-clock) than Federated Learning on constrained IoT networks (56kbps), consuming **99% less bandwidth**.
- **Scalability:** Secure Aggregation proven linearly scalable (O(N)) for trusted peers, with optional Byzantine Fault Tolerance (Krum) for adversarial environments.

The system is architected as three interlocking layers:

1. **The Body (`qres_core`):** A `no_std` Rust library for deterministic inference and fragmentation.
2. **The Mind (`swarm_sim`):** An ECS-based simulator demonstrating emergent healing behavior.
3. **The Hippocampus:** A persistent storage layer enabling Lamarckian evolution across reboots.

---

## Why QRES?

- **Consensus-first math:** Q16.16 fixed-point removes cross-arch float drift, keeping 1,000 heterogeneous devices in deterministic lockstep.
- **Bandwidth as the bottleneck:** By shrinking updates to kilobyte genes, swarms converge faster in wall-clock time on LoRa/NB-IoT even if they need more epochs.
- **Swarm-scale parallelism:** Parallelism lives at the node level (many daemons), while each node stays single-threaded for determinism (see docs/adrs/ADR-004-concurrency-model.md).
- **End-to-end resilience:** MTU-aware gene gossip and persistence mean mutations survive drops and reboots; see docs/CLOUD_BENCHMARK_RESULTS.md for measured gains.

---

## ⚡ Verified Performance (Azure Cloud v18.0)

![Scalability Chart](docs/images/scalability_chart.png)

| Metric | Result | vs. Baseline | Verified On |
| :--- | :--- | :--- | :--- |
| **Scalability** | **10,000 Nodes** (100% Success) | **20x** Scale | Azure Standard_D2s (2 vCPU) |
| **Compression** | **31.8x** (SmoothSine) | **1.5x** Neural Gain | Azure Standard_F2s |
| **Memory** | **< 1 KB / Node** | **O(1)** Overhead | Rust `no_std` Runtime |

> *Benchmarks verified on January 16, 2026 using automated Azure CI/CD pipelines. See [docs/BENCHMARKS.md](docs/BENCHMARKS.md) for full results.*

---

## Key Features

### The Body: Deterministic Core

Located in `crates/qres_core/`, this is a `no_std` Rust library implementing:

- **Q16.16 Fixed-Point Arithmetic**: Eliminates floating-point drift across x86, ARM, and WASM platforms.
- **Deterministic Compression**: All prediction errors use integer math, making behavior reproducible across devices.
- **SwarmNeuron Trait**: Abstract interface for neural processors with signature methods: `predict()`, `check_surprise()`, `adapt()`, `export_gene()`, `install_gene()`.
- **LinearNeuron Implementation**: 8-lag linear predictor with entropy tracking and refractory periods.
- **Regime Switching**: Automatic transition between Calm, Storm, and Adapting states based on entropy thresholds.

See: [API Reference](docs/API_REFERENCE.md) | [Specification](docs/SPEC.md)

### The Mind: Emergent Swarm Simulator

Located in `tools/swarm_sim/`, this Bevy-based 3D simulator demonstrates:

- **Living Brain Visualization**: 150 nodes forming a force-directed spherical network with organic physics.
- **Interactive Camera**: Mouse-drag to orbit, scroll to zoom, spacebar for auto-rotation.
- **Noise Zone Physics**: A moving interference zone that induces storms and forces mutations.
- **Gene Gossip Protocol**: Panicked (Red) nodes request cure genes from evolved (Purple) neighbors.
- **Dense Neural Web**: Real-time connection visualization with color-coded synapse activity.
- **HUD Metrics**: Live display of node counts, entropy, synapses, and packet traffic.
- **Reset Function**: Press R to clear saved genes and restart evolution from scratch.
- **Cinematic Rendering**: HDR bloom, TonyMcMapface tonemapping, pulsing evolved nodes.

See: [Theory of Emergence](docs/theory/THEORY.md) | [P2P Implementation](docs/guides/P2P_IMPLEMENTATION.md)

### The Hippocampus: Persistent Memory

Located in `crates/qres_core/src/cortex/storage.rs`, this layer provides:

- **GeneStorage Trait**: Abstract persistence interface, `no_std` compatible.
- **DiskGeneStorage Implementation**: Saves evolved genes to `./swarms_memory/` directory.
- **Auto-Loading on Spawn**: Nodes check disk for saved genes on initialization; if found, spawn as evolved.
- **Periodic Persistence**: Every 5 seconds, calm evolved nodes save their bytecode to disk.
- **Lamarckian Evolution**: Learned strategies survive simulation restarts.

---

## Performance

![Swarm Singularity](docs/images/singularity_zero_shot.png)

Convergence benchmarks showing 100 nodes reaching consensus on a shared predictive model in under 30 epochs, using only 8 KB of bandwidth per day per node. Traditional federated learning requires **60,000x more bandwidth** and **12x longer wall-clock time** on constrained IoT networks.

---

## Getting Started

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Cargo (included with Rust)
- Optional: OBS or Windows Game Bar for recording simulations

### Run the Neural Swarm Simulator

![Neural Swarm Visualization](docs/images/SwarmVisual.gif)

```bash
cargo run -p swarm_sim --release
```

A Bevy window opens titled "QRES Living Brain: Neural Swarm Visualization." You will observe:

- **Force-Directed Brain**: 150 nodes form a cohesive spherical neural network with organic movement.
- **Noise Zone**: A red interference zone orbits through the brain, inducing storms in affected nodes.
- **Evolution Cascade**: Purple nodes spontaneously mutate and spread their cure to panicking red neighbors.
- **Dense Neural Web**: Connections visualize communication paths—purple for evolved pairs, cyan for mixed.

#### Interactive Controls

| Key | Action |
|-----|--------|
| **Left-click + drag** | Orbit camera around the brain |
| **Scroll wheel** | Zoom in/out |
| **Spacebar** | Toggle auto-rotation on/off |
| **R** | Reset simulation (clears all evolved genes) |

#### HUD Metrics

The top-left overlay displays real-time swarm statistics:
- **Nodes**: Total count and evolved percentage
- **Entropy**: Storm activity level (0% = calm, 100% = chaos)
- **Synapses**: Active communication channels
- **Packets**: Gene transfers in-flight

### Verify Persistence (The Hippocampus)

After running for 30+ seconds:

1. Stop the simulator (Ctrl+C).
2. List saved genes: `ls swarms_memory/`
3. Restart: `cargo run -p swarm_sim --release`
4. Observe: Nodes that were purple before now spawn purple immediately. Their learned strategies persisted.

---

## Build Verification

All crates compile to both `std` and `no_std` targets:

```bash
# Core crate (no_std)
cargo build -p qres_core --no-default-features --release

# Daemon (std)
cargo build -p qres_daemon --release

# Simulator (std + Bevy)
cargo build -p swarm_sim --release

# Run all tests
cargo test --all
```

---

## Documentation

Complete documentation is organized in the [docs/](docs/) directory:

| Category | Files |
|----------|-------|
| **Core Architecture** | [SPEC.md](docs/SPEC.md), [API_REFERENCE.md](docs/API_REFERENCE.md) |
| **Theory & Research** | [THEORY.md](docs/theory/THEORY.md), [SNN_ENERGY_ANALYSIS.md](docs/theory/SNN_ENERGY_ANALYSIS.md) |
| **Implementation Guides** | [P2P_IMPLEMENTATION.md](docs/guides/P2P_IMPLEMENTATION.md), [SECURITY_IMPLEMENTATION_GUIDE.md](docs/guides/SECURITY_IMPLEMENTATION_GUIDE.md) |
| **Process** | [CONTRIBUTING.md](docs/CONTRIBUTING.md), [SECURITY_ROADMAP.md](docs/SECURITY_ROADMAP.md) |
| **Benchmarks** | [BENCHMARKS.md](docs/BENCHMARKS.md), [CLOUD_BENCHMARK_RESULTS.md](docs/CLOUD_BENCHMARK_RESULTS.md) |
| **Media** | [IMAGES.md](docs/IMAGES.md) |

For a complete index, see [docs/README.md](docs/README.md).

---

## Project Structure

```
QRES/
├── crates/
│   ├── qres_core/           # Core no_std library (Body + Hippocampus)
│   │   └── src/cortex/      # SwarmNeuron trait, LinearNeuron, GeneStorage
│   ├── qres_daemon/         # Daemon service for edge deployment
│   └── qres_wasm/           # WebAssembly bindings
├── tools/
│   └── swarm_sim/           # Bevy-based God View simulator (Mind)
├── docs/                    # Comprehensive documentation
├── tests/                   # Integration tests
└── README.md               # This file
```

---

## Architecture Layers

### Layer 1: The Body (crates/qres_core)

Deterministic fixed-point arithmetic core. No floating-point operations. Runs on x86, ARM, WASM. Constraint: `no_std` with `alloc`.

Key modules:
- `cortex/`: Neural computing (SwarmNeuron trait, LinearNeuron, GeneStorage)
- `adaptive/`: Regime switching and entropy tracking
- `compression/`: Deterministic compression algorithms
- `crypto/`: Curve25519-based zero-knowledge proofs

### Layer 2: The Mind (tools/swarm_sim)

Bevy-based ECS simulator demonstrating emergent swarm behavior under network constraints. This is where evolution, mutation, and healing are visualized.

Key systems:
- `simulate_cortex_reaction`: React to noise zone
- `trigger_evolution`: Random mutations
- `gossip_protocol`: Gene requests between neighbors
- `packet_physics_system`: MTU fragmentation and packet loss
- `process_incoming_packets`: Gene installation

### Layer 3: The Hippocampus (crates/qres_core/src/cortex/storage.rs)

Persistence layer enabling learned strategies to survive reboots. Trait-based design allows swapping implementations (disk, cloud, IPFS).

---

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines, development workflow, and architecture decision records (ADRs).

---

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.

---

## Citation

If you use QRES in research, please cite:

```bibtex
@software{qres2026,
  author = {Krenik, Cavin},
  title = {QRES: Neural Swarm Operating System},
  url = {https://github.com/CavinKrenik/QRES},
  doi = {10.5281/zenodo.18261441},
  year = {2026}
}
```

See [CITATION.cff](CITATION.cff) for additional metadata.

---

## Publications

| Paper | Venue | DOI |
|-------|-------|-----|
| **Deterministic Rematerialization: Convergent Evolution in Cloud Kernels and Edge Swarms** | Preprint (2026) | [![DOI](https://img.shields.io/badge/DOI-10.5281%2Fzenodo.18305656-blue)](https://doi.org/10.5281/zenodo.18305656) |

This paper formalizes the theoretical foundation of QRES, identifying a structural isomorphism between IO-aware GPU kernels (FlashAttention, Fused Cross-Entropy) and QRES's silent consensus protocol. Both converge on **Deterministic Rematerialization**—discarding intermediate state and recomputing on demand—to achieve orders-of-magnitude reductions in data movement.

---

**Status**: Stable. Version 18.0 (Neural Swarm Architecture) complete. The pivot from deterministic compression to emergent swarms is verified in simulation. Ready for edge deployment.
