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

### Phase 1: Authentication (v13)

- ✅ **ed25519 signatures** for all model updates (Implemented in `qres_daemon/src/security.rs`)
- ✅ **Node identity verification** via public key infrastructure (Implemented in `qres_daemon/src/peer_keys.rs`)
- ✅ **Replay attack prevention** with nonces and timestamps (Implemented with signature module)

### Phase 2: Robust Aggregation (v14)

- **Krum algorithm** for outlier rejection in federated averaging
- Median/trimmed mean weight averaging instead of simple mean
- Pre-merge validation on local test sets before accepting updates

### Phase 3: Privacy (v15)

- **Differential privacy** for shared weights (ε-DP guarantees)
- Secure aggregation protocols (sum of weights without revealing individual)
- Zero-knowledge proofs of model quality

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

## Timeline

| Phase | Target | Status |
|-------|--------|--------|
| Authentication | v13 | 📋 Planned |
| Robust Aggregation | v14 | 📋 Planned |
| Privacy | v15 | 📋 Planned |

---

*See [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) for current production readiness.*
