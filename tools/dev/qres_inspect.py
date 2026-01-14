import sys
import json
import struct
import colorama
from colorama import Fore, Style

colorama.init(autoreset=True)

def inspect_report(path):
    # Try JSON first
    try:
        with open(path, 'r') as f:
            stats = json.load(f)
        # It's a JSON Race Report
        inspect_json(stats)
        return
    except (UnicodeDecodeError, json.JSONDecodeError):
        pass # Not JSON
        
    # Try Binary QRES
    try:
        with open(path, 'rb') as f:
            magic = f.read(4)
            if magic != b'QRES':
                print("Not a QRES file or JSON report.")
                return
            
            len_bytes = f.read(4)
            header_len = struct.unpack('<I', len_bytes)[0]
            header_bytes = f.read(header_len)
            
            # Simple manual parse of bincode structure? 
            # Bincode struct: version(u8), flags(u8), predictor_id(u8), ...
            # We can just read the bytes directly if we assume standard packing.
            # Rust bincode default:
            # version: 1 byte
            # flags: 1 byte
            # predictor_id: 1 byte
            
            predictor_id = header_bytes[2]
            print_winner(predictor_id, "Psychic Selection (Binary Header)")
            
    except Exception as e:
        print(f"Error reading binary: {e}")

def inspect_json(stats):
    print(f"\n{Style.BRIGHT}⚔️  QRES BATTLE REPORT ⚔️{Style.RESET_ALL}\n")
    try:
        winner_id = stats['winner_id']
        print_winner(winner_id, "Race Winner")
    except KeyError:
        print("Invalid Report Format")

def print_winner(winner_id, label):
    w_name = "Unknown"
    w_color = Fore.WHITE
    if winner_id == 1:
        w_name = "LINEAR (Native)"
        w_color = Fore.CYAN
    elif winner_id == 3:
        w_name = "LSTM (Neural)"
        w_color = Fore.MAGENTA
    elif winner_id == 4:
        w_name = "TENSOR (Quantum)"
        w_color = Fore.YELLOW
    
    print(f"{Fore.WHITE}{label}: {Style.BRIGHT}{w_color}{w_name}")
    print("\n")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python qres_inspect.py <file.qres | race_stats.json>")
    else:
        inspect_report(sys.argv[1])
