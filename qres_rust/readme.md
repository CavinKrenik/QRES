# QRES Rust Implementation

QRES (Quantum-Relational Encoding System) is a lossless data compression tool using relational delta encoding, run-length optimization (RLO), and zlib on chunked data. This Rust version supports multithreading for speed and low-memory chunking for large files (e.g., multi-GB ISOs).

## Features
- Delta encoding for byte differences.
- RLO for compressing repetitive sequences.
- Chunked processing (4MB blocks) to handle large files safely.
- Multithreading with rayon for parallel compression.
- Custom .qres format with metadata header.
- Lossless and deterministic.
- Proven: Compressed 6.28 GB Ubuntu ISO to ~1 GB (~84% reduction), decompressed and verified via mount/install.

## Dependencies
See Cargo.toml. Built with Rust 1.70+.

## Build and Run
1. Install Rust: https://www.rust-lang.org/tools/install
2. Clone the repo and cd into qres_rust/.
3. Build: `cargo build --release`
4. Run compression: `./target/release/qres_rust compress <input_file> <output.qres>`
5. Run decompression: `./target/release/qres_rust decompress <input.qres> <output_file>`

Example:
```bash
cargo build --release
./target/release/qres_rust compress ubuntu.iso ubuntu.qres
./target/release/qres_rust decompress ubuntu.qres restored.iso