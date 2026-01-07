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
    import time
    
    print("=== QES Swarm Test ===\n")
    print(f"Nodes: {num_nodes}")
    print(f"Shared Seed: {seed}")
    print(f"Epochs: {num_epochs}\n")
    
    # Create nodes with same seed
    nodes = [QesSyncManager(seed) for _ in range(num_nodes)]
    
    all_passed = True
    total_time = 0
    
    for epoch in range(1, num_epochs + 1):
        start = time.perf_counter()
        
        # Generate deltas for each node
        all_deltas = [node.generate_weight_deltas(6) for node in nodes]
        
        elapsed = time.perf_counter() - start
        total_time += elapsed
        
        # Display first node's deltas
        d = all_deltas[0]
        print(f"Epoch {epoch:2d}: [{d[0]:+.4f}, {d[1]:+.4f}, ...] ({elapsed*1000:.2f}ms)")
        
        # Check synchronization
        first = all_deltas[0]
        synced = all(d == first for d in all_deltas[1:])
        
        if not synced:
            print("  ❌ DESYNC!")
            all_passed = False
    
    print(f"\n=== Results ===")
    print(f"Nodes: {num_nodes}")
    print(f"Epochs: {num_epochs}")
    print(f"Total Time: {total_time*1000:.2f}ms")
    print(f"Avg/Epoch: {total_time/num_epochs*1000:.2f}ms")
    print(f"Sync Rate: {'100%' if all_passed else 'FAILED'}")
    
    return all_passed


if __name__ == "__main__":
    import sys
    
    # Parse args
    num_nodes = int(sys.argv[1]) if len(sys.argv) > 1 else 3
    num_epochs = int(sys.argv[2]) if len(sys.argv) > 2 else 10
    
    success = run_swarm_test(num_nodes=num_nodes, num_epochs=num_epochs)
    sys.exit(0 if success else 1)
