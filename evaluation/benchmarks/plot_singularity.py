import matplotlib.pyplot as plt
import pandas as pd
import os

def plot_singularity():
    csv_path = os.path.expanduser('~/.qres/singularity_metrics.csv')
    if not os.path.exists(csv_path):
        print(f"❌ CSV file not found: {csv_path}")
        return

    try:
        df = pd.read_csv(csv_path)
    except Exception as e:
        print(f"❌ Failed to read CSV: {e}")
        return

    # Create professional style
    plt.style.use('seaborn-v0_8-darkgrid' if 'seaborn-v0_8-darkgrid' in plt.style.available else 'default')

    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(12, 8), sharex=True)

    # Convert timestamp to relative time (seconds from start)
    df['time'] = (df['timestamp'] - df['timestamp'].iloc[0])

    # Plot 1: Loss over time
    ax1.plot(df['time'], df['local_loss'], 'r-', linewidth=2, label='Local Loss')
    ax1.axhline(y=0.01, color='red', linestyle='--', alpha=0.7, label='Singularity Threshold (0.01)')
    ax1.fill_between(df['time'], 0, 0.01, where=(df['local_loss'] <= 0.01), color='green', alpha=0.3, label='Singularity Achieved')
    ax1.set_ylabel('Local Loss', fontsize=12)
    ax1.set_title('Swarm Singularity: Federated Learning Convergence', fontsize=14, pad=20)
    ax1.legend()
    ax1.grid(True, alpha=0.3)

    # Plot 2: Consensus variance and active peers
    ax2.plot(df['time'], df['swarm_consensus_variance'], 'b-', linewidth=2, label='Consensus Variance')
    ax2.set_ylabel('Consensus Variance', fontsize=12, color='blue')
    ax2.tick_params(axis='y', labelcolor='blue')

    ax3 = ax2.twinx()
    ax3.plot(df['time'], df['active_peers'], 'g-', linewidth=2, label='Active Peers')
    ax3.set_ylabel('Active Peers', fontsize=12, color='green')
    ax3.tick_params(axis='y', labelcolor='green')

    ax2.set_xlabel('Time (seconds)', fontsize=12)
    ax2.grid(True, alpha=0.3)

    # Add annotations
    singularity_time = df[df['local_loss'] <= 0.01]['time'].iloc[0] if any(df['local_loss'] <= 0.01) else None
    if singularity_time is not None:
        ax1.annotate('Singularity Achieved', xy=(singularity_time, 0.01),
                    xytext=(singularity_time + 50, 0.05),
                    arrowprops=dict(facecolor='black', shrink=0.05),
                    fontsize=10, fontweight='bold',
                    bbox=dict(boxstyle="round,pad=0.3", fc="yellow", ec="black", alpha=0.8))

    plt.tight_layout()

    # Ensure docs/images directory exists
    os.makedirs('docs/images', exist_ok=True)
    plt.savefig('docs/images/singularity_zero_shot.png', dpi=300, bbox_inches='tight')
    print("✅ Generated docs/images/singularity_zero_shot.png")

if __name__ == "__main__":
    plot_singularity()
