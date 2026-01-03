import argparse
import sys
import os

# Path hack
sys.path.append(os.path.join(os.getcwd(), 'python'))
from qres.api import QRES_API

def main():
    parser = argparse.ArgumentParser(description="QRES v7.5 CLI - Quantum Mode")
    parser.add_argument("input", help="Input file path", nargs='?')
    parser.add_argument("--mode", choices=["standard", "quantum"], default="standard", help="Compression mode")
    parser.add_argument("--optimize", action="store_true", help="Run Neural/Ethical optimization first")
    parser.add_argument("--broadcast", action="store_true", help="Broadcast output to QRES Swarm (via quantum_outbox)")
    parser.add_argument("--save-state", metavar="VERSION", help="Save current world state with version name")
    parser.add_argument("--load-state", metavar="VERSION", help="Load world state (use 'latest' for most recent)")
    parser.add_argument("--broadcast-state", metavar="VERSION", help="Broadcast world state to swarm (None = current)")
    
    args = parser.parse_args()
    
    if not args.input and not args.optimize and not args.save_state and not args.load_state:
        parser.print_help()
        return

    api = QRES_API(mode=args.mode)
    
    # Handle state loading first
    if args.load_state:
        version = None if args.load_state == "latest" else args.load_state
        if api.load_world_state(version):
            print("✅ World state loaded successfully")
        else:
            print("❌ Failed to load world state")
            return
    
    if args.optimize:
        api.load_brain()
        api.optimize_system()
    
    # Handle state saving
    if args.save_state:
        version = api.save_world_state(args.save_state)
        if version:
            print(f"✅ World state saved as {version}")
        return
    
    # Handle state broadcasting
    if args.broadcast_state:
        version = args.broadcast_state if args.broadcast_state != "current" else None
        if api.broadcast_world_state(version):
            print(f"✅ World state broadcast queued")
        return
        
    if args.input:
        if not os.path.exists(args.input):
            print(f"Error: {args.input} not found.")
            return
            
        with open(args.input, "rb") as f:
            data = f.read()
            
        print(f"Compressing {len(data)} bytes in {args.mode} mode...")
        compressed = api.compress(data)
        
        if args.broadcast:
            # Write to quantum_outbox/
            if not os.path.exists("quantum_outbox"):
                os.makedirs("quantum_outbox")
            
            # Filename needs to be unique enough to avoid collisions before processing
            import time
            out_name = f"quantum_outbox/qv7_{int(time.time()*1000)}.qres"
            with open(out_name, "wb") as f:
                f.write(compressed)
            print(f"Packaged for Broadcast -> {out_name}")
        else:
            out_name = args.input + ".qres"
            with open(out_name, "wb") as f:
                f.write(compressed)
            print(f"Saved to {out_name} ({len(compressed)} bytes)")

if __name__ == "__main__":
    main()
