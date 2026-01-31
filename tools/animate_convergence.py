"""
QRES Swarm Convergence Animation
Shows honest nodes converging while ignoring coordinated attackers.

Usage: python animate_convergence.py
Output: docs/images/consensus_evolution.gif
"""

import numpy as np
import matplotlib.pyplot as plt
import matplotlib.animation as animation
from matplotlib.patches import Ellipse

# --- Configuration ---
N_HONEST = 15
N_MALICIOUS = 3
FRAMES = 50
LEARNING_RATE = 0.1

# --- Krum Logic ---
def krum_aggregate(vectors, f):
    n = len(vectors)
    if n < 2 * f + 3: 
        return vectors[0]
    k = n - f - 2
    
    scores = []
    for i in range(n):
        dists = np.sum((vectors - vectors[i])**2, axis=1)
        dists_sorted = np.sort(dists)
        score = np.sum(dists_sorted[:k+1])  # k nearest (excluding self)
        scores.append(score)
    
    best_idx = np.argmin(scores)
    return vectors[best_idx]

# --- Setup Data ---
np.random.seed(42)
honest_nodes = np.random.normal(1.0, 1.5, (N_HONEST, 2))
malicious_nodes = np.random.normal(8.0, 0.2, (N_MALICIOUS, 2))

# --- Setup Plot ---
fig, ax = plt.subplots(figsize=(10, 8))
ax.set_xlim(-4, 10)
ax.set_ylim(-4, 10)
ax.set_title("QRES Swarm Convergence vs. Coordinated Attack", fontsize=16, fontweight='bold')
ax.grid(True, linestyle='--', alpha=0.3)

honest_scatter = ax.scatter([], [], c='green', s=100, alpha=0.6, label='Honest Nodes')
malicious_scatter = ax.scatter([], [], c='red', marker='X', s=150, label='Malicious Attackers')
mean_marker, = ax.plot([], [], 'x', color='orange', markersize=15, markeredgewidth=3, label='Naive Mean (Poisoned)')
krum_marker, = ax.plot([], [], 'P', color='blue', markersize=15, markeredgewidth=3, label='QRES Consensus')

zone = Ellipse((0,0), width=0, height=0, color='blue', alpha=0.1)
ax.add_patch(zone)

ax.legend(loc='upper left', frameon=True)

# --- Animation ---
def update(frame):
    global honest_nodes
    
    all_nodes = np.vstack([honest_nodes, malicious_nodes])
    
    target_krum = krum_aggregate(all_nodes, f=N_MALICIOUS)
    target_mean = np.mean(all_nodes, axis=0)
    
    # Move honest nodes toward Krum consensus
    move_vectors = target_krum - honest_nodes
    honest_nodes = honest_nodes + move_vectors * LEARNING_RATE
    
    # Add thermal noise
    honest_nodes = honest_nodes + np.random.normal(0, 0.05, honest_nodes.shape)
    
    # Update visuals
    honest_scatter.set_offsets(honest_nodes)
    malicious_scatter.set_offsets(malicious_nodes)
    
    mean_marker.set_data([target_mean[0]], [target_mean[1]])
    krum_marker.set_data([target_krum[0]], [target_krum[1]])
    
    center = np.mean(honest_nodes, axis=0)
    spread = np.max(np.linalg.norm(honest_nodes - center, axis=1)) * 2.5
    zone.set_center(center)
    zone.width = spread
    zone.height = spread
    
    return honest_scatter, malicious_scatter, mean_marker, krum_marker, zone

print("Rendering animation (10-20 seconds)...")
ani = animation.FuncAnimation(fig, update, frames=FRAMES, interval=100, blit=True)
ani.save('docs/images/consensus_evolution.gif', writer='pillow', fps=15)
print("Saved to docs/images/consensus_evolution.gif")
