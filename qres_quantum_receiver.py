
import os
import time
import sys
import argparse

# Path hack
sys.path.append(os.path.join(os.getcwd(), 'python'))
from qres.api import QRES_API

def receiver_loop(inbox_path="quantum_inbox", interval=2.0):
    print(f"🌌 [Quantum Receiver] Watching {inbox_path} for teleported tensors...")
    
    if not os.path.exists(inbox_path):
        os.makedirs(inbox_path)
        
    api = QRES_API(mode="quantum")
    
    try:
        while True:
            # Simple polling (In prod use watchdog)
            for filename in os.listdir(inbox_path):
                file_path = os.path.join(inbox_path, filename)
                
                # Check extension or mock
                if os.path.isfile(file_path):
                    if filename.endswith(".qt") or filename.endswith(".qres") or "qv7" in filename:
                        print(f"\n📩 Detected Incoming Tensor: {filename}")
                        
                        try:
                            # Read
                            with open(file_path, "rb") as f:
                                data = f.read()
                            
                            # Process
                            if api.merge_quantum_state(data):
                                print(f"✅ Successfully integrated {filename} into Hive Mind.")
                            else:
                                print(f"⚠️  Rejected malformed tensor: {filename}")
                                
                            # Delete (Consume)
                            os.remove(file_path)
                            
                        except Exception as e:
                            print(f"❌ Error processing {filename}: {e}")
            
            time.sleep(interval)
            
    except KeyboardInterrupt:
        print("\n🛑 Receiver stopped.")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="QRES v8.0 Quantum Receiver")
    parser.add_argument("--dir", default="quantum_inbox", help="Inbox directory to watch")
    args = parser.parse_args()
    
    receiver_loop(args.dir)
