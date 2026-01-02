import os
import time
import subprocess
import requests
import sys
import json
import csv
import random
import math
import struct
import matplotlib.pyplot as plt

# Configuration
QRES_BIN = r"qres_rust\target\release\qres-cli.exe"
DATA_DIR = "benchmarks/datasets"
RESULTS_DIR = "benchmarks/results"

def ensure_dirs():
    if not os.path.exists(DATA_DIR):
        os.makedirs(DATA_DIR)
    if not os.path.exists(RESULTS_DIR):
        os.makedirs(RESULTS_DIR)

# --- Synthetic Data Generators ---

def gen_sine_wave(filename, size_mb=1):
    print(f"[Gen] Sine Wave -> {filename}")
    size = size_mb * 1024 * 1024
    t = [math.sin(2 * math.pi * i / 256.0) for i in range(size)]
    data = bytearray([int((x * 127) + 128) for x in t])
    with open(filename, 'wb') as f:
        f.write(data)

def gen_json_logs(filename, size_mb=1):
    print(f"[Gen] JSON Logs -> {filename}")
    target_size = size_mb * 1024 * 1024
    with open(filename, 'w') as f:
        while f.tell() < target_size:
            log = {
                "timestamp": time.time(),
                "level": random.choice(["INFO", "DEBUG", "ERROR", "WARN"]),
                "service": random.choice(["auth", "db", "api", "frontend"]),
                "msg": "Processed request successfully",
                "latency": random.random() * 0.5,
                "user_id": random.randint(1000, 9999)
            }
            f.write(json.dumps(log) + "\n")

def gen_csv_data(filename, size_mb=1):
    print(f"[Gen] CSV Data -> {filename}")
    target_size = size_mb * 1024 * 1024
    with open(filename, 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow(["id", "value1", "value2", "category", "flag"])
        current_size = 0
        while current_size < target_size:
            row = [
                random.randint(0, 1000000),
                random.random(),
                random.random() * 100,
                random.choice(["A", "B", "C", "D"]),
                random.choice([True, False])
            ]
            writer.writerow(row)
            # Rough estimation to avoid constant ftell
            current_size += 50 

def gen_random(filename, size_mb=1):
    print(f"[Gen] Random Data -> {filename}")
    size = size_mb * 1024 * 1024
    with open(filename, 'wb') as f:
        f.write(os.urandom(size))

def gen_text(filename, size_mb=1):
    print(f"[Gen] English Text -> {filename}")
    target_size = size_mb * 1024 * 1024
    vocab = ["the", "quick", "brown", "fox", "jumps", "over", "lazy", "dog", "compression", "is", "awesome", "data", "mining", "neural", "network"]
    with open(filename, 'w') as f:
        while f.tell() < target_size:
            sentence = " ".join([random.choice(vocab) for _ in range(10)]) + ". "
            f.write(sentence)

def gen_xml_data(filename, size_mb=1):
    print(f"[Gen] XML Data -> {filename}")
    with open(filename, 'w') as f:
        f.write("<?xml version=\"1.0\"?>\n<root>\n")
        while f.tell() < size_mb * 1024 * 1024:
            f.write(f"  <record id=\"{random.randint(0,99999)}\">\n")
            f.write(f"    <data>{random.random()}</data>\n")
            f.write(f"    <description>Sample XML entry for benchmark</description>\n")
            f.write("  </record>\n")
        f.write("</root>")

def gen_source_code(filename, size_mb=1):
    print(f"[Gen] C-like Code -> {filename}")
    keywords = ["if", "while", "for", "return", "int", "void", "float", "struct"]
    with open(filename, 'w') as f:
        while f.tell() < size_mb * 1024 * 1024:
            f.write(f"{random.choice(keywords)} function_{random.randint(0,100)}() {{\n")
            for _ in range(random.randint(2, 10)):
                f.write(f"    int x = {random.randint(0,100)};\n")
            f.write("}\n")

def gen_exe_data(filename, size_mb=1):
    print(f"[Gen] Mock Executable -> {filename}")
    header = b"\x4D\x5A\x90\x00" * 16 # PE Header pattern
    with open(filename, 'wb') as f:
        while f.tell() < size_mb * 1024 * 1024:
            f.write(header)
            f.write(os.urandom(4096))

# --- Benchmarking Core ---

def get_bin_path():
    # Try absolute
    if os.path.exists(QRES_BIN):
        return os.path.abspath(QRES_BIN)
    # Try relative to CWD
    rel = os.path.join(os.getcwd(), QRES_BIN)
    if os.path.exists(rel):
        return rel
    # Try finding in typical rust target
    for d in ["qres_rust/target/release/qres-cli.exe", "../qres_rust/target/release/qres-cli.exe"]:
        if os.path.exists(d):
            return os.path.abspath(d)
    return None

def run_cmd(cmd):
    start = time.time()
    try:
        subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        return time.time() - start
    except subprocess.CalledProcessError:
        return None
    except FileNotFoundError:
        return None

def benchmark():
    ensure_dirs()
    
    datasets = [
        ("sine_1mb.dat", gen_sine_wave, 1),
        ("logs_1mb.json", gen_json_logs, 1),
        ("data_1mb.csv", gen_csv_data, 1),
        ("random_1mb.bin", gen_random, 1),
        ("text_1mb.txt", gen_text, 1),
        ("data_1mb.xml", gen_xml_data, 1),
        ("source_1mb.c", gen_source_code, 1),
        ("app_1mb.exe", gen_exe_data, 1)
    ]
    
    # Generate if missing
    for name, gen_func, size in datasets:
        path = os.path.join(DATA_DIR, name)
        if not os.path.exists(path):
            gen_func(path, size)

    qres_exe = get_bin_path()
    if not qres_exe:
        print("CRITICAL: QRES binary not found. Build with `cargo build --release`.")
        return

    print(f"\nBenchmark Suite v2.0")
    print(f"Target: {qres_exe}")
    print(f"{'Dataset':<15} | {'Orig(KB)':<8} | {'Comp(KB)':<8} | {'Ratio':<6} | {'Speed(MB/s)':<12} | {'Zstd Ratio':<10}")
    print("-" * 85)

    report_lines = []
    report_lines.append("| Dataset | Size (KB) | QRES Ratio | QRES Speed (MB/s) | Zstd Ratio | Winner |")
    report_lines.append("|---|---|---|---|---|---|")

    stats = []

    for name, _, size_mb in datasets:
        input_path = os.path.join(DATA_DIR, name)
        orig_size = os.path.getsize(input_path)
        
        # 1. QRES
        qres_out = os.path.join(RESULTS_DIR, name + ".qres")
        cmd = [qres_exe, "compress", input_path, qres_out]
        
        duration = run_cmd(cmd)
        
        if duration:
            comp_size = os.path.getsize(qres_out)
            ratio = comp_size / orig_size
            speed = (orig_size / 1024 / 1024) / duration
        else:
            ratio = 1.0; speed = 0.0; comp_size = orig_size

        # 2. Zstd Comparison
        zstd_out = os.path.join(RESULTS_DIR, name + ".zst")
        # Try zstd command line
        z_start = time.time()
        z_duration = run_cmd(["zstd", "-f", "-1", input_path, "-o", zstd_out]) # -1 for speed comparison
        
        z_ratio = 1.0
        if z_duration:
            z_size = os.path.getsize(zstd_out)
            z_ratio = z_size / orig_size
        
        # Display
        winner = "QRES" if ratio < z_ratio else "Zstd"
        if abs(ratio - z_ratio) < 0.01: winner = "Tie"
        
        print(f"{name:<15} | {orig_size/1024:<8.0f} | {comp_size/1024:<8.1f} | {ratio:<6.1%} | {speed:<12.1f} | {z_ratio:<10.1%}")
        
        report_lines.append(f"| {name} | {orig_size/1024:.1f} | {ratio:.2%} | {speed:.1f} | {z_ratio:.2%} | **{winner}** |")

        stats.append({
            "dataset": name,
            "qres_ratio": ratio,
            "zstd_ratio": z_ratio,
            "speed": speed
        })

    # Save Report
    with open(os.path.join(RESULTS_DIR, "benchmark_report.md"), "w") as f:
        f.write("# QRES v5 Benchmark Report\n\n")
        f.write("\n".join(report_lines))
        f.write("\n\n*Generated by titan_bench.py*")
    
    print(f"\nReport saved to {RESULTS_DIR}/benchmark_report.md")
    
    # Plotting
    names = [s["dataset"] for s in stats]
    q_ratios = [s["qres_ratio"] for s in stats]
    z_ratios = [s["zstd_ratio"] for s in stats]
    
    x = range(len(names))
    width = 0.35
    
    plt.figure(figsize=(10, 6))
    plt.bar([i - width/2 for i in x], q_ratios, width, label='QRES')
    plt.bar([i + width/2 for i in x], z_ratios, width, label='Zstd')
    
    plt.ylabel('Compression Ratio (Lower is Better)')
    plt.title('QRES vs Zstd Compression Ratio')
    plt.xticks(x, names, rotation=45)
    plt.legend()
    plt.tight_layout()
    plt.savefig(os.path.join(RESULTS_DIR, 'ratio_comparison.png'))

if __name__ == "__main__":
    benchmark()
