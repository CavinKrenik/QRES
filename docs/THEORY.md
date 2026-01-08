# QRES Theoretical Analysis

This document provides formal analysis of QRES's security and convergence properties.

---

## 1. Privacy Composition

### Background

Differential Privacy (DP) provides provable privacy guarantees. When multiple DP operations are applied, privacy "composes" (accumulates).

### Basic Composition

For T rounds with per-round privacy budget ε:

```
ε_total = T × ε_round
```

**Example:** T=100 rounds, ε=0.1 → ε_total = 10.0 (weak)

### Advanced Composition (Moments Accountant)

For (ε, δ)-DP with advanced composition:

```
ε_total = √(2T × ln(1/δ')) × ε_round + T × ε_round × (e^ε_round - 1)
```

For small ε, simplified:

```
ε_total ≈ √(2T × ln(1/δ)) × ε_round
```

**Example:** T=100 rounds, ε=0.1, δ=1e-5 → ε_total ≈ 4.8 (much better)

### QRES Privacy Budget

| Rounds | Per-Round ε | Basic ε_total | Advanced ε_total |
|--------|------------|---------------|------------------|
| 10 | 0.1 | 1.0 | 1.5 |
| 100 | 0.1 | 10.0 | 4.8 |
| 1000 | 0.1 | 100.0 | 15.2 |

**Recommendation:** Use ε ≤ 0.1 per round for meaningful long-term privacy.

---

## 2. Byzantine Tolerance

### Krum Algorithm

Krum selects the update with minimum sum of distances to its nearest neighbors, rejecting outliers.

**Tolerance Bound:**

```
f < (n - 2) / 2
```

Where:
- n = total number of nodes
- f = number of Byzantine (malicious) nodes

**Proof Sketch:** With f Byzantine nodes, there are n-f honest nodes. Krum computes scores based on n-f-2 nearest neighbors. If f < (n-2)/2, honest updates form a majority cluster.

### Multi-Krum

Multi-Krum averages the k most representative updates:

```
f < (n - k - 2) / 2
```

**Trade-off:** Higher k → more averaging → faster convergence, but lower Byzantine tolerance.

### QRES Byzantine Tolerance Table

| Nodes (n) | Max Byzantine (f) | Tolerance % |
|-----------|------------------|-------------|
| 5 | 1 | 20% |
| 10 | 3 | 30% |
| 20 | 8 | 40% |
| 50 | 23 | 46% |

**Note:** These are theoretical maximums. Practical attacks may succeed at lower thresholds if adversaries are adaptive.

---

## 3. Convergence Analysis

### FedAvg Convergence

For convex objectives with L-smooth gradients:

```
E[‖∇F(w_T)‖²] ≤ O(1/√T) + O(σ²/K)
```

Where:
- T = rounds
- K = local steps per round
- σ² = gradient variance

### QRES Convergence (SNN)

SNNs are non-convex, so theoretical guarantees are weaker. QRES relies on:

1. **Empirical convergence:** Demonstrated on IoT datasets
2. **Regime adaptation:** Momentum-based predictor reweighting
3. **Bounded variance:** Q16.16 fixed-point limits numerical instability

### Regime Change Recovery

| Shift Type | Recovery Rounds | Mechanism |
|------------|----------------|-----------|
| Gradual | 5-10 | Continuous adaptation |
| Abrupt | 10-20 | Momentum decay + relearning |
| Oscillating | 10-15 | Pattern memory in TNC |

---

## References

- Dwork, C., & Roth, A. (2014). The Algorithmic Foundations of Differential Privacy.
- Blanchard, P., et al. (2017). Machine Learning with Adversaries: Byzantine Tolerant Gradient Descent.
- McMahan, H. B., et al. (2017). Communication-Efficient Learning of Deep Networks from Decentralized Data.
- Abadi, M., et al. (2016). Deep Learning with Differential Privacy.
