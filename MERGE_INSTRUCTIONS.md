# QRES v18.0.0 Merge Checklist

## Pre-Merge Verification

Run these checks before merging:

```bash
# Verify all changes are committed
git status
# Expected: working tree clean on architecture/neural-swarm

# Verify workspace builds
cargo check --workspace
cargo build --workspace --release

# Verify tests pass
cargo test --workspace

# Verify simulator runs
cargo run -p swarm_sim --release
# Should launch Bevy window with 100-node grid
```

## Merge Command Sequence

Execute these commands in order from the project root:

### Option A: Automated (Recommended)

**On macOS/Linux:**
```bash
bash MERGE_COMMANDS.sh
```

**On Windows (PowerShell):**
```powershell
.\MERGE_COMMANDS.ps1
```

### Option B: Manual Commands

**Step 1: Verify we're on the feature branch**
```bash
git status
# Expected branch: architecture/neural-swarm
```

**Step 2: Push final changes**
```bash
git push origin architecture/neural-swarm
```

**Step 3: Switch to main and update**
```bash
git checkout main
git pull origin main
```

**Step 4: Merge with annotated commit**
```bash
git merge --no-ff architecture/neural-swarm -m "Merge branch 'architecture/neural-swarm' into main

v18.0.0: The Neural Swarm Pivot

- Emergent Intelligence: Bevy-based swarm simulator with self-healing networks
- Hippocampus: Persistent evolutionary memory layer (GeneStorage trait)
- No_Std Core: SwarmNeuron trait for embedded neural computing
- Gossip Protocol: Decentralized gene propagation under MTU constraints
- Breaking: Predictor trait refactored to SwarmNeuron
- New crate: tools/swarm_sim for God View visualization
- Ready for production edge deployment"
```

**Step 5: Create release tag**
```bash
git tag -a v18.0.0 -m "v18.0.0: The Neural Swarm Pivot

Release Date: 2026-01-15

This release pivots from deterministic compression (v17) to a fully decentralized neural swarm architecture with emergent self-healing and persistent evolutionary memory.

Key Components:
- SwarmNeuron trait (qres_core)
- God View Simulator (tools/swarm_sim)
- GeneStorage persistence layer
- Bevy-based 3D visualization
- MTU-constrained gossip protocol

Breaking Changes:
- Predictor trait -> SwarmNeuron trait
- Simulator moved to tools/swarm_sim
- New storage module in cortex subsystem

For details: see docs/releases/RELEASE_NOTES.md"
```

**Step 6: Push to remote**
```bash
git push origin main
git push origin v18.0.0
```

## Post-Merge Tasks

1. **Create GitHub Release**
   - Navigate to: https://github.com/CavinKrenik/QRES/releases/new?tag=v18.0.0
   - Copy release notes from [docs/releases/RELEASE_NOTES.md](docs/releases/RELEASE_NOTES.md#v1800-the-neural-swarm-pivot)
   - Upload simulator binary (optional)

2. **Verify CI Passes**
   - Check GitHub Actions workflows on main branch
   - All tests should pass: test.yml, lint.yml, release.yml

3. **Publish to Registries**
   - Publish to crates.io: `cargo publish -p qres_core`
   - Publish Python bindings: `cd bindings/python && python -m build && twine upload dist/*`

4. **Announce Release**
   - Discord: #releases channel
   - Twitter/X: @QRES_project
   - Reddit: r/rust, r/rust_embedded
   - GitHub Discussions: Create announcement thread

5. **Branch Cleanup** (after verification)
   - Delete feature branch locally: `git branch -d architecture/neural-swarm`
   - Delete remote branch: `git push origin --delete architecture/neural-swarm`

## Files Changed Summary

### Core Configuration
- **Cargo.toml**: Added `tools/swarm_sim` to workspace members, cleaned formatting
- **docs/releases/RELEASE_NOTES.md**: Added comprehensive v18.0.0 section with highlights, breaking changes, and migration guide

### CI/CD Status
- **test.yml**: Already includes `--workspace --exclude qres-studio` (covers swarm_sim)
- **lint.yml**: Already includes `--workspace --exclude qres-studio` (covers swarm_sim)
- **release.yml**: Standard release workflow (no changes needed)

### Documentation
- **README.md**: Updated to v18.0 with three-layer architecture explanation
- **docs/README.md**: Created comprehensive documentation index

### New Crate
- **tools/swarm_sim/**: Complete Bevy-based swarm simulator with all 4 phases
  - Phase 1: Cortex Foundation (SwarmNeuron trait)
  - Phase 2: God View Simulator (100-node grid)
  - Phase 3: Emergent Swarm Evolution (viral cure narrative)
  - Phase 4: Persistent Memory (Hippocampus layer)

## Verification Checklist

Before executing merge:

- [ ] All code is committed: `git status` shows clean working tree
- [ ] Tests pass: `cargo test --workspace`
- [ ] Release build succeeds: `cargo build --workspace --release`
- [ ] Simulator runs: `cargo run -p swarm_sim --release`
- [ ] Documentation builds: Check docs/ folder is complete
- [ ] No merge conflicts expected (feature branch created from recent main)

After merge:

- [ ] main branch is updated
- [ ] v18.0.0 tag created
- [ ] Both branch and tag pushed to remote
- [ ] GitHub Actions green on main
- [ ] Release notes visible in GitHub Releases
- [ ] Feature branch deleted

---

**Status**: Ready to merge
**Date**: 2026-01-15
**Release**: v18.0.0 (The Neural Swarm Pivot)
