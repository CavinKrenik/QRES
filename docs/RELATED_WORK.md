# Related Work & Original Contributions

This document positions QRES within the distributed systems landscape and details its specific original contributions to the field of Edge AI.

---

## 1. Federated Learning Foundations

### FedAvg (McMahan et al., 2017)
- **Key Idea:** Clients train locally for multiple epochs, server averages weights.
- **Limitation:** Assumes IID data, high bandwidth, and no Byzantine tolerance.
- **Relation to QRES:** QRES replaces the central server with a gossip protocol and replaces weight averaging with deterministic seed synchronization.

### FedProx (Li et al., 2020)
- **Key Idea:** Adds proximal term to handle heterogeneous data.
- **Limitation:** Still requires heavy runtimes (TensorFlow/PyTorch).
- **Relation to QRES:** QRES solves heterogeneity via "Regime Switching" rather than proximal regularization, allowing it to run on microcontrollers.

---

## 2. Secure Aggregation

### Bonawitz et al. (2017)
- **Key Idea:** Pairwise masking with secret sharing for dropout tolerance.
- **Implementation:** Used in Google's Gboard.
- **Relation to QRES:** QRES implements a lighter, non-interactive variant of pairwise masking using X25519 shared secrets to fit within IoT packet limits.

---

## 3. Byzantine-Tolerant Aggregation

### Krum (Blanchard et al., 2017)
- **Key Idea:** Select update with minimum distance to neighbors to reject outliers.
- **Tolerance:** f < (n-2)/2.
- **Relation to QRES:** QRES implements Krum as a "Gatekeeper" to reject malicious genes before they enter the local population.

---

## 4. Differential Privacy in FL

### DP-SGD (Abadi et al., 2016)
- **Key Idea:** Clip gradients and add Gaussian noise during training.
- **Relation to QRES:** QRES implements Node-Level DP by adding noise to the transmitted gene residuals, ensuring no single node's data can be reconstructed from the swarm's evolution.

---

## 5. Spiking Neural Networks (SNNs)

### Theoretical Foundations (Maass, 1997)
- **Key Idea:** SNNs are computationally universal and energy-efficient.
- **Relation to QRES:** QRES uses a simplified "SwarmNeuron" model inspired by SNNs, where "Surprise" (prediction error) acts as the spike that triggers learning.

---

## 6. Original Contributions in QRES

While QRES builds on the foundations above, it introduces several novel architectural patterns specifically for **Adversarial Edge Environments**.

### A. Consensus-First Determinism (`Q16.16`)
Most FL frameworks treat floating-point non-determinism as a minor noise source. QRES treats it as a **consensus failure**.
* **Innovation:** By implementing a custom `Q16.16` fixed-point arithmetic engine from scratch in `no_std` Rust, QRES guarantees that `result_x86 == result_arm`.
* **Impact:** This allows model states to be treated as **Merkle Trees**. Nodes can verify swarm synchronization instantly via hashes, eliminating the need for complex reconciliation protocols.

### B. Lamarckian "Hippocampus" Persistence
Standard Evolutionary Strategies (ES) are Darwinian: agents die, and only their offspring inherit traits. This is inefficient for IoT devices that frequently reboot.
* **Innovation:** The **Hippocampus** layer (implemented via the `GeneStorage` trait) enables **Lamarckian Evolution**. Nodes serialize their "learned instincts" (bytecode) to non-volatile storage before rebooting.
* **Impact:** A swarm can survive a total power failure and resume evolution exactly where it left off, preventing "Knowledge Collapse" in unstable energy environments.

### C. Prediction-as-Consensus (Proof-of-Understanding)
QRES reframes compression and intelligence as identical problems.
* **Innovation:** Instead of solving a Proof-of-Work puzzle (hashing), nodes provide a **Proof-of-Understanding** by compressing sensor data. A node that broadcasts a small residual packet proves it has a superior predictive model.
* **Impact:** High compression ratios serve as a unforgeable metric of intelligence, allowing the swarm to automatically weight "smarter" nodes higher during aggregation without a trusted central authority.

### D. Emergent Gene Gossip under Physics Constraints
Existing P2P learning simulations often ignore network physics (MTU, packet loss).
* **Innovation:** QRES simulates the physical "viral" spread of intelligence. Evolved bytecode ("Genes") must be fragmented into 1400-byte packets to traverse the simulated network. High-entropy noise zones cause packet loss, physically preventing large, complex models from spreading.
* **Impact:** This creates an **evolutionary pressure for compactness**. The swarm naturally selects for smaller, more efficient models that can survive the hostile network environment, demonstrating emergent architectural search.

---

## 7. Framework Comparison

| Feature | QRES | FedML | Flower | TensorFlow Federated |
|---------|------|-------|--------|----------------------|
| **Primary Target** | **Microcontrollers (Edge)** | Research / Cloud | Mobile / Cloud | Mobile / Cloud |
| **Math Engine** | **Deterministic Q16.16** | Float32 | Float32 | Float32 |
| **Consensus Model** | **Implicit (Seed Sync)** | Central Server | Central Server | Central Server |
| **Persistence** | **Lamarckian (Hippocampus)** | Checkpoints | Checkpoints | Checkpoints |
| **Runtime** | **`no_std` Rust (Bare Metal)** | Python | Python/C++ | Python/C++ |
| **Bandwidth** | **~1KB / update** | MBs / update | MBs / update | MBs / update |

---

## References

See `references.bib` for full BibTeX entries.