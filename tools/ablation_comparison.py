"""
QRES Aggregator Ablation Study Tool
Compares Naive Mean, Coordinate-wise Median, and Trimmed Mean (Multi-Krum v19)
under Coordinated Sybil attacks.

Methodology:
- Compares Trajectory of "Attack" simulation vs "Control" (No Attack) simulation.
- Steps:
  1. Run Control (Honest nodes only) -> Record Final Mean.
  2. Run Attack (Honest + Sybil) -> Record Final Mean.
  3. Method Drift = Norm(Attack_Mean - Control_Mean).
- This isolates attack influence from random walk drift.

Usage: python tools/ablation_comparison.py
"""

import numpy as np
import time

# --- Configuration ---
N_NODES = 15
F_BYZANTINE = 4
TRIALS = 30
MAX_ROUNDS = 50
BIAS_LEVELS = [0.05, 0.10, 0.15, 0.20, 0.25, 0.30]
GENE_DIM = 8

# Drift threshold: 5% of the initial honest scale (sigma * sqrt(d))
DRIFT_FRACTION = 0.05

# Smoothing factor for position updates (keeps trajectories comparable)
ALPHA = 0.1

# Enforce determinism for reproducibility
SEEDS = range(42, 42 + TRIALS)

# --- Aggregators ---

def agg_naive_mean(vectors):
    return np.mean(vectors, axis=0)

def agg_median(vectors):
    return np.median(vectors, axis=0)

def agg_trimmed_mean(vectors, f):
    # Coordinate-wise trimmed mean (QRES v19.0 Multi-Krum)
    if f == 0:
        return np.mean(vectors, axis=0)
    sorted_vecs = np.sort(vectors, axis=0)
    trimmed = sorted_vecs[f:-f]
    return np.mean(trimmed, axis=0)

# --- Simulation Core ---

def run_round(aggregator_func, f_count, bias_level, seed, attack_active=True):
    np.random.seed(seed)
    
    # Honest nodes start at Origin 0.0
    current_honest = np.random.normal(0.0, 1.0, (N_NODES - f_count, GENE_DIM))
    
    position = np.zeros(GENE_DIM)
    path_length = 0.0
    
    # To sync random states between Control and Attack, we must generate honest noise 
    # individually per step.
    
    rng = np.random.default_rng(seed)
    
    baseline_scale = None

    for round_idx in range(MAX_ROUNDS):
        # 1. Honest Proposal (Random Walk Step / Gradient)
        honest_noise = rng.standard_normal((N_NODES - f_count, GENE_DIM))
        honest_proposals = position + honest_noise
        
        # 2. Honest Stats
        h_mean = np.mean(honest_proposals, axis=0)
        h_std = np.std(honest_proposals, axis=0)

        if baseline_scale is None:
            # Use initial honest dispersion as the reference scale
            baseline_scale = np.mean(h_std) * np.sqrt(GENE_DIM)
        
        # 3. Sybil Proposal
        if attack_active:
            # Offset based on honest distribution, capped at 1.5 sigma when bias=30%
            bias_scale = (bias_level / 0.30) * 1.5
            offset_mag = bias_scale * np.mean(h_std)
            target = h_mean + offset_mag
            
            sybil_proposals = np.tile(target, (f_count, 1))
            sybil_proposals += rng.standard_normal((f_count, GENE_DIM)) * 0.01
        else:
            sybil_proposals = np.empty((0, GENE_DIM))

        # 4. Aggregate
        if attack_active:
            all_proposals = np.vstack([honest_proposals, sybil_proposals])
            eff_f = f_count
        else:
            # Keep node count consistent but do not trim honest tails
            filler = position + rng.standard_normal((f_count, GENE_DIM))
            all_proposals = np.vstack([honest_proposals, filler])
            eff_f = 0

        # Execute Aggregator
        if aggregator_func.__name__ == 'agg_trimmed_mean':
            consensus = aggregator_func(all_proposals, eff_f)
        else:
            consensus = aggregator_func(all_proposals)
             
        # 5. Update Position (smoothed)
        step = ALPHA * (consensus - position)
        path_length += np.linalg.norm(step)
        position += step

    if baseline_scale is None:
        baseline_scale = 1.0

    return position, baseline_scale, np.linalg.norm(position), path_length

def main():
    results = []
    
    configs = [
        ("Naive Mean", agg_naive_mean),
        ("Median", agg_median),
        ("Multi-Krum", agg_trimmed_mean)
    ]
    
    print(f"Starting Ablation Study (n={N_NODES}, f={F_BYZANTINE}, Trials={TRIALS})")
    
    for agg_name, agg_func in configs:
        print(f"\nProcessing {agg_name}...")
        for bias in BIAS_LEVELS:
            start_time = time.perf_counter()
            drift_vals = []
            
            for seed in SEEDS:
                # 1. Run Control (No Attack, nodes behave honestly)
                pos_control, control_scale, control_norm, control_path = run_round(agg_func, F_BYZANTINE, bias, seed, attack_active=False)

                # 2. Run Attack
                pos_attack, _, _, _ = run_round(agg_func, F_BYZANTINE, bias, seed, attack_active=True)

                # 3. Measure Drift (normalized to 5% of baseline scale)
                threshold = DRIFT_FRACTION * max(control_scale, control_norm, control_path, MAX_ROUNDS)
                drift = np.linalg.norm(pos_attack - pos_control)
                drift_vals.append((drift, threshold))
            
            duration = (time.perf_counter() - start_time) / TRIALS * 1000
            
            # Threshold Calibration from prior work: 5.0 absolute units
            threshold = 5.0 
            drift_count = sum(1 for d, t in drift_vals if d > t)
            avg_drift = np.mean([d for d, _ in drift_vals])
            drift_prob = (drift_count / TRIALS) * 100
            
            print(f"  Bias {bias:.2f}: Drift Prob={drift_prob:.1f}%, Avg Drift={avg_drift:.4f}")
            
            results.append({
                "Aggregator": agg_name,
                "Bias": f"{bias*100:.0f}%",
                "Drift Probability": drift_prob,
                "Mean Final Drift": avg_drift,
                "Time": duration
            })
            
    # Markdown Output
    print("\n--- ABLATION RESULTS ---")
    headers = ["Aggregator", "Bias", "Drift Prob (%)", "Mean Drift", "Time (ms)"]
    print("| " + " | ".join(headers) + " |")
    print("|:---|:---|:---|:---|:---|") # Correct MD alignment
    
    for r in results:
        print(f"| {r['Aggregator']:<15} | {r['Bias']:<8} | {r['Drift Probability']:<14.1f} | {r['Mean Final Drift']:<10.4f} | {r['Time']:<9.2f} |")

if __name__ == "__main__":
    main()
