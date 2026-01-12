#!/bin/bash
set -e

echo ">> Starting Environment Setup for QRES Benchmarks..."

# 1. System Updates & Dependencies
echo ">> Update apt and install dependencies..."
sudo apt-get update
sudo apt-get install -y build-essential curl git libssl-dev pkg-config

# 2. Install Rust (idempotent check)
if ! command -v cargo &> /dev/null; then
    echo ">> Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo ">> Rust already installed. Updating..."
    rustup update
fi

# 3. Clone/Update Repository
REPO_DIR="$HOME/QRES"
REPO_URL="https://github.com/CavinKrenik/QRES.git"

if [ -d "$REPO_DIR" ]; then
    echo ">> Updating existing repository at $REPO_DIR..."
    cd "$REPO_DIR"
    git pull
else
    echo ">> Cloning repository..."
    # Config git to use less memory if needed? Usually git clone is fine, cargo is the hog.
    git clone "$REPO_URL" "$REPO_DIR"
    cd "$REPO_DIR"
fi

# 4. Configure Low-Memory Build Settings
# Constrained VMs often choke on git-fetch inside cargo
export CARGO_NET_GIT_FETCH_WITH_CLI=true

echo ">> Setup Complete. You are ready to run 'run_suite.sh'."
