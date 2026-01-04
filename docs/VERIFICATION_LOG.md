# QRES Verification Log

## Breakthrough 1: IoT Compression & Interleave Smart-Splitting
**Date:** Jan 3, 2026
**Component:** `qres_rust/src/lib.rs` (Interleave Detection) & `qres_rust/src/spectral.rs` (Detrending)

### Impact Analysis
- **Metric:** Compression Ratio on `iot_telemetry.dat` (20MB interleaved signal).
- **Baseline (QRES v6):** 61.37% (Fails to beat Zstd @ 57%).
- **New Result (QRES v7):** **51.42%** (Beats Zstd by ~10%).
- **Status:** **CONFIRMED BREAKTHROUGH**.

### Methodology
1. **Diagnosis:** Identified that single-stream predictors fail on interleaved data (Temp/Vib) due to high-freq switching noise.
2. **Solution:** 
   - Added `0x03` codec flag for "Smart Split": If Lag-2 variance < 70% of Lag-1, recursively compress Even/Odd streams.
   - Added Linear Detrending to `SpectralPredictor` to handle the slow drift in the Temperature channel.
3. **Verification:**
   - `benchmarks/iot_benchmark.py`: Confirmed 51.42% ratio.
   - `tests/verify_iot_fidelity.py`: Confirmed SHA256 match on 20MB file.

### Next Steps
- Address throughput (currently ~1.5 MB/s).
- Proceed to Phase 2 (Quantum Tensors).
