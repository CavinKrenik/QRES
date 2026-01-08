# ADR-002: ed25519 vs Dilithium for Signatures

**Status:** Accepted  
**Date:** 2026-01-08  
**Context:** Choosing digital signature scheme for node authentication

---

## Decision

Use **ed25519** for all model update signatures in v13-v15.

## Rationale

| Factor | ed25519 | Dilithium (PQC) |
|--------|---------|-----------------|
| **Speed** | ~50μs sign/verify | ~500μs sign/verify |
| **Size** | 64B signature, 32B pubkey | 2.4KB signature, 1.3KB pubkey |
| **Maturity** | 10+ years, audited | NIST finalist, newer |
| **Quantum** | Broken by Shor's | Secure |

## Why Not Post-Quantum Now?

1. **Bandwidth:** IoT networks can't afford 2KB signatures per update
2. **Timeline:** Practical quantum computers are 10+ years away
3. **Migration:** Plan to add Dilithium option in v16+ as hybrid

## Consequences

- v13-v15 use `ed25519-dalek` crate
- Signature size is 64 bytes (acceptable for IoT)
- Future: Add `dilithium_enabled` config flag for hybrid mode
