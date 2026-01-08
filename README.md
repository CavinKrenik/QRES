# QRES

> A neural compression engine for time-series data

[![License](https://img.shields.io/badge/license-Apache%202.0-blue)](LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/CavinKrenik/QRES/release.yml?style=flat)](https://github.com/CavinKrenik/QRES/actions)
[![Version](https://img.shields.io/badge/version-v15.0.0-brightgreen)](https://github.com/CavinKrenik/QRES/releases)

## The Core Idea

What if we compressed data the way brains compress memories—by predicting patterns and encoding only surprises?

QRES started with a simple question: **Can you compress data using only three comparisons: `<`, `>`, `=`?**

That question led to the exploration of predictive compression, deterministic neural networks, and distributed model synchronization.

## How It Works

**Traditional compression:** "How do I represent this data in fewer bits?"  
**QRES approach:** "How often does this data differ from what I expected?"

When encoder and decoder share the same predictor, you only need to transmit the surprises—the moments when reality diverged from expectation.

```mermaid
graph LR
    A[Sensor Data] --> B[Predictor]
    B --> C[Expected Value]
    A --> D[Compare]
    C --> D
    D --> E{Surprise?}
    E -->|No| F[Send 0 bit]
    E -->|Yes| G[Encode Residual]
```

## Key Features

- **Deterministic Architecture**: Q16.16 fixed-point math ensures bit-perfect reproducibility across x86, ARM, RISC-V, WASM
- **Federated Learning**: Nodes share models peer-to-peer without transmitting raw data
- **Edge-Native**: `no_std` compatible, runs on bare-metal embedded devices
- **Hybrid Runtime**: Seamlessly switch between native performance and portable WebAssembly

## When to Use QRES

| ✅ Use QRES For | ❌ Don't Use QRES For |
|----------------|----------------------|
| IoT telemetry (repetitive sensor data) | High-entropy data (encryption, random) |
| Structured logs (timestamps, patterns) | Existing archives (.zip, .jpg, .mp4) |
| Edge networks (bandwidth > compute cost) | Files < 1KB (header overhead) |
| Archival (deterministic restoration) | Maximum speed (use LZ4) |

## Quick Start

```bash
# Clone and install
git clone https://github.com/CavinKrenik/QRES.git
cd QRES
pip install .

# Compress a file
python3 -c "import qres; print(f'Size: {len(qres.compress(open(\"README.md\", \"rb\").read()))} bytes')"
```

[Full installation docs →](docs/guides/P2P_IMPLEMENTATION.md)

## The Journey

This project evolved through 13 major iterations:

| Version | Milestone |
|---------|-----------|
| v1 | Simple ternary encoding (left/right/equal) |
| v2 | Added delta prediction |
| v3 | Experimented with neural predictors |
| v4 | Switched to SNNs for edge compatibility |
| v8 | P2P swarm architecture |
| v10 | Tensor network correlators, Q16.16 determinism |
| v12 | Federated swarms, zero-bandwidth synchronization |
| v13 | Security hardening: ed25519 signatures, Krum aggregation |
| v14 | Robust aggregation: Multi-Krum, Trimmed Mean, Median |
| v15 | Privacy: Differential Privacy, Secure Aggregation, ZK Proofs |

[Read the full story →](docs/PHILOSOPHY.md)

## Architecture

```mermaid
graph TD
    A[Node A] -.Share Models.-> B[Node B]
    B -.Share Models.-> C[Node C]
    C -.Share Models.-> A
    A --> D[Local Data]
    B --> E[Local Data]
    C --> F[Local Data]
    D --> G[Collective Intelligence]
    E --> G
    F --> G
```

[See WHITEPAPER.md for technical details →](docs/WHITEPAPER.md)

## Performance

Benchmarks on structured datasets (Intel Ice Lake):

| Dataset | Type | Ratio | Speed |
|---------|------|-------|-------|
| **Sensor Stream** | IoT Telemetry | **~0.15** (6.6x) | 300+ MB/s |
| **Server Logs** | Text/Time-series | **~0.19** (5.2x) | 200 MB/s |
| **CSV Data** | Correlated Numerics | ~0.76 (1.3x) | 150 MB/s |

[Full benchmarks →](docs/BENCHMARKS.md)

## Implementation Status

| Status | Components |
|--------|------------|
| ✅ **Production Ready** | Core engine, Python bindings, WASM decoder |
| 🧪 **Experimental** | Federated dreaming, regime adaptation |
| 📋 **Roadmap** | Security defenses, arithmetic coding, FPGA |

[Detailed status →](docs/IMPLEMENTATION_STATUS.md)

## Project Structure

- `qres_rust/` – Rust workspace containing core engine and daemon
  - `qres_core`: High-performance, `no_std` compression library
  - `qres_daemon`: P2P node and API service
- `python/` – Python bindings and experimental ML models
- `qres-studio/` – Cross-platform GUI (Tauri/Svelte)
- `docs/` – Documentation hub

## Documentation

- [Philosophy & Origin Story](docs/PHILOSOPHY.md)
- [Implementation Status](docs/IMPLEMENTATION_STATUS.md)
- [Technical Deep Dives](docs/TECHNICAL_DEEP_DIVES.md)
- [Security Roadmap](docs/SECURITY_ROADMAP.md)
- [Contributing Guide](docs/CONTRIBUTING.md)

## Frequently Asked Questions

**Q: Is this production-ready?**  
A: The core engine is stable and deterministic. Federated learning is experimental. Security features are on the roadmap. See [Implementation Status](docs/IMPLEMENTATION_STATUS.md).

**Q: How does this compare to Zstd/Gzip?**  
A: QRES is specialized for repetitive time-series data. For general-purpose compression, use Zstd. QRES shines when data patterns are predictable and bandwidth is constrained.

**Q: Does this use quantum computing?**  
A: No. QRES runs on classical hardware. Early versions had misleading naming that has been removed.

**Q: What's the learning curve?**  
A: For basic use (compress/decompress): low, similar to any compression tool. For federated swarm deployment: moderate, requires understanding of distributed systems.

**Q: Can I use this in production?**  
A: The compression engine is solid. The P2P/federated features assume trusted nodes (no Byzantine defenses yet). Safe for private networks, not public/adversarial environments.

## License

Apache 2.0 – See [LICENSE](LICENSE)

---

## What I Learned

**Before:** I thought compression was about clever bit-packing  
**After:** Compression is prediction. Better predictions = better compression

**Challenge that surprised me:** Handling regime changes gracefully. Static predictors fail when patterns shift.

**What I'd do differently:** Start with security/Byzantine tolerance from day one. Adding it retroactively is painful.

[Read more about the development journey →](docs/PHILOSOPHY.md)
