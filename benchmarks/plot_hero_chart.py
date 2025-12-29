import matplotlib.pyplot as plt
import pandas as pd
import numpy as np
import os

# Phase 20: The "Hero Chart" Generator
# Leads: QRES Data Visualization Team
# Goal: Scientific proof of Zero-Shot Adapatation

def plot_hero():
    print("🎨 Generating Hero Chart...")
    
    # Check Data
    if not os.path.exists("agent_a.csv") or not os.path.exists("agent_b.csv"):
        print("❌ Data missing!")
        return

    df_a = pd.read_csv("agent_a.csv")
    df_b = pd.read_csv("agent_b.csv")

    # Data Parsing
    # We need to map Active Confidence
    # Agent A
    conf_active_a = []
    for i, row in df_a.iterrows():
        eid = int(row['EngineID'])
        # Map EngineID to Column Index? 
        # CSV has ConfLinear (1) and ConfIPEPS (5).
        # We need logic: if ID=1 use ConfLinear. If ID=5 use ConfIPEPS.
        # But wait, CSV only has ConfLinear, ConfIPEPS columns.
        # What about LSTM (3)?
        # The CSV logging in lib.rs prints: self.living_brain.confidence[1] and [5].
        # It does NOT print [3].
        # However, we can infer confidence=1.0 for LSTM if it was picked fresh.
        # Let's just use a synthesized "System Confidence" metric.
        # If Ratio > 1.0, Confidence is Low.
        # If Ratio < 0.75, Confidence is High.
        # Actually, let's stick to the Ratio plot as the HERO.
        # And plot "Quantum Activation" (0 or 1) on Right Axis.
        pass

    # Plot 1: Compression Ratio (The Struggle) - Left Axis
    # Invert Y axis? No, Lower is Better.
    l1 = ax.plot(chunks, df_a['Ratio'], color='#D55E00', linestyle=':', linewidth=2, label='Agent A: Learning (Search Phase)')
    l2 = ax.plot(chunks, df_b['Ratio'], color='#009E73', linestyle='-', linewidth=3, label='Agent B: Zero-Shot (Instant Quantum)')
    
    ax.set_ylim(0.0, 1.4)
    ax.set_ylabel('Compression Ratio (Lower is Better)', fontsize=12, fontweight='bold')
    ax.set_xlabel('Time (Chunks)', fontsize=12, fontweight='bold')
    
    # Shade the "Pain Zone"
    ax.fill_between(chunks, 0.75, 1.4, color='red', alpha=0.05, label='Inefficient Zone')

    # Annotations for Agent A
    # Find transition points
    # Linear (ID 1) -> LSTM (ID 3) happens at ~36
    # LSTM (ID 3) -> iPEPS (ID 5) happens at ~38
    
    ax.annotate('Adopting iPEPS', xy=(38, 0.64), xytext=(50, 0.9),
                arrowprops=dict(facecolor='black', arrowstyle='->'), fontsize=10)
                
    ax.annotate('Punishment Loop', xy=(34, 1.19), xytext=(10, 1.3),
                arrowprops=dict(facecolor='red', arrowstyle='->'), color='#D55E00', fontsize=10)

    # Singularity Moment
    ax.annotate('The Singularity:\nInstant Optimal State', xy=(32, 0.64), xytext=(60, 0.3),
                arrowprops=dict(facecolor='#009E73', shrink=0.05), fontsize=12, fontweight='bold', color='#009E73',
                bbox=dict(boxstyle="round,pad=0.3", fc="white", ec="#009E73", alpha=0.9))

    # Right Axis: Engine State
    ax2 = ax.twinx()
    # Plot Engine ID Activity
    # 1=Linear, 3=LSTM, 5=Quantum
    # We want to show "Quantumness".
    # Map IDs: 1->0, 3->0.5, 5->1.0
    quantumness_a = df_a['EngineID'].map({1:0.0, 3:0.3, 5:1.0})
    quantumness_b = df_b['EngineID'].map({1:0.0, 3:0.3, 5:1.0})
    
    l3 = ax2.step(chunks, quantumness_a, color='gray', linestyle='--', alpha=0.3, where='post', label='Agent A: Model Complexity')
    # l4 = ax2.step(chunks, quantumness_b, color='green', alpha=0.1, where='post')
    
    ax2.set_yticks([0.0, 0.3, 1.0])
    ax2.set_yticklabels(['Linear', 'LSTM', 'Quantum (iPEPS)'])
    ax2.set_ylabel('Active Model Architecture', fontsize=12, color='gray')
    ax2.spines['right'].set_color('gray')
    ax2.tick_params(axis='y', colors='gray')
    ax2.set_ylim(-0.1, 1.2)

    # Legend
    lines = l1 + l2 + [l3[0]]
    labels = [l.get_label() for l in lines]
    ax.legend(lines, labels, loc='lower center', bbox_to_anchor=(0.5, 1.02), ncol=3, frameon=False)
    
    plt.tight_layout()
    plt.savefig('benchmarks/results/singularity_zero_shot.png', dpi=300)
    print("Generated Hero Chart.")

if __name__ == "__main__":
    plot_hero()
