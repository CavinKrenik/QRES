#!/bin/bash
set -e # Exit on error

echo "🚀 QRES Release Manager Initialized"

# 1. Verify Clean Git State
if [[ -n $(git status -s) ]]; then
  echo "❌ Error: Working directory not clean. Commit changes first."
  exit 1
fi

# 2. Run The Battle Royale (Safety Gate)
echo "⚔️  Running Battle Royale Verification..."
python benchmarks/battle_royale.py
if [ $? -eq 0 ]; then
    echo "✅ Core Codec Integrity Verified."
else
    echo "❌ CRITICAL: Codec Regression Detected. Aborting."
    exit 1
fi

# 3. Extract Version
VERSION=$(grep -m 1 'version =' qres_rust/qres_core/Cargo.toml | cut -d '"' -f 2)
echo "📦 Detected Version: v$VERSION"

# 4. Confirmation
echo "Are you ready to create tag v$VERSION and push? (y/n)"
read REPLY
if [[ $REPLY =~ ^[Yy]$ ]]; then
    git tag -a "v$VERSION" -m "QRES Engineering Release v$VERSION"
    git push origin "v$VERSION"
    echo "🎉 Release v$VERSION Pushed!"
else
    echo "🛑 Aborted."
fi