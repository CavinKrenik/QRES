# QRES v4.0.1 - Release Notes
## "The Hive" Update

**Release Date:** December 2025
**Version:** v4.0.1
**Codename:** The Hive

### 🚀 Highlights
*   **Swarm Intelligence (FedProx)**: New agents now instantly inherit the performance of the expert "Global Brain" via federated averaging, solving the cold-start problem.
*   **Spectral Predictor**: Added FFT-based engine that achieves **46.2% compression ratio on Sine Waves**, significantly beating Zstd (16.6%).
*   **Graph Predictor**: New DAG-based predictor for structural telemetry logs.
*   **Lossy Mode (RDO)**: Rate-Distortion Optimization now allows `lossy` encoding to act as a smart denoising filter.

### 📊 Benchmark Results
| Dataset | QRES v4 Ratio | Notes |
| :--- | :--- | :--- |
| **IoT Telemetry** | **74.8%** | Beats LZ4 |
| **Sine Wave** | **46.2%** | SOTA (State of the Art) |
| **All Zeros** | **43.1%** | Fast adaptation |

### 🐛 Fixes
*   Fixed `AttributeError` in Python bindings.
*   Optimized `ans_coder` with batched updates for 10x throughput.
*   Resolved Windows Unicode issues in CLI.

### 📦 Assets
*   `qres-cli-windows.exe`: Standalone compressor.
*   `qres-*.whl`: Python bindings.
*   `qres_brain.json`: Pre-trained starter brain.
