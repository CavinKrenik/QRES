import numpy as np
import os
import sys
from sklearn.tree import DecisionTreeClassifier, export_text
from sklearn.model_selection import train_test_split
import qres
import math

# --- 1. CONFIG ---
SAMPLES_PER_TYPE = 50
CHUNK_SIZE = 4096  # 4KB Analysis Window
FULL_SIZE = 64 * 1024 # 64KB for race

# --- 2. DATA GENERATORS ---
def gen_sine(size):
    x = np.linspace(0, 100 * np.pi, size)
    y = np.sin(x) * 100 + 128
    return y.astype(np.uint8).tobytes()

def gen_linear(size):
    # Ramp
    return np.arange(size, dtype=np.uint8).tobytes()

def gen_noise(size):
    return np.random.randint(0, 256, size, dtype=np.uint8).tobytes()

def gen_text(size):
    # Geometric distribution for ASCII
    return np.random.geometric(p=0.01, size=size).astype(np.uint8).tobytes() # Rough approx

def gen_sparse(size):
    return np.zeros(size, dtype=np.uint8).tobytes()

# --- 3. FEATURE EXTRACTION ---
def get_features(data):
    # First 4KB
    chunk = np.frombuffer(data[:CHUNK_SIZE], dtype=np.uint8)
    if len(chunk) < 2: return [0,0,0,0]
    
    mean = np.mean(chunk)
    var = np.var(chunk)
    
    # Entropy
    counts = np.bincount(chunk, minlength=256)
    probs = counts[counts > 0] / len(chunk)
    entropy = -np.sum(probs * np.log2(probs))
    
    # Zero Crossing Rate (Approx raw changes)
    diffs = np.diff(chunk.astype(np.int16))
    zcr = np.sum(np.abs(diffs) > 10) / len(chunk) # Changes > 10
    
    return [mean, var, entropy, zcr]

# --- 4. LABELING (THE RACE) ---
def get_winner(data):
    # Compress with all 3 engines
    try:
        # qres.encode_bytes(bytes, predictor_id, weights)
        # ID 1 = Linear, 3 = LSTM, 4 = Tensor
        # Note: Weights must be handled. 
        # For this training, we use default/empty weights for Linear
        # For LSTM/Tensor, we hopefully have the weights in the lib or need to pass them?
        # The python binding expects weights option.
        # Problem: We don't have easy access to the embedded weights from Python 
        # unless we extracted them or `qres` exposes them.
        # HACK: Use ID 1 (Linear) vs ID 0 (Previous) vs Zlib?
        # Wait, the user wants Linear vs Tensor vs LSTM.
        # We need the weights.
        # Let's assume the installed qres package has the latest build where we might not need to pass weights 
        # IF the python binding had a high level compress function. It does not.
        # It has `encode_bytes(data, id, weights)`.
        
        # ACTUALLY, the Rust `qres` lib has `LSTM_WEIGHTS` embedded. 
        # But `encode_bytes` takes `weights: Option<&[u8]>`. 
        # If we pass None, it initializes new (empty) weights which is BAD for LSTM/Tensor.
        
        # WORKAROUND: We will read the weights from the file system since we are in the repo.
        pass
    except:
        pass

# Load Weights
def load_weights():
    base = os.path.dirname(os.path.abspath(__file__))
    assets = os.path.join(base, "../qres_rust/assets")
    
    with open(os.path.join(assets, "lstm.qnn"), "rb") as f:
        lstm_w = f.read()
    with open(os.path.join(assets, "tensor.qnn"), "rb") as f:
        tensor_w = f.read()
    return lstm_w, tensor_w

lstm_weights, tensor_weights = load_weights()

def race(data):
    # 1. Linear
    c1 = qres.encode_bytes(data, 1, None)
    l1 = len(c1)
    
    # 3. LSTM
    c3 = qres.encode_bytes(data, 3, lstm_weights)
    l3 = len(c3)
    
    # 4. Tensor
    c4 = qres.encode_bytes(data, 4, tensor_weights)
    l4 = len(c4)
    
    # Winner?
    scores = {1: l1, 3: l3, 4: l4}
    winner = min(scores, key=scores.get)
    return winner

# --- 5. MAIN LOOP ---
print("Generating Data & Racing...")
X = []
y = []

generators = [gen_sine, gen_linear, gen_noise, gen_text, gen_sparse]

for gen in generators:
    for _ in range(SAMPLES_PER_TYPE):
        data = gen(FULL_SIZE)
        feats = get_features(data)
        label = race(data)
        X.append(feats)
        y.append(label)

print(f"Collected {len(X)} samples.")

# --- 6. TRAINING ---
clf = DecisionTreeClassifier(max_depth=5, random_state=42)
clf.fit(X, y)

accuracy = clf.score(X, y)
print(f"Model Accuracy: {accuracy:.2f}")

# --- 7. EXPORT TO RUST ---
# We need to walk the tree and print Rust code
tree = clf.tree_
feature_names = ["mean", "var", "entropy", "zcr"]

def tree_to_rust(node, depth, conditions=[]):
    indent = "    " * depth
    if tree.children_left[node] == tree.children_right[node]: # Leaf
        # Class index
        class_idx = np.argmax(tree.value[node])
        class_val = clf.classes_[class_idx]
        
        # Explainability: Join conditions
        if not conditions:
            reason = "Default"
        else:
            # Simplification: Take the last 2 important conditions or all?
            # Let's simple take all for true neuro-symbolic trace.
            reason = ", ".join(conditions)
            
        return f'{indent}({class_val}, "{reason}")\n'
    else:
        threshold = tree.threshold[node]
        feat_idx = tree.feature[node]
        feat_name = feature_names[feat_idx]
        
        cond_left = f"{feat_name} <= {threshold:.2f}"
        cond_right = f"{feat_name} > {threshold:.2f}"
        
        left = tree_to_rust(tree.children_left[node], depth + 1, conditions + [cond_left])
        right = tree_to_rust(tree.children_right[node], depth + 1, conditions + [cond_right])
        
        return f"{indent}if {feat_name} <= {threshold:.4f} {{\n{left}{indent}}} else {{\n{right}{indent}}}\n"

rust_code = f"""// Generated by ai/train_meta.py
// Accuracy: {accuracy:.2f}

pub fn predict(mean: f32, var: f32, entropy: f32, zcr: f32) -> (u8, &'static str) {{
{tree_to_rust(0, 1)}}}
"""

with open("qres_rust/src/meta_brain.rs", "w") as f:
    f.write(rust_code)

print("Generated qres_rust/src/meta_brain.rs")
