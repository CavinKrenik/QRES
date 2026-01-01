#!/bin/bash
# QRES v5.0 Release Manager
# Usage: ./release_manager.sh "v5.0.0" "Commit message here"

VERSION=$1
MSG=$2

if [ -z "$VERSION" ] || [ -z "$MSG" ]; then
    echo "Usage: ./release_manager.sh <version> <commit_message>"
    echo "Example: ./release_manager.sh v5.0.0 'Implement LzMatch and SIMD'"
    exit 1
fi

echo "🚀 Starting QRES Release Sequence for $VERSION..."

# 1. Run Local Tests
echo "🧪 Running Rust Core Tests..."
cd qres_rust
cargo test
if [ $? -ne 0 ]; then
    echo "❌ Rust tests failed! Aborting release."
    exit 1
fi
cd ..

echo "🐍 Running Python Integration Tests..."
# Assuming virtualenv is active or available
if command -v python3 &> /dev/null; then
    python3 benchmarks/test_final_suite.py
    if [ $? -ne 0 ]; then
        echo "❌ Python benchmarks failed! Aborting release."
        exit 1
    fi
else
    echo "⚠️ Python3 not found, skipping integration tests..."
fi

# 2. Update Version in Cargo.toml (Basic sed replacement)
# Adjust path if needed
if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "s/^version = \".*\"/version = \"${VERSION#v}\"/" qres_rust/Cargo.toml
else
    sed -i "s/^version = \".*\"/version = \"${VERSION#v}\"/" qres_rust/Cargo.toml
fi
echo "📝 Updated Cargo.toml version to ${VERSION#v}"

# 3. Git Operations
echo "📦 Committing changes..."
git add .
git commit -m "feat: $MSG"

echo "🏷️ Tagging $VERSION..."
git tag -a "$VERSION" -m "Release $VERSION: $MSG"

echo "📤 Pushing to GitHub..."
git push origin main
git push origin "$VERSION"

echo "✅ Done! GitHub Action should now be building $VERSION."
echo "Check status here: https://github.com/CavinKrenik/QRES/actions"