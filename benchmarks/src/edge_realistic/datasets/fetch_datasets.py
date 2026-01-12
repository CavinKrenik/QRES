"""Script to fetch Edge-Realistic datasets (Jena Climate and ETTh1).

This script downloads and extracts standard datasets used for benchmarking
time-series processing on edge devices.
"""

import os
import requests
import zipfile
import io
from pathlib import Path

# Constants
DATASETS_DIR = Path(__file__).parent
JENA_URL = "https://storage.googleapis.com/tensorflow/tf-keras-datasets/jena_climate_2009_2016.csv.zip"
ETT_URL = "https://raw.githubusercontent.com/zhouhaoyi/ETDataset/main/ETT-small/ETTh1.csv"

def download_file(url, target_path):
    """Downloads a file from a URL to a target path with a progress bar.

    Args:
        url: The URL to download from.
        target_path: The local path to save the file.
    """
    print(f"Downloading {url} to {target_path}...")
    try:
        response = requests.get(url, stream=True)
        response.raise_for_status()
        
        total_size = int(response.headers.get("content-length", 0))
        block_size = 8192
        downloaded = 0

        with open(target_path, "wb") as f:
            for chunk in response.iter_content(chunk_size=block_size):
                if chunk:
                    f.write(chunk)
                    downloaded += len(chunk)
                    if total_size > 0:
                        percent = int(50 * downloaded / total_size)
                        print(f"\r[{'=' * percent}{' ' * (50 - percent)}] {downloaded}/{total_size} bytes", end="")
        print()  # Newline after progress bar
        print("Download complete.")

    except requests.exceptions.RequestException as e:
        print(f"Error downloading {url}: {e}")
        if os.path.exists(target_path):
            os.remove(target_path)
        raise

def fetch_jena_climate():
    """Downloads and extracts the Jena Climate dataset."""
    csv_path = DATASETS_DIR / "jena_climate_2009_2016.csv"
    if csv_path.exists():
        print(f"Jena Climate dataset already exists at {csv_path}")
        return

    zip_path = DATASETS_DIR / "jena_climate.zip"
    try:
        download_file(JENA_URL, zip_path)
        
        print(f"Extracting {zip_path}...")
        with zipfile.ZipFile(zip_path, "r") as zip_ref:
            zip_ref.extractall(DATASETS_DIR)
        
        # Cleanup zip
        os.remove(zip_path)
        print("Jena Climate dataset ready.")

    except Exception as e:
        print(f"Failed to fetch Jena Climate dataset: {e}")

def fetch_ett_electricity():
    """Downloads the ETT (Electricity) dataset."""
    target_path = DATASETS_DIR / "ETTh1.csv"
    if target_path.exists():
        print(f"ETT dataset already exists at {target_path}")
        return

    try:
        download_file(ETT_URL, target_path)
        print("ETT dataset ready.")
    except Exception as e:
        print(f"Failed to fetch ETT dataset: {e}")

def main():
    """Main execution function."""
    print(f"Fetching datasets to {DATASETS_DIR}...")
    
    # Ensure directory exists (redundant if running from within, but safe)
    DATASETS_DIR.mkdir(parents=True, exist_ok=True)
    
    fetch_jena_climate()
    fetch_ett_electricity()
    
    print("All datasets processed.")

if __name__ == "__main__":
    main()
