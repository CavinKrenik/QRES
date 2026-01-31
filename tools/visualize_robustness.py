"""
QRES Krum Robustness Visualization
Creates a multi-panel figure showing all three stress test scenarios.

Usage: python visualize_robustness.py
Output: robustness_comparison.png
"""

import matplotlib.pyplot as plt
import numpy as np

# --- Krum Implementation ---
def euclidean_dist_sq(v1, v2):
    return np.sum((v1 - v2) ** 2)

def krum(vectors, f):
    n = len(vectors)
    if n < 2 * f + 3:
        return None, float('inf')
    
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
    best_idx, best_score = scores[0]
    return vectors[best_idx], best_score

# --- Data ---
np.random.seed(42)

# Scenario A: Subtle Poisoning
honest_A = np.array([[1.0, 1.0], [0.95, 1.05], [1.05, 0.95], [1.0, 1.0]])
malicious_A = np.array([[1.5, 1.5]])

# Scenario B: Coordination Attack
honest_B = np.array([[1.0, 1.0], [0.9, 1.0], [1.1, 1.0], [1.0, 0.9]])
malicious_B = np.array([[5.0, 5.0], [5.0, 5.0]])

# Scenario C: 8D (project to 2D for viz)
honest_C_8d = np.random.normal(1.0, 0.05, (10, 8))
malicious_C_8d = np.ones((2, 8)) * 10.0
# PCA-like projection: just take first 2 dims
honest_C = honest_C_8d[:, :2]
malicious_C = malicious_C_8d[:, :2]

scenarios = [
    ("A: Subtle Poisoning (1.5x)", honest_A, malicious_A, 1),
    ("B: Coordination Attack (2 Attackers)", honest_B, malicious_B, 1),
    ("C: 8D Gene Vector (projected)", honest_C, malicious_C, 2),
]

# --- Create Figure ---
fig, axes = plt.subplots(1, 3, figsize=(15, 5))
plt.style.use('ggplot')

for ax, (title, honest, malicious, f) in zip(axes, scenarios):
    swarm = np.vstack([honest, malicious])
    
    # Calculate metrics
    naive_mean = np.mean(swarm, axis=0)
    krum_winner, _ = krum(swarm, f)
    honest_center = np.mean(honest, axis=0)
    
    # Plot
    ax.scatter(honest[:, 0], honest[:, 1], c='green', s=150, alpha=0.7, 
               label='Honest Nodes', zorder=3)
    ax.scatter(malicious[:, 0], malicious[:, 1], c='red', marker='X', s=200, 
               label='Malicious', zorder=3)
    ax.scatter(naive_mean[0], naive_mean[1], c='orange', marker='x', s=200, 
               linewidth=3, label='Naive Mean', zorder=4)
    ax.scatter(krum_winner[0], krum_winner[1], c='blue', marker='P', s=200, 
               edgecolors='black', label='Krum Selection', zorder=5)
    
    # Formatting
    ax.set_title(title, fontsize=12, fontweight='bold')
    ax.set_xlabel("Dimension X")
    ax.set_ylabel("Dimension Y")
    ax.legend(loc='upper left', fontsize=8)
    ax.grid(True, linestyle='--', alpha=0.6)
    
    # Metrics annotation
    mean_err = np.linalg.norm(naive_mean - honest_center)
    krum_err = np.linalg.norm(krum_winner - honest_center)
    ax.annotate(f'Mean Err: {mean_err:.2f}\nKrum Err: {krum_err:.2f}', 
                xy=(0.98, 0.02), xycoords='axes fraction',
                ha='right', va='bottom', fontsize=9,
                bbox=dict(boxstyle='round', facecolor='white', alpha=0.8))

plt.suptitle("QRES Krum Robustness: All Scenarios Pass", fontsize=14, fontweight='bold', y=1.02)
plt.tight_layout()
plt.savefig('docs/images/robustness_comparison.png', dpi=300, bbox_inches='tight')
print("Saved to docs/images/robustness_comparison.png")
plt.show()
