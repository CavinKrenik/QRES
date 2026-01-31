"""
Figure 3: Temporal Evolution of QRES Swarm Consensus
Creates a 2x2 panel showing convergence at T=0, 10, 20, 50 with trajectory trails.
Suitable for PDF publication (no GIF).

Usage: python figure3_static_evolution.py
Output: docs/images/figure3_static_evolution.png
"""

import numpy as np
import matplotlib.pyplot as plt

# --- Configuration ---
N_HONEST = 20
N_MALICIOUS = 3
TOTAL_STEPS = 51
SNAPSHOTS = [0, 10, 20, 50]
LEARNING_RATE = 0.08
NOISE_LEVEL = 0.1

# --- Krum Logic ---
def krum_aggregate(vectors, f):
    n = len(vectors)
    k = n - f - 2
    if k < 1: return vectors[0]
    
    scores = []
    for i in range(n):
        dists = np.sum((vectors - vectors[i])**2, axis=1)
        dists.sort()
        score = np.sum(dists[:k]) 
        scores.append(score)
    return vectors[np.argmin(scores)]

# --- Simulation ---
np.random.seed(42)
honest_nodes = np.random.normal(1.0, 2.0, (N_HONEST, 2))
malicious_nodes = np.random.normal(8.0, 0.2, (N_MALICIOUS, 2))
history_honest = [honest_nodes.copy()]
targets_krum = []
targets_mean = []

for t in range(TOTAL_STEPS):
    all_nodes = np.vstack([honest_nodes, malicious_nodes])
    
    t_krum = krum_aggregate(all_nodes, f=N_MALICIOUS)
    t_mean = np.mean(all_nodes, axis=0)
    targets_krum.append(t_krum)
    targets_mean.append(t_mean)
    
    move = (t_krum - honest_nodes) * LEARNING_RATE
    noise = np.random.normal(0, NOISE_LEVEL, honest_nodes.shape)
    honest_nodes = honest_nodes + move + noise
    history_honest.append(honest_nodes.copy())

# --- Plotting 2x2 Grid ---
fig, axes = plt.subplots(2, 2, figsize=(12, 10))
axes = axes.flatten()
plt.style.use('ggplot')

history_honest = np.array(history_honest)

for i, t in enumerate(SNAPSHOTS):
    ax = axes[i]
    ax.set_xlim(-3, 10)
    ax.set_ylim(-3, 10)
    ax.grid(True, linestyle='--', alpha=0.3)
    ax.set_title(f"Time Step: T={t}", fontsize=12, fontweight='bold')
    
    # Trajectories
    if t > 0:
        for node_idx in range(N_HONEST):
            path = history_honest[:t+1, node_idx, :]
            ax.plot(path[:,0], path[:,1], c='green', alpha=0.15, linewidth=1)

    # Current Positions
    current_honest = history_honest[t]
    ax.scatter(current_honest[:,0], current_honest[:,1], c='forestgreen', s=60, alpha=0.8, 
               label='Honest Node' if i==0 else "")
    ax.scatter(malicious_nodes[:,0], malicious_nodes[:,1], c='crimson', marker='X', s=100, 
               label='Attacker' if i==0 else "")
    
    # Targets
    curr_krum = targets_krum[min(t, len(targets_krum)-1)]
    curr_mean = targets_mean[min(t, len(targets_mean)-1)]
    
    ax.plot(curr_krum[0], curr_krum[1], 'P', c='blue', ms=12, mew=2, 
            label='QRES Consensus' if i==0 else "")
    ax.plot(curr_mean[0], curr_mean[1], 'x', c='orange', ms=12, mew=2, 
            label='Naive Mean' if i==0 else "")
    
    # Byzantine Pull Arrow
    honest_center = np.mean(current_honest, axis=0)
    ax.annotate("", xy=curr_mean, xytext=honest_center,
                arrowprops=dict(arrowstyle="->", color="orange", lw=2, linestyle='--'))
    
    if i == 0:
        ax.legend(loc='upper left', fontsize=9)

    # Stats Box
    variance = np.var(current_honest)
    ax.text(0.95, 0.05, f"Cluster Var: {variance:.2f}", transform=ax.transAxes, 
            ha='right', fontsize=9, bbox=dict(boxstyle='round', facecolor='white', alpha=0.8))

plt.suptitle("Figure 3: Temporal Evolution of QRES Swarm Consensus vs. Attack", fontsize=16, y=0.98)
plt.tight_layout(rect=[0, 0.03, 1, 0.95])
plt.savefig('docs/images/figure3_static_evolution.png', dpi=300)
print("Saved to docs/images/figure3_static_evolution.png")
plt.show()
