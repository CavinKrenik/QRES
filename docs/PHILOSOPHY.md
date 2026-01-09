# Why QRES Exists

## The Origin Story

This project started with the simplest possible idea: can you compress data using only three comparisons? `<`, `>`, `=`.

At first, it was just comparing the current value to what I expected. If it matched, nothing needed to be sent. If it didn't, only the fact of change mattered.

That framing shifted compression away from "how do I encode data efficiently" toward **"how often is reality surprising?"**

Even in its most primitive form, this was already predictive: compression existed only when expectation and reality diverged.

## The Evolution

From there, expectation stopped being static and became learned. The project evolved through three distinct phases:

### Phase 1: Deterministic Foundation (v1–v3)

Basic comparison and prediction—establishing that compression is really about measuring surprise.

| Version | Approach |
|---------|----------|
| **v1** | Static comparison (current == expected?) |
| **v2** | Learned predictor (what SHOULD come next?) |
| **v3** | Synchronized predictors (encoder/decoder share model) |

### Phase 2: Adaptive Intelligence (v4–v9)

SNNs, weighting, and pruning—compression becomes neuromorphic.

| Version | Approach |
|---------|----------|
| **v4** | Spiking Neural Networks for edge compatibility |
| **v7** | Multimodal memory, PPO agent for adaptive weighting |
| **v8** | P2P swarm architecture, Hive Mind federation |
| **v9** | GIF neurons, 97% sparsity via OSBC pruning |

### Phase 3: Hive Mind (v10–v15)

Swarm sync and federated dreaming—devices learn together without congesting the network.

| Version | Approach |
|---------|----------|
| **v10** | Q16.16 fixed-point determinism, architecture decoupling |
| **v11** | Portable SIMD (ARM/x86/WASM), browser support |
| **v12** | Zero-bandwidth swarm sync, federated dreaming |
| **v15** | Differential privacy, Byzantine fault tolerance, secure aggregation |

Instead of comparing against a fixed value, the system began predicting what should come next based on time, patterns, and prior behavior. Differences shrank into residuals, and residuals became the only thing transmitted.

## Design Principles

1. **Determinism over performance**: Reproducibility matters for embedded systems
2. **Prediction over encoding**: Better models → better compression
3. **Cooperation over isolation**: Swarms learn faster than individuals
4. **Surprise as signal**: Encode what's unexpected, not what's expected

## The Biological Inspiration

Human brains don't store raw sensory data—they store compressed representations and prediction models. When you remember an event, you're not replaying a video; you're reconstructing it from compressed patterns.

QRES applies this principle:

| Brain Concept | QRES Analogy |
|---------------|--------------|
| Synaptic weights | Predictor model weights |
| Memory consolidation | Federated model sharing |
| Surprise signals | Prediction errors that get encoded |
| Adaptation | Learning from swarm intelligence |
| Sleep/dreaming | **Federated dreaming**—devices update their internal models during downtime, ensuring the swarm evolves without congesting the network |

This isn't just poetic—it's technically grounded in neuroscience's **predictive coding theory**.

## Why This Matters: The Data Problem

Every day, billions of IoT devices transmit telemetry. Most of this data is highly redundant—your thermostat reading doesn't change much minute-to-minute. Yet we transmit every value as if it were novel information.

### The Math

For a temperature sensor reporting every minute:

| Approach | Daily Data |
|----------|------------|
| Traditional | 8 bytes/reading × 1,440 readings/day = **11.5 KB/day** |
| QRES (stable environment) | **~0.5 KB/day** (95% reduction) |

Multiply by 1 million sensors: **11 GB/day → 500 MB/day**

The bandwidth savings compound across swarms.

### The Hidden Cost: Energy

Bandwidth is only part of the story. Sending radio signals (WiFi/LoRa/5G) consumes **significantly more battery than running local calculations**. A constrained device might spend 90% of its energy budget on transmission alone.

By reducing what needs to be sent, QRES extends the battery life of remote sensors from **months to years**—making truly autonomous IoT deployments possible.

## Core Philosophy

> **"Compress surprise, not data"**

Encode prediction errors, not raw values. When your model is good, most values aren't surprising—so most bits aren't needed.
