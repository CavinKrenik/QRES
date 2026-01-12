# QRES: Adaptive Hybrid Compression for Edge IoT

[![CI](https://github.com/cavinkrenik/QRES/actions/workflows/rust.yml/badge.svg)](https://github.com/cavinkrenik/QRES/actions)
[![Version](https://img.shields.io/badge/version-v16.0.0-blue)](https://crates.io/crates/qres)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)

**QRES (Quantized Residual Entropy System)** is a `no_std`, biologically-inspired compression engine designed for the constrained edge. 

It uses a **Hybrid Conditional Pipeline** that dynamically switches between high-speed **Bit-Packing** and **Neural Residual Prediction** based on real-time data entropy. This guarantees minimal latency on noisy data while squeezing maximum compression from structured signals.

## 🚀 Validated Performance (v16 Benchmarks)
Benchmarks run on single-core generic hardware. QRES automatically bypasses Neural prediction when entropy is high (e.g., ETTh1) to save CPU.

| Dataset | Domain | Compression Ratio | Architecture Used |
|:---|:---|:---:|:---|
| **SmoothSine** | Synthetic | **24.9x** | Neural + BitPack |
| **Jena Climate** | Weather | **4.9x** | BitPack Dominant |
| **ItalyPower** | Smart Grid | **4.6x** | Neural + BitPack |
| **Wafer** | Manufacturing | **4.2x** | Neural + BitPack |
| **ECG5000** | Medical | **4.0x** | Neural + BitPack |
| **ETTh1** | Grid Sensor | **2.8x** | BitPack Only (Bypass) |

## 🌟 Key Features
* **Hybrid Gatekeeper:** Automatically detects if data is "predictable" (< 7.5 bits/byte). If yes, runs Neural network. If no, falls back to fast Bit-Packing.
* **Bit-Perfect Determinism:** Uses `Q16.16` fixed-point arithmetic for identical results across x86, ARM, and RISC-V.
* **Edge Native:** Pure `no_std` Rust, allocation-free options, and compiles to WASM.

## 📦 Installation

```toml
[dependencies]
qres = "0.16.0"
```

## ⚡ Quick Start

```rust
use qres_core::{compress_adaptive, decompress_adaptive};

fn main() {
    let sensor_data: Vec<f32> = vec![22.0, 22.1, 22.1, 22.3, 24.5];
    
    // QRES automatically decides whether to use Neural or Bit-Pack path
    let compressed = compress_adaptive(&sensor_data).unwrap();
    
    println!("Compressed {} bytes -> {} bytes", 
             sensor_data.len() * 4, compressed.len());
             
    let recovered = decompress_adaptive(&compressed).unwrap();
    assert_eq!(sensor_data, recovered);
}
```
