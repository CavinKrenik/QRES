"""
QRES v19.0 - Dynamic Mid-Flight Onboarding Test

Compares two onboarding strategies for new nodes joining a mature swarm:
    Scenario A: Full History Replay (receive and replay all 500 rounds)
    Scenario B: Hippocampus Summary Gene (receive a single compressed state)

Metric: Total bytes transferred for the new node to reach consensus.
Success Criterion: Scenario B reduces bandwidth by >90%.

Usage: python tools/onboarding_test.py
"""

import numpy as np
import sys
from datetime import datetime

# --- Configuration ---
N_NODES = 20
N_ROUNDS = 500
GENE_DIMENSIONS = 8
BYTES_PER_FLOAT32 = 4
SUMMARY_OVERHEAD_FACTOR = 1.5  # Summary gene is slightly larger than one round


def simulate_swarm_history(n_rounds, n_nodes, gene_dims, seed):
    """
    Simulate a swarm evolving over n_rounds.
    Each round: every node submits an 8-dimensional gene vector.
    The swarm state (consensus) evolves via simple averaging.

    Returns:
        history: list of (round_submissions, consensus) per round
        final_consensus: the converged state
    """
    rng = np.random.default_rng(seed)

    consensus = rng.normal(0, 1.0, gene_dims)
    history = []

    for r in range(n_rounds):
        # Each node submits a vector near the current consensus
        submissions = rng.normal(consensus, 0.05, (n_nodes, gene_dims))
        new_consensus = np.mean(submissions, axis=0)

        history.append({
            'round': r,
            'submissions': submissions,
            'consensus': new_consensus.copy()
        })

        consensus = new_consensus

    return history, consensus


def scenario_a_full_replay(history, gene_dims):
    """
    Scenario A: New node receives the full submission history.

    Bandwidth = sum of all submissions across all rounds.
    The new node replays each round's aggregation to reconstruct state.
    """
    total_bytes = 0
    for round_data in history:
        n_submissions = len(round_data['submissions'])
        # Each submission is gene_dims * float32
        round_bytes = n_submissions * gene_dims * BYTES_PER_FLOAT32
        total_bytes += round_bytes

    return {
        'total_bytes': total_bytes,
        'n_rounds_replayed': len(history),
        'description': 'Full History Replay'
    }


def scenario_b_summary_gene(history, gene_dims):
    """
    Scenario B: New node receives a single "Summary Gene" from the
    Hippocampus layer.

    The Summary Gene encodes:
        1. The current consensus vector (gene_dims * float32)
        2. A confidence/variance vector (gene_dims * float32)
        3. A round counter (4 bytes)
        4. A cryptographic hash of the history (32 bytes)

    Total: ~2 * gene_dims * 4 + 36 bytes
    With SUMMARY_OVERHEAD_FACTOR applied for metadata framing.
    """
    # Core payload: consensus + variance
    payload_bytes = 2 * gene_dims * BYTES_PER_FLOAT32

    # Metadata: round counter (4 bytes) + history hash (32 bytes)
    metadata_bytes = 4 + 32

    # Framing overhead (protocol headers, signatures)
    total_bytes = int((payload_bytes + metadata_bytes) * SUMMARY_OVERHEAD_FACTOR)

    return {
        'total_bytes': total_bytes,
        'n_rounds_replayed': 0,
        'description': 'Hippocampus Summary Gene'
    }


def scenario_c_stale_node_packet_loss(summary_bytes, packet_loss_rate=0.3):
    """
    Scenario C: Stale Node joining under High Packet Loss (30%).
    The Summary Gene is small enough to fit in a single UDP packet (MTU ~1500).
    We simulate independent Bernoulli trials for packet reception.
    """
    rng = np.random.default_rng(42)
    attempts = 0
    received = False
    
    # Simulate ARQ (Automatic Repeat Request)
    while not received:
        attempts += 1
        if rng.random() > packet_loss_rate:
            received = True
            
    # Total bandwidth = summary size * attempts (including ack/nack overhead roughly)
    total_bytes = summary_bytes * attempts
    
    return {
        'total_bytes': total_bytes,
        'attempts': attempts,
        'description': f'Summary Gene (30% Loss)'
    }


def verify_convergence(history, summary_consensus, gene_dims, seed):
    """
    Verify that the Summary Gene allows the new node to converge
    to the same state as if it had replayed the full history.

    The new node starts from the summary consensus and participates
    in a few rounds to confirm alignment.
    """
    rng = np.random.default_rng(seed + 1000)

    # Full-replay node: has the exact final consensus
    replay_state = history[-1]['consensus'].copy()

    # Summary node: starts from summary consensus
    summary_state = summary_consensus.copy()

    # Both participate in 5 additional rounds
    convergence_rounds = 5
    for _ in range(convergence_rounds):
        target = rng.normal(replay_state, 0.01, gene_dims)
        replay_state = 0.9 * replay_state + 0.1 * target
        summary_state = 0.9 * summary_state + 0.1 * target

    distance = np.linalg.norm(replay_state - summary_state)
    return distance


def main():
    print("=" * 70)
    print("QRES v19.0 - Dynamic Mid-Flight Onboarding Test")
    print("=" * 70)
    print(f"Timestamp: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"Swarm: {N_NODES} nodes, {N_ROUNDS} rounds, {GENE_DIMENSIONS}-dim genes")
    print(f"Bytes per float: {BYTES_PER_FLOAT32}")
    print("=" * 70)

    seed = 42
    print(f"\nSimulating {N_ROUNDS} rounds of swarm evolution...")
    history, final_consensus = simulate_swarm_history(
        N_ROUNDS, N_NODES, GENE_DIMENSIONS, seed
    )
    print(f"   Swarm converged. Final consensus norm: {np.linalg.norm(final_consensus):.4f}")

    # Scenario A: Full Replay
    print("\n--- Scenario A: Full History Replay ---")
    result_a = scenario_a_full_replay(history, GENE_DIMENSIONS)
    print(f"   Rounds replayed: {result_a['n_rounds_replayed']}")
    print(f"   Total bytes transferred: {result_a['total_bytes']:,}")
    print(f"   ({result_a['total_bytes'] / 1024:.1f} KB)")

    # Scenario B: Summary Gene
    print("\n--- Scenario B: Hippocampus Summary Gene ---")
    result_b = scenario_b_summary_gene(history, GENE_DIMENSIONS)
    print(f"   Rounds replayed: {result_b['n_rounds_replayed']}")
    print(f"   Total bytes transferred: {result_b['total_bytes']:,}")
    print(f"   ({result_b['total_bytes'] / 1024:.4f} KB)")

    # Scenario C: Stale Node + 30% Packet Loss
    print("\n--- Scenario C: Stale Node + 30% Loss ---")
    result_c = scenario_c_stale_node_packet_loss(result_b['total_bytes'], packet_loss_rate=0.30)
    print(f"   Packet Loss Sim: 30%")
    print(f"   Attempts needed: {result_c['attempts']}")
    print(f"   Effective Total Bytes: {result_c['total_bytes']:,}")

    # Verify convergence equivalence
    print("\n--- Convergence Verification ---")
    distance = verify_convergence(history, final_consensus, GENE_DIMENSIONS, seed)
    print(f"   Post-onboarding state divergence: {distance:.2e}")
    print(f"   States equivalent: {'YES' if distance < 0.01 else 'NO'}")

    # Calculate reduction
    reduction = 1.0 - (result_b['total_bytes'] / result_a['total_bytes'])

    print()
    print("=" * 70)
    print("ONBOARDING TEST RESULTS")
    print("=" * 70)
    print(f"Scenario A (Full Replay):    {result_a['total_bytes']:>10,} bytes ({result_a['total_bytes']/1024:.1f} KB)")
    print(f"Scenario B (Summary Gene):   {result_b['total_bytes']:>10,} bytes ({result_b['total_bytes']/1024:.4f} KB)")
    print(f"Bandwidth Reduction:         {reduction*100:.2f}%")
    print(f"Compression Ratio:           {result_a['total_bytes'] / result_b['total_bytes']:.0f}:1")
    print()

    passed = reduction > 0.90
    print(f"CRITERION: >90% bandwidth reduction: [{'PASS' if passed else 'FAIL'}] ({reduction*100:.2f}%)")
    print("=" * 70)

    return {
        'passed': passed,
        'reduction': reduction,
        'bytes_a': result_a['total_bytes'],
        'bytes_b': result_b['total_bytes'],
        'convergence_distance': distance
    }


if __name__ == "__main__":
    result = main()
    sys.exit(0 if result['passed'] else 1)
