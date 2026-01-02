# QRES Research Notes & Citations (v6.0 Alpha)

## 1. Context-Aware Neural Compression (LLM Integration)
**Goal:** Outperform traditional entropy coders (Zstd, LZMA) on source code by leveraging semantic understanding.

### Relevant Papers
*   **"Language Models are Universal Compressors" (Delétang et al., 2024):** Demonstrates that LLMs can achieve compression ratios significantly better than Gzip on text by using next-token prediction probabilities as entropy model distributions.
    *   *Insight for QRES:* We can use a small, local LLM (e.g., CodeLlama-7B quantized, or even tiny-llama) to predict the next byte/token. The `Mixer` can treat the LLM as a high-latency, high-accuracy predictor.
    *   *Implementation Strategy:* Python-side predictor using `transformers` + `bitsandbytes`. Since inference is slow, apply only to high-entropy blocks or asynchronously pre-fetch.

## 2. Attention Mechanisms for Byte Streams
**Goal:** Capture long-range dependencies (>1MB) that sliding windows miss.

### Relevant Papers
*   **"Linear Transformers for Long-Range Interaction" (Katharopoulos et al., 2020):** Proposed O(N) attention by using kernel feature maps.
    *   *QRES v5.1 Implementation:* We implemented a simplified "Soft-LZ" attention in `transformer.rs`.
    *   *Improvement:* Use a "Memory Token" approach (like Transformer-XL) to persist state across chunks without re-processing.

## 3. Distributed Swarm Learning
**Goal:** Optimize compression weights without transmitting private data.

### Relevant Papers
*   **"Federated Optimization in Heterogeneous Networks" (FedProx, Li et al., 2018):** Adds a proximal term to the local objective to handle non-IID data.
    *   *QRES Implementation:* The `Mixer` already includes a specific "FedProx" update step (`global_weights` pull). We need to verify the `mu` hyperparameter (currently 0.001) against empirical convergence rates.

## 4. Hardware Acceleration
*   **Idea:** Using `Compel` (Rust GPU compute) or `wgpu` to offload the FFT in `SpectralPredictor`.
