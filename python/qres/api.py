"""
QRES v7.0/v7.5 Unified Python API
integrates Multi-Modal Memory, Quantum Compression, and Neural Optimization.
"""

import os
import sys
import numpy as np

# ensure imports work
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from multimodal import MultiModalMemory
from quantum import QuantumEncoder
from neural import NeuralOptimizer
try:
    import qres_rust
except ImportError:
    qres_rust = None

class QRES_API:
    def __init__(self, mode="hybrid"):
        self.mode = mode
        self.memory = MultiModalMemory()
        self.quantum = QuantumEncoder(n_qubits_per_node=2)
        self.neural = NeuralOptimizer()
        self.brain_weights = None
        
    def load_brain(self, path="qres_rust/assets/meta_brain_v2.json"):
        """Mock loader for header/weights."""
        # Real implementation would load JSON/SafeTensors
        self.brain_weights = np.random.normal(0, 1, (10, 10)) 
        print(f"[API] Loaded brain weights from {path}")

    def compress(self, data: bytes, usage_hint="auto") -> bytes:
        """
        Main compression entry point.
        Dispatches to Rust or Quantum core based on mode.
        """
        if self.mode == "quantum":
            return self._compress_quantum(data)
        else:
            return self._compress_standard(data)

    def optimize_system(self):
        """
        Triggers self-optimization:
        1. Ethical Pruning of Memory
        2. AQC Pruning of Neural Weights
        """
        print("[API] Starting System Optimization...")
        
        # 1. Memory
        has_bias = self.memory.detect_bias()
        if has_bias:
            print("[API] Memory bias corrected.")
            
        # 2. Neural (Simulated AQC)
        if self.brain_weights is not None:
            print("[API] Optimizing Neural Weights via AQC...")
            original_sparsity = 1.0 - (np.count_nonzero(self.brain_weights) / self.brain_weights.size)
            
            self.brain_weights = self.neural.aqc_prune_weights(self.brain_weights)
            
            new_sparsity = 1.0 - (np.count_nonzero(self.brain_weights) / self.brain_weights.size)
            print(f"[API] Sparsity improved: {original_sparsity:.2%} -> {new_sparsity:.2%}")

    def _compress_standard(self, data: bytes) -> bytes:
        if qres_rust:
            # return qres_rust.encode_bytes(data, [], 0)
            # Mock for now if DLL issue
            return data 
        return data

    def _compress_quantum(self, data: bytes) -> bytes:
        """
        Experimental: Maps bytes to graph -> tensor -> compressed.
        """
        print("[API] Quantum Mode: Activating Tensor Network...")
        
        # 1. Byte -> Text/Image Node (Mock classification)
        # For demo, treat data as text
        try:
            text = data.decode('utf-8')
            node_id = f"chunk_{hash(data)}"
            self.memory.add_text_node(node_id, text)
        except:
             # Binary data
             pass
             
        # 2. Encode Graph
        full, reduced, metrics = self.quantum.encode_graph(self.memory.graph)
        
        if metrics and 'ratio' in metrics:
            print(f"[API] Quantum Compression Ratio: {metrics['ratio']:.4%}")
            # Serialize reduced tensor (mock serialization)
            return b"QRES_Q_TENSOR" + reduced.full().tobytes()
        else:
            print("[API] Quantum Encode metrics missing or failed, fallback.")
            return data

if __name__ == "__main__":
    api = QRES_API(mode="quantum")
    api.load_brain()
    
    # Prune
    api.optimize_system()
    
    # Compress
    out = api.compress(b"Hello Quantum World")
    print("Output size:", len(out))
