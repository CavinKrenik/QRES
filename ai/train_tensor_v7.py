import sys
import os
sys.path.append(os.path.join(os.getcwd(), 'python'))
from qres.quantum import QuantumEncoder
from qres.multimodal import MultiModalMemory
import numpy as np

def test_multimodal_compression():
    print("="*60)
    print("Quantum-Enhanced Multi-Modal Compression (v7.5)")
    print("="*60)
    
    # 1. Setup Phase 1 Memory
    print("[1] Initializing MultiModal Memory...")
    mm = MultiModalMemory()
    mm.add_text_node("doc1", "Quantum computing uses qubits.")
    mm.add_text_node("doc2", "QRES optimizes data storage with AI.")
    mm.add_text_node("img1", "path/to/virtual_image.png") # Virtual, embedding will be 0s or we mock it
    
    # Mock embedding for image since path doesn't exist really
    import torch
    mm.graph.nodes["img1"]["embedding"] = torch.rand(384) 
    
    # 2. Initialize Phase 2 Quantum Encoder
    # Using 2 qubits per node for faster simulation of noise calculation
    qe = QuantumEncoder(n_qubits_per_node=2)
    
    # 3. Compress
    print("[2] Encoding Graph to Quantum Tensor Network...")
    full, reduced, metrics = qe.encode_graph(mm.graph)
    
    if metrics:
        print(f"\n[3] Compression Results:")
        print(f"  - Simulated Qubits: {metrics['qubits_simulated']}")
        print(f"  - Original State Size: {metrics['original_size'] / 1024:.2f} KB")
        print(f"  - Compressed Size: {metrics['compressed_size'] / 1024:.2f} KB")
        print(f"  - Compression Ratio: {metrics['ratio']:.4%}")
        print(f"  - Von Neumann Entropy: {metrics['entropy']:.4f}")
        
    # 4. Noise Simulation
    print("\n[4] Simulating Decoherence (Noise)...")
    noisy = qe.simulate_noise(full, error_prob=0.1)
    # properly check fidelity using qutip
    # qutip.fidelity(A, B) = trace(sqrt(sqrt(A) * B * sqrt(A)))
    try:
        import qutip as qt
        fid = qt.fidelity(full, noisy)
        print(f"  - Fidelity (Original vs Noisy): {fid:.4f}")
    except Exception as e:
        print(f"  - Fidelity check failed: {e}")

if __name__ == "__main__":
    test_multimodal_compression()
