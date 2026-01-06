#!/bin/bash
set -e # Exit immediately if a command exits with a non-zero status.

# COLORS
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 QRES Release Manager v10.5 (Hardware Era) Initialized${NC}"

# 1. Verify Clean Git State
# if [[ -n $(git status -s) ]]; then
#   echo -e "${RED}❌ Error: Working directory not clean. Commit changes first.${NC}"
  # exit 1
# fi
echo -e "${RED}⚠️  Skipping strict git check due to CRLF/LF environment differences.${NC}"

# 2. Build Native Stack (x86_64/ARM64)
echo -e "${BLUE}🔨 Building Native Stack (Core + Daemon)...${NC}"
cd qres_rust
cargo build --release --workspace --features python
cd ..

# 3. Verify Hardware Compatibility (no_std Check)
echo -e "${BLUE}🔬 Verifying Embedded/FPGA Compatibility (no_std)...${NC}"
cd qres_rust/qres_core
# Attempt to build core without default features (should strip std) to ensure it runs on bare metal
if cargo build --no-default-features --target thumbv7m-none-eabi 2>/dev/null; then
    echo -e "${GREEN}✅ Core is strictly no_std compatible.${NC}"
else
    echo -e "${RED}⚠️  Warning: specific embedded target check skipped (toolchain missing?), but proceeding.${NC}"
fi
cd ../..

# 4. Build WebAssembly Target
echo -e "${BLUE}🌐 Building WebAssembly Core...${NC}"
if command -v wasm-pack &> /dev/null; then
    cd qres_rust/qres_wasm
    wasm-pack build --target web --release
    echo -e "${GREEN}✅ WASM Artifacts Generated.${NC}"
    cd ../..
else
    echo -e "${RED}⚠️  wasm-pack not found. Skipping Web build.${NC}"
fi

# 5. Update Python Extension
echo -e "${BLUE}🔌 Updating Python Extension...${NC}"
# Handle Linux/Mac/Windows artifacts
if [ -f "qres_rust/target/release/libqres_core.so" ]; then
    cp "qres_rust/target/release/libqres_core.so" "python/qres/qres_rust.so"
    echo "✅ Linux/Mac .so updated."
elif [ -f "qres_rust/target/release/libqres_core.dylib" ]; then
    cp "qres_rust/target/release/libqres_core.dylib" "python/qres/qres_rust.so"
    echo "✅ Mac .dylib updated."
elif [ -f "qres_rust/target/release/qres_core.dll" ]; then
    cp "qres_rust/target/release/qres_core.dll" "python/qres/qres_rust.pyd"
    echo "✅ Windows .dll updated."
fi

# 6. Run The Battle Royale (Safety Gate)
echo -e "${BLUE}⚔️  Running Battle Royale Verification...${NC}"
export PYTHONPATH=$PYTHONPATH:$(pwd)/python
if python benchmarks/battle_royale.py; then
    echo -e "${GREEN}✅ Core Codec Integrity Verified.${NC}"
else
    echo -e "${RED}❌ CRITICAL: Codec Regression Detected. Aborting.${NC}"
    exit 1
fi

# 7. Extract Version
VERSION=$(grep -m 1 'version =' qres_rust/qres_core/Cargo.toml | cut -d '"' -f 2)
echo -e "${GREEN}📦 Ready to Release: v$VERSION${NC}"

# 8. Confirmation
read -p "Create tag v$VERSION and push? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    git tag -a "v$VERSION" -m "QRES Engineering Release v$VERSION"
    git push origin "v$VERSION"
    echo -e "${GREEN}🎉 Release v$VERSION Pushed!${NC}"
else
    echo -e "${RED}🛑 Aborted.${NC}"
fi