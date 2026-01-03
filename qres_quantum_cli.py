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
    
    args = parser.parse_args()
    
    if not args.input and not args.optimize:
        parser.print_help()
        return

    api = QRES_API(mode=args.mode)
    
    if args.optimize:
        api.load_brain()
        api.optimize_system()
        
    if args.input:
        if not os.path.exists(args.input):
            print(f"Error: {args.input} not found.")
            return
            
        with open(args.input, "rb") as f:
            data = f.read()
            
        print(f"Compressing {len(data)} bytes in {args.mode} mode...")
        compressed = api.compress(data)
        
        out_name = args.input + ".qres"
        with open(out_name, "wb") as f:
            f.write(compressed)
            
        print(f"Saved to {out_name} ({len(compressed)} bytes)")

if __name__ == "__main__":
    main()
