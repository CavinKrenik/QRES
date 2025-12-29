import subprocess
import time
import os
import json
import sys
import shutil

# Phase 19: The Swarm Simulation
# Demonstrates "Zero-Shot Adaptation" via Federated Intelligence.

CLI = "qres_rust/target/debug/qres-cli"
if sys.platform == "win32": CLI += ".exe"

HIVE_SERVER = "utils/hive_server.py"
HIVE_SYNC = "utils/hive_sync.py"
DATA_GEN = "benchmarks/drifting_signal.py"
BRAIN_FILE = "qres_brain.json"

def run_command(cmd, capture=False):
    print(f"🚀 Running: {' '.join(cmd)}")
    if capture:
        return subprocess.run(cmd, capture_output=True, text=True)
    return subprocess.run(cmd)

def check_brain_confidence(id, threshold):
    if not os.path.exists(BRAIN_FILE):
        print("❌ Brain file missing!")
        return False
    
    with open(BRAIN_FILE) as f:
        data = json.load(f)
        conf = data['confidence'][id]
        print(f"🧠 Engine {id} Confidence: {conf:.4f}")
        return conf > threshold

def main():
    print("=== 🐝 Phase 19: Swarm Simulation ===")
    
    # 0. Setup
    if os.path.exists(BRAIN_FILE): os.remove(BRAIN_FILE)
    run_command(["python", DATA_GEN]) # Generate drift.bin
    
    # Start Hive
    print("🐝 Starting Hive Server...")
    # Remove DEVNULL to see errors
    hive_proc = subprocess.Popen(["python", HIVE_SERVER]) 
    time.sleep(5) # Warmup
    
    try:
        # 1. Agent A (The Teacher)
        print("\n🎓 Agent A: Learning from Experience...")
        # Run compress. This will trigger punishment/learning logic.
        run_command([CLI, "compress", "drift.bin", "a.qres"])
        
        # Verify A learned iPEPS (ID 5)
        if not check_brain_confidence(5, 0.8):
             print("⚠️ Agent A failed to learn iPEPS (Confidence too low).")
        
        # 2. Sync A -> Hive
        print("\n📤 Agent A: Uploading Wisdom to Hive...")
        run_command(["python", HIVE_SYNC])
        
        # 3. Agent B (The Student)
        print("\n👶 Agent B: Born (Fresh State)...")
        if os.path.exists(BRAIN_FILE): os.remove(BRAIN_FILE) # Wipe memory
        
        # 4. Sync Hive -> B
        print("\n📥 Agent B: Downloading Wisdom from Hive...")
        run_command(["python", HIVE_SYNC])
        
        # Verify B has the knowledge BEFORE ever seeing data
        print("\n🔎 Verifying Agent B's Zero-Shot Intuition...")
        is_smart = check_brain_confidence(5, 0.9) # Should be high (avg of 1.0 and A's score)
        
        if is_smart:
            print("\n✅ SUCCESS: Agent B acquired iPEPS intuition without training!")
        else:
            print("\n❌ FAILURE: Agent B did not acquire intuition.")
            
        # 5. Run B
        print("\n🏃 Agent B: Compressing with Inherited Wisdom...")
        run_command([CLI, "compress", "drift.bin", "b.qres", "--explain"])
        
    finally:
        print("\n🧹 Cleaning up...")
        hive_proc.terminate()
        # if os.path.exists(BRAIN_FILE): os.remove(BRAIN_FILE)

if __name__ == "__main__":
    main()
