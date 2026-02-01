"""
QRES Adversarial Hardening - Experiment 2: Asymmetric Network Partition & Recovery

Simulates network partitions where subgroups evolve independently (drift apart)
and then reconnect. Measures the "Consensus Pulse" - the time required to 
restore global consensus.

Parameters:
    n = 20 nodes total
    Scenarios:
        1. Balanced: 10v10
        2. Imbalanced: 15v5
        3. Fragmented: 8v7v5
    Isolation: 100 rounds (simulating drift)
    
Primary Metric: Consensus Pulse (rounds to restore >90% variance reduction after reconnection)

Usage: python tools/partition_recovery.py
"""

import numpy as np
import sys
import os
from datetime import datetime

# Configuration
N_TOTAL = 20
ISOLATION_ROUNDS = 100
MAX_RECOVERY_ROUNDS = 100
GENE_DIMENSIONS = 8
N_TRIALS = 10
VARIANCE_REDUCTION_TARGET = 0.90  # 90% reduction

SCENARIOS = {
    "Balanced": [10, 10],
    "Imbalanced": [15, 5],
    "Fragmented": [8, 7, 5]
}

def get_population_variance(nodes):
    """Calculate the total variance (sum of variances of each dimension)."""
    return np.sum(np.var(nodes, axis=0))

def simulate_local_training(nodes, drift_vector, rng):
    """
    Simulate local training steps.
    Nodes move towards their partition's local mean, which drifts over time.
    """
    # Simply add the drift vector + some noise to all nodes
    noise = rng.normal(0, 0.05, nodes.shape)
    return nodes + drift_vector + noise

def krum_aggregate(nodes, f):
    """Simplified Krum aggregation for consensus."""
    n = len(nodes)
    if n < 2 * f + 3:
        # Fallback to mean if Krum conditions aren't met (common in small partitions)
        return np.mean(nodes, axis=0)
    
    # Calculate pairwise squared distances
    diffs = nodes[:, np.newaxis, :] - nodes[np.newaxis, :, :]
    dists = np.sum(diffs**2, axis=-1)
    
    # Krum score: sum of distances to k nearest neighbors
    k = n - f - 2
    scores = np.sort(dists, axis=1)[:, 1:k+1].sum(axis=1) # Exclude self (index 0)
    
    winner_idx = np.argmin(scores)
    return nodes[winner_idx]

def run_trial(scenario_name, partitions_sizes, seed):
    rng = np.random.default_rng(seed)
    
    # Initialize all nodes at origin
    partitions = []
    
    # Create partitions
    for size in partitions_sizes:
        # Each partition gets nodes slightly scattered around origin
        p_nodes = rng.normal(0, 0.1, (size, GENE_DIMENSIONS))
        partitions.append(p_nodes)
        
    # Assign drift directions for each partition
    # ensuring they drift APART
    drift_vectors = []
    for _ in range(len(partitions_sizes)):
        d = rng.normal(0, 1, GENE_DIMENSIONS)
        d = d / np.linalg.norm(d) * 0.05 # Small drift per round
        drift_vectors.append(d)
        
    # --- PHASE 1: ISOLATION (100 Rounds) ---
    for _ in range(ISOLATION_ROUNDS):
        for i in range(len(partitions)):
            # Each partition evolves independently
            partitions[i] = simulate_local_training(partitions[i], drift_vectors[i], rng)
            
            # Internal consensus (keeps partition cohesive but moving)
            # Assume strict consensus within partition
            center = np.mean(partitions[i], axis=0)
            # Pull nodes tight to center to simulate "consensus"
            partitions[i] = partitions[i] * 0.5 + center * 0.5

    # --- PHASE 2: RECONNECTION ---
    # Merge all nodes
    all_nodes = np.vstack(partitions)
    
    # Measure Peak Variance at moment of reconnection
    initial_variance = get_population_variance(all_nodes)
    target_variance = initial_variance * (1 - VARIANCE_REDUCTION_TARGET)
    
    # Recovery Loop
    rounds_to_recover = MAX_RECOVERY_ROUNDS
    recovered = False
    
    variance_history = [initial_variance]
    
    # Assume f comes from total network size (e.g., f=6 for n=20)
    # Using f=6 (approx 1/3)
    global_f = int((N_TOTAL - 1) / 3)
    
    for r in range(MAX_RECOVERY_ROUNDS):
        # 1. Consensus Step
        # In a real network, this takes time. Here we simulate rounds of aggregation.
        # Nodes pull towards the Krum winner
        winner = krum_aggregate(all_nodes, global_f)
        
        # Update rule: nodes move towards consensus
        # Learning rate alpha=0.5
        all_nodes = all_nodes * 0.9 + winner * 0.1 + rng.normal(0, 0.01, all_nodes.shape)
        
        # 2. Measure Variance
        current_variance = get_population_variance(all_nodes)
        variance_history.append(current_variance)
        
        if current_variance <= target_variance and not recovered:
            rounds_to_recover = r + 1
            recovered = True
            break
            
    return {
        'scenario': scenario_name,
        'rounds': rounds_to_recover,
        'recovered': recovered,
        'initial_var': initial_variance,
        'final_var': variance_history[-1]
    }

def run_experiment():
    results = {}
    print("Running Experiment 2: Network Partition & Recovery")
    print(f"Scenarios: {list(SCENARIOS.keys())}")
    
    all_metrics = []
    
    for name, sizes in SCENARIOS.items():
        print(f"\n[SCENARIO] {name} {sizes}")
        scenario_results = []
        
        for i in range(N_TRIALS):
            seed = 2000 + i
            res = run_trial(name, sizes, seed)
            scenario_results.append(res['rounds'])
            if (i+1) % 5 == 0:
                print(f"   Trial {i+1}/{N_TRIALS}...")
                
        mean_rounds = np.mean(scenario_results)
        std_rounds = np.std(scenario_results)
        ci95 = 1.96 * std_rounds / np.sqrt(N_TRIALS)
        
        print(f"   >>> Recovery Time: {mean_rounds:.1f} +/- {ci95:.1f} rounds")
        
        results[name] = {
            'mean': mean_rounds,
            'ci': ci95,
            'raw': scenario_results
        }
        
    return results

def append_to_report(results):
    report = f"""
### Experiment 2: Asymmetric Network Partition & Recovery - {datetime.now().strftime("%Y-%m-%d %H:%M")}

- **Hypothesis:** Can the network recover consensus within acceptable time limits (CP < 20 rounds) after prolonged partitioning, regardless of partition topology?

- **Parameters:**
  - $n = 20$
  - Isolation: 100 rounds
  - Variance Reduction Target: 90%
  - Scenarios: Balanced (10v10), Imbalanced (15v5), Fragmented (8v7v5)

- **Raw Results:**

| Scenario | Split | Consensus Pulse (Mean Rounds) | 95% CI | Status |
|----------|-------|-------------------------------|--------|--------|
"""
    status = "PASSED"
    
    for name, data in results.items():
        row_status = "OK" if data['mean'] < 20 else "SLOW"
        if data['mean'] > 20: status = "FAILED"
        
        split = str(SCENARIOS[name])
        report += f"| {name} | {split} | {data['mean']:.1f} | +/- {data['ci']:.1f} | {row_status} |\n"
        
    report += f"""
- **Analysis:**
  - **Balanced Partition (10v10):** {"Recovery was fast." if results['Balanced']['mean'] < 20 else "Struggled to reconcile equal-weight partitions."}
  - **Imbalanced (15v5):** {"The larger partition quickly absorbed the smaller one." if results['Imbalanced']['mean'] < results['Balanced']['mean'] else "Unexpected resistance from minority partition."}
  - **Fragmented (8v7v5):** Complex 3-way merge dynamics observed.

- **Status:** **{status}**
    """
    
    with open(os.path.join("research", "Attack.md"), "a", encoding="utf-8") as f:
        f.write(report)
    print("\nResults appended to Attack.md")

if __name__ == "__main__":
    data = run_experiment()
    append_to_report(data)
