# Security Implementation Guide

This guide provides a structured, iterative workflow to implement the items in `docs/SECURITY_ROADMAP.md`. It is designed for use in prompts (e.g., with AI assistants) or manual development. The process is thorough, incorporating research, cross-referencing with repo elements (code, configs, docs), implementation, testing, and updates. It's progressive: complete phases sequentially, update docs after each item/phase, and iterate as needed.

Assumptions: Access to the repo via GitHub or local clone. Commit changes after each item, bump versions (e.g., v13 post-Phase 1), and push. Use tools like code linters, benchmarks, or simulations for validation.

## General Workflow for Each Item

1. **Research and Planning**:
   - Research concepts (e.g., web searches for crates/examples).
   - Cross-reference: Check code (e.g., `crates/qres_daemon/src` for P2P), configs (e.g., `qres_daemon.toml.example`), docs (e.g., `IMPLEMENTATION_STATUS.md` for FedProx status).
   - Plan: Outline integration into architecture (e.g., FedProx update flow).

2. **Implementation**:
   - Code the feature (Rust for core/daemon, Python for bindings if impacted).
   - Add dependencies (e.g., crates in `Cargo.toml`).
   - Ensure compatibility (e.g., `no_std`, portable SIMD, async with Tokio).

3. **Testing**:
   - Add unit/integration tests (in `tests/` or `crates/benches/`).
   - Simulate scenarios (e.g., poisoning attacks from `SECURITY_ROADMAP.md` demo).
   - Validate cross-platform (Linux/macOS/Windows/WASM).
   - Run benchmarks to check performance impact.

4. **Documentation Updates**:
   - Update `SECURITY_ROADMAP.md` (e.g., status to "Implemented").
   - Update `IMPLEMENTATION_STATUS.md` (e.g., move to "Fully Implemented").
   - Cross-update others: `ROADMAP.md`, `WHITEPAPER.md`, config examples.
   - Add new docs/examples (e.g., keygen usage).
   - Commit with descriptive messages (e.g., `feat(security): add ed25519 signatures`).

5. **Review and Iteration**:
   - Check impacts (e.g., on swarm sync or FedProx).
   - Iterate if issues: Re-research/test/update.
   - Update timelines in `SECURITY_ROADMAP.md`.

Repeat per item within a phase. Post-phase: Release new version, update `CHANGELOG.md`.

---

## Phases

### Phase 1: Authentication (Completed v16.5)

Focus: Secure model updates in trusted-node setup.  
Cross-ref: P2P in `qres_daemon` (libp2p/GossipSub), FedProx in core, config in `examples/qres_daemon.toml.example`.

#### Item 1: ed25519 signatures for all model updates

| Step | Details |
|------|---------|
| **Research** | `ed25519-dalek` crate; sign/verify model weights |
| **Cross-ref** | Model update logic in `crates/qres_daemon/src`; `require_signature = false` in config |
| **Implementation** | Sign before sending; verify on receipt in P2P messages |
| **Testing** | Unit tests for sign/verify; simulate invalid signatures |
| **Updates** | Enable `require_signature = true`; status to implemented |
| **Iteration** | Check DoS impact |

#### Item 2: Node identity verification via public key infrastructure

| Step | Details |
|------|---------|
| **Research** | libp2p peer IDs with pubkeys; simple PKI |
| **Cross-ref** | Trusted peers in config; libp2p setup |
| **Implementation** | Key gen/storage; verify on connect |
| **Testing** | Simulate unauthorized peers |
| **Updates** | Add key mgmt to config/docs |
| **Iteration** | Compat with whitelist mode |

#### Item 3: Replay attack prevention with nonces and timestamps

| Step | Details |
|------|---------|
| **Research** | Nonce/timestamp in distributed systems |
| **Cross-ref** | Message formats in P2P |
| **Implementation** | Add to signed updates; reject duplicates/old |
| **Testing** | Replay sims; clock skew handling |
| **Updates** | Add config options; update attack demo |
| **Iteration** | Cross-check with gradient attacks |

**Post-Phase**: v13 bump; full benchmarks.

---

### Phase 1.5: Reputation & Trust (Completed v16.5)

Focus: Filtering malicious actors based on historical behavior.
Cross-ref: `ReputationManager` in `qres_daemon`.

#### Item 1: Long-term Reputation Scoring

| Step | Details |
|------|---------|
| **Research** | Reputation systems (EigenTrust, etc.); persistent tracking |
| **Cross-ref** | `PeerId` usage in aggregator |
| **Implementation** | JSON-backed Score DB; Reward (+0.01)/Punish (-0.1) logic |
| **Testing** | Simulate "sleeper agent" behavior |
| **Updates** | `reputation.json` config |
| **Iteration** | Ban thresholds |

---

### Phase 2: Robust Aggregation (Completed v16.5)

Focus: Byzantine faults.  
Cross-ref: FedProx in `qres_core`; Krum in roadmap.

#### Item 1: Krum algorithm for outlier rejection

| Step | Details |
|------|---------|
| **Research** | Krum impl in Rust |
| **Cross-ref** | Averaging code |
| **Implementation** | Replace mean with Krum |
| **Testing** | Poisoning sims |
| **Updates** | Add to robust section |
| **Iteration** | Non-IID data tests |

#### Item 2: Median/trimmed mean averaging

| Step | Details |
|------|---------|
| **Research** | Robust stats in Rust |
| **Cross-ref** | Weight vectors |
| **Implementation** | Configurable modes |
| **Testing** | Outlier sims |
| **Updates** | Config options |
| **Iteration** | Perf checks |

#### Item 3: Pre-merge validation on local test sets

| Step | Details |
|------|---------|
| **Research** | FL validation techniques |
| **Cross-ref** | Compression monitoring |
| **Implementation** | Validate incoming weights |
| **Testing** | Corruption sims |
| **Updates** | Validation config |
| **Iteration** | Ensemble cross-tests |

**Post-Phase**: v14; robustness changelog.

---

### Phase 3: Privacy (Completed v16.5 - "The Ghost Protocol")

Focus: Data protection.  
Cross-ref: DP in roadmap; gradient attacks.

#### Item 1: Differential privacy for shared weights

| Step | Details |
|------|---------|
| **Research** | `opendp` or Manual Gaussian Mechanism |
| **Cross-ref** | Weight sharing in `privacy.rs` |
| **Implementation** | Noise addition (Gaussian) on I16F16 |
| **Testing** | Privacy audits; `I16F16` precision checks |
| **Updates** | DP section |
| **Iteration** | Utility benchmarks |

#### Item 2: Secure aggregation protocols

| Step | Details |
|------|---------|
| **Research** | Pairwise X25519 Masking |
| **Cross-ref** | Aggregation in `secure_agg.rs` |
| **Implementation** | Masked summing; `mask_update_fixed` |
| **Testing** | perfect cancellation checks |
| **Updates** | Protocols expansion |
| **Iteration** | Scale tests |

#### Item 3: Zero-knowledge proofs of model quality

| Step | Details |
|------|---------|
| **Research** | Pedersen Commitments |
| **Cross-ref** | Snapshots in `zk_proofs.rs` |
| **Implementation** | Proof gen/verify; `ProofBundle` |
| **Testing** | Validity/time; batch verification |
| **Updates** | ZK section |
| **Iteration** | Edge device opt |

**Post-Phase**: v16.5; security audit.

---

## Additional Notes

- **Repo-Wide Updates**: Search for terms (e.g., "signature") to catch missed cross-refs.
- **Progress Tracking**: Use GitHub issues/PRs.
- **Thoroughness**: 100% coverage per item (e.g., all attacks mitigated, tests >95%).
