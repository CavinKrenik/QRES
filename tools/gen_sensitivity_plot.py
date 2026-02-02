import matplotlib.pyplot as plt

biases = [5, 10, 15, 20, 25, 30]
drift_probs = {
    "Naive Mean": [0.0, 0.0, 93.3, 100.0, 100.0, 100.0],
    "Median": [100.0, 100.0, 100.0, 100.0, 100.0, 100.0],
    "Multi-Krum": [0.0, 100.0, 100.0, 100.0, 100.0, 100.0],
}

plt.figure(figsize=(8, 5))
for agg, probs in drift_probs.items():
    plt.plot(biases, probs, marker="o", label=agg)
plt.xlabel("Bias (%)")
plt.ylabel("Drift Probability (%)")
plt.title("Sensitivity: Bias vs. Drift Probability (n=15, f=4)")
plt.grid(True)
plt.legend()
plt.ylim(0, 110)
plt.tight_layout()
plt.savefig("docs/images/sensitivity_plot.png", dpi=150)
print("Saved docs/images/sensitivity_plot.png")
