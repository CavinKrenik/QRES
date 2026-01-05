#!/bin/bash
set -e # Exit on error

echo "🚀 QRES Release Manager Initialized"

# 1. Verify Clean Git State
if [[ -n $(git status -s) ]]; then
  echo "❌ Error: Working directory not clean. Commit changes first."
  exit 1
fi

# 2. Build Core Codec
echo "🔨 Building qres_core (Release)..."
cd qres_rust
cargo build --release --features python
cd ..

# 3. Install/Update Extension
echo "🔌 Updating Python Extension..."
# Windows specific copy
if [ -f "qres_rust/target/release/qres_core.dll" ]; then
    cp "qres_rust/target/release/qres_core.dll" "python/qres/qres_rust.pyd"
    echo "✅ qres_rust.pyd updated."
elif [ -f "qres_rust/target/release/qres_core.so" ]; then
    cp "qres_rust/target/release/qres_core.so" "python/qres/qres_rust.so"
    echo "✅ qres_rust.so updated."
else
    # Fallback to checking deps if dll name differs
    echo "⚠️  Warning: DLL not found in expected path. Proceeding with existing extension."
fi

# 4. Run The Battle Royale (Safety Gate)
echo "⚔️  Running Battle Royale Verification..."
export PYTHONPATH=$PYTHONPATH:$(pwd)/python
python benchmarks/battle_royale.py
if [ $? -eq 0 ]; then
    echo "✅ Core Codec Integrity Verified."
else
    echo "❌ CRITICAL: Codec Regression Detected. Aborting."
    exit 1
fi

# 5. Extract Version
VERSION=$(grep -m 1 'version =' qres_rust/qres_core/Cargo.toml | cut -d '"' -f 2)
echo "📦 Detected Version: v$VERSION"

# 6. Confirmation
echo "Are you ready to create tag v$VERSION and push? (y/n)"
read REPLY
if [[ $REPLY =~ ^[Yy]$ ]]; then
    git tag -a "v$VERSION" -m "QRES Engineering Release v$VERSION"
    git push origin "v$VERSION"
    echo "🎉 Release v$VERSION Pushed!"
else
    echo "🛑 Aborted."
fi