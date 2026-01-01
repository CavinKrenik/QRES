import pandas as pd
import numpy as np
import json
import os
from sklearn.neural_network import MLPRegressor
from sklearn.model_selection import train_test_split
from sklearn.preprocessing import StandardScaler

# 1. Load Data
DATA_PATH = "benchmarks/training_data.csv"
if not os.path.exists(DATA_PATH):
    print(f"Error: {DATA_PATH} not found.")
    exit(1)

df = pd.read_csv(DATA_PATH)
print(f"Loaded {len(df)} samples.")

# 2. Preprocessing
X = df[["entropy", "mean", "variance", "autocorr_1"]].values
y = df[["w_linear", "w_simple", "w_graph", "w_spectral", "w_lz"]].values

# Filter out rows where weights don't sum to roughly 1 (validity check)
# Actually, the Mixer weights might diverge if not normalized, but update() normalizes them periodically.
# Let's normalize y just in case.
y_sums = y.sum(axis=1, keepdims=True)
y = y / (y_sums + 1e-6)

# Scale Input
scaler = StandardScaler()
X_scaled = scaler.fit_transform(X)

# 3. Train MLP
# Tiny architecture: 4 -> 16 -> 8 -> 5
print("Training MLP...")
mlp = MLPRegressor(
    hidden_layer_sizes=(16, 8),
    activation="relu",
    solver="adam",
    max_iter=5000,
    random_state=42
)
mlp.fit(X_scaled, y)

print(f"Score: {mlp.score(X_scaled, y):.4f}")

# 4. Export Weights
# We need to export: Scaler params (mean, scale) AND MLP weights (coefs_, intercepts_)
model_data = {
    "scaler_mean": scaler.mean_.tolist(),
    "scaler_scale": scaler.scale_.tolist(),
    "layer_0_weights": mlp.coefs_[0].tolist(), # 4x16
    "layer_0_bias": mlp.intercepts_[0].tolist(), # 16
    "layer_1_weights": mlp.coefs_[1].tolist(), # 16x8
    "layer_1_bias": mlp.intercepts_[1].tolist(), # 8
    "layer_2_weights": mlp.coefs_[2].tolist(), # 8x5
    "layer_2_bias": mlp.intercepts_[2].tolist(), # 5
}

# Ensure assets dir
os.makedirs("qres_rust/assets", exist_ok=True)
OUT_PATH = "qres_rust/assets/meta_brain_v2.json"

with open(OUT_PATH, "w") as f:
    json.dump(model_data, f, indent=2)

print(f"Model exported to {OUT_PATH}")
