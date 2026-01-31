"""
QRES Krum Robustness Verification Suite
Tests subtle poisoning, coordination attacks, and high-dimensional cases.

Usage: python verify_robustness.py
"""

import numpy as np

# --- 1. Re-implement Krum Logic (Python Prototype) ---
def euclidean_dist_sq(v1, v2):
    return np.sum((v1 - v2) ** 2)

def krum(vectors, f):
    n = len(vectors)
    if n < 2 * f + 3:
        return None, float('inf') # Fail condition
    
    k_neighbors = n - f - 2
    scores = []

    for i in range(n):
        distances = []
        for j in range(n):
            if i == j: continue
            distances.append(euclidean_dist_sq(vectors[i], vectors[j]))
        
        distances.sort()
        # Krum Score = Sum of distances to k nearest neighbors
        score = sum(distances[:k_neighbors])
        scores.append((i, score))
    
    # Winner is index with lowest score
    scores.sort(key=lambda x: x[1])
    best_idx, best_score = scores[0]
    return vectors[best_idx], best_score

# --- 2. Test Scenarios ---

def run_test(name, honest_nodes, malicious_nodes, f):
    swarm = np.vstack([honest_nodes, malicious_nodes])
    
    print(f"\n[TEST] {name}")
    print(f"   Nodes: {len(swarm)} (Honest: {len(honest_nodes)}, Malicious: {len(malicious_nodes)})")
    print(f"   Tolerance (f): {f}")
    
    # 1. Calculate Naive Mean (The Control)
    naive_mean = np.mean(swarm, axis=0)
    
    # 2. Run Krum (The Experiment)
    winner, score = krum(swarm, f)
    
    # 3. Analyze Results
    honest_center = np.mean(honest_nodes, axis=0)
    dist_mean_to_honest = np.linalg.norm(naive_mean - honest_center)
    dist_krum_to_honest = np.linalg.norm(winner - honest_center)
    
    print(f"   [X] Naive Mean Error: {dist_mean_to_honest:.4f}")
    print(f"   [+] Krum Error:       {dist_krum_to_honest:.4f}")
    
    if dist_krum_to_honest < 0.1:
        print("   >>> RESULT: PASSED (Robust)")
    else:
        print("   >>> RESULT: FAILED (Compromised)")

print("=" * 60)
print("QRES Krum Robustness Verification Suite")
print("=" * 60)

# --- SCENARIO A: The "Subtle Poison" (Smart Attack) ---
# Attackers inject values just slightly off (1.5) to drift the model 
# without looking like obvious outliers.
honest_A = np.array([[1.0, 1.0], [0.95, 1.05], [1.05, 0.95], [1.0, 1.0]])
malicious_A = np.array([[1.5, 1.5]]) # Only 50% off, not 10000%
run_test("Subtle Poisoning (1.5x)", honest_A, malicious_A, f=1)

# --- SCENARIO B: The "Coordination Attack" (Collusion) ---
# Two malicious nodes work together to pull the mean.
# n=6, f=1 (Wait, n >= 2f + 3 -> 6 >= 5. This is valid for f=1)
honest_B = np.array([[1.0, 1.0], [0.9, 1.0], [1.1, 1.0], [1.0, 0.9]])
malicious_B = np.array([[5.0, 5.0], [5.0, 5.0]]) # Two nodes pulling hard
# NOTE: If we set f=1 but there are 2 attackers, Krum *should* fail or struggle.
run_test("Coordination Attack (2 Attackers, f=1)", honest_B, malicious_B, f=1)

# --- SCENARIO C: High-Dimensional Drift (8-Lag Gene) ---
# Testing if distance metrics hold up in higher dimensions (like your QRES genes)
np.random.seed(42)  # For reproducibility
honest_C = np.random.normal(1.0, 0.05, (10, 8)) # 10 nodes, 8 dimensions
malicious_C = np.ones((2, 8)) * 10.0 # 2 attackers at 10.0
run_test("8D Gene Vector (n=12, f=2)", honest_C, malicious_C, f=2)

print("\n" + "=" * 60)
print("Suite Complete")
print("=" * 60)
