
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
                    if filename.endswith(".qt") or filename.endswith(".qres") or filename.endswith(".qws") or "qv7" in filename or "world_" in filename:
                        print(f"\n📩 Detected Incoming: {filename}")
                        
                        try:
                            # Read
                            with open(file_path, "rb") as f:
                                data = f.read()
                            
                            # Check if it's a world state broadcast
                            if data.startswith(b"QRES_WORLD_STATE"):
                                print("🌍 Processing World State Broadcast...")
                                import pickle
                                state_data = pickle.loads(data[len(b"QRES_WORLD_STATE"):])
                                
                                # Merge with local state
                                local_version = api.world_state.get_latest_version()
                                
                                # Import the received state
                                remote_version = state_data['version']
                                api.world_state.states[remote_version] = state_data
                                api.world_state._save_db()
                                
                                if local_version:
                                    # Merge states
                                    print(f"  Merging {local_version} + {remote_version}...")
                                    merged_version = api.world_state.merge_world_states(
                                        local_version,
                                        remote_version,
                                        fidelity_threshold=0.98
                                    )
                                    print(f"✅ Merged world state: {merged_version}")
                                else:
                                    # First state, just adopt it
                                    api.load_world_state(remote_version)
                                    print(f"✅ Adopted world state: {remote_version}")
                            
                            # Process quantum tensor
                            elif api.merge_quantum_state(data):
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
