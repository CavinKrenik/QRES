from flask import Flask, request, jsonify
import json
import logging
import os
import numpy as np

# QRES v4.2 Hive Server
# Implements Federated Learning (FedProx-inspired weighted aggregation)
# Features:
# - Weighted averaging based on contribution size (compressions count)
# - Persistence (saves global state to disk)
# - Version checking

app = Flask(__name__)
log = logging.getLogger('werkzeug')
log.setLevel(logging.ERROR)

STATE_FILE = "global_brain_state.json"

# In-Memory State
global_state = {
    "weights": None,          # Current global weights
    "round": 0,               # FL Round
    "total_samples": 0,       # Total samples seen
    "min_clients": 1,         # Min clients to update
    "metrics_history": [],    # Track convergence over time
    "active_clients": set()   # Track unique participant IDs
}

def load_state():
    global global_state
    if os.path.exists(STATE_FILE):
        try:
            with open(STATE_FILE, 'r') as f:
                data = json.load(f)
                # Filter out sets/complex types before update
                if "active_clients" in data:
                    data["active_clients"] = set(data["active_clients"])
                global_state.update(data)
                # Ensure weights are list if loaded
                if global_state["weights"] and isinstance(global_state["weights"], list):
                     global_state["weights"] = np.array(global_state["weights"])
            print(f"[Hive] Loaded Global Brain (Round {global_state['round']})")
        except Exception as e:
            print(f"[Hive] Failed to load state: {e}")

def save_state():
    # Convert numpy to list for JSON
    save_data = global_state.copy()
    if isinstance(save_data["weights"], np.ndarray):
        save_data["weights"] = save_data["weights"].tolist()
    if isinstance(save_data["active_clients"], set):
        save_data["active_clients"] = list(save_data["active_clients"])
    
    with open(STATE_FILE, 'w') as f:
        json.dump(save_data, f)

# Load state on startup
load_state()

# Pending contributions buffer
pending_contributions = []

@app.route('/contribute', methods=['POST'])
def contribute():
    """
    Accepts client contribution:
    {
        "confidence": [float],
        "samples": int,      # How many compressions/interactions
        "client_id": str
    }
    """
    data = request.json
    if not data or 'confidence' not in data:
        return jsonify({"status": "error", "message": "Invalid Payload"}), 400
    
    # Validate dimensions
    weights = np.array(data['confidence'], dtype=np.float32)
    samples = data.get('samples', 1) 
    client_id = data.get('client_id', 'anon')
    
    global_state["active_clients"].add(client_id)
    
    pending_contributions.append({
        "weights": weights,
        "samples": samples,
        "client_id": client_id
    })
    
    print(f"[Hive] Contribution from {client_id} (n={samples}). Pending: {len(pending_contributions)}")
    
    # Aggregation Trigger (FedProx-ish)
    # If we have enough updates, aggregate immediately for this demo
    if len(pending_contributions) >= global_state["min_clients"]:
        aggregate_updates()
        
    return jsonify({
        "status": "accepted", 
        "round": global_state["round"],
        "global_ver": global_state["round"]
    })

def aggregate_updates():
    """
    Performs Weighted Federated Averaging
    W_global = (Sum(W_i * n_i)) / Sum(n_i)
    """
    global global_state
    
    if not pending_contributions:
        return

    # 1. Initialize Global if empty
    first_contrib = pending_contributions[0]["weights"]
    if global_state["weights"] is None:
        global_state["weights"] = np.zeros_like(first_contrib)

    # 2. Weighted Sum
    total_samples_round = sum(c["samples"] for c in pending_contributions)
    weighted_sum = np.zeros_like(global_state["weights"])
    
    # Track variance for metrics
    stacked_weights = []
    
    for c in pending_contributions:
        # Match dimensions if needed (safety)
        w = c["weights"]
        stacked_weights.append(w)
        if w.shape != weighted_sum.shape:
             # Basic versioning/truncation logic
             common_len = min(len(w), len(weighted_sum))
             weighted_sum[:common_len] += w[:common_len] * c["samples"]
        else:
            weighted_sum += w * c["samples"]
            
    # Calculate Round Variance (Convergence Metric)
    # High variance = agents disagree (divergence)
    # Low variance = consensus
    if len(stacked_weights) > 1:
        # Clean shapes
        min_len = min(len(w) for w in stacked_weights)
        clean_stack = [w[:min_len] for w in stacked_weights]
        round_variance = np.var(clean_stack, axis=0).mean()
    else:
        round_variance = 0.0

    # 3. Mixing with previous global (Momentum/Stability)
    aggregated_update = weighted_sum / max(1, total_samples_round)
    
    if global_state["total_samples"] == 0:
        global_state["weights"] = aggregated_update
    else:
        # FedProx: Proximal term simulation via heavy momentum
        alpha = 0.7 
        global_state["weights"] = (alpha * global_state["weights"]) + ((1 - alpha) * aggregated_update)

    global_state["total_samples"] += total_samples_round
    global_state["round"] += 1
    
    # Record Metrics
    metric_entry = {
        "round": global_state["round"],
        "clients": len(pending_contributions),
        "total_samples": global_state["total_samples"],
        "variance": float(round_variance),
        "weights_mean": float(np.mean(global_state["weights"]))
    }
    global_state["metrics_history"].append(metric_entry)
    
    # Clear buffer
    pending_contributions.clear()
    save_state()
    
    print(f"[Hive] Aggregated Round {global_state['round']}. Var: {round_variance:.4f}")

@app.route('/global_brain', methods=['GET'])
def get_global_brain():
    if global_state["weights"] is None:
        return jsonify({"confidence": [1.0] * 4, "round": 0})
        
    return jsonify({
        "confidence": global_state["weights"].tolist(),
        "round": global_state["round"]
    })

@app.route('/metrics', methods=['GET'])
def get_metrics():
    return jsonify({
        "history": global_state["metrics_history"],
        "active_clients": len(global_state["active_clients"]),
        "current_round": global_state["round"]
    })

@app.route('/reset', methods=['POST'])
def reset():
    global global_state
    global_state = {
        "weights": None,
        "round": 0,
        "total_samples": 0,
        "min_clients": 1,
        "metrics_history": [],
        "active_clients": set()
    }
    if os.path.exists(STATE_FILE):
        os.remove(STATE_FILE)
    print("[Hive] Brain Pool Reset")
    return jsonify({"status": "reset"})

if __name__ == '__main__':
    print(f"[Hive] Server v4.2 active on port 5000. PID: {os.getpid()}")
    print("[Hive] Ready to aggregate collective intelligence.")
    app.run(port=5000, debug=False)
