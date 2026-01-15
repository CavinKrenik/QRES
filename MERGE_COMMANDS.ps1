# QRES v18.0.0 Merge Script (PowerShell)
# Execute these commands to merge architecture/neural-swarm into main on Windows

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "QRES v18.0.0: Neural Swarm Pivot Merge" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Verify branch
Write-Host "[1/6] Verifying current branch..." -ForegroundColor Yellow
$currentBranch = git rev-parse --abbrev-ref HEAD
if ($currentBranch -ne "architecture/neural-swarm") {
    Write-Host "ERROR: Not on architecture/neural-swarm branch (currently on: $currentBranch)" -ForegroundColor Red
    exit 1
}
Write-Host "✓ On architecture/neural-swarm" -ForegroundColor Green
Write-Host ""

# Step 2: Push final changes
Write-Host "[2/6] Pushing final changes to remote..." -ForegroundColor Yellow
git push origin architecture/neural-swarm
Write-Host "✓ Pushed final changes" -ForegroundColor Green
Write-Host ""

# Step 3: Switch to main
Write-Host "[3/6] Switching to main branch..." -ForegroundColor Yellow
git checkout main
git pull origin main
Write-Host "✓ On main (up-to-date)" -ForegroundColor Green
Write-Host ""

# Step 4: Merge
Write-Host "[4/6] Merging architecture/neural-swarm into main..." -ForegroundColor Yellow
git merge --no-ff architecture/neural-swarm -m @"
Merge branch 'architecture/neural-swarm' into main

v18.0.0: The Neural Swarm Pivot

- Emergent Intelligence: Bevy-based swarm simulator with self-healing networks
- Hippocampus: Persistent evolutionary memory layer (GeneStorage trait)
- No_Std Core: SwarmNeuron trait for embedded neural computing
- Gossip Protocol: Decentralized gene propagation under MTU constraints
- Breaking: Predictor trait refactored to SwarmNeuron
- New crate: tools/swarm_sim for God View visualization
- Ready for production edge deployment
"@
Write-Host "✓ Merged successfully" -ForegroundColor Green
Write-Host ""

# Step 5: Tag
Write-Host "[5/6] Creating release tag v18.0.0..." -ForegroundColor Yellow
git tag -a v18.0.0 -m @"
v18.0.0: The Neural Swarm Pivot

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

For details: see docs/releases/RELEASE_NOTES.md
"@
Write-Host "✓ Tagged as v18.0.0" -ForegroundColor Green
Write-Host ""

# Step 6: Push
Write-Host "[6/6] Pushing main and tags to remote..." -ForegroundColor Yellow
git push origin main
git push origin v18.0.0
Write-Host "✓ Pushed to remote" -ForegroundColor Green
Write-Host ""

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "✓ Merge Complete!" -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Create a GitHub Release at: https://github.com/CavinKrenik/QRES/releases/new?tag=v18.0.0"
Write-Host "  2. Use content from docs/releases/RELEASE_NOTES.md"
Write-Host "  3. Attach simulator binary or build instructions"
Write-Host "  4. Announce on Discord/Twitter/forums"
