#!/bin/bash
# QRES Raspberry Pi Cluster Deployment Script
# Deploys ARM binary to multiple Pi nodes

set -e

# Configuration - Edit these for your cluster
PI_USER="pi"
PI_NODES=("pi-node1.local" "pi-node2.local" "pi-node3.local")
PI_PASSWORD=""  # Leave empty for SSH key auth
BINARY_PATH="/mnt/c/Dev/QRES/qres_rust/target/aarch64-unknown-linux-gnu/release/qres_daemon"
REMOTE_DIR="/home/pi/qres"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo "=== QRES Raspberry Pi Cluster Deployment ==="
echo ""

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${RED}Error: ARM binary not found at $BINARY_PATH${NC}"
    echo "Build with: cargo build --target aarch64-unknown-linux-gnu -p qres_daemon --release"
    exit 1
fi

echo "Binary found: $BINARY_PATH"
echo "Deploying to ${#PI_NODES[@]} nodes..."
echo ""

# Deploy to each node
for node in "${PI_NODES[@]}"; do
    echo "--- Deploying to $node ---"
    
    # Create remote directory
    ssh "$PI_USER@$node" "mkdir -p $REMOTE_DIR" 2>/dev/null || {
        echo -e "${RED}Failed to connect to $node${NC}"
        continue
    }
    
    # Copy binary
    scp "$BINARY_PATH" "$PI_USER@$node:$REMOTE_DIR/qres_daemon"
    
    # Make executable
    ssh "$PI_USER@$node" "chmod +x $REMOTE_DIR/qres_daemon"
    
    echo -e "${GREEN}Deployed to $node${NC}"
done

echo ""
echo "=== Deployment Complete ==="
echo ""
echo "To start swarm on each node:"
echo "  ssh pi@<node> '$REMOTE_DIR/qres_daemon swarm --port 8080'"
