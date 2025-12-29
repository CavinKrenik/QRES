import requests
import subprocess
import json
import os
import sys
import time

# Phase 19: The Synapse (Network Bridge)
# Connects local qres-cli to the Hive.

HIVE_URL = str(os.getenv("HIVE_URL", "http://localhost:5000"))
CLI_PATH = "qres_rust/target/debug/qres-cli" # Assuming Dev environment
if sys.platform == "win32":
    CLI_PATH += ".exe"

def run_cli(args):
    result = subprocess.run([CLI_PATH] + args, capture_output=True, text=True)
    if result.returncode != 0:
        print(f"CLI Error: {result.stderr}")
        return None
    return result.stdout.strip()

def sync():
    print(f"🔌 Connecting to Hive at {HIVE_URL}...")
    
    # 1. Export Local Brain
    print("📤 Exporting Local Intuition...")
    local_json = run_cli(["brain-export"])
    if not local_json:
        print("Failed to export brain.")
        return
    
    try:
        brain_data = json.loads(local_json)
    except:
        print("Invalid JSON from CLI")
        return

    # 2. Push to Hive
    try:
        res = requests.post(f"{HIVE_URL}/contribute", json=brain_data)
        if res.status_code == 200:
            print("✅ Contribution Accepted.")
        else:
            print(f"❌ Push Failed: {res.text}")
    except Exception as e:
        print(f"❌ Hive Unreachable: {e}")
        return

    # 3. Pull from Hive
    print("📥 Downloading Global Wisdom...")
    try:
        res = requests.get(f"{HIVE_URL}/global_brain")
        if res.status_code == 200:
            global_brain = res.json()
            
            # Save to temp file
            with open("global_brain.json", "w") as f:
                json.dump(global_brain, f)
            
            # 4. Import (Merge)
            print("🧠 Assimilating Knowledge...")
            out = run_cli(["brain-import", "global_brain.json"])
            print(out)
            
            # Cleanup
            os.remove("global_brain.json")
        else:
            print(f"❌ Pull Failed: {res.text}")
    except Exception as e:
        print(f"❌ Pull Error: {e}")

if __name__ == "__main__":
    sync()
