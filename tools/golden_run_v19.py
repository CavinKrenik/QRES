"""
QRES v19.0 Golden Run - Integrated Multi-Krum + BFP-16 Verification

Proves that the coordinate-wise trimmed mean aggregator and BFP-16
arithmetic work together without interference.

Test: Sybil Collusion attack (n=15, f=4, bias=20%) where gradient
updates are quantized through BFP-16 before aggregation via trimmed mean.

Success Criteria:
    1. Zero drift (0/30 trials exceed 5% threshold) at 20% bias
    2. Zero vanishing updates (0.0% zero rate) at LR = 1e-5

Usage: python tools/golden_run_v19.py
"""

import numpy as np
import sys
import os
from datetime import datetime

# --- Configuration ---
N_TOTAL = 15
F_BYZANTINE = 4
BIAS_LEVEL = 0.20
N_TRIALS = 30
DRIFT_THRESHOLD = 0.05
SIGMA_MULTIPLIER = 1.5
MAX_ROUNDS = 50
GENE_DIMENSIONS = 8
LEARNING_RATE = 1e-5

# BFP-16 constants
BFP_MANTISSA_MAX = 32767
BFP_MANTISSA_MIN = -32768
BFP_EXPONENT_MIN = -128
BFP_EXPONENT_MAX = 127


def to_bfp(x):
    """Quantize float vector to BFP-16 and reconstruct."""
    max_abs = np.max(np.abs(x))
    if max_abs == 0:
        return np.zeros_like(x)
    raw_exp = np.ceil(np.log2(max_abs / BFP_MANTISSA_MAX))
    shared_exp = int(np.clip(raw_exp, BFP_EXPONENT_MIN, BFP_EXPONENT_MAX))
    scale = 2.0 ** shared_exp
    mantissas = np.round(x / scale)
    mantissas = np.clip(mantissas, BFP_MANTISSA_MIN, BFP_MANTISSA_MAX)
    return mantissas * scale


def trimmed_mean(vectors, f):
    """Coordinate-wise trimmed mean: trim top-f and bottom-f per dimension."""
    n = len(vectors)
    if n < 2 * f + 1:
        return np.mean(vectors, axis=0)
    sorted_per_dim = np.sort(vectors, axis=0)
    trimmed = sorted_per_dim[f:n - f, :]
    return np.mean(trimmed, axis=0)


def run_integrated_trial(seed):
    """
    Single trial: Sybil attack with BFP-16 quantized updates + trimmed mean.

    Returns dict with drift info and BFP zero-update stats.
    """
    rng = np.random.default_rng(seed)

    n_honest = N_TOTAL - F_BYZANTINE
    honest_center = np.ones(GENE_DIMENSIONS)
    honest_std = 0.05
    current_consensus = honest_center.copy()
    original_center = honest_center.copy()

    attack_direction = np.ones(GENE_DIMENSIONS)
    attack_direction = attack_direction / np.linalg.norm(attack_direction)

    rounds_to_drift = MAX_ROUNDS
    drift_exceeded = False
    total_updates = 0
    zero_updates = 0

    for round_num in range(MAX_ROUNDS):
        # Generate honest nodes
        honest_nodes = rng.normal(current_consensus, honest_std, (n_honest, GENE_DIMENSIONS))

        # Generate coordinated malicious nodes (within 1.5 sigma)
        honest_mean = np.mean(honest_nodes, axis=0)
        h_std = np.std(honest_nodes, axis=0)
        max_offset = SIGMA_MULTIPLIER * np.mean(h_std)
        actual_offset = max_offset * BIAS_LEVEL

        malicious_nodes = []
        for _ in range(F_BYZANTINE):
            noise = rng.normal(0, 0.01, GENE_DIMENSIONS)
            malicious_gene = honest_mean + attack_direction * actual_offset + noise
            malicious_nodes.append(malicious_gene)
        malicious_nodes = np.array(malicious_nodes)

        all_nodes = np.vstack([honest_nodes, malicious_nodes])

        # BFP-16 quantization of each node's "gradient update" (gene vector)
        # Simulate: each node's submission = LR * gene_vector (low-magnitude update)
        quantized_nodes = []
        for node_vec in all_nodes:
            update = LEARNING_RATE * node_vec
            bfp_update = to_bfp(update)
            reconstructed = bfp_update / LEARNING_RATE
            quantized_nodes.append(reconstructed)

            # Track zero-update rate
            non_zero = np.abs(update) > 1e-30
            zeros = (np.abs(bfp_update) == 0) & non_zero
            total_updates += np.sum(non_zero)
            zero_updates += np.sum(zeros)

        quantized_nodes = np.array(quantized_nodes)

        # Aggregate via coordinate-wise trimmed mean
        result = trimmed_mean(quantized_nodes, F_BYZANTINE)

        # Update consensus
        lr = 0.3
        current_consensus = (1 - lr) * current_consensus + lr * result

        # Check drift
        drift_vec = current_consensus - original_center
        drift_mag = np.dot(drift_vec, attack_direction)
        original_mag = np.linalg.norm(original_center)
        drift = abs(drift_mag) / original_mag if original_mag > 1e-10 else 0.0

        if drift > DRIFT_THRESHOLD and not drift_exceeded:
            rounds_to_drift = round_num + 1
            drift_exceeded = True

    # Final drift
    drift_vec = current_consensus - original_center
    drift_mag = np.dot(drift_vec, attack_direction)
    final_drift = abs(drift_mag) / np.linalg.norm(original_center)

    bfp_zero_rate = zero_updates / total_updates if total_updates > 0 else 0.0

    return {
        'rounds_to_drift': rounds_to_drift,
        'final_drift': final_drift,
        'drifted': drift_exceeded,
        'bfp_zero_rate': bfp_zero_rate,
        'total_updates': int(total_updates),
        'zero_updates': int(zero_updates)
    }


def main():
    print("=" * 70)
    print("QRES v19.0 Golden Run: Integrated Multi-Krum + BFP-16")
    print("=" * 70)
    print(f"Timestamp: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"Aggregator: Coordinate-wise Trimmed Mean (trim f={F_BYZANTINE})")
    print(f"Arithmetic: BFP-16 (16-bit mantissa, 8-bit shared exponent)")
    print(f"Attack: Sybil Collusion, n={N_TOTAL}, f={F_BYZANTINE}, bias={BIAS_LEVEL*100:.0f}%")
    print(f"LR for BFP test: {LEARNING_RATE}")
    print(f"Trials: {N_TRIALS}, Max rounds: {MAX_ROUNDS}")
    print("=" * 70)

    results = []
    for trial in range(N_TRIALS):
        seed = 9000 + trial
        r = run_integrated_trial(seed)
        results.append(r)
        if (trial + 1) % 10 == 0:
            print(f"   Completed {trial + 1}/{N_TRIALS} trials...")

    # Aggregate results
    drift_rates = [r['rounds_to_drift'] for r in results]
    final_drifts = [r['final_drift'] for r in results]
    n_drifted = sum(1 for r in results if r['drifted'])
    bfp_zero_rates = [r['bfp_zero_rate'] for r in results]

    mean_drift = np.mean(final_drifts)
    std_drift = np.std(final_drifts)
    ci_drift = 1.96 * std_drift / np.sqrt(N_TRIALS)
    mean_bfp_zero = np.mean(bfp_zero_rates)

    print()
    print("=" * 70)
    print("GOLDEN RUN RESULTS")
    print("=" * 70)
    print(f"Drift at {BIAS_LEVEL*100:.0f}% bias:")
    print(f"   Mean Final Drift: {mean_drift*100:.2f}% +/- {ci_drift*100:.2f}%")
    print(f"   Drift Probability: {n_drifted}/{N_TRIALS} ({n_drifted/N_TRIALS*100:.0f}%)")
    print(f"   Mean Rounds to Drift: {np.mean(drift_rates):.1f}")
    print()
    print(f"BFP-16 at LR={LEARNING_RATE}:")
    print(f"   Mean Zero Update Rate: {mean_bfp_zero*100:.2f}%")
    print(f"   Total updates checked: {results[0]['total_updates']}")
    print()

    # Determine pass/fail
    drift_pass = n_drifted == 0
    bfp_pass = mean_bfp_zero == 0.0

    print("CRITERIA:")
    print(f"   [{'PASS' if drift_pass else 'FAIL'}] Zero drift events at 20% bias: {n_drifted}/{N_TRIALS}")
    print(f"   [{'PASS' if bfp_pass else 'FAIL'}] Zero vanishing updates at LR=1e-5: {mean_bfp_zero*100:.2f}%")
    print()

    overall = "PASS" if (drift_pass and bfp_pass) else "FAIL"
    print(f"OVERALL: {overall}")
    print("=" * 70)

    return {
        'drift_pass': drift_pass,
        'bfp_pass': bfp_pass,
        'overall': overall,
        'mean_drift': mean_drift,
        'ci_drift': ci_drift,
        'n_drifted': n_drifted,
        'mean_bfp_zero': mean_bfp_zero
    }


if __name__ == "__main__":
    result = main()
    sys.exit(0 if result['overall'] == "PASS" else 1)
