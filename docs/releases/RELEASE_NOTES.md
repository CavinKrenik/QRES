# QRES v15.0.0 "Privacy"

**Release Date:** January 8, 2026

---

## Overview

v15.0.0 completes **Phase 3: Privacy** of the QRES security roadmap. This release adds provable privacy guarantees to federated swarms, defending against inference attacks and enabling aggregation without revealing individual contributions.

---

## Key Features

### Differential Privacy (`privacy.rs`)
- **Gaussian Mechanism:** Adds calibrated noise to model updates before sharing.
- **Provable Parameters:** Epsilon (ε) and Delta (δ) configurable via `[privacy]` section.
- **no_std Compatible:** Uses Box-Muller fallback for WASM/embedded.

### Secure Aggregation (`secure_agg.rs`)
- **Pairwise Masking:** Updates are masked via X25519 shared secrets + ChaCha20 RNG.
- **Zero-Sum Masks:** Masks cancel out upon aggregation, revealing only the global sum.
- **Privacy Guarantee:** Hides individual updates from honest-but-curious aggregators.

### Zero-Knowledge Proofs (`zk_proofs.rs`)
- **Pedersen Commitments:** Binds values without revealing them.
- **Proof of Norm:** Proves `||update||_2 <= threshold` without revealing the update.
- **Poisoning Defense:** Prevents malicious nodes from submitting extreme updates.

---

## Configuration

```toml
[privacy]
enabled = true
epsilon = 1.0
delta = 1e-5
clipping_threshold = 1.0
secure_aggregation = false  # Enable for pairwise masking

[security]
# See v13.0.0 for authentication settings
```

---

## Dependencies

- **Pinned:** `curve25519-dalek =4.1.3` with `rand_core`, `zeroize` features.
- **Added:** `x25519-dalek 2.0`, `rand_chacha 0.3`, `blake3 1.5`.

---

## Upgrade Notes

1. Update `Cargo.toml` to use `qres_core = "15.0"` and `qres_daemon = "15.0"`.
2. Add `[privacy]` section to your `qres_daemon.toml` to configure DP/SecAgg.
3. Existing configurations remain compatible with defaults.

---

## What's Next (v15.1)

- **Pluggable Aggregators:** Trait abstraction for custom aggregation strategies.
- **PRNG Seed Sync:** Drift mitigation for deterministic swarm replay.
- **Architecture Decision Records (ADRs):** Documenting key design choices.

---

*See [CHANGELOG.md](../CHANGELOG.md) for the full list of changes.*
