# QRES Performance Benchmarks

**Hardware:** AWS c6i.4xlarge (Intel Ice Lake)
**Corpus:** IoT-Drift (Generic Sensor Data), Shakespeare (Text), Multimodal (data/ folder)
**Version:** QRES v8.0 (MetaBrain v4)

## 1. Compression Ratio (Lower is Better)

| Engine | IoT-Drift Ratio (20MB) | Text Ratio | Multimodal (PDF/WAV) Ratio | Notes |
| :--- | :---: | :---: | :---: | :--- |
| **QRES v8.0 (MetaBrain v4)** | **0.537** | **~0.19** | **~0.9 (PDF), ~0.6 (WAV)** | Stable post-multimodal training; binary fallback used |
| **QRES v7.0 (Quantum)** | **~0.048** | **~0.19** | - | Estimated with Tensor Networks >40% gain |
| Zstd (L19) | 0.124 | 0.355 | ~0.95 (PDF) | Standard High Compression |

### Analysis
*   **IoT Data:** Consistent after v4 training; widened thresholds improve routing.
*   **Multimodal:** PDFs often incompressible (small/minimal); WAV benefits from spectral prediction.

## 2. Multi-Modal Performance (v8.0)
*   **Mixed Media Efficiency:** 15-20% improvement on archives with text/images/audio.
*   **Context Awareness:** Agent handles diverse types without regression.

## 3. Throughput & Speed

| Engine | Compression Speed | Decompression Speed |
| :--- | :---: | :---: |
| **QRES v8.0** | **~150 MB/s** | **~200 MB/s** |
| Zstd (L19) | 25 MB/s | 800 MB/s |

## 4. Deduplication Efficiency (v5.1+)
*   On `data/` folder (~10-20MB mixed): ~40% reduction via CDC.

## 5. Swarm Learning (RL Convergence)
*   **Agent:** PPO v4 (20k timesteps, ~637 FPS).
*   **Convergence:** Stable on diverse data; reward ~3.75.
