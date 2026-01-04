# QRES Research Notes & Citations (v9.0)

## 1. Context-Aware Neural Compression (LLM Integration)
**Goal:** Outperform traditional entropy coders on source code by leveraging semantic understanding.

### Relevant Papers
*   **"Language Models are Universal Compressors"** (Delétang et al., 2024): Demonstrates that LLMs can achieve compression ratios significantly better than Gzip on text by using next-token prediction probabilities as entropy model distributions.
    *   *Insight for QRES:* Used in MetaBrain v4 for multimodal prediction.

## 2. Attention Mechanisms for Byte Streams
*   **"Linear Transformers for Long-Range Interaction"** (Katharopoulos et al., 2020): O(N) attention via kernel maps.
    *   *QRES Implementation:* In `transformer.rs`; extended for binary/multimodal.

## 3. Distributed Swarm Learning
*   **"Federated Optimization in Heterogeneous Networks"** (FedProx, Li et al., 2018): Proximal term for non-IID data.
    *   *QRES Implementation:* In swarm merging; supports MetaBrain sharing.
*   **"Fed2Com: Compressed Federated Learning"** (ICNC 2024): Delta encoding for 70% bandwidth reduction.
    *   *QRES v8.1:* Implemented in `swarm_cli.py:delta_compress()`.
*   **"KL-FedDis: Divergence-Aware Federated Distillation"** (2024): Reject high-divergence updates.
    *   *QRES v8.1:* Implemented in `hive_mind.py:compute_kl_divergence()`.

## 4. Reinforcement Learning for Compression
*   **"Implementation Matters in Deep Policy Gradient Methods"** (Engstrom et al., 2021): PPO best practices.
    *   *Insight:* Guided MetaBrain v4 training (vec envs, reward shaping).

## 5. Multimodal Embeddings
*   **"Learning Transferable Visual Models From Natural Language Supervision"** (Radford et al., 2021 - CLIP): Shared embeddings for images/text.
    *   *QRES v8:* Used in training for diverse data generalization.

## 6. Hardware Acceleration
*   *Idea:* `wgpu` for FFT in SpectralPredictor; extended for RL inference.

---

## 7. Spiking Neural Networks (v9.0 - NEW)

*   **"SpikeLLM: Scaling Up Spiking Neural Networks with Generalized Integrate-and-Fire"** (ICLR 2025): GIF neurons for scalable spiking architectures.
    *   *QRES v9.0:* Upgraded `snn_predictor.py` with `GIFNeuron` for adaptive thresholds.
*   **"Optimal Brain Spiking Compression (OSBC)"** (OpenReview 2025): Second-order pruning for 97% sparsity.
    *   *QRES v9.0:* Implemented in `prune_second_order()` method.
*   **"Energy-Efficient Intelligence: SNN Survey"** (Oulu Univ., Oct 2025): 1,000-10,000x energy reduction vs ANNs.
    *   *Insight:* Edge viability for QRES IoT deployment.

## 8. Quantum Machine Learning Compression (v9.0 - NEW)

*   **"Equivariant Quantum Operator Compression"** (NeurIPS 2025): Preserve O(3) symmetries in quantum tensors.
    *   *QRES v9.0:* Implemented in `qnn_vqc.py:equivariant_lattice()`.
*   **"Quantum Autoencoders for Hidden Subgroup Compression"** (QuantumZeitgeist, Nov 2025): Data reduction via symmetry groups.
    *   *Insight:* Future work for v10.0 true quantum hardware integration.

