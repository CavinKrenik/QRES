"""
Visual Analysis Tool for Error-Bounded Compression.

This script loads time-series data, simulates the QRES v16 lossy compression logic,
and plots the original signal vs. the reconstructed signal to visualize the
"accuracy vs. bandwidth" trade-off.

Usage:
    python tools/visual_analysis.py --file benchmarks/datasets/jena_climate_2009_2016.csv
"""

import argparse
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt


def simulate_compression(data: np.ndarray, error_bound: float):
    """
    Simulates the Rust ErrorBoundedCompressor logic.

    Args:
        data: The input time-series array.
        error_bound: Maximum allowable deviation.

    Returns:
        reconstructed: The decompressed signal.
        mask_skipped: Boolean array (True where values were skipped).
        compression_ratio: Estimated ratio (bytes out / bytes in).
    """
    reconstructed = []
    mask_skipped = []
    
    # Byte tracking (Est: 1 byte tag + 4 bytes payload per event)
    bytes_in = len(data) * 4
    bytes_out = 0

    if len(data) == 0:
        return [], [], 0

    # First value is always sent
    last_val = data[0]
    reconstructed.append(last_val)
    mask_skipped.append(False)
    bytes_out += 5  # Tag + f32

    skip_count = 0

    for val in data[1:]:
        prediction = last_val
        diff = abs(val - prediction)

        if diff <= error_bound:
            skip_count += 1
            # Decoder just holds the last value
            reconstructed.append(last_val) 
            mask_skipped.append(True)
        else:
            if skip_count > 0:
                bytes_out += 5 # Tag + u32 (Run Length)
                skip_count = 0
            
            # Update state
            last_val = val
            reconstructed.append(val)
            mask_skipped.append(False)
            bytes_out += 5 # Tag + f32

    # Flush trails
    if skip_count > 0:
        bytes_out += 5

    ratio = bytes_in / bytes_out if bytes_out > 0 else 0
    return np.array(reconstructed), np.array(mask_skipped), ratio


def main():
    parser = argparse.ArgumentParser(description="Visualize QRES Compression")
    parser.add_argument("--file", type=str, required=True, help="Path to CSV")
    parser.add_argument("--bound", type=float, default=0.5, help="Error bound")
    parser.add_argument("--col", type=str, default="T (degC)", help="Column")
    args = parser.parse_args()

    # Load Data (Limit to 500 points for clear visualization)
    print(f"Loading {args.file}...")
    try:
        df = pd.read_csv(args.file)
        if args.col not in df.columns:
            print(f"Column '{args.col}' not found. Available columns: {list(df.columns)}")
            return
        data = df[args.col].values[:500]
    except Exception as e:
        print(f"Error loading file: {e}")
        return

    # Run Simulation
    rec, mask, ratio = simulate_compression(data, args.bound)

    print(f"Original Points: {len(data)}")
    print(f"Skipped Points:  {np.sum(mask)}")
    print(f"Comp. Ratio:     {ratio:.2f}x")

    # Plot
    plt.style.use('dark_background')
    plt.figure(figsize=(12, 6))
    
    # Plot Original (faint line)
    plt.plot(data, color='#333333', label='Original', linewidth=1)
    
    # Plot Reconstructed (Cyan line)
    plt.plot(rec, color='#00ff9d', label='Reconstructed', linewidth=1.5, alpha=0.8)

    # Plot Skipped Points (Red dots, small)
    skipped_x = np.where(mask)[0]
    skipped_y = rec[mask]
    plt.scatter(skipped_x, skipped_y, c='#ff00ff', s=10, 
                label='Skipped (Predicted)', marker='.', alpha=0.5)

    plt.title(f"QRES Adaptive Lossy (Bound={args.bound}) | Ratio: {ratio:.2f}x")
    plt.legend()
    plt.grid(color='#222')
    plt.tight_layout()
    plt.show()


if __name__ == "__main__":
    main()
