# QRES Performance Benchmarks

Performance metrics for the Quantum-Relational Encoding System.

---

## Test Environment

| Property | Value |
|----------|-------|
| **Hardware** | Intel Ice Lake (AWS c6i.4xlarge) |
| **Version** | QRES v10.0 |
| **Agent** | MetaBrain v5 (SNN+QNN) |

---

## Compression Ratio

*Lower is better. Ratio = Compressed Size / Original Size.*

| Dataset | QRES v9.0 | Zstd (L19) | Notes |
|---------|-----------|------------|-------|
| **IoT Telemetry** (20MB) | **0.537** | 0.124 | Adaptive prediction |
| **Text/Code** | **~0.19** | 0.355 | 46% better than Zstd |
| **PDF Documents** | ~0.9 | ~0.95 | Already compressed |
| **WAV Audio** | ~0.6 | ~0.8 | Spectral benefits |

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

---

*Benchmarks run on standardized test corpus. See `benchmarks/` for raw data.*
