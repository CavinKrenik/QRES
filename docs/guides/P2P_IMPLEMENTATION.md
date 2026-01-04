# P2P Swarm Implementation Guide

This document details the architecture and implementation of the QRES P2P Swarm (v8.0+), enabling distributed learning ("Hive Mind") and quantum state broadcasting.

## Overview

The QRES Swarm uses **libp2p** to create a decentralized network where nodes share:
1.  **Epiphanies:** Learned model weights (small tensors).
2.  **Quantum States:** Compressed "World States" for synchronization.
3.  **Discovery:** Peer finding via Kademlia DHT.

**Privacy First:** Only *model weights* and *state metadata* are shared. Raw file content never leaves the local machine.

## Architecture

### 1. Network Stack (Rust)
Located in `qres_rust/src/swarm.rs`.

- **Transport:** TCP/QUIC with Noise encryption (Yamux multiplexing).
- **Discovery:** Kademlia DHT (Distributed Hash Table) for finding peers without a central server.
  - *Bootstrap Mode:* Nodes can act as bootstrap servers for WAN discovery.
- **PubSub:** GossipSub v1.1 for efficient message broadcasting.

### 2. Topics & Protocols

| Topic | Description | Payload |
| :--- | :--- | :--- |
| `qres/v1/epiphany` | Shared model weights | `Epiphany { model_type, weights, accuracy }` |
| `qres/v1/quantum` | Quantum State updates | `QuantumState { timestamp, fidelity, tensor_blob }` |
| `qres/v1/heartbeat` | Node status updates | `Heartbeat { uptime, version }` |

### 3. Usage

#### Starting a Swarm Node (Receiver)
The Python receiver script listens for incoming quantum states and reconstructs them.

```bash
python qres_quantum_receiver.py --dir ./quantum_inbox --port 4001
```

#### Broadcasting to the Swarm
When using the CLI with the `--swarm` flag, QRES broadcasts the resulting state after compression.

```bash
qres archive --dir ./logs --out logs.qrar --swarm
```

#### WAN Bootstrap
To run a stable node for others to discover:

```bash
qres-daemon --mode bootstrap --port 4001
```

## Quantum Tensor Broadcasting (v8.1)

Phase 4 introduced **Persistent World Compression**. The Swarm now synchronizes these persistent states.
- **Sender:** The CLI serializes the internal state (`WorldStateManager`) and pushes it to the Rust Swarm Outbox.
- **Receiver:** The `qres_quantum_receiver.py` script polls the Rust Inbox and applies updates if Fidelity > 0.98.

## Troubleshooting

- **No Peers Found:** Ensure port 4001 (default) is open. Use `--bootstrap <IP>` to connect to a known node.
- **Version Mismatch:** Swarm protocol versions must match (check `major.minor`).
