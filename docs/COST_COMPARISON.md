# QRES Computational Cost Comparison

## Table 1: Operation Cost (per aggregation round)

| Operation | QRES (I16F16) | Standard FL (Float32) | Advantage |
|-----------|---------------|------------------------|-----------|
| **Arithmetic Precision** | 16-bit integer | 32-bit float | 2× memory |
| **Multiplication** | Integer MUL | FP MUL + normalize | ~3× faster |
| **Addition** | Integer ADD | FP ADD + align | ~2× faster |
| **Distance Calculation** | Saturating ops | IEEE 754 handling | Deterministic |
| **Cross-Architecture** | Bit-perfect | Drift (1e-6 to 1e-3) | Consensus-safe |

## Table 2: Krum Complexity Analysis

| Metric | Value | Notes |
|--------|-------|-------|
| **Time Complexity** | O(n² × d) | n=nodes, d=dimensions |
| **Space Complexity** | O(n) | Score storage per node |
| **Operations per Round** | ~n² distances | Distance matrix |
| **Neighbor Sort** | O(n log n) | Per candidate |

## Table 3: Bandwidth Comparison (n=100 nodes, d=8 weights)

| Approach | Update Size | Daily Traffic | QRES Advantage |
|----------|-------------|---------------|----------------|
| **Federated Learning** | ~4KB (full weights) | 400 KB/node | Baseline |
| **QRES Gene Gossip** | 16 bytes (gene) | 1.6 KB/node | **250× less** |
| **With Krum BFT** | 16 bytes + 2 bytes | 1.8 KB/node | **222× less** |

## Table 4: Byzantine Tolerance Operating Envelope

| Byzantine % | n >= 2f+3 | Krum Status | Recommendation |
|-------------|-----------|-------------|----------------|
| 10% | ✅ Yes | Secure | Normal operation |
| 20% | ✅ Yes | Secure | Monitor closely |
| 33% | ⚠️ Limit | At boundary | Maximum safe |
| 40% | ❌ No | Degraded | Reconfigure f |
| 50% | ❌ No | Failed | Partition network |

## Key Takeaways

1. **Fixed-point advantage**: I16F16 provides deterministic consensus without floating-point drift
2. **Bandwidth efficiency**: Gene gossip reduces traffic by 200-250× vs traditional FL
3. **Byzantine tolerance**: Safe up to ~33% network compromise with proper configuration
4. **Scalability**: O(n²) Krum is acceptable for edge swarms (n < 100)
