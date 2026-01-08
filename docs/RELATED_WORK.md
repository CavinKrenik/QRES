# Related Work

This document surveys relevant literature and positions QRES within the federated learning landscape.

---

## 1. Federated Learning Foundations

### FedAvg (McMahan et al., 2017)
- **Key Idea:** Clients train locally for multiple epochs, server averages weights
- **Limitation:** Assumes IID data, no privacy/Byzantine tolerance
- **Relation to QRES:** QRES extends with Krum aggregation and DP

### FedProx (Li et al., 2020)
- **Key Idea:** Adds proximal term to handle heterogeneous data
- **Limitation:** Still no security guarantees
- **Relation to QRES:** QRES targets edge deployment where FedProx is too heavy

---

## 2. Secure Aggregation

### Bonawitz et al. (2017)
- **Key Idea:** Pairwise masking with secret sharing for dropout tolerance
- **Implementation:** Used in Google's Gboard
- **Relation to QRES:** QRES uses simplified pairwise X25519 masking (Phase 3)

### Bell et al. (2020)
- **Key Idea:** Improved efficiency via structured random seed agreement
- **Relation to QRES:** Potential future optimization

---

## 3. Byzantine-Tolerant Aggregation

### Krum (Blanchard et al., 2017)
- **Key Idea:** Select update with minimum distance to neighbors
- **Tolerance:** f < (n-2)/2
- **Relation to QRES:** Implemented in v14

### Trimmed Mean / Median (Yin et al., 2018)
- **Key Idea:** Coordinate-wise robust statistics
- **Tolerance:** f < n/2
- **Relation to QRES:** Implemented as alternatives to Krum

### Byzantine-Resilient SGD (El-Mhamdi et al., 2020)
- **Key Idea:** Combines Krum with momentum
- **Relation to QRES:** Potential enhancement for v16

---

## 4. Differential Privacy in FL

### DP-SGD (Abadi et al., 2016)
- **Key Idea:** Clip gradients, add Gaussian noise
- **Relation to QRES:** Core privacy mechanism in v15

### User-Level DP (Geyer et al., 2017)
- **Key Idea:** Privacy per user, not per sample
- **Relation to QRES:** QRES implements node-level DP

### Federated DP (Kairouz et al., 2021)
- **Key Idea:** Distributed noise with central DP guarantees
- **Relation to QRES:** Future integration possible

---

## 5. Spiking Neural Networks

### Theoretical Foundations (Maass, 1997)
- **Key Idea:** SNNs are computationally universal
- **Relation to QRES:** Justifies SNN choice for edge

### Neuromorphic Computing (Pfeiffer & Pfeil, 2018)
- **Key Idea:** Event-driven, low-power computation
- **Relation to QRES:** Aligns with edge deployment goals

### SNNs for Time-Series (Shrestha & Orchard, 2018)
- **Key Idea:** Temporal coding natural for sequential data
- **Relation to QRES:** Core predictor architecture

---

## 6. Edge Federated Learning

### TinyFL (Mills et al., 2019)
- **Key Idea:** FL on microcontrollers
- **Limitation:** Limited model size
- **Relation to QRES:** QRES targets similar devices with compression

### Edge FL Surveys (Kairouz et al., 2019)
- **Key Idea:** Comprehensive FL challenges
- **Relation to QRES:** Positions QRES in the landscape

---

## 7. FL Frameworks Comparison

| Feature | QRES | FedML | Flower | PySyft | TFF |
|---------|------|-------|--------|--------|-----|
| **Edge/no_std** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Deterministic** | ✅ Q16.16 | ❌ | ❌ | ❌ | ❌ |
| **Byzantine Tol.** | ✅ Krum | ❌ | ✅ | ❌ | ❌ |
| **Differential Privacy** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Secure Aggregation** | ✅ | ✅* | ✅* | ✅ | ✅* |
| **ZK Proofs** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **WASM Support** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Bio-Inspired** | ✅ SNN | ❌ | ❌ | ❌ | ❌ |

*Limited or plugin-based

---

## 8. Key Differentiators

QRES is unique in combining:

1. **Edge-Native Design:** `no_std` Rust, Q16.16 fixed-point, WASM
2. **Complete Security Stack:** Auth + Byzantine + DP + SecAgg + ZK
3. **Bio-Inspired Architecture:** SNNs, swarm synchronization, neural plasticity
4. **Deterministic Reproducibility:** Bit-exact compression across platforms

No existing framework provides all four properties simultaneously.

---

## References

See `references.bib` for full BibTeX entries.
