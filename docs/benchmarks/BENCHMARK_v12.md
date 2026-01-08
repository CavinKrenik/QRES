# QRES v12.0 Benchmark Report

## Test Environment

| Property | Value |
|----------|-------|
| **Hardware** | Intel Ice Lake (AWS c6i.4xlarge) |
| **Version** | QRES v12.0 (Swarm Scaling Era) |
| **Agent** | MetaBrain v5 (SNN + Tensor Predictor) |

---

## Federated Swarm Synchronization

### Zero-Bandwidth Sync Performance

| Nodes | Epochs | Total Time | Avg/Epoch | Sync Rate |
|-------|--------|-----------|-----------|-----------|
| **10** | 50 | 0.50ms | 0.01ms | **100%** |
| **10** | 20 | 0.44ms | 0.02ms | **100%** |
| **3** | 10 | 0.15ms | 0.02ms | **100%** |

### Bandwidth Comparison

| Approach | Daily Bandwidth (1000 nodes) |
|----------|------------------------------|
| Full weight sync | **2.3 GB/day** |
| QRES PRNG seed sync | **8 KB/day** |

*99.996% bandwidth reduction via deterministic seed-based synchronization.*

---

## Regime Change Resilience

### Frequency Shift Test
Dataset: Sine wave with frequency shift at midpoint

| Time | Pattern | QRES Ratio |
|------|---------|-----------|
| 0-100s | 10Hz sine | 8.2x |
| 100s | Shift to 50Hz | 1.9x (degradation) |
| 100-120s | Learning | 3.5x (recovering) |
| 120s+ | Adapted | 7.8x (restored) |

### Recovery Comparison

| Compressor | Avg Ratio | Shift Penalty | Recovery Time |
|------------|-----------|---------------|---------------|
| Zstd | 2.1x | None | N/A |
| QRES (solo) | 5.8x | -60% | 48 hours |
| QRES (swarm) | 6.2x | -40% | 12 hours |

---

## Compression Ratios

### IoT Telemetry

| Dataset | Size | Compressed | Ratio |
|---------|------|------------|-------|
| iot_trending.dat | 15MB | 7.7MB | **0.489** |
| iot_anomaly.dat | 15MB | 11.6MB | 0.735 |
| iot_correlated.dat | 15MB | 13.3MB | 0.846 |
| iot_mixed.dat | 15MB | 7.4MB | **0.473** |

### Cross-Format

| Format | QRES v12 | Zstd (L19) | Winner |
|--------|----------|------------|--------|
| Text/Code | ~0.19 | 0.355 | QRES |
| IoT Telemetry | ~0.49 | 0.45 | Zstd* |
| WAV Audio | ~0.6 | ~0.8 | QRES |

*\*Zstd slightly better on diverse IoT; QRES better on stable patterns.*

---

## Neural Metrics

| Metric | Value |
|--------|-------|
| **SNN Sparsity** | 97% (OSBC pruning) |
| **Tensor Dimensions** | 4 |
| **Training FPS** | ~500 |
| **Fidelity** | >0.99 |

---

## Known Issues

- **Pure Sine Waves:** Q16.16 quantization limits ratio to ~0.47
- **High Entropy:** Incompressible data returns ratio 1.0 (passthrough)
- **Small Files:** Header overhead makes files <1KB inefficient

---

*Benchmarks run on standardized test corpus. Raw data in `benchmarks/` directory.*
