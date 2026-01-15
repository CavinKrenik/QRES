#!/bin/bash
# QRES v18.0.0 Merge Script
# Execute these commands to merge architecture/neural-swarm into main

set -e

echo "=========================================="
echo "QRES v18.0.0: Neural Swarm Pivot Merge"
echo "=========================================="
echo ""

# Step 1: Ensure we're on the architecture/neural-swarm branch
echo "[1/5] Verifying current branch..."
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$CURRENT_BRANCH" != "architecture/neural-swarm" ]; then
    echo "ERROR: Not on architecture/neural-swarm branch (currently on: $CURRENT_BRANCH)"
    echo "Run: git checkout architecture/neural-swarm"
    exit 1
fi
echo "✓ On architecture/neural-swarm"
echo ""

# Step 2: Push final changes
echo "[2/5] Pushing final changes to remote..."
git push origin architecture/neural-swarm
echo "✓ Pushed final changes"
echo ""

# Step 3: Switch to main
echo "[3/5] Switching to main branch..."
git checkout main
git pull origin main
echo "✓ On main (up-to-date)"
echo ""

# Step 4: Merge architecture/neural-swarm
echo "[4/5] Merging architecture/neural-swarm into main..."
git merge --no-ff architecture/neural-swarm -m "Merge branch 'architecture/neural-swarm' into main

v18.0.0: The Neural Swarm Pivot

- Emergent Intelligence: Bevy-based swarm simulator with self-healing networks
- Hippocampus: Persistent evolutionary memory layer (GeneStorage trait)
- No_Std Core: SwarmNeuron trait for embedded neural computing
- Gossip Protocol: Decentralized gene propagation under MTU constraints
- Breaking: Predictor trait refactored to SwarmNeuron
- New crate: tools/swarm_sim for God View visualization
- Ready for production edge deployment"
echo "✓ Merged successfully"
echo ""

# Step 5: Tag the release
echo "[5/5] Creating release tag v18.0.0..."
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
- Predictor trait → SwarmNeuron trait
- Simulator moved to tools/swarm_sim
- New storage module in cortex subsystem

For details: see docs/releases/RELEASE_NOTES.md"
echo "✓ Tagged as v18.0.0"
echo ""

# Step 6: Push to remote
echo "[6/6] Pushing main and tags to remote..."
git push origin main
git push origin v18.0.0
echo "✓ Pushed to remote"
echo ""

echo "=========================================="
echo "✓ Merge Complete!"
echo "=========================================="
echo ""
echo "Next steps:"
echo "  1. Create a GitHub Release: https://github.com/CavinKrenik/QRES/releases/new?tag=v18.0.0"
echo "  2. Use content from docs/releases/RELEASE_NOTES.md"
echo "  3. Attach simulator binary or build instructions"
echo "  4. Announce on Discord/Twitter/forums"
echo ""
