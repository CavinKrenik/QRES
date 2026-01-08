import subprocess
import time
import os
import shutil
import json
import urllib.request
import urllib.error

# Setup
BRAIN_1 = "benchmarks/results/brain_node_1.json"
BRAIN_2 = "benchmarks/results/brain_node_2.json"
CLI = "qres_rust/target/release/qres_daemon.exe"
PORT_1 = 8081
PORT_2 = 8082

if os.path.exists(BRAIN_1): os.remove(BRAIN_1)
if os.path.exists(BRAIN_2): os.remove(BRAIN_2)

# Create 2 distinct brains
brain1 = {
    "version": 1,
    "confidence": [0.8, 0.1, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0],
    "stats": {"compressions": 10},
    "predictors": ["lstm", "graph"]
}
brain2 = {
    "version": 1,
    "confidence": [0.0, 0.0, 0.8, 0.2, 0.0, 0.0, 0.0, 0.0],
    "stats": {"compressions": 20},
    "predictors": ["lstm", "graph"]
}

with open(BRAIN_1, "w") as f: json.dump(brain1, f)
with open(BRAIN_2, "w") as f: json.dump(brain2, f)

# Helper keys
KEY_1 = "benchmarks/results/node1.key"
KEY_2 = "benchmarks/results/node2.key"

# Clean up keys if exist (force regen by daemon if it did it, 
# but daemon won't auto-regen if file provided. We rely on SecurityManager creating it)
if os.path.exists(KEY_1): os.remove(KEY_1)
if os.path.exists(KEY_2): os.remove(KEY_2)

print(f"[Sim] Launching Node 1 on port {PORT_1}...")
p1 = subprocess.Popen([CLI, "swarm", "--brain", BRAIN_1, "--port", str(PORT_1), "--key", KEY_1], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)

print(f"[Sim] Launching Node 2 on port {PORT_2}...")
p2 = subprocess.Popen([CLI, "swarm", "--brain", BRAIN_2, "--port", str(PORT_2), "--key", KEY_2], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)

def get_status(port):
    try:
        with urllib.request.urlopen(f"http://127.0.0.1:{port}/status") as response:
            return json.loads(response.read().decode())
    except Exception as e:
        return None

print("[Sim] Polling API for connectivity...")
connected = False
for i in range(30):
    s1 = get_status(PORT_1)
    s2 = get_status(PORT_2)
    
    if s1 and s2:
        print(f"[{i}s] Node 1 Peers: {s1['connected_peers']} | Node 2 Peers: {s2['connected_peers']}")
        if s1['connected_peers'] > 0 and s2['connected_peers'] > 0:
            connected = True
            break
    else:
        print(f"[{i}s] Waiting for API...")
    time.sleep(1)

if connected:
    print("[Success] Nodes connected via P2P!")
else:
    print("[Fail] Nodes failed to connect within timeout.")

# Wait a bit for sync
print("[Sim] Waiting for sync logic (15s)...")
time.sleep(15)

print("[Sim] Fetching Brain state via API...")
b1_api = get_status(PORT_1)

print("[Sim] Killing nodes...")
p1.terminate()
p2.terminate()

# Verify brains on disk
print("[Sim] Verifying disk state...")
with open(BRAIN_1, "r") as f: b1_disk = json.load(f)

print(f"Start Conf: {brain1['confidence']}")
if b1_api:
    print(f"API Conf:   {b1_api['brain_confidence']}")
print(f"Disk Conf:  {b1_disk['confidence']}")

if b1_disk['confidence'] != brain1['confidence']:
    print("[Success] Brain evolved!")
else:
    print("[Fail] Brain did not change.")

# Log output
print("\n--- Node 1 Output ---")
try:
    print(p1.stderr.read())
except: pass
print("\n--- Node 2 Output ---")
try:
    print(p2.stderr.read())
except: pass
