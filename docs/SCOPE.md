# Scope & Non-Goals: Defining the Negative Space of QRES

A mature systems design is defined as much by what it excludes as by what it enables. Explicitly defining the negative space of QRES prevents scope creep, clarifies its value proposition, and inoculates the system against inappropriate comparisons.

The following are explicit non-goals of QRES.

---

## 1. QRES Is Not a Blockchain

**Misconception**
> "QRES uses consensus and hashes, so it's a blockchain-like ledger for IoT."

**The Reality**
Blockchains optimize for *trustlessness*—they assume participants are adversarial. QRES optimizes for *bandwidth efficiency* under the assumption that the majority of nodes are cooperative but severely resource-constrained.

**Why this matters**
Blockchain consensus mechanisms (Proof of Work, Proof of Stake) deliberately burn compute and bandwidth to establish trust. QRES exists to eliminate unnecessary data movement. Introducing ledger history, global ordering, or economic incentives would directly violate its core design objective.

> QRES nodes do not trust a chain of history; they trust deterministic math and replay.

## 2. QRES Is Not Model Parallelism (Sharding)

**Misconception**
> "With enough nodes, QRES can train arbitrarily large models by splitting them up."

**The Reality**
QRES uses data parallelism, not model parallelism. Every node must hold a complete copy of the model weights in order to deterministically replay training steps and maintain consensus.

**Hard Constraint**
Model size is strictly bounded by the RAM of the smallest device in the swarm. Every node must maintain a complete model copy to deterministically replay training steps from **identical initial conditions**—partial models would break the consensus mechanism entirely. A deployment on ESP32-class hardware (~320 KB RAM) caps the model at roughly that size, regardless of swarm size.

> Adding more nodes increases coverage and learning diversity, not model capacity.

## 3. QRES Is Not a Safety-Critical, Hard Real-Time Controller

**Misconception**
> "QRES can coordinate high-speed physical systems like drone swarms or medical devices."

**The Reality**
QRES provides soft real-time consensus. Silence is used as a signal. Genuine packet loss can temporarily cause divergence until integrity checks detect inconsistency and trigger resynchronization.

**Why this is dangerous**
In safety-critical systems, even brief nondeterministic delays (e.g., a 500 ms resync window) are unacceptable. QRES is designed for learning, optimization, and anomaly detection—not sub-millisecond control loops or life-safety applications.

> **Warning:** QRES cannot and should not be used where failure to converge could result in physical harm or catastrophic system failure.

## 4. QRES Is Not Homomorphic Encryption or Perfect Privacy

**Misconception**
> "Because only residuals are transmitted, the data is perfectly private."

**The Reality**
QRES is *privacy-preserving*, not privacy-proof. While raw data never leaves the node, a sufficiently capable adversary observing residual streams could theoretically reconstruct local signals through gradient leakage attacks.

**Why this trade-off exists**
True homomorphic encryption or secure multi-party computation introduces extreme computational and bandwidth overhead that would negate the primary advantage of QRES. For the **threat model** QRES targets—cooperative deployments under common administrative control—differential privacy via local noise injection and residual-only transmission provides appropriate protection without destroying efficiency.

> QRES intentionally trades absolute secrecy for efficiency and deployability on microcontroller-class hardware.

## 5. QRES Is Not a General-Purpose Database or CRDT

**Misconception**
> "QRES can synchronize arbitrary state like logs, settings, or text."

**The Reality**
QRES is specialized for continuous, differentiable signals. Its predictive model relies on gradients and smooth temporal evolution.

**Failure Mode**
Discrete data—booleans, strings, categorical states—cannot be meaningfully differentiated. Attempting to sync such data breaks the prediction logic entirely. QRES is a learning protocol, not a state replication system.

> If your data doesn't have meaningful derivatives, QRES is the wrong tool.

## 6. QRES Is Not "Serverless" — It Is Server-Free

**Misconception**
> "QRES is like AWS Lambda, but on edge devices."

**The Reality**
"Serverless" usually means the server exists but is abstracted away. QRES has no orchestrator, no coordinator, and no authoritative node. It is fully peer-to-peer.

**Engineering Burden**
Topology management, partition healing, and convergence behavior are the responsibility of the system designer. In the event of a network split, there is no central authority to declare which partition is canonical. This is a deliberate design choice, not a convenience feature.

> QRES gives you full control at the cost of full responsibility.

---

## Summary: Explicit Non-Goals

QRES is explicitly **not** designed for:
* Universal untrusted environments (e.g. Public Blockchains)
* Large-scale model storage beyond single-node RAM
* Discrete state synchronization
* Hard real-time or safety-critical control systems
* Perfect cryptographic privacy guarantees
* Centrally managed deployments requiring orchestration

**v19.0 Resilience Bound**
- Resistant to coordinated bias up to **30%** via coordinate-wise trimmed mean aggregation, assuming a cooperative majority (f < 33%).
- Precision engine: **BFP-16** matches Float32 (0.00% delta) with an effective resolution floor of ~1.5×10^-5.
- Capacity constraint: Models are capped by the smallest node’s RAM (e.g., ~320 KB on ESP32-class hardware). QRES is not intended for LLM-scale architectures.

---

# What QRES Is

**QRES (Quantum-Resilient Evolutionary Swarm)** is a bandwidth-first distributed learning runtime designed for microcontroller-class edge devices (e.g., ESP32, ARM Cortex-M).

It addresses the weight bottleneck in federated learning by replacing state transmission with deterministic rematerialization. Instead of sending model updates, nodes exchange minimal residuals ("surprises") and locally replay training steps to converge on identical model states.

**In Systems Terms:**
> QRES is a soft real-time, data-parallel, peer-to-peer learning protocol that intentionally trades surplus local compute for massive reductions in RF usage.

---

## The QRES Litmus Test

QRES is the correct tool only if **all** of the following are true:

- [ ] Bandwidth is expensive relative to local computation.
- [ ] The full model fits within the smallest node's RAM.
- [ ] Input signals are continuous and differentiable.
- [ ] The majority of nodes are cooperative (f < 33%).
- [ ] Eventual consistency is acceptable (soft real-time is sufficient).
- [ ] Privacy concerns are about casual observation, not state-level adversaries.

**Failing any one of these conditions strongly suggests a different architecture.**

---

## Ideal Use Cases ("The Golden Path")

### A. Remote Environmental Sensing
A mesh of solar-powered sensors detecting wildfire conditions where satellite bandwidth is scarce or costly.
* **Benefit:** Shared anomaly detection with near-zero uplink traffic.
* **Example:** Fire weather indices computed collaboratively across 100+ nodes with **<1 KB/hour** total RF usage.

### B. Industrial Predictive Maintenance
Thousands of vibration sensors monitoring machinery without saturating local networks.
* **Benefit:** Fault detection with orders-of-magnitude less data movement.
* **Example:** Bearing wear detection across factory floor with **5.14x compression** vs. raw telemetry.

### C. Privacy-Preserving Wearables
On-device learning for fall detection or cardiac anomalies without exporting raw biometric data.
* **Benefit:** Personalization without cloud dependency or data exfiltration.
* **Example:** Multi-user fall detection learning without sharing accelerometer traces.

---

## Adjacent Technologies: When to Use Something Else

| If you need… | Use instead | Why |
| :--- | :--- | :--- |
| **Large model training** (>100 MB) | Parameter server architectures (PyTorch DDP, Horovod) | Centralized aggregation handles models exceeding edge RAM. |
| **Byzantine fault tolerance** | BFT consensus (Tendermint, HotStuff, PBFT) | Designed for adversarial nodes with cryptographic guarantees. |
| **Hard real-time control** (<10 ms jitter) | Dedicated RTOS + deterministic networking (FreeRTOS + TSN) | Bounded worst-case latency, no probabilistic convergence. |
| **Perfect privacy guarantees** | Federated learning + secure aggregation (FATE, PySyft, TFF) | Cryptographic privacy with formal guarantees (at significant cost). |
| **Arbitrary state synchronization** | CRDTs (Automerge, Yjs) or consensus databases (etcd, Consul) | Designed for discrete, eventually-consistent state. |
| **Cloud-scale orchestration** | Kubernetes, Ray, Apache Spark | Centralized control plane manages distributed execution. |

---

## The Warning Label

> **Do not use QRES for:**
> * Financial systems, cryptographic ledgers, or adversarial environments.
> * Large language models, transformers, or models exceeding single-device RAM.
> * Safety-critical control systems (medical devices, autonomous vehicles, industrial safety interlocks).
> * Applications requiring cryptographic privacy guarantees or protection from state-level adversaries.
> * Discrete data types (logs, configuration, categorical states).

---

## Design Philosophy

> **QRES exists because some networks are so bandwidth-constrained that it's cheaper to recompute state than to transmit it—and because some data is too sensitive to leave the device, even encrypted.**

*This document is a living specification. As QRES evolves, so too will its boundaries. Proposals to expand scope must demonstrate that the expansion preserves the core value proposition: extreme bandwidth efficiency on resource-constrained cooperative networks.*