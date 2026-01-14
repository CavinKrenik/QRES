# P2P Swarm Implementation Guide

This document details the architecture and implementation of the QRES P2P Swarm (v8.0+), enabling distributed learning ("Hive Mind") and model synchronization.

## Overview
The QRES Swarm uses `libp2p` to create a decentralized network where nodes share:
1.  **Epiphanies:** Learned model weights (small tensors, including MetaBrain v4).
2.  **Hilbert Embeddings:** Compressed "World States" (high-dimensional tensors) for synchronization.
3.  **Discovery:** Peer finding via Kademlia DHT.

*Privacy First: Only model weights and state metadata are shared. Raw file content never leaves the local node.*

## Architecture

### 1. Network Stack (Rust)
Located in `crates/qres_daemon/src/swarm.rs`.
*   **Transport:** TCP/QUIC with Noise encryption (Yamux multiplexing).
*   **Discovery:** Kademlia DHT (Distributed Hash Table) for finding peers without a central server.
    *   **Bootstrap Mode:** Nodes can act as bootstrap servers for WAN discovery.
*   **PubSub:** GossipSub v1.1 for efficient message broadcasting.

### 2. Topics & Protocols

| Topic | Description | Payload |
| :--- | :--- | :--- |
| `qres/v1/epiphany` | Shared model weights | `Epiphany { model_type, weights, fidelity_score }` |
| `qres/v1/state` | State synchronization | `State { timestamp, fidelity, tensor_blob }` |
| `qres/v1/heartbeat` | Node status updates | `Heartbeat { uptime, version }` |

### 3. Hive Mind (Continual Learning)
Implemented in `ai/hive_mind.py`.
*   **FedProx:** Federated Averaging with Proximal term to handle non-IID data stability.
*   **Cycle:**
    1.  **Local Train:** Node evolves SNN/Tensor predictor locally on new data.
    2.  **Epiphany:** Weights extract -> Quantize -> Broadcast on `qres/v1/epiphany`.
    3.  **Assimilate:** Receiver averages parameter vectors: $W_{new} = \frac{1}{N} \sum W_i$.
    4.  **Evolve:** Local model updated with community knowledge.

## 3. Usage

### Starting a Swarm Node (qres-daemon)
To start a background node that listens for model updates and contributes to the Hive Mind:

```bash
qres-daemon --mode node --port 4001
```

### Broadcasting to the Swarm
To archive data and broadcast the weight "epiphany" to the network:

```bash
qres pack --input ./data --out archive.qrar --swarm
```

### WAN Bootstrap
To run as a stable bootstrap peer for other nodes to discover:

```bash
qres-daemon --mode bootstrap --port 4001
```

### Tensor State Broadcasting (v10.0)
Persistent states now include multimodal embeddings; sync with >0.99 fidelity.

## Troubleshooting
*   **No Peers Found:** Ensure port 4001 is open (UDP/TCP). Use `--bootstrap <IP>` to connect to a known peer.
*   **Version Mismatch:** Swarm protocol enforces major version compatibility (v10.x).
