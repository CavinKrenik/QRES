"""
Phase 2: Quantum-Inspired Tensor Networks for Compression
Focus: Research & Prototyping
"""

import numpy as np
import sys

try:
    import qutip
    from qutip import tensor, rand_dm, Qobj
    # Try different locations for partial_trace
    try:
        from qutip import ptrace as partial_trace
    except ImportError:
        try:
            from qutip.core import ptrace as partial_trace
        except ImportError:
            # Maybe it's a function on Qobj or different name
            print("Warning: partial_trace not found in standard paths")
            partial_trace = None
except ImportError as e:
    print(f"QuTiP Import Error: {e}")
    sys.exit(1)

def simulate_tensor_compression(num_qubits=4):
    print(f"🔬 Simulating Quantum Tensor Network with {num_qubits} qubits...")
    
    # 1. Create a random state (density matrix) representing a complex data chunk
    # In reality, we would map data to this state via amplitude encoding.
    dims_list = [2] * num_qubits
    # rand_dm(N) returns an NxN density matrix. 
    full_state = rand_dm(2**num_qubits)
    full_state.dims = [dims_list, dims_list]
    
    original_size = full_state.shape[0] * full_state.shape[1] * 16 # Complex128 bytes
    print(f"  Original State Size (Memory): {original_size} bytes (Virtual)")
    
    # 2. Schmidt Decomposition via Partial Trace
    # We trace out half the system to find the reduced density matrix of the "core"
    keep_indices = list(range(num_qubits // 2))
    reduced_state = partial_trace(full_state, keep_indices)
    
    compressed_size = reduced_state.shape[0] * reduced_state.shape[1] * 16
    print(f"  Reduced State Size: {compressed_size} bytes")
    
    # 3. Calculate Von Neumann Entropy (Information Content)
    # Lower entropy = higher compressibility potential using quantum tensors
    entropy = qutip.entropy_vn(reduced_state)
    print(f"  Von Neumann Entropy: {entropy:.4f}")
    
    ratio = compressed_size / original_size
    print(f"  Theoretical Compression Ratio: {ratio:.2%}")
    
    return ratio, entropy

def map_data_to_state(data_bytes):
    """
    Experimental: Map 4 bytes to a 2-qubit state.
    """
    # Normalize bytes to amplitudes
    floats = np.frombuffer(data_bytes, dtype=np.uint8) / 255.0
    # Pad to power of 2
    target_len = 2**np.ceil(np.log2(len(floats))).astype(int)
    padded = np.zeros(target_len)
    padded[:len(floats)] = floats
    padded /= np.linalg.norm(padded) # Normalize state
    
    # Create Qobj
    psi = Qobj(padded)
    return psi

if __name__ == "__main__":
    print("="*60)
    print("Quantum-Inspired Tensor Network Research (v7.5)")
    print("="*60)
    
    simulate_tensor_compression(4)
    simulate_tensor_compression(8)
    
    print("\n🧪 Mapping Real Data...")
    data = b"test_data_chunk"
    state = map_data_to_state(data)
    print(f"  Mapped '{data.decode()}' to Quantum State: {state.shape}")
