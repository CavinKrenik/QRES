import os
import time
import subprocess
import requests
import sys
import matplotlib.pyplot as plt

# Configuration
QRES_BIN = r"qres_rust\target\release\qres-cli.exe"
DATA_DIR = "titan_data"
RESULTS_DIR = "results"

# Datasets (Silesia Corpus snippet)
DATASETS = {
    "dickens": "https://sun.aei.polsl.pl//~sdeor/corpus/dickens",
    "mozilla": "https://sun.aei.polsl.pl//~sdeor/corpus/mozilla",
    "webster": "https://sun.aei.polsl.pl//~sdeor/corpus/webster",
    "samba": "https://sun.aei.polsl.pl//~sdeor/corpus/samba"
}

def download_file(url, dest):
    if os.path.exists(dest):
        print(f"[Skip] {dest} exists")
        return
    print(f"[Download] {url} -> {dest}")
    try:
        r = requests.get(url, stream=True)
        with open(dest, 'wb') as f:
            for chunk in r.iter_content(chunk_size=8192):
                f.write(chunk)
    except Exception as e:
        print(f"[Error] Failed to download {url}: {e}")

def run_bench(cmd, input_file):
    start = time.time()
    try:
        # Run command
        subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        duration = time.time() - start
        
        # Get Size
        # output file is usually input_file + extension
        # We need to know the output file name
        return duration
    except Exception as e:
        print(f"Bench failed: {cmd} - {e}")
        return 0.0

def get_size(path):
    if os.path.exists(path):
        return os.path.getsize(path)
    return 0

def benchmark():
    if not os.path.exists(DATA_DIR):
        os.makedirs(DATA_DIR)
    if not os.path.exists(RESULTS_DIR):
        os.makedirs(RESULTS_DIR)

    # 1. Prepare Data
    for name, url in DATASETS.items():
        download_file(url, os.path.join(DATA_DIR, name))

    results = []

    # 2. Run Benchmarks
    abs_qres_bin = os.path.abspath(QRES_BIN)
    if not os.path.exists(abs_qres_bin):
        print(f"Error: QRES binary not found at {abs_qres_bin}")
        return

    tools = [
        {"name": "QRES (Hybrid)", "cmd": [abs_qres_bin, "compress", "INPUT", "OUTPUT"], "ext": ".qres"},
        # We need external tools for comparison. 
        # If zstd is not in path, this will fail. We'll wrap in try.
        {"name": "Zstd (Default)", "cmd": ["zstd", "-f", "INPUT", "-o", "OUTPUT"], "ext": ".zst"},
        {"name": "Brotli (Default)", "cmd": ["brotli", "-f", "INPUT", "-o", "OUTPUT"], "ext": ".br"},
    ]

    print(f"{'Dataset':<10} | {'Tool':<15} | {'Size (KB)':<10} | {'Ratio':<10} | {'Time (s)':<10} | {'Speed (MB/s)':<10}")
    print("-" * 80)

    for name in DATASETS.keys():
        input_path = os.path.join(DATA_DIR, name)
        orig_size = get_size(input_path)
        if orig_size == 0: continue

        for tool in tools:
            output_path = os.path.join(RESULTS_DIR, name + tool["ext"])
            cmd = [arg.replace("INPUT", input_path).replace("OUTPUT", output_path) for arg in tool["cmd"]]
            
            try:
                duration = run_bench(cmd, input_path)
                compressed_size = get_size(output_path)
                
                if compressed_size > 0:
                    ratio = orig_size / compressed_size
                    speed = (orig_size / 1024 / 1024) / duration if duration > 0 else 0
                    print(f"{name:<10} | {tool['name']:<15} | {compressed_size/1024:<10.1f} | {ratio:<10.2f} | {duration:<10.3f} | {speed:<10.2f}")
                    
                    results.append({
                        "dataset": name,
                        "tool": tool["name"],
                        "ratio": ratio,
                        "speed": speed
                    })
            except Exception as e:
                pass # Tool likely missing

    # 3. Plot
    plot_results(results)

def plot_results(results):
    if not results:
        print("No results to plot.")
        return
        
    datasets = sorted(list(set(r["dataset"] for r in results)))
    tool_names = sorted(list(set(r["tool"] for r in results)))
    
    # Pareto Plot: Speed vs Ratio
    plt.figure(figsize=(10, 6))
    
    colors = {"QRES (Hybrid)": "red", "Zstd (Default)": "blue", "Brotli (Default)": "green"}
    
    for tool in tool_names:
        subset = [r for r in results if r["tool"] == tool]
        avg_ratio = sum(r["ratio"] for r in subset) / len(subset)
        avg_speed = sum(r["speed"] for r in subset) / len(subset)
        
        plt.scatter(avg_speed, avg_ratio, label=tool, color=colors.get(tool, "gray"), s=100)
        plt.text(avg_speed, avg_ratio, f" {tool}", fontsize=9)

    plt.title("Compression Efficiency: Speed vs Ratio (Titan Bench)")
    plt.xlabel("Speed (MB/s)")
    plt.ylabel("Compression Ratio")
    plt.grid(True, linestyle="--", alpha=0.6)
    plt.legend()
    plt.savefig("benchmarks/titan_bench_pareto.png")
    print("\nChart saved to benchmarks/titan_bench_pareto.png")

if __name__ == "__main__":
    benchmark()
