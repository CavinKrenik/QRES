# QRES: Quantum-Relational Encoding System

[![Rust](https://img.shields.io/badge/built_with-Rust-dca282.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/status-Alpha_v2-yellow.svg)]()

**QRES** is a next-generation lossless compression framework that flips the traditional model on its head. Instead of storing absolute states (bytes), QRES stores the **"Waveform of Change"**: a stream of relational transitions (↑ Rise, ↓ Fall, = Plateau).

Originally prototyped in Python, the core engine has been rewritten in **Rust** to target high-performance telemetry, IoT logs, and time-series data where local continuity is high.

## 🚀 Key Features

- **Relational Encoding:** Compresses the *derivative* of the data. If a sensor reads `100, 101, 102, 103`, QRES sees `+1, +1, +1`.
- **Quantum-State Bit Packing (v2):** Uses a highly efficient 2-bit protocol to encode the *shape* of the data:
    - `00`: Flat (No Change)
    - `01`: Rise (+1)
    - `10`: Fall (-1)
    - `11`: Escape (Followed by 8-bit literal)
- **.qres Binary Format:** A framed, streamable container format with Zlib-compressed chunks and CRC32 checksums.
- **Quantum-Ready:** Designed to align with quantum computing concepts (entanglement analogs) for future hybrid applications.

## 📦 The .qres Binary Specification (v2)

The QRES file format is designed for streaming and random access.

| Segment | Size | Description |
| :--- | :--- | :--- |
| **Magic** | 4 bytes | ASCII `QRES` |
| **Meta Len** | 4 bytes | Big-endian integer (N) |
| **Metadata** | N bytes | JSON Header (Version=2, Timestamp, Original Size) |
| **Chunk 1** | Var | [Len: u32] [Data: Compressed Payload] |
| **Chunk N** | Var | ... |

## 🛠️ Installation & Usage

### Prerequisites
- Rust (Cargo) 1.70+

### Build from Source
```bash
git clone https://github.com/cavinkrenik/qres.git
cd qres/qres_rust
cargo build --release
```

### CLI Usage
Compress a file:

```bash
./target/release/qres_rust compress data.csv data.qres
```

Decompress a file:

```bash
./target/release/qres_rust decompress data.qres restored.csv
```

## ⚡ Performance Strategy
QRES outperforms general-purpose compressors (gzip) in specific domains:

**Telemetry/Logs:** Where values change slowly or predictably.

**Sensor Data:** Where noise is low relative to signal magnitude.

**Gradients:** Image data with smooth transitions.

## 🤝 Contributing
We are actively looking for contributors to help with:

**Hardware Acceleration:** AVX2/SIMD implementation of the bit-packer.

**Python Bindings:** wrapping the Rust core with PyO3 for pip install qres.

## License
MIT
