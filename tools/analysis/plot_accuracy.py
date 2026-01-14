
import pandas as pd
import matplotlib.pyplot as plt
import os

def main():
    csv_path = "benchmarks/accuracy_results.csv"
    # Support running from root or benchmarks dir
    if not os.path.exists(csv_path):
        csv_path = "accuracy_results.csv"
    
    if not os.path.exists(csv_path):
        print(f"Error: CSV file not found at {csv_path}")
        return

    print(f"Loading data from {csv_path}...")
    df = pd.read_csv(csv_path)

    # Calculate MSE
    mse_heuristic = ((df['actual_value'] - df['heuristic_pred']) ** 2).mean()
    mse_neural = ((df['actual_value'] - df['neural_pred']) ** 2).mean()

    print("=" * 40)
    print("Predictions Accuracy Showdown")
    print("=" * 40)
    print(f"Heuristic MSE: {mse_heuristic:.6f}")
    print(f"Neural MSE:    {mse_neural:.6f}")
    print(f"Improvement:   {(mse_heuristic - mse_neural) / mse_heuristic * 100:.2f}%")
    print("=" * 40)

    # Plot
    plt.figure(figsize=(12, 6))
    # Plot a subset for clarity if too long
    subset = df.iloc[0:200] 
    
    plt.plot(subset['step'], subset['actual_value'], label='Actual', color='black', alpha=0.5, linestyle='--')
    plt.plot(subset['step'], subset['heuristic_pred'], label=f'Heuristic (MSE={mse_heuristic:.4f})', color='red', alpha=0.7)
    plt.plot(subset['step'], subset['neural_pred'], label=f'Neural (MSE={mse_neural:.4f})', color='green', alpha=0.9)
    
    plt.title('Prediction Accuracy: Neural vs Heuristic (First 200 steps)')
    plt.xlabel('Step')
    plt.ylabel('Value')
    plt.legend()
    plt.grid(True, alpha=0.3)
    
    output_path = "benchmarks/accuracy_plot.png"
    if not os.path.exists("benchmarks") and os.path.basename(os.getcwd()) == "benchmarks":
        output_path = "accuracy_plot.png"
        
    plt.savefig(output_path)
    print(f"Plot saved to {output_path}")

if __name__ == "__main__":
    main()
