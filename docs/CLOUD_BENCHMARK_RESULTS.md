# Cloud Edge Benchmark Results (Azure B1ls)

**Date:** 2026-01-12
**Environment:** Azure Standard_B1ls (1 vCPU, 0.5 GiB RAM)
**OS:** Ubuntu 24.04 LTS

## 1. Executive Summary
We deployed the QRES `ResourceUsagePredictor` to a highly constrained cloud environment to simulate an edge IoT gateway. The system successfully compiled and executed under strict memory limits (Swap required for compilation, but not for execution).

## 2. Performance Metrics

| Metric | Value | Notes |
| :--- | :--- | :--- |
| **Peak RAM Usage** | **16.12 MB** | Fits easily on 512MB Pi Zero |
| **Heuristic Latency** | 0.04 µs | Near-instantaneous |
| **Neural Latency** | 1,707 µs | ~1.7ms per inference |
| **Hybrid Latency** | **845 µs** | **50% Reduction** vs Pure Neural |
| **CPU Usage** | 99% | Efficient single-core utilization |

## 3. Cost Analysis
The benchmark ran on the cheapest Azure instance available (~$3.80/month).
* **Compilation:** Required 16m 31s (with `-j1` flag).
* **Inference:** Stable at ~1200 predictions/second (Hybrid mode).

## 4. Conclusion
QRES is verified "Edge Ready." The memory footprint (~16MB) is negligible, and the Hybrid Predictor successfully scales down to weak hardware, maintaining sub-millisecond average latency.
