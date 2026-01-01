import subprocess
import time
import os
import shutil
import json

# Setup
BRAIN_1 = "benchmarks/results/brain_node_1.json"
BRAIN_2 = "benchmarks/results/brain_node_2.json"
CLI = "qres_rust/target/release/qres-cli.exe"

if os.path.exists(BRAIN_1): os.remove(BRAIN_1)
if os.path.exists(BRAIN_2): os.remove(BRAIN_2)

# Create 2 distinct brains with different confidence to verify merging
brain1 = {
    "version": 1,
    "confidence": [0.8, 0.1, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0], # Likes Linear
    "stats": {"compressions": 10},
    "predictors": ["lstm", "graph"]
}
brain2 = {
    "version": 1,
    "confidence": [0.0, 0.0, 0.8, 0.2, 0.0, 0.0, 0.0, 0.0], # Likes Graph
    "stats": {"compressions": 20},
    "predictors": ["lstm", "graph"]
}

with open(BRAIN_1, "w") as f: json.dump(brain1, f)
with open(BRAIN_2, "w") as f: json.dump(brain2, f)

print("[Sim] Launching Node 1...")
p1 = subprocess.Popen([CLI, "swarm", "--brain", BRAIN_1], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)

print("[Sim] Launching Node 2...")
p2 = subprocess.Popen([CLI, "swarm", "--brain", BRAIN_2], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)

print("[Sim] Waiting for mDNS discovery and sync (30s)...")
time.sleep(30)

print("[Sim] Killing nodes...")
p1.terminate()
p2.terminate()

# Verify brains changed
print("[Sim] Verifying brains...")
with open(BRAIN_1, "r") as f: b1_new = json.load(f)
with open(BRAIN_2, "r") as f: b2_new = json.load(f)

print(f"Node 1 Conf: {b1_new['confidence']}")
print(f"Node 2 Conf: {b2_new['confidence']}")

# Check convergence (approximate due to 10s tick)
c1 = b1_new["confidence"]
c2 = b2_new["confidence"]

# Did they change?
changed1 = c1 != brain1["confidence"]
changed2 = c2 != brain2["confidence"]

if changed1 or changed2:
    print("[Success] Brains evolved via P2P Swarm!")
else:
    print("[Fail] Brains identical to start. P2P sync failed.")

# Log output
print("\n--- Node 1 Output ---")
try:
    print(p1.stderr.read())
except: pass
print("\n--- Node 2 Output ---")
try:
    print(p2.stderr.read())
except: pass
