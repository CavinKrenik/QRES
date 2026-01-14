# QRES Security Roadmap (2026)

This document outlines the phased security hardening of the QRES distributed system, transitioning from a trusted research prototype to a robust, adversarial-resistant production network.

> **Implementation Guide:** See [docs/guides/SECURITY_IMPLEMENTATION_GUIDE.md](guides/SECURITY_IMPLEMENTATION_GUIDE.md) for step-by-step dev workflows.

---

## Phase 1: Authentication & Identity (Target v13)
**Focus:** Secure the P2P layer against unauthorized access and tampering in a semi-trusted environment.

- [x] **Item 1: Ed25519 Signatures**
  - **Goal:** Guarantee authenticity of all model updates.
  - **Tech:** `ed25519-dalek` for signing weight buffers.
  - **Attack Mitigation:** Spoofing, Man-in-the-Middle.

- [x] **Item 2: Node PKI (Public Key Infrastructure)**
  - **Goal:** Enforce node identity verification during handshake.
  - **Tech:** `libp2p` PeerId / Noise protocol.
  - **Attack Mitigation:** Sybil attacks (partial).

- [x] **Item 3: Replay Prevention**
  - **Goal:** Prevent attackers from rebroadcasting old valid updates.
  - **Tech:** Nonces + Timestamps in protocol headers.
  - **Attack Mitigation:** Replay attacks.

---

## Phase 1.5: Reputation & Trust (Target v16.5)
**Focus:** Build long-term trust metrics to punish bad actors and reward honest contributors.

- [ ] **Item 1: Long-term Reputation Scoring**
  - **Goal:** Filter out nodes that consistently provide poor or malicious updates.
  - **Tech:** `ReputationManager` (Persistent JSON DB).
  - **Logic:**
    - **Reward:** Trust += 0.01 (accepted update).
    - **Punish:** Trust -= 0.1 (rejected by Krum).
    - **Ban:** Trust < 0.2 (Gatekeeper Block).
  - **Attack Mitigation:** Sleeper agents, intermittent poisoning.

---

## Phase 2: Robust Aggregation (Target v14)
**Focus:** Resilience against Byzantine faults (malicious or faulty nodes) sending bad data.
> **Note:** Integration with Phase 1 (Identity) is currently active via the 'Gatekeeper' logic in v16.5.

- [x] **Item 1: Krum Algorithm**
  - **Goal:** Replace simple averaging with outlier-resistant aggregation.
  - **Tech:** Multi-Krum (selects $n-f-2$ vectors closest to geometric median).
  - **Attack Mitigation:** Model Poisoning (Gaussian noise injection).

- [ ] **Item 2: Trimmed Mean / Median**
  - **Goal:** Statistical robustness for scalar updates.
  - **Tech:** Dimension-wise sorting and trimming.
  - **Attack Mitigation:** Extreme value outliers.

- [ ] **Item 3: Pre-Merge Validation**
  - **Goal:** Filter updates that degrade model performance on a local validation set.
  - **Tech:** "Gatekeeper" check before aggregation.
  - **Attack Mitigation:** Subtle poisoning / Backdoor attacks.

---

## Phase 3: Privacy & Zero-Knowledge (Target v15)
**Focus:** Protecting the confidentiality of raw data and individual updates.

- [ ] **Item 1: Differential Privacy (DP)**
  - **Goal:** Mathematically guarantee bounds on information leakage.
  - **Tech:** Gaussian Mechanism (noise addition) on gradients.
  - **Attack Mitigation:** Membership Inference, Gradient Inversion.

- [ ] **Item 2: Secure Aggregation**
  - **Goal:** Aggregator sees only the sum, not individual updates.
  - **Tech:** Masking protocols (e.g., Bonawitz et al.).
  - **Attack Mitigation:** Honest-but-curious server/peers.

- [ ] **Item 3: Zero-Knowledge Proofs (ZK)**
  - **Goal:** Prove training happened correctly without revealing data.
  - **Tech:** zk-SNARKs (e.g., `bellman` / `halo2`).
  - **Attack Mitigation:** Compute spoofing ("Lazy Worker").

---

## Future Considerations
- Hardware Enclaves (TEE/SGX)
- Homomorphic Encryption
