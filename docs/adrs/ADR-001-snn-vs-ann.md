# ADR-001: SNN vs ANN for Edge Deployment

**Status:** Accepted  
**Date:** 2026-01-08  
**Context:** Choosing neural network architecture for edge/IoT compression

---

## Decision

Use **Spiking Neural Networks (SNNs)** instead of traditional Artificial Neural Networks (ANNs) for QRES's predictive compression engine.

## Rationale

| Factor | SNN | ANN |
|--------|-----|-----|
| **Determinism** | Spike timing is discrete, easier to make bit-exact | Floating-point accumulation can drift |
| **Power** | Event-driven, low power on neuromorphic hw | Always-on computation |
| **Edge Fit** | Natural for time-series, matches sensor data | Requires batching, more memory |
| **Complexity** | Simpler activation (spike/no-spike) | Backprop, gradients, more state |

## Trade-offs

- **Expressivity:** ANNs can represent more complex functions
- **Ecosystem:** ANN tooling (PyTorch, TensorFlow) is more mature
- **Training:** SNNs are harder to train via gradient descent

## Consequences

- Core engine uses Q16.16 fixed-point SNN
- Training can still use ANNs, converted to SNN weights
- Edge deployment prioritized over cloud GPU optimization
