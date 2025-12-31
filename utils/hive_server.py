from flask import Flask, request, jsonify
import json
import logging

# Phase 19: The Aggregator
# Receives "Living Brains" (Confidence Scores) and performs Federated Averaging.

app = Flask(__name__)
log = logging.getLogger('werkzeug')
log.setLevel(logging.ERROR) # Silence request logs

# In-Memory Storage of Brains
# List of Brain Objects: {"confidence": [f32; 6]}
contributions = []

@app.route('/contribute', methods=['POST'])
def contribute():
    brain = request.json
    if not brain or 'confidence' not in brain:
        return jsonify({"status": "error", "message": "Invalid Brain"}), 400
    
    # Store
    contributions.append(brain)
    print(f"[Hive] Received Contribution. Total contributions: {len(contributions)}")
    return jsonify({"status": "accepted", "pool_size": len(contributions)})

@app.route('/global_brain', methods=['GET'])
def get_global_brain():
    if not contributions:
        # Return default neutral brain if pool is empty
        return jsonify({"confidence": [1.0] * 6})
    
    # Federated Averaging (FedAvg)
    # Determine dimension from first contribution
    if not contributions:
         return jsonify({"confidence": []})
         
    num_engines = len(contributions[0]['confidence'])
    totals = [0.0] * num_engines
    count = len(contributions)
    
    for brain in contributions:
        # Handle mismatch if any (truncate or pad?)
        # For now, assume consistent version. Cap at num_engines.
        b_conf = brain.get('confidence', [])
        for i in range(min(len(b_conf), num_engines)):
            totals[i] += b_conf[i]
            
    # Average
    averaged = [t / count for t in totals]
    print(f"[Hive] Distributing Global Brain (Avg of {count} agents): {averaged}")
    return jsonify({"confidence": averaged})

@app.route('/reset', methods=['POST'])
def reset():
    contributions.clear()
    print("[Hive] Brain Pool Reset")
    return jsonify({"status": "reset"})

if __name__ == '__main__':
    print("[Hive] Hive Server active on port 5000...")
    app.run(port=5000, debug=False)
