"""
QRES Tolerance Threshold Analysis
Shows where Krum breaks down as Byzantine percentage increases.

Usage: python visualize_tolerance.py
Output: docs/images/tolerance_curve.png
"""

import matplotlib.pyplot as plt
import numpy as np

# --- Krum Implementation ---
def euclidean_dist_sq(v1, v2):
    return np.sum((v1 - v2) ** 2)

def krum(vectors, f):
    n = len(vectors)
    if n < 2 * f + 3:
        return None
    
    k_neighbors = n - f - 2
    scores = []

    for i in range(n):
        distances = []
        for j in range(n):
            if i == j: continue
            distances.append(euclidean_dist_sq(vectors[i], vectors[j]))
        
        distances.sort()
        score = sum(distances[:k_neighbors])
        scores.append((i, score))
    
    scores.sort(key=lambda x: x[1])
    best_idx, _ = scores[0]
    return vectors[best_idx]

# --- Run Tolerance Tests ---
np.random.seed(42)

byzantine_pcts = [10, 20, 30, 40, 50]
krum_errors = []
naive_errors = []
n_total = 12  # Total swarm size

for pct in byzantine_pcts:
    n_byz = int(n_total * pct / 100)
    n_honest = n_total - n_byz
    f = n_byz  # Expected byzantines
    
    # Generate nodes
    honest = np.random.normal(1.0, 0.1, (n_honest, 8))
    malicious = np.ones((n_byz, 8)) * 10.0
    swarm = np.vstack([honest, malicious])
    
    # Calculate metrics
    honest_center = np.mean(honest, axis=0)
    naive_mean = np.mean(swarm, axis=0)
    krum_result = krum(swarm, f)
    
    naive_err = np.linalg.norm(naive_mean - honest_center)
    naive_errors.append(naive_err)
    
    if krum_result is not None:
        krum_err = np.linalg.norm(krum_result - honest_center)
    else:
        krum_err = naive_err  # Fallback
    krum_errors.append(krum_err)
    
    print(f"{pct}% Byzantine: Krum={krum_err:.2f}, Mean={naive_err:.2f}")

# --- Plot ---
plt.figure(figsize=(10, 6))
plt.style.use('ggplot')

# Plot lines
plt.plot(byzantine_pcts, naive_errors, 
         linestyle='--', color='orange', alpha=0.7, linewidth=2, 
         marker='s', markersize=8, label='Naive Mean (Unprotected)')

plt.plot(byzantine_pcts, krum_errors, 
         marker='o', linewidth=3, color='blue', markersize=10, 
         label='QRES Krum (Protected)')

# Theoretical limit line (n >= 2f + 3 -> f <= (n-3)/2 = 4.5 for n=12 -> ~37.5%)
plt.axvline(x=37.5, color='red', linestyle=':', linewidth=2, 
            label='Theoretical Limit (n < 2f+3)')

# Safe zone shading
plt.axvspan(0, 33, alpha=0.1, color='green', label='Safe Zone (<33%)')

# Annotations
plt.annotate('Safe Zone\n(Robust)', 
             xy=(20, krum_errors[1]), xytext=(8, 3),
             arrowprops=dict(facecolor='green', shrink=0.05),
             fontsize=11, color='green', fontweight='bold')

plt.annotate('Degradation\nBegins', 
             xy=(40, krum_errors[3]), xytext=(42, 2),
             arrowprops=dict(facecolor='red', shrink=0.05),
             fontsize=11, color='red', fontweight='bold')

# Formatting
plt.title("QRES Tolerance Analysis: The 'Breakdown' Curve", fontsize=14, fontweight='bold')
plt.xlabel("Byzantine Node Percentage (%)", fontsize=12)
plt.ylabel("Aggregation Error (Euclidean Distance)", fontsize=12)
plt.legend(loc='upper left', frameon=True, shadow=True)
plt.xticks(byzantine_pcts)
plt.ylim(0, max(naive_errors) * 1.1)

plt.tight_layout()
plt.savefig('docs/images/tolerance_curve.png', dpi=300)
print("Saved to docs/images/tolerance_curve.png")
plt.show()
