# QRES Progress Log

## Cycle 1: Phase 1 Validation (Jan 3, 2026)
**Focus:** IoT Data Compression & RL Adaptation

### Challenges
1. **IoT Benchmark Gap:**
   - Target: <50% ratio (beating Zstd @ 57%).
   - Actual: 61.37%.
   - **Root Cause Analysis:** The IoT data is interleaved (Temp/Vib). The `SpectralPredictor` processes it as a single 1D stream. While `GraphPredictor` has Lag-2 capability, the default Mixer (AR2) isn't converging fast enough or the weights aren't favoring the Graph model.
   - **Attempted Fix:** Implemented linear detrending in `spectral.rs`. Result: No significant change (61.37%). This confirms the issue isn't just drift, but structural interleaving.

2. **RL Agent Convergence:**
   - PPO Agent stuck at ~81% ratio.
   - **Hypothesis:** The reward signal (improvement over baseline) is too sparse or the environment (`CompressionMixingEnv`) generates chunks that are too small/random for the predictor state to warm up effectively.

### Solutions & Tuning Plan
- **De-interleaving Strategy:** QRES v7.0 needs a "Smart Pre-Pass" (as hinted in lib.rs) to detect interleaving and split streams, OR we need the Graph Predictor to start with higher weights for Lag-2.
- **RL Environment:** Increase chunk size in `rl_mixer_env.py` (currently 4KB) or carry over state to allow predictors to learn.

### Decisions
- Documented v7.0 performance as "Pre-Alpha" in `BENCHMARK_v7.md`.
- Will proceed to Phase 2 (Quantum) to see if Tensor Networks handle the interleaving naturally (MPS is great for this).

## Experiment 2: Text Compression Tuning (Jan 3, 2026)
**Feature:** Text Prediction (SimplePredictor Order-2, LzMatch Hash Fix)

### Outcome
- **Baseline (V6):** 91.70% ratio on text_1mb.txt.
- **Improved (V7):** 91.36% ratio.
- **Analysis:** Simple predictive coding + ANS is insufficient for text. Residuals are not Laplacian. Text requires FSE (Finite State Entropy) or explicit Dictionary Coding (zstd style) or LLM-based token probabilities (not byte residuals).
- **Decision:** Deferred to Phase 2 (Quantum/Tensor) or separate Entropy Coding overhaul. The code improvements (Order-2, Better Hash) are kept as they are technically superior, even if the gain is marginal.


## Experiment 3: Quantum/Structured Compression (Jan 3, 2026)
**Feature:** MpsCompressor (Haar Wavelet Transform) for floating point tensors.

### Outcome
- **Dataset:** 1024x1024 Rank-5 Correlated Matrix (8MB).
- **Zstd:** 95.14% Ratio (Fails to compress floats).
- **QRES (Quantum v7.5):** 64.62% Ratio (Sparse Approx).
- **Result:** **30% Improvement over Zstd**. Validates Structured Data capabilities.
- **Next Steps:** Implement true MPS (SVD-based) with Candle or ndarray-linalg for higher ratios on non-smooth data.

