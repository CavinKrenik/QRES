#!/usr/bin/env python3
"""
QRES QES Swarm Simulation
Tests Quantum-Entangled Swarms weight synchronization across virtual nodes.
"""

import random

class QesSyncManager:
    """Simulates QES PRNG-seeded weight synchronization."""
    
    def __init__(self, seed: int):
        self.rng = random.Random(seed)
        self.epoch = 0
    
    def generate_weight_deltas(self, num_weights: int) -> list[float]:
        """Generate synchronized weight deltas."""
        self.epoch += 1
        return [self.rng.uniform(-0.01, 0.01) for _ in range(num_weights)]
    
    def apply_to_weights(self, weights: list[float]) -> list[float]:
        """Apply deltas and normalize."""
        deltas = self.generate_weight_deltas(len(weights))
        weights = [max(0, min(1, w + d)) for w, d in zip(weights, deltas)]
        total = sum(weights)
        if total > 0.001:
            weights = [w / total for w in weights]
        return weights


def run_swarm_test(num_nodes: int = 3, num_epochs: int = 5, seed: int = 42):
    """Run QES swarm synchronization test."""
    print("=== QES Swarm Test ===\n")
    print(f"Nodes: {num_nodes}")
    print(f"Shared Seed: {seed}")
    print(f"Epochs: {num_epochs}\n")
    
    # Create nodes with same seed
    nodes = [QesSyncManager(seed) for _ in range(num_nodes)]
    
    all_passed = True
    
    for epoch in range(1, num_epochs + 1):
        print(f"--- Epoch {epoch} ---")
        
        # Generate deltas for each node
        all_deltas = [node.generate_weight_deltas(6) for node in nodes]
        
        # Display first node's deltas
        d = all_deltas[0]
        print(f"  Deltas: [{d[0]:.4f}, {d[1]:.4f}, {d[2]:.4f}, ...]")
        
        # Check synchronization
        first = all_deltas[0]
        synced = all(d == first for d in all_deltas[1:])
        
        if synced:
            print("  ✅ All nodes synchronized!")
        else:
            print("  ❌ Synchronization FAILED!")
            all_passed = False
        print()
    
    print("=== Test Complete ===")
    if all_passed:
        print(f"Result: PASSED - {num_epochs} epochs, {num_nodes} nodes in sync")
    else:
        print("Result: FAILED - Nodes fell out of sync")
    
    return all_passed


if __name__ == "__main__":
    import sys
    
    # Parse args
    num_nodes = int(sys.argv[1]) if len(sys.argv) > 1 else 3
    num_epochs = int(sys.argv[2]) if len(sys.argv) > 2 else 10
    
    success = run_swarm_test(num_nodes=num_nodes, num_epochs=num_epochs)
    sys.exit(0 if success else 1)
