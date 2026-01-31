"""
QRES Byzantine Fault Tolerance Visualization
Generates a scatter plot showing Krum's outlier rejection in action.

Usage: python visualize_attack.py
Output: krum_defense.png
"""

import matplotlib.pyplot as plt
import numpy as np

# --- Data from your Log ---
honest_vectors = np.array([
    [1.00, 1.10],
    [0.90, 1.00],
    [1.05, 0.95],
    [1.00, 1.00]
])
malicious_vector = np.array([[100.0, 100.0]])  # Far outlier
compromised_mean = np.array([20.79, 20.81])
krum_selection = np.array([1.00, 1.00])  # Selected honest node

# --- Setup Plot ---
plt.figure(figsize=(10, 6))
plt.style.use('ggplot')  # Clean, professional style

# 1. Plot Honest Nodes (Green Cluster)
plt.scatter(honest_vectors[:, 0], honest_vectors[:, 1], 
            c='green', s=150, alpha=0.7, label='Honest Nodes (Consensus)')

# 2. Plot Malicious Node (Red Outlier)
plt.scatter(malicious_vector[:, 0], malicious_vector[:, 1], 
            c='red', marker='X', s=200, label='Malicious Outlier')

# 3. Plot Naive Mean (Orange X)
plt.scatter(compromised_mean[0], compromised_mean[1], 
            c='orange', marker='x', s=200, linewidth=3, label='Naive Mean (Poisoned)')

# 4. Plot Krum Selection (Blue Shield)
plt.scatter(krum_selection[0], krum_selection[1], 
            c='blue', marker='P', s=200, edgecolors='black', label='Krum Selection (Protected)')

# --- Formatting ---
plt.title("QRES Byzantine Defense: Krum vs. Poisoned Mean", fontsize=14, fontweight='bold')
plt.xlabel("Gradient Dimension X")
plt.ylabel("Gradient Dimension Y")
plt.legend(loc='upper right', frameon=True, shadow=True)
plt.grid(True, linestyle='--', alpha=0.6)

# Annotate the "Save"
plt.annotate('Protected Consensus\n(Remains in Cluster)', 
             xy=(1.0, 1.0), xytext=(2.5, 2.5),
             arrowprops=dict(facecolor='blue', shrink=0.05))

plt.annotate('Poisoned Average\n(Pulled towards Attack)', 
             xy=(20.79, 20.81), xytext=(15, 15),
             arrowprops=dict(facecolor='orange', shrink=0.05))

# Use symlog scale to show both the cluster (at ~1.0) and the outlier (at 100)
plt.xscale('symlog')
plt.yscale('symlog')

plt.tight_layout()
plt.savefig('docs/images/krum_defense.png', dpi=300)
print("✅ Visualization saved to 'docs/images/krum_defense.png'")
plt.show()
