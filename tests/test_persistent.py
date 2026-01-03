
import unittest
import os
import sys
import shutil
import numpy as np
import networkx as nx

# Path hack
# sys.path.append(os.path.join(os.getcwd(), 'python'))

from qres.persistent import WorldStateManager

try:
    import qutip as qt
    QUTIP_AVAILABLE = True
except ImportError:
    QUTIP_AVAILABLE = False

class TestPersistentWorldState(unittest.TestCase):
    
    def setUp(self):
        self.test_db = "test_world_state.db"
        if os.path.exists(self.test_db):
            os.remove(self.test_db)
        self.manager = WorldStateManager(self.test_db)
    
    def tearDown(self):
        if os.path.exists(self.test_db):
            os.remove(self.test_db)
    
    def test_serialize_and_load(self):
        print("\n[Test] Serialize and Load World State")
        
        # Create test graph
        graph = nx.Graph()
        graph.add_node("n1", embedding=np.random.rand(4))
        graph.add_node("n2", embedding=np.random.rand(4))
        graph.add_edge("n1", "n2", weight=0.8)
        
        # Create test neural weights
        weights = np.random.randn(5, 5)
        
        # Serialize
        version = self.manager.serialize_world_state(
            graph,
            neural_weights=weights,
            version="test_v1"
        )
        
        self.assertEqual(version, "test_v1")
        
        # Load
        loaded = self.manager.load_world_state("test_v1")
        
        self.assertIsNotNone(loaded)
        self.assertEqual(loaded['graph'].number_of_nodes(), 2)
        self.assertEqual(loaded['graph'].number_of_edges(), 1)
        self.assertTrue(np.allclose(loaded['neural_weights'], weights))
        
        print("✅ Serialization and loading verified")
    
    @unittest.skipIf(not QUTIP_AVAILABLE, "QuTiP not available")
    def test_quantum_tensor_persistence(self):
        print("\n[Test] Quantum Tensor Persistence")
        
        # Create test tensor
        tensor = qt.rand_dm(4)
        graph = nx.Graph()
        graph.add_node("t1")
        
        # Serialize
        version = self.manager.serialize_world_state(
            graph,
            tensor_state=tensor,
            version="test_quantum"
        )
        
        # Load
        loaded = self.manager.load_world_state("test_quantum")
        
        self.assertIsNotNone(loaded['tensor'])
        
        # Check fidelity
        fidelity = qt.fidelity(tensor, loaded['tensor'])
        print(f"  Fidelity: {fidelity:.6f}")
        self.assertGreater(fidelity, 0.99)  # Should be near-perfect
        
        print("✅ Quantum tensor persistence verified")
    
    @unittest.skipIf(not QUTIP_AVAILABLE, "QuTiP not available")
    def test_merge_states(self):
        print("\n[Test] Merge World States")
        
        # Create two states
        g1 = nx.Graph()
        g1.add_node("a")
        t1 = qt.rand_dm(4)
        
        g2 = nx.Graph()
        g2.add_node("b")
        t2 = qt.rand_dm(4)
        
        v1 = self.manager.serialize_world_state(g1, t1, version="state1")
        v2 = self.manager.serialize_world_state(g2, t2, version="state2")
        
        # Merge
        merged_v = self.manager.merge_world_states("state1", "state2")
        
        # Verify
        merged = self.manager.load_world_state(merged_v)
        self.assertEqual(merged['graph'].number_of_nodes(), 2)  # Union
        
        print("✅ State merging verified")
    
    def test_version_management(self):
        print("\n[Test] Version Management")
        
        g = nx.Graph()
        g.add_node("test")
        
        # Create multiple versions
        v1 = self.manager.serialize_world_state(g, version="v1")
        v2 = self.manager.serialize_world_state(g, version="v2")
        
        versions = self.manager.list_versions()
        self.assertEqual(len(versions), 2)
        self.assertIn("v1", versions)
        self.assertIn("v2", versions)
        
        latest = self.manager.get_latest_version()
        self.assertEqual(latest, "v2")
        
        print("✅ Version management verified")

if __name__ == '__main__':
    unittest.main()
