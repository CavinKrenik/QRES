
import unittest
import sys
import os
import shutil
import numpy as np

# Ensure root is in path
sys.path.append(os.getcwd())
# sys.path.append(os.path.join(os.getcwd(), 'python'))

class TestPhase2(unittest.TestCase):

    def test_quantum_encoder(self):
        print("\n[Test] Quantum Encoder")
        from qres.tensor import TensorEncoder
        from qres.multimodal import MultiModalMemory
        
        # 1. Setup Data
        mm = MultiModalMemory()
        mm.add_text_node("t1", "test")
        mm.add_text_node("t2", "data")
        
        # 2. Encode
        qe = TensorEncoder(n_qubits_per_node=2)
        full, reduced, metrics = qe.encode_graph(mm.graph)
        
        # 3. Validation
        self.assertIsNotNone(full)
        self.assertIsNotNone(reduced)
        self.assertLess(metrics['ratio'], 0.10) # Expect high compression
        
        # 4. Noise
        noisy = qe.simulate_noise(full, error_prob=0.01)
        self.assertNotEqual(full, noisy)

if __name__ == '__main__':
    unittest.main()
