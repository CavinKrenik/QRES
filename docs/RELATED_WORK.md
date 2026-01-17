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

While QRES builds on the foundations above, it introduces several **novel architectural patterns** that constitute genuine systems-level innovations for **Adversarial Edge Environments**. These are not incremental improvements—they represent fundamental re-designs required for swarm intelligence at the edge.

### A. Consensus-First Determinism (`Q16.16`) — *"Math as Law"*

**Problem:** Most FL frameworks treat floating-point non-determinism as a minor noise source. In reality, even 1 bit of drift compounds catastrophically across a swarm, making consensus verification impossible.

**Innovation:** QRES implements a custom `Q16.16` fixed-point arithmetic engine from scratch in `no_std` Rust. Every mathematical operation—sin, cos, exp, sqrt—produces **bit-identical results** regardless of hardware: `result_x86 == result_arm == result_risc_v`.

**Why It Matters:**
- Model states become **Merkle Trees**. Nodes verify synchronization instantly via hashes.
- Eliminates complex reconciliation protocols required by float-based systems.
- Enables **cryptographic proofs** of consensus that are legally auditable.

### B. Lamarckian "Hippocampus" Persistence — *"Memories Survive Death"*

**Problem:** Standard Evolutionary Strategies (ES) are Darwinian: agents die, and only their offspring inherit traits. This is inefficient for IoT devices that frequently reboot due to power instability.

**Innovation:** The **Hippocampus** layer (implemented via the `GeneStorage` trait) enables **Lamarckian Evolution**. Nodes serialize their "learned instincts" (bytecode) to non-volatile storage before shutdown.

**Why It Matters:**
- A swarm survives **total power failure** and resumes evolution exactly where it left off.
- Prevents "Knowledge Collapse" in solar/battery-powered deployments.
- Creates continuity of intelligence across hardware replacements—a new node inherits the swarm's collective memory.

### C. Prediction-as-Consensus (Proof-of-Understanding) — *"Compression is Intelligence"*

**Problem:** How do you establish trust in a decentralized swarm without a central authority? Proof-of-Work (hashing) wastes energy and proves nothing about model quality.

**Innovation:** QRES reframes **compression and intelligence as identical problems**. Instead of solving a PoW puzzle, nodes provide a **Proof-of-Understanding** by compressing sensor data. A node that broadcasts a small residual packet proves it has a superior predictive model.

**Why It Matters:**
- High compression ratios serve as an **unforgeable metric of intelligence**.
- The swarm automatically weights "smarter" nodes higher during aggregation.
- Eliminates the need for trusted oracles or central coordinators.
- Creates a natural incentive structure: better predictions = more influence.

### D. Emergent Gene Gossip under Physics Constraints — *"Evolution in the Wild"*

**Problem:** Existing P2P learning simulations often ignore network physics (MTU limits, packet loss, latency). This leads to unrealistic assumptions about model transfer.

**Innovation:** QRES simulates the **physical "viral" spread** of intelligence. Evolved bytecode ("Genes") must be fragmented into 1400-byte packets to traverse the network. High-entropy noise zones cause packet loss, physically preventing large, complex models from spreading.

**Why It Matters:**
- Creates an **evolutionary pressure for compactness**. The swarm naturally selects for smaller, more efficient models.
- Demonstrates **emergent architectural search**—the network topology shapes the model architecture.
- Hostile environments (noisy radio channels) become a feature, not a bug: they prune bloated models.

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