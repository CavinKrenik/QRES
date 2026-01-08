# Security Model & Roadmap

## Current Threat Model

QRES currently operates in a **trusted-node** environment:

- All nodes assumed benign
- No cryptographic authentication of model updates
- No Byzantine fault tolerance

> [!WARNING]
> Do not deploy QRES swarms on public/adversarial networks without implementing the defenses described below.

## Why This Matters

In production IoT deployments, malicious nodes could:

| Attack | Impact |
|--------|--------|
| **Weight poisoning** | Inject corrupted weights to degrade compression swarm-wide |
| **Gradient attacks** | Leak information about training data via model updates |
| **DoS flooding** | Overwhelm network with junk model updates |
| **Free-riding** | Consume shared models without contributing |

## Planned Defenses

### Phase 1: Authentication (v13) ✅ Complete

- ✅ **ed25519 signatures** for all model updates (Implemented in `qres_daemon/src/security.rs`)
- ✅ **Node identity verification** via public key infrastructure (Implemented in `qres_daemon/src/peer_keys.rs`)
- ✅ **Replay attack prevention** with nonces and timestamps (Implemented with signature module)
- ✅ **P2P integration** - Signed broadcasts, verified receives in `swarm_p2p.rs`

### Phase 2: Robust Aggregation (v14)

- ✅ **Krum algorithm** for outlier rejection in federated averaging (Implemented in `qres_core/src/aggregation.rs`)
- ✅ **Multi-Krum** - averages the k most representative updates
- ✅ **Median/trimmed mean** weight averaging instead of simple mean
- **Pre-merge validation** on local test sets before accepting updates

### Phase 3: Privacy (v15)

- ✅ **Differential Privacy** for shared weights (ε-DP guarantees, Gaussian mechanism)
- ✅ **Secure Aggregation** protocols (Pairwise masking via X25519 + ChaCha20)
- ✅ **Zero-knowledge proofs** (Pedersen Commitments + Proof of Norm via EdwardsPoint)

## Current Best Practices

For production deployment today:

1. **Use VPN/private networks** – Isolate swarm traffic from public internet
2. **Node whitelisting** – Configure libp2p peer list to known nodes only
3. **Monitor compression ratios** – Anomalous drops may indicate poisoning
4. **Regular model snapshots** – Enable rollback if corruption detected

## Configuration Example

```toml
# qres_daemon.toml
[security]
mode = "whitelist"
trusted_peers = [
    "12D3KooWExamplePeerId1...",
    "12D3KooWExamplePeerId2...",
]
require_signature = false  # Enable when Phase 1 ships
```

## Attack Simulation (Demo)

A simple poisoning attack demonstration:

```python
# Simulated weight poisoning attack
normal_weights = [0.1, 0.2, 0.3]  # Legitimate update
poisoned_weights = [100.0, -100.0, 0.0]  # Malicious update

# Without defense: naive averaging
compromised = [(n + p) / 2 for n, p in zip(normal_weights, poisoned_weights)]
# Result: [50.05, -49.9, 0.15] ← Model destroyed

# With Krum defense: outlier rejection
# Poisoned update detected and rejected
# Model remains: [0.1, 0.2, 0.3]
```

## Assumed Adversaries

This section documents the threat actors QRES is designed to defend against, their capabilities, and the limitations of current defenses.

### Byzantine Adversary
| Aspect | Details |
|--------|---------|
| **Capability** | Send arbitrary malicious gradients/weights to poison the global model |
| **Example Attack** | Submit `[1e6, -1e6, 0]` to maximize damage via averaging |
| **Defense** | Krum algorithm rejects outlier updates based on pairwise distances |
| **Limitation** | Breaks if >20% of nodes collude (f < n/2 - 1 for Krum) |

### Honest-but-Curious Aggregator
| Aspect | Details |
|--------|---------|
| **Capability** | Observe all model updates to infer private training data |
| **Example Attack** | Gradient inversion to reconstruct images/text from updates |
| **Defense** | Differential Privacy (ε-DP noise) + Secure Aggregation (pairwise masking) |
| **Limitation** | Colluding majority can reconstruct individual updates |

### Sybil Attacker
| Aspect | Details |
|--------|---------|
| **Capability** | Create multiple fake identities to gain disproportionate influence |
| **Example Attack** | Register 100 fake nodes to dominate aggregation |
| **Defense** | PKI with trusted seed nodes; ed25519 identity verification |
| **Limitation** | Open networks without identity authority are vulnerable |

### Out of Scope

The following are **not** addressed by current QRES defenses:

- **Side-channel attacks** (timing, power analysis)
- **Supply chain compromise** (malicious dependencies)
- **Quantum adversaries** (post-quantum crypto planned for v16+)
- **Physical access** to nodes
- **Social engineering** of node operators

## Timeline

| Phase | Target | Status |
|-------|--------|--------|
| Authentication | v13 | ✅ Complete |
| Robust Aggregation | v14 | ✅ Complete |
| Privacy | v15 | ✅ Complete |

---

*See [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) for current production readiness.*
