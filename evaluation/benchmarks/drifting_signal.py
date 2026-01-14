import numpy as np
import subprocess
import os
import sys

def generate_drift_signal(filename):
    print("Generating Drifting Signal (Sine -> Chaos)...")
    
    # Phase 1: Pure Sine (Easy for Linear/Tensor) - 2MB
    t = np.linspace(0, 1000, 2000000) 
    msg1 = (np.sin(t) * 100 + 128).astype(np.uint8)
    
    # Phase 2: Interleaved Zero/Random (The "Comb" Pattern) - 8MB
    # Linear (Stride 1): Sees 0, R, 0, R. Predicts garbage. Ratio ~1.0.
    # iPEPS (Stride 2): Sees 0->0 (Perfect) and R->R (Fail). Ratio ~0.5.
    
    n_points = 4000000
    msg2 = np.zeros(n_points * 2, dtype=np.uint8)
    np.random.seed(42)
    msg2[1::2] = np.random.randint(0, 255, n_points, dtype=np.uint8)
    
    with open(filename, 'wb') as f:
        f.write(msg1.tobytes())
        f.write(msg2.tobytes())
        
    print(f"Created {filename} (400KB)")

def run_test():
    IN_FILE = "drift.bin"
    OUT_FILE = "drift.qres"
    
    generate_drift_signal(IN_FILE)
    
    # Compress with Watchdog (Threshold 50 for punishment logic check? Or just run it)
    # The watchdog logs to stderr. We want to capture that.
    
    cmd = [
        "qres_rust/target/release/qres-cli", 
        "compress", 
        IN_FILE, 
        OUT_FILE, 
        "--auto-tune",
        # "--detect-anomalies", "255" # Removed to silence spam
    ]
    
    print(f"Running: {' '.join(cmd)}")
    
    result = subprocess.run(cmd, capture_output=True, text=True)
    
    print("\n--- CLI Output ---")
    print(result.stdout)
    print("\n--- Watchdog Logs (Stderr) ---")
    print(result.stderr)
    
    if "Punishment!" in result.stderr:
        print("\n✅ SUCCESS: Online Learning triggered (Punishment detected).")
    else:
        print("\n❌ FAILURE: No punishment triggered during drift.")

    # Verify Decompression (Agile Reader)
    cmd_dec = ["qres_rust/target/release/qres-cli", "decompress", OUT_FILE, "drift_restored.bin"]
    subprocess.run(cmd_dec, check=True)
    
    with open(IN_FILE, 'rb') as f1, open("drift_restored.bin", 'rb') as f2:
        if f1.read() == f2.read():
            print("✅ SUCCESS: Decompression verified (Data match).")
        else:
            print("❌ FAILURE: Data mismatch!")

if __name__ == "__main__":
    # run_test() # Disabled to prevent brain pollution
    generate_drift_signal("drift.bin")
