# QRES v15.2.0 "Publication Era"

**Release Date:** January 8, 2026

---

## Overview

v15.2.0 focuses on **Scientific Reproducibility and Validation**, preparing the QRES ecosystem for publication at FLICS 2026. It includes comprehensive benchmarks, scalability analysis, and complete theoretical documentation.

---

## Key Features

### 📊 Comprehensive Benchmarks (docs/BENCHMARKS.md)
- **Scalability:** Validated 100-node swarm performance (~9.4MB protocol state).
- **Throughput:** ~15ms update latency (10 nodes).
- **Privacy Cost:** Quantified 3.1x runtime overhead for Full Privacy Stack.
- **Compression:** ~22:1 ratio on IoT telemetry.

### 📚 Documentation Overhaul
- **Theory:** New `THEORY.md` detailing privacy composition and Byzantine proofs.
- **Related Work:** `RELATED_WORK.md` with 30+ citations.
- **Clean Structure:** Pruned obsolete files, organized `docs/archive`.

### 🧪 Reproducibility Code
- **Docker:** Production-grade `Dockerfile`.
- **Scripts:** `run_all_benchmarks.sh` for one-click validation.
- **Examples:** `examples/swarm_scale.rs` for load testing.

---

## What's Next (v16.0)

- **Post-Quantum Security:** Dilithium signatures.
- **FPGA Acceleration:** Hardware offload for SNN inference.

---

*See [CHANGELOG.md](../../CHANGELOG.md) for full details.*
