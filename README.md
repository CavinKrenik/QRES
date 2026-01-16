# QRES: Neural Swarm Operating System

[![v18.0](https://img.shields.io/badge/version-18.0-blue.svg)](https://github.com/CavinKrenik/QRES/releases)
[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.18261441.svg)](https://doi.org/10.5281/zenodo.18261441)
[![no_std](https://img.shields.io/badge/no_std-compatible-green.svg)](https://docs.rust-embedded.org/book/intro/no-std.html)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust 2021](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)

---

## Emergent Intelligence in Action

![Neural Swarm Emergence](docs/images/neural_swarm_emergence.gif)

Visualizing a decentralized neural swarm recovering from a 15% packet loss interference zone. A single mutation propagates its evolved bytecode to heal the network through hardware-constrained gossip. This behavior emerges from network physics constraints, not central orchestration.

---

## Executive Summary

QRES is a decentralized operating system for edge AI swarms. It combines deterministic consensus (via fixed-point arithmetic), emergent healing (via MTU-constrained gene gossip), and persistent evolutionary memory (via the Hippocampus layer). In v18 cloud benchmarks, swarms converged an order of magnitude faster than baseline federated learning while holding bandwidth near ~8 KB/day; see docs/CLOUD_BENCHMARK_RESULTS.md for scenarios and metrics.

The system is architected as three interlocking layers:

1. **The Body**: A `no_std` Rust core using Q16.16 fixed-point math for deterministic computation across heterogeneous hardware.

2. **The Mind**: An ECS-based swarm simulator demonstrating emergent behavior where linear neural predictors mutate under stress and propagate evolved bytecode through gossip protocols.

3. **The Hippocampus**: A persistence layer enabling Lamarckian evolution—learned strategies survive across reboots via disk storage.

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

- **God View Visualization**: 100 nodes in a 10x10 grid, each running a SwarmNeuron instance.
- **Noise Zone Physics**: A moving interference zone that induces packet loss and forces mutations.
- **Gene Gossip Protocol**: Panicked (Red) nodes request cure genes from evolved (Purple) neighbors within transmission range.
- **MTU Fragmentation**: Simulates ESP32 Wi-Fi 1400-byte MTU limit; large genes (1600 bytes) drop at 15% rate, creating evolutionary pressure for compact bytecode.
- **Cinematic Rendering**: HDR bloom, TonyMcMapface tonemapping, gizmo visualization of the noise threat and neural web connections.

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

Convergence benchmarks showing 100 nodes reaching consensus on a shared predictive model in under 30 epochs, using only 8 KB of bandwidth per day per node. Traditional federated learning requires 8-10x the data and 100x the time.

---

## Getting Started

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Cargo (included with Rust)
- Optional: OBS or Windows Game Bar for recording simulations

### Run the Neural Swarm Simulator

```bash
cargo run -p swarm_sim --release
```

A Bevy window opens titled "QRES Phase 3: Emergent Swarm Evolution." You will observe:

- **Seconds 0-10**: Blue nodes at rest (Calm state). Red force field (noise zone) begins orbiting.
- **Seconds 10-15**: Red nodes appear where noise intersects. Panic state activated.
- **Seconds 15-20**: A purple node spontaneously mutates (EvolvedNeuron). This is the spark.
- **Seconds 20+**: Purple nodes spread to neighboring Red nodes via gossip. The cure propagates, but stutters due to MTU fragmentation losses. This is emergent healing under physical constraints.

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

MIT License. See [LICENSE](LICENSE) for details.

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

**Status**: Stable. Version 18.0 (Neural Swarm Architecture) complete. The pivot from deterministic compression to emergent swarms is verified in simulation. Ready for edge deployment.
