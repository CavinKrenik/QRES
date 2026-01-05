# Tensor Network Correlation Analysis (TNC)

**Formerly "Quantum VQC"**

## Overview
Standard compression relies on statistical probability (Huffman/FSE) or linear history (LZ77). QRES TNC employs **Matrix Product States (MPS)**—a concept borrowed from quantum many-body physics—to model non-linear, long-range correlations in data streams without requiring quantum hardware.

## The Engineering Thesis
In a byte stream, standard Markov chains model $P(x_t | x_{t-1})$. Tensor Networks model the entire sequence as a high-dimensional state vector $|\Psi\rangle$ living in a Hilbert space, decomposed into low-rank tensors:

$$|\Psi\rangle \approx \sum_{i_1, i_2, ...} \text{Trace}(A^{[1]}_{i_1} A^{[2]}_{i_2} \dots A^{[N]}_{i_N}) |i_1 i_2 \dots i_N\rangle$$

### Why This Matters for Compression
1.  **"Entanglement" as Correlation:** In physics, entanglement describes strong correlations between particles. In QRES, we treat widely separated bytes (e.g., a JSON opening brace and its closing brace 4KB later) as "entangled."
2.  **Dimensionality Reduction:** By limiting the "Bond Dimension" ($\chi$) of the tensors, we force the model to learn only the most salient features of the data, filtering out noise.
3.  **Fixed-Point Execution:** Unlike actual Quantum Circuits which are probabilistic and noisy, our TNC is simulated using Q16.16 fixed-point arithmetic on CPU. It is deterministic, stable, and bit-perfect across architectures.

## Implementation Details
* **Engine:** `qres_core::tensor` (Pure Rust)
* **Structure:** 1D Tensor Train (TT-Decomposition)
* **Contraction:** Greedy optimization path for $O(N \chi^3)$ complexity.
* **Optimization:** SIMD-accelerated linear algebra (no GPU requirement).

## Benchmark Reality
TNC excels at "Structural Inference"—predicting valid syntax in code (JSON/XML/Rust) where relationships are strict but spatially distant. It is less effective on high-entropy streams (compressed video/encrypted data), where we fallback to standard entropy coding.
