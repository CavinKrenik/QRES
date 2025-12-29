import sys
import json
import colorama
from colorama import Fore, Style

colorama.init(autoreset=True)

def inspect_report(path):
    try:
        with open(path, 'r') as f:
            stats = json.load(f)
    except Exception as e:
        print(f"Error reading report: {e}")
        return

    # Header
    print(f"\n{Style.BRIGHT}⚔️  QRES BATTLE REPORT ⚔️{Style.RESET_ALL}\n")
    
    # Extract Scores
    try:
        linear = stats['linear_score']
        lstm = stats['lstm_score']
        tensor = stats['tensor_score']
        winner_id = stats['winner_id']
    except KeyError:
        print("Invalid Report Format")
        return

    # Determine Winner Name & Color
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

    print(f"{Fore.WHITE}Strategy Selected: {Style.BRIGHT}{w_color}{w_name}")
    
    # Scoreboard
    print(f"\n{Style.BRIGHT}Scoreboard (Lower is Better):{Style.RESET_ALL}")
    
    scores = [
        ("Linear", linear, Fore.CYAN),
        ("Tensor", tensor, Fore.YELLOW),
        ("LSTM  ", lstm, Fore.MAGENTA)
    ]
    scores.sort(key=lambda x: x[1])

    for rank, (name, score, color) in enumerate(scores):
        prefix = "👑" if rank == 0 else f"{rank+1}."
        print(f"  {prefix} {color}{name}: {score:.2f}")

    print("\n")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python qres_inspect.py <race_stats.json>")
    else:
        inspect_report(sys.argv[1])
