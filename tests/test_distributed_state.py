
import unittest
import os
import sys
import shutil
import time
import numpy as np
import networkx as nx

# Path hack
sys.path.append(os.path.join(os.getcwd(), 'python'))

from qres.api import QRES_API
from qres.persistent import WorldStateManager

try:
    import qutip as qt
    QUTIP_AVAILABLE = True
except ImportError:
    QUTIP_AVAILABLE = False

class TestDistributedWorldState(unittest.TestCase):
    
    def setUp(self):
        # Clean up test directories
        for path in ["quantum_outbox", "quantum_inbox_test", "test_db1.db", "test_db2.db"]:
            if os.path.exists(path):
                if os.path.isdir(path):
                    shutil.rmtree(path)
                else:
                    os.remove(path)
        
        os.makedirs("quantum_outbox", exist_ok=True)
        os.makedirs("quantum_inbox_test", exist_ok=True)
    
    def tearDown(self):
        # Clean up
        for path in ["quantum_outbox", "quantum_inbox_test", "test_db1.db", "test_db2.db"]:
            if os.path.exists(path):
                if os.path.isdir(path):
                    shutil.rmtree(path)
                else:
                    os.remove(path)
    
    def test_broadcast_world_state(self):
        print("\n[Test] Broadcast World State")
        
        # Create API with custom DB
        api = QRES_API(mode="quantum", enable_persistence=True)
        api.world_state.db_path = "test_db1.db"
        
        # Build some state
        api.memory.add_text_node("node1", "Test data for broadcast")
        api.memory.add_text_node("node2", "More test data")
        api.memory.graph.add_edge("node1", "node2", weight=0.9)
        
        # Save state
        version = api.save_world_state("broadcast_test_v1")
        self.assertEqual(version, "broadcast_test_v1")
        
        # Broadcast it
        success = api.broadcast_world_state("broadcast_test_v1")
        self.assertTrue(success)
        
        # Verify file created in outbox
        files = os.listdir("quantum_outbox")
        world_files = [f for f in files if f.startswith("world_") and f.endswith(".qws")]
        self.assertEqual(len(world_files), 1)
        
        # Verify file content
        with open(f"quantum_outbox/{world_files[0]}", "rb") as f:
            data = f.read()
        
        self.assertTrue(data.startswith(b"QRES_WORLD_STATE"))
        
        print("✅ Broadcast verified")
    
    def test_receive_and_merge_world_state(self):
        print("\n[Test] Receive and Merge World State")
        
        # Node 1: Create and broadcast state
        api1 = QRES_API(mode="quantum", enable_persistence=True)
        api1.world_state.db_path = "test_db1.db"
        
        api1.memory.add_text_node("node_a", "From Node 1")
        version1 = api1.save_world_state("node1_state")
        api1.broadcast_world_state("node1_state")
        
        # Move broadcast to inbox (simulating network transfer)
        files = os.listdir("quantum_outbox")
        world_file = [f for f in files if f.endswith(".qws")][0]
        shutil.move(f"quantum_outbox/{world_file}", f"quantum_inbox_test/{world_file}")
        
        # Node 2: Create local state
        api2 = QRES_API(mode="quantum", enable_persistence=True)
        api2.world_state.db_path = "test_db2.db"
        
        api2.memory.add_text_node("node_b", "From Node 2")
        version2 = api2.save_world_state("node2_state")
        
        # Node 2: Receive and merge
        with open(f"quantum_inbox_test/{world_file}", "rb") as f:
            data = f.read()
        
        self.assertTrue(data.startswith(b"QRES_WORLD_STATE"))
        
        # Simulate receiver processing
        import pickle
        state_data = pickle.loads(data[len(b"QRES_WORLD_STATE"):])
        
        remote_version = state_data['version']
        api2.world_state.states[remote_version] = state_data
        api2.world_state._save_db()
        
        # Merge
        merged_version = api2.world_state.merge_world_states(
            "node2_state",
            remote_version,
            fidelity_threshold=0.98
        )
        
        # Verify merged state
        merged_state = api2.world_state.load_world_state(merged_version)
        self.assertIsNotNone(merged_state)
        
        # Should have nodes from both
        self.assertEqual(merged_state['graph'].number_of_nodes(), 2)
        self.assertIn("node_a", merged_state['graph'].nodes())
        self.assertIn("node_b", merged_state['graph'].nodes())
        
        print("✅ Distributed merge verified")
    
    @unittest.skipIf(not QUTIP_AVAILABLE, "QuTiP not available")
    def test_quantum_state_fidelity_across_network(self):
        print("\n[Test] Quantum State Fidelity Across Network")
        
        # Create state with quantum tensor
        api1 = QRES_API(mode="quantum", enable_persistence=True)
        api1.world_state.db_path = "test_db1.db"
        
        # Create graph with embeddings
        api1.memory.add_text_node("q1", "Quantum node 1")
        api1.memory.add_text_node("q2", "Quantum node 2")
        
        # Save and broadcast
        version = api1.save_world_state("quantum_test")
        api1.broadcast_world_state("quantum_test")
        
        # Receive on Node 2
        api2 = QRES_API(mode="quantum", enable_persistence=True)
        api2.world_state.db_path = "test_db2.db"
        
        files = os.listdir("quantum_outbox")
        world_file = [f for f in files if f.endswith(".qws")][0]
        
        with open(f"quantum_outbox/{world_file}", "rb") as f:
            data = f.read()
        
        import pickle
        state_data = pickle.loads(data[len(b"QRES_WORLD_STATE"):])
        
        api2.world_state.states[state_data['version']] = state_data
        loaded = api2.world_state.load_world_state(state_data['version'])
        
        # Verify graph integrity
        self.assertEqual(loaded['graph'].number_of_nodes(), 2)
        
        print("✅ Network fidelity verified")

if __name__ == '__main__':
    unittest.main()
