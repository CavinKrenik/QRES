# ADR-004: Adversarial Hardening (Trimmed Mean & BFP-16)

## Status
Accepted (v19.0)

## Context
During the "Adversarial Hardening" phase (Experiments 1-4), we identified two critical vulnerabilities in the QRES v18.0 architecture:

1. **Drift Vulnerability (Attack.md):** The Multi-Krum consensus algorithm, while robust against outliers, was falsified by "Inlier Bias Attacks" (Experiment 1). Attackers operating within 1.5$\sigma$ of the honest distribution could shift the global model by 5% over 50 rounds, as Krum selects a single "valid" update rather than averaging, making it susceptible to choosing a biased vector.

2. **Vanishing Gradient Precision (Experiment 3):** The `I16F16` fixed-point format (min step $1.5 \times 10^{-5}$) proved insufficient for fine-tuning. At learning rates below $1 \times 10^{-4}$, 100% of gradient updates underflowed to zero, halting convergence.

## Decision

### 1. Adopt Coordinate-wise Trimmed Mean
We replaced Multi-Krum with a **Coordinate-wise Trimmed Mean** (`TrimmedMeanByz`) aggregator.
- **Logic:** For each dimension $d$, sort the values from all $n$ peers, remove the top $f$ and bottom $f$ values, and average the rest.
- **Param:** $f_{byz} < n/2$ (typically $f < n/3$ for provable safety).
- **Benefit:** Provides statistical robustness against outliers while utilizing the information from all "honest" survivors, unlike Krum's single-vector selection.

### 2. Introduce Block Floating Point (BFP-16)
We introduced a custom `Bfp16Vec` encoding for gradient transmission.
- **Format:** One shared 8-bit signed exponent per vector (block), and $N$ 16-bit signed integer mantissas.
- **Dynamics:** $Value_i = Mantissa_i \times 2^{SharedExponent}$.
- **Benefit:** Maintains the range of `f32` (via exponent) while keeping the storage density of `i16`. Solves the vanishing gradient problem (Experiment 4 verified 0% zero-rate at LR $10^{-5}$).

## Consequences

### Positive
- **Convergence:** MNIST training converges stably at LR $10^{-5}$ (verified in `tools/mnist_real_world.py`).
- **Drift Tolerance:** Significantly improved over Krum, though perfect immunity to inlier bias is theoretically impossible without an external trust anchor.
- **Bandwidth:** BFP-16 adds only 1 byte (exponent) overhead per vector.

### Negative
- **Quantization Noise:** BFP-16 introduces quantization error relative to full `f32`.
- **Drift Residue:** The "Golden Run" simulation shows random drift events (~10% probability) where intelligent attackers can still influence the mean slightly. This is deemed an acceptable trade-off for the massive gain in convergence stability.

## References
- `crates/qres_core/src/aggregation.rs`
- `crates/qres_core/src/consensus/krum.rs`
- `tools/golden_run_v19.py`
