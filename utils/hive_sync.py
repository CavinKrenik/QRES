import requests
import subprocess
import json
import os
import sys
import time

# Phase 2: Hive-Optimized Swarm (FedProx)
# Connects local qres-cli to the Hive with intelligent brain merging.

HIVE_URL = str(os.getenv("HIVE_URL", "http://localhost:5000"))
CLI_PATH = str(os.getenv("QRES_CLI", "qres_rust/target/release/qres-cli")) # Default fallback
if sys.platform == "win32" and not CLI_PATH.endswith(".exe"):
    CLI_PATH += ".exe"

def run_cli(args):
    """Run the Rust QRES CLI."""
    # Check debug if release not found (fallback)
    cmd = CLI_PATH
    if not os.path.exists(cmd):
        cmd = cmd.replace("release", "debug")
        
    if not os.path.exists(cmd):
        print(f"[Error] CLI binary not found at {CLI_PATH}")
        return None

    try:
        result = subprocess.run([cmd] + args, capture_output=True, text=True)
        if result.returncode != 0:
            print(f"CLI Error: {result.stderr}")
            return None
        return result.stdout.strip()
    except Exception as e:
        print(f"Failed to run CLI: {e}")
        return None

def fed_prox_merge(local_brain, global_brain):
    """
    Implements FedProx-style merging.
    Rules:
    1. Zero-Shot Adaptation: If local agent is new (low stats), inherit Global perfectly.
    2. Proximal Optimization: If local is experienced, pull towards Global but respect Local variance.
    """
    print("[FedProx] Computing merge...")
    
    # 1. Check Experience
    local_conf = local_brain.get("confidence", [0.5]*6)
    local_stats = local_brain.get("stats", {})
    compressions = local_stats.get("compressions", 0)
    
    global_conf = global_brain.get("confidence", [0.5]*6)
    
    # Align lengths
    min_len = min(len(local_conf), len(global_conf))
    local_conf = local_conf[:min_len]
    global_conf = global_conf[:min_len]
    
    merged_conf = []

    # 2. Logic
    if compressions < 1000:
        print(f"   Shape-Shifting: Agent is young ({compressions} ops). Inheriting Global Wisdom.")
        # Zero-Shot Adaptation: Agent B becomes Agent A instantly
        merged_conf = global_conf
    else:
        print(f"   Proximal Update: Agent is experienced ({compressions} ops). Merging.")
        # FedProx: w_new = w_local - mu * (w_local - w_global)
        # Effectively: w_new = (1-mu)*w_local + mu*w_global
        
        # Calculate divergence
        divergence = sum(abs(l - g) for l, g in zip(local_conf, global_conf))
        print(f"   Divergence: {divergence:.4f}")
        
        # Adaptive mu based on divergence (higher divergence = trust global less? or more?)
        # FedProx prevents straying too far from global.
        # Let's say we trust Global (The Hive) as the anchor.
        mu = 0.3 # Moderate pull towards global
        
        for l, g in zip(local_conf, global_conf):
            val = (1.0 - mu) * l + mu * g
            merged_conf.append(val)
            
            
    # Update local brain structure
    local_brain["confidence"] = merged_conf
    local_brain["global_confidence"] = global_conf # Persistence for continuous FedProx
    return local_brain

def sync():
    print(f"[Sync] Connecting to Hive at {HIVE_URL}...")
    
    # 1. Export Local Brain
    # 1. Export Local Brain
    print("[Export] Exporting Local Intuition...")
    # CLI requires file argument: qres-cli export-brain <FILE>
    _ = run_cli(["export-brain", "temp_brain.json"])
    
    if not os.path.exists("temp_brain.json"):
        print("Failed to export brain (file missing).")
        return

    try:
        with open("temp_brain.json", "r") as f:
            local_brain = json.load(f)
        os.remove("temp_brain.json")
    except Exception as e:
        print(f"Invalid JSON/Read Error: {e}")
        return

    # 2. Push to Hive (Contribution)
    # We contribute BEFORE merging so the hive sees our raw local learnings
    try:
        res = requests.post(f"{HIVE_URL}/contribute", json=local_brain)
        if res.status_code == 200:
            print("[OK] Contribution Accepted.")
        else:
            print(f"[Error] Push Failed: {res.text}")
    except Exception as e:
        print(f"[Error] Hive Unreachable: {e}")
        return

    # 3. Pull from Hive
    print("[Download] Downloading Global Wisdom...")
    try:
        res = requests.get(f"{HIVE_URL}/global_brain")
        if res.status_code == 200:
            global_brain = res.json()
            
            # 4. FedProx Merge
            merged_brain = fed_prox_merge(local_brain, global_brain)
            
            # Save to temp file
            with open("merged_brain.json", "w") as f:
                json.dump(merged_brain, f)
            
            # 5. Import (Overwrite)
            print("[Merge] Assimilating Knowledge...")
            # Use 'import-brain' (kebab-case)
            out = run_cli(["import-brain", "merged_brain.json"])
            print(out)
            
            # Cleanup
            if os.path.exists("merged_brain.json"):
                os.remove("merged_brain.json")
        else:
            print(f"[Error] Pull Failed: {res.text}")
    except Exception as e:
        print(f"[Error] Pull Error: {e}")

if __name__ == "__main__":
    sync()
