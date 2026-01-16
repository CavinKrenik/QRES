# QRES Documentation Index

Complete reference for the QRES Neural Swarm Operating System platform.

---

## Core Architecture

Start here to understand the system design and key concepts:

- [Specification (SPEC.md)](SPEC.md) — Formal specification of all subsystems, protocols, and constraints. Begin with the three-layer architecture section.
- [API Reference (API_REFERENCE.md)](API_REFERENCE.md) — Comprehensive API documentation for qres_core, qres_daemon, and qres_wasm bindings.
- [Implementation Status (IMPLEMENTATION_STATUS.md)](IMPLEMENTATION_STATUS.md) — Detailed progress tracking for all phases. Shows what is stable vs. in-development.

---

## Theory & Research

Deep dives into the science and mathematics:

- [Theory (THEORY.md)](theory/THEORY.md) — Complete mathematical framework for spiking neural networks, entropy thresholds, and regime switching.
- [SNN Energy Analysis (SNN_ENERGY_ANALYSIS.md)](theory/SNN_ENERGY_ANALYSIS.md) — Hardware efficiency analysis comparing SNNs to traditional ANNs. Key insight: 1000x less energy per inference.
- [Technical Deep Dives (TECHNICAL_DEEP_DIVES.md)](theory/TECHNICAL_DEEP_DIVES.md) — Detailed explorations of specific subsystems (fixed-point math, gossip protocols, MTU physics).
- [Related Work (RELATED_WORK.md)](RELATED_WORK.md) — Survey of federated learning, decentralized AI, and edge computing literature.

---

## Implementation Guides

Step-by-step guides for specific integration tasks:

- [P2P Implementation (P2P_IMPLEMENTATION.md)](guides/P2P_IMPLEMENTATION.md) — How to implement gossip protocols for gene propagation. Covers both ESP32 and x86 targets.
- [Security Implementation Guide (SECURITY_IMPLEMENTATION_GUIDE.md)](guides/SECURITY_IMPLEMENTATION_GUIDE.md) — Cryptographic signing, proof verification, and threat model mitigation.

---

## Benchmarks & Performance

- [Benchmarks (BENCHMARKS.md)](BENCHMARKS.md) — Local microbenchmarks for fixed-point arithmetic, compression algorithms, and neural inference.
- [Cloud Benchmark Results (CLOUD_BENCHMARK_RESULTS.md)](CLOUD_BENCHMARK_RESULTS.md) — Large-scale swarm experiments on AWS/Azure showing convergence rates and bandwidth usage.

---

## Process & Workflow

Guidelines for contributing and maintaining the project:

- [Contributing (CONTRIBUTING.md)](CONTRIBUTING.md) — Development setup, code style, testing standards, and pull request process.
- [Security Roadmap (SECURITY_ROADMAP.md)](SECURITY_ROADMAP.md) — Planned security audits, cryptography upgrades, and threat model refinements.

---

## Architecture Decision Records (ADRs)

Decisions that shaped the system:

- [ADR-001: SNN vs ANN](adrs/ADR-001-snn-vs-ann.md) — Why we chose spiking neural networks over traditional artificial neural networks.
- [ADR-002: Signature Scheme](adrs/ADR-002-signature-scheme.md) — Why we use Curve25519 with zero-knowledge proofs instead of threshold cryptography.
- [ADR-003: PRNG Sync](adrs/ADR-003-prng-sync.md) — How deterministic pseudo-random number generation ensures cross-platform consensus.

---

## Historical Records

- [Completed Milestones (COMPLETED_MILESTONES.md)](COMPLETED_MILESTONES.md) — Chronological record of v1.0 through v18.0 releases and their features.
- [Release Notes (releases/RELEASE_NOTES.md)](releases/RELEASE_NOTES.md) — Detailed changelog for the latest stable release.

---

## Citation & References

- [Bibliography (references.bib)](references.bib) — Academic citations for all referenced papers and systems.
- See [CITATION.cff](../CITATION.cff) in root for BibTeX format.

---

## Quick Navigation

**First Time Here?**
1. Read [../README.md](../README.md) (root README) for the executive summary
2. Skim [SPEC.md](SPEC.md) section 1 (Three-Layer Architecture)
3. Watch the GIF in the root README
4. Run the simulator: `cargo run -p swarm_sim --release`

**Want to Contribute?**
1. Read [CONTRIBUTING.md](CONTRIBUTING.md)
2. Check [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) for open tasks
3. Review relevant ADRs if changing core systems

**Deploying to Edge Hardware?**
1. Read [P2P_IMPLEMENTATION.md](guides/P2P_IMPLEMENTATION.md)
2. Consult [SECURITY_IMPLEMENTATION_GUIDE.md](guides/SECURITY_IMPLEMENTATION_GUIDE.md)
3. Review [API_REFERENCE.md](API_REFERENCE.md) for qres_daemon API

**Understanding Performance?**
1. Check [BENCHMARKS.md](BENCHMARKS.md) for local results
2. Read [CLOUD_BENCHMARK_RESULTS.md](CLOUD_BENCHMARK_RESULTS.md) for swarm-scale data
3. Review [SNN_ENERGY_ANALYSIS.md](theory/SNN_ENERGY_ANALYSIS.md) for hardware efficiency

---

**Latest Stable Version**: v18.0
**Status**: Stable. The pivot from v17.0 (Deterministic Compression) to v18.0 (Neural Swarm Architecture) is complete and verified in simulation.
