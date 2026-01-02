
# QRES v5 Performance Benchmarks

**Hardware:** AWS c6i.4xlarge (Intel Ice Lake)
**Corpus:** IoT-Drift (Generic Sensor Data), Shakespeare (Text)

## 1. Compression Ratio (Lower is Better)

| Engine | IoT-Drift Ratio | Text Ratio |
| :--- | :---: | :---: |
| **QRES v5.1** | **0.081** | **0.322** |
| QRES v4.0 | 0.095 | 0.380 |
| Zstd (L19) | 0.124 | 0.355 |
| LZMA (7z) | 0.118 | 0.340 |
| Gzip (L9) | 0.280 | 0.421 |

**Analysis:**
QRES v5.1 outperforms Zstd by **35%** on IoT data due to its ability to model non-linear sensor drift using the Spectral Predictor. On text, the new `DedupEngine` combined with the Graph Predictor edges out Zstd and LZMA.

## 2. Throughput (MB/s)

| Engine | Compression Speed | Decompression Speed |
| :--- | :---: | :---: |
| **QRES v5.1** | **180 MB/s** | **220 MB/s** |
| Zstd (L19) | 25 MB/s | 800 MB/s |
| LZ4 | 800 MB/s | 4500 MB/s |

**Analysis:**
While slower than LZ4, QRES provides a "Goldilocks" balance, offering archival-grade compression at speeds acceptable for real-time network transmission (1Gbps+). The Neural-Symbolic approach is computationally heavier than LZ4's simple dictionary match but significantly lighter than deep learning alternatives.

## 3. Deduplication Efficiency

On a dataset of 100 log files with 50% redundancy:

- **Original Size:** 100 MB
- **Deduplicated Size:** 52 MB (Reference Chunks)
- **Final Compressed Size:** 4.1 MB

The CDC engine successfully identified 98% of the duplicate content across file boundaries.
