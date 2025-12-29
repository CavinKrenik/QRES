import os
import time
import subprocess
import matplotlib.pyplot as plt

# Configuration
QRES_BIN = r"qres_rust\target\release\qres-cli.exe"
DATA_DIR = "titan_data"
LOG_FILE = os.path.join(DATA_DIR, "apache_access.log")

def fetch_log_data():
    if not os.path.exists(DATA_DIR):
        os.makedirs(DATA_DIR)
    
    if not os.path.exists(LOG_FILE):
        print("Creating synthetic Apache Log file...")
        # Synthesize repetitive log data
        with open(LOG_FILE, "wb") as f:
            for i in range(10000):
                line = f'192.168.0.{i%255} - - [29/Dec/2025:12:00:{i%60} +0000] "GET /index.html HTTP/1.1" 200 {1000+i} "http://example.com" "Mozilla/5.0"\n'
                f.write(line.encode('utf-8'))
        print(f"Created {LOG_FILE} ({os.path.getsize(LOG_FILE)/1024:.2f} KB)")

def run_bench():
    fetch_log_data()
    
    abs_qres_bin = os.path.abspath(QRES_BIN)
    if not os.path.exists(abs_qres_bin):
        print(f"Error: Binary not found at {abs_qres_bin}")
        return

    # Tools: 
    # QRES (Semantic) vs Zstd (Level 9)
    # Zstd is standard for logs.
    
    results = []
    
    # 1. QRES Semantic
    start = time.time()
    subprocess.run([abs_qres_bin, "compress", LOG_FILE, "test.qres_sem", "--mode", "semantic"], check=True)
    opts_time = time.time() - start
    size_qres = os.path.getsize("test.qres_sem")
    results.append(("QRES Semantic", size_qres, opts_time))
    
    # 2. Zstd -9
    try:
        start = time.time()
        subprocess.run(["zstd", "-9", "-f", LOG_FILE, "-o", "test.zst"], check=True)
        zstd_time = time.time() - start
        size_zstd = os.path.getsize("test.zst")
        results.append(("Zstd -9", size_zstd, zstd_time))
    except FileNotFoundError:
        print("Zstd binary not found. Skipping comparison.")

    # 3. Brotli (if avail)
    try:
        start = time.time()
        subprocess.run(["brotli", "-q", "9", "-f", LOG_FILE, "-o", "test.br"], check=True)
        br_time = time.time() - start
        size_br = os.path.getsize("test.br")
        results.append(("Brotli -9", size_br, br_time))
    except FileNotFoundError:
        pass

    print("\n--- Semantic Benchmark Results ---")
    print(f"{'Tool':<15} | {'Size (KB)':<10} | {'Ratio':<6} | {'Time (s)':<8}")
    orig_size = os.path.getsize(LOG_FILE)
    
    for name, size, duration in results:
        ratio = orig_size / size
        print(f"{name:<15} | {size/1024:<10.2f} | {ratio:<6.2f} | {duration:<8.4f}")

if __name__ == "__main__":
    run_bench()
