# ADR-003: PRNG Synchronization Strategy

**Status:** Proposed  
**Date:** 2026-01-08  
**Context:** Maintaining deterministic reproducibility across federated nodes

---

## Problem

QRES compression relies on deterministic PRNG for encoder/decoder synchronization. In federated swarms, nodes may:
- Join at different times
- Miss gossip messages
- Have different seed histories

This causes **PRNG drift**, leading to decompression failures.

## Decision

Implement **versioned seed state** with gossip-based synchronization.

## Design

```rust
struct SeedState {
    version: u32,           // Monotonic epoch
    seed: [u8; 32],         // ChaCha20 seed
    timestamp: u64,         // Unix millis
}
```

### Sync Modes

| Mode | Behavior | Use Case |
|------|----------|----------|
| `diff` | Gossip only version+hash; request full if behind | Bandwidth-constrained |
| `full` | Gossip full seed on every epoch | Small swarms, low latency |

## Trade-offs

- **Diff mode:** Lower bandwidth, but requires request-response for catch-up
- **Full mode:** Simpler, but doesn't scale beyond ~50 nodes

## Consequences

- Add `seed_sync_mode` to config
- Implement `SeedState` in `prng.rs`
- Gossip via existing GossipSub channel
