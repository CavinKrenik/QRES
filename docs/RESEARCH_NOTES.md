# QRES Research Notes & Citations (v8.0)

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

## 4. Reinforcement Learning for Compression
*   **"Implementation Matters in Deep Policy Gradient Methods"** (Engstrom et al., 2021): PPO best practices.
    *   *Insight:* Guided MetaBrain v4 training (vec envs, reward shaping).

## 5. Multimodal Embeddings
*   **"Learning Transferable Visual Models From Natural Language Supervision"** (Radford et al., 2021 - CLIP): Shared embeddings for images/text.
    *   *QRES v8:* Used in training for diverse data generalization.

## 6. Hardware Acceleration
*   *Idea:* `wgpu` for FFT in SpectralPredictor; extended for RL inference.
