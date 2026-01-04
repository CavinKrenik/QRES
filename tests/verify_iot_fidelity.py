import os
import hashlib
import subprocess
import shutil

def verify_fidelity():
    test_file = "iot_telemetry.dat"
    compressed_file = test_file + ".qres"
    decompressed_file = test_file + ".restored"
    
    # Ensure test file exists
    if not os.path.exists(test_file):
        print("Generating test file...")
        # (Assuming benchmark script ran recently, but let's just make a small one)
        with open(test_file, "wb") as f:
            f.write(os.urandom(1024*1024)) # Logic relies on interleaved detection, random won't trigger it 
            # We need interleaved data to trigger 0x03 path
            # But the benchmark generated the big one. Let's use it if exists.
    
    if not os.path.exists(test_file):
        print("Error: iot_telemetry.dat missing")
        return

    base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    cli = os.path.join(base_dir, "qres_rust", "target", "release", "qres-cli.exe")
    
    print("Compressing...")
    subprocess.run([cli, "compress", test_file, compressed_file], check=True)
    
    print("Decompressing...")
    subprocess.run([cli, "decompress", compressed_file, decompressed_file], check=True)
    
    # Verify
    print("Verifying hash...")
    
    def get_hash(fname):
        sha = hashlib.sha256()
        with open(fname, "rb") as f:
            while True:
                data = f.read(65536)
                if not data: break
                sha.update(data)
        return sha.hexdigest()
        
    h1 = get_hash(test_file)
    h2 = get_hash(decompressed_file)
    
    if h1 == h2:
        print(f"✅ SUCCESS: Hashes match ({h1[:8]}). Fidelity verified.")
    else:
        print(f"❌ FAILURE: Hashes mismatch!\nOriginal: {h1}\nRestored: {h2}")
        exit(1)

if __name__ == "__main__":
    verify_fidelity()
