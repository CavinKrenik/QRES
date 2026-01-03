
import unittest
import sys
import os
import shutil
import numpy as np

# Ensure root is in path
sys.path.append(os.getcwd())
sys.path.append(os.getcwd())
# sys.path.append(os.path.join(os.getcwd(), 'python')) -> Removed to avoid shadowing installed package

class TestPhase1(unittest.TestCase):

    def test_multimodal_memory(self):
        print("\n[Test] Multi-Modal Memory")
        from qres.multimodal import MultiModalMemory
        mm = MultiModalMemory() # Defaults to cpu if no cuda
        
        # Test Text Node
        mm.add_text_node("t1", "hello world")
        self.assertTrue("t1" in mm.graph.nodes)
        
        # Test Bias Detection (Simulate)
        # Create a distribution where median is low but one edge is huge
        mm.graph.add_edge("t1", "bias_target", weight=1.0)
        mm.graph.add_edge("t1", "other1", weight=0.01)
        mm.graph.add_edge("t1", "other2", weight=0.01)
        mm.graph.add_edge("t1", "other3", weight=0.01)
        
        # Weights: [1.0, 0.01, 0.01, 0.01]
        # Median: 0.01
        # Outlier Threshold: 0.02
        # 1.0 > 0.02 -> Should prune
        
        # Should detect bias
        has_bias = mm.detect_bias(threshold=0.4)
        print("Bias Detected:", has_bias)
        self.assertTrue(has_bias)
        
        # Verify Pruning
        e_data = mm.graph.get_edge_data("t1", "bias_target")
        self.assertTrue(e_data['decayed'])
        self.assertLess(e_data['weight'], 0.9)

    def test_rl_env(self):
        print("\n[Test] RL Environment")
        from ai.rl_mixer_env import CompressionMixingEnv
        env = CompressionMixingEnv()
        obs, _ = env.reset()
        
        # Check observation shape
        self.assertEqual(obs.shape, (5,))
        
        # Check Step
        action = np.array([0.25, 0.25, 0.25, 0.25], dtype=np.float32)
        obs, reward, done, _, info = env.step(action)
        
        print(f"Step Reward: {reward}, Ratio: {info['ratio']}")
        self.assertIsInstance(reward, float)
        self.assertIsInstance(info['ratio'], float)

    def test_tensor_sim(self):
        print("\n[Test] Quantum Tensor Sim (Legacy)")
        # Since train_tensor_v7 was refactored to use QuantumEncoder, we check that directly (or skip)
        from qres.quantum import QuantumEncoder
        qe = QuantumEncoder(n_qubits_per_node=2)
        # Mock Graph
        import networkx as nx 
        import torch
        g = nx.Graph()
        g.add_node("n1", embedding=torch.rand(4))
        g.add_node("n2", embedding=torch.rand(4))
        
        full, reduced, metrics = qe.encode_graph(g)
        self.assertIsNotNone(full)
        self.assertLess(metrics['ratio'], 1.0)

if __name__ == '__main__':
    unittest.main()
