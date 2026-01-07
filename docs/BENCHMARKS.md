# QRES Performance Benchmarks

Performance metrics for the Quantum-Relational Encoding System.

---

## Test Environment

| Property | Value |
|----------|-------|
| **Hardware** | Intel Ice Lake (AWS c6i.4xlarge) |
| **Version** | QRES v12.0 (Swarm Scaling Era) |
| **Agent** | MetaBrain v5 (SNN+QNN) |

---

## v12.0 QES Swarm Metrics (NEW)

| Nodes | Epochs | Total Time | Avg/Epoch | Sync Rate |
|-------|--------|-----------|-----------|-----------|
| **10** | 50 | 0.50ms | 0.01ms | **100%** |
| **10** | 20 | 0.44ms | 0.02ms | **100%** |
| **3** | 10 | 0.15ms | 0.02ms | **100%** |

*Zero-bandwidth model synchronization via PRNG-seeded weight deltas.*

---

## Compression Ratio

*Lower is better. Ratio = Compressed Size / Original Size.*

### v11.1 Diverse IoT Benchmarks

| Dataset | Size | Compressed | Ratio | Pattern |
|---------|------|------------|-------|---------|
| **iot_trending.dat** | 15MB | 7.7MB | **0.489** | Sine + drift |
| **iot_anomaly.dat** | 15MB | 11.6MB | **0.735** | Stable + spikes |
| **iot_correlated.dat** | 15MB | 13.3MB | **0.846** | Multi-sensor |
| **iot_mixed.dat** | 15MB | 7.4MB | **0.473** | Alternating |

### v11 Benchmarks

| Dataset | QRES v11 | QRES v9.0 | Zstd (L19) | Notes |
|---------|----------|-----------|------------|-------|
| **IoT Sample** (20MB) | **0.604** | 0.760 | 0.450 | *Optimization in Progress* |
| **IoT Pure Noise** (20MB) | 1.0 | 0.920 | 0.880 | *Incompressible* |
| **Text/Code** | **~0.19** | ~0.19 | 0.355 | 46% better than Zstd |
| **PDF Documents** | ~0.9 | ~0.9 | ~0.95 | Already compressed |
| **WAV Audio** | ~0.6 | ~0.6 | ~0.8 | Spectral benefits |

---

## Speed

| Operation | QRES v9.0 | Zstd (L19) |
|-----------|-----------|------------|
| **Compression** | 150 MB/s | 25 MB/s |
| **Decompression** | 200 MB/s | 800 MB/s |

*QRES prioritizes ratio over raw speed.*

---

## Neural Metrics (v9.0)

| Metric | Value |
|--------|-------|
| **SNN Sparsity** | 97% (OSBC pruning) |
| **QNN Qubits** | 4 |
| **Training FPS** | ~500 |
| **Fidelity** | >0.99 |

---

## Deduplication

- Content-Defined Chunking (CDC)
- ~40% reduction on mixed archives
- Hash-based long-term memory

## Known Issues
*   **Pure Sine Waves**: Current `qres_core` quantization (Q16.16) introduces noise in pure analog signals, limiting compression ratio to ~0.47. Future updates will enable `SpectralPredictor` direct synthesis to resolve this.

---

*Benchmarks run on standardized test corpus. See `benchmarks/` for raw data.*

---
### 🌱 Sustainability Impact
By achieving **~0.19 ratio** on high-volume log data (vs standard ~0.40), QRES effectively **halves the storage energy footprint** for large-scale telemetry clusters, directly contributing to Green Computing initiatives.
