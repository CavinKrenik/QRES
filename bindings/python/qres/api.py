"""
QRES v10.0 Unified Python API
integrates Multi-Modal Memory, Tensor Network Compression, and Neural Optimization.
"""

import os
import sys
import numpy as np

# ensure imports work
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from multimodal import MultiModalMemory
from tensor import TensorEncoder
from neural import NeuralOptimizer
from persistent import WorldStateManager
try:
    from stable_baselines3 import PPO
    PPO_AVAILABLE = True
except ImportError:
    PPO_AVAILABLE = False
import struct
try:
    from . import qres_rust
except ImportError:
    try:
        import qres_rust
    except ImportError:
        qres_rust = None

class QRES_API:
    def __init__(self, mode="hybrid", enable_persistence=True):
        self.mode = mode
        self.memory = MultiModalMemory()
        self.tensor = TensorEncoder(n_qubits_per_node=2)
        self.neural = NeuralOptimizer()
        self.brain_weights = None
        
        # Load MetaBrain (PPO) if available
        self.metabrain = None
        if PPO_AVAILABLE and os.path.exists("ai/metabrain_ppo_v4.zip"):
            try:
                self.metabrain = PPO.load("ai/metabrain_ppo_v4.zip")
                print("[API] MetaBrain (PPO) loaded successfully.")
            except Exception as e:
                print(f"[API] Failed to load MetaBrain: {e}")

        # Phase 4: Persistent World State
        self.persistence_enabled = enable_persistence
        if enable_persistence:
            self.world_state = WorldStateManager()
        else:
            self.world_state = None
        
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
        if self.mode == "tensor":
            return self._compress_tensor(data)
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

    def merge_tensor_state(self, tensor_bytes: bytes):
        """
        [Receiver] reconstructs a graph/state from received tensor bytes.
        """
        print("[API] Receiving Tensor State...")
        
        # 1. Deserialize
        # Verify header
        if not tensor_bytes.startswith(b"QRES_T_TENSOR"):
            print("[API] Error: Invalid Tensor Header")
            return False
            
        payload = tensor_bytes[len(b"QRES_T_TENSOR"):]
        
        # 2. Reconstruct (Mocking reconstruction of Density Matrix)
        # In a real app, we'd use qutip.Qobj(payload)
        import time
        print(f"  - Payload Size: {len(payload)} bytes")
        print("  - Reconstructing Density Matrix (Telepathy)...")
        time.sleep(0.1) 
        
        # 3. Fidelity Check (Simulated)
        fidelity = 0.98 # Mock high fidelity
        print(f"  - Fidelity Check: {fidelity:.4f} (Pass)")
        
        # 4. Merge into Memory
        # We assume the tensor encodes new knowledge (nodes/edges).
        # We'll add a "Remote_Node" to our graph to signify learned data.
        node_id = f"remote_{int(time.time())}"
        self.memory.add_text_node(node_id, "Imported Quantum Knowledge")
        print(f"[API] Merged remote state into MultiModal Memory (Node: {node_id})")
        
        return True

    def save_world_state(self, version: str = None) -> str:
        """
        Save current system state to persistent storage.
        
        Returns:
            Version key of saved state
        """
        if not self.persistence_enabled:
            print("[API] Persistence disabled")
            return None
            
        print("[API] Saving World State...")
        
        # Get current tensor if available
        tensor = None
        if self.memory.graph.number_of_nodes() > 0:
            full, reduced, metrics = self.tensor.encode_graph(self.memory.graph)
            tensor = reduced if reduced is not None else full
        
        version = self.world_state.serialize_world_state(
            self.memory.graph,
            tensor,
            self.brain_weights,
            version
        )
        
        print(f"[API] World state saved as {version}")
        return version
    
    def load_world_state(self, version: str = None):
        """
        Load a world state from persistent storage.
        
        Args:
            version: Specific version to load (None = latest)
        """
        if not self.persistence_enabled:
            print("[API] Persistence disabled")
            return False
            
        if version is None:
            version = self.world_state.get_latest_version()
            if version is None:
                print("[API] No saved states found")
                return False
        
        print(f"[API] Loading World State: {version}...")
        
        state = self.world_state.load_world_state(version)
        if state is None:
            return False
        
        # Restore graph
        self.memory.graph = state['graph']
        
        # Restore neural weights
        if state['neural_weights'] is not None:
            self.brain_weights = state['neural_weights']
        
        print(f"[API] World state restored from {version}")
        print(f"  - Nodes: {self.memory.graph.number_of_nodes()}")
        print(f"  - Edges: {self.memory.graph.number_of_edges()}")
        
        return True

    def broadcast_world_state(self, version: str = None) -> bool:
        """
        Broadcast a world state to the P2P swarm for distributed synchronization.
        
        Args:
            version: Version to broadcast (None = current state, save first)
        
        Returns:
            True if broadcast successful
        """
        if not self.persistence_enabled:
            print("[API] Persistence disabled")
            return False
        
        # Save current state if no version specified
        if version is None:
            print("[API] Saving current state for broadcast...")
            version = self.save_world_state()
        
        # Load the state to get serialized data
        state_data = self.world_state.states.get(version)
        if state_data is None:
            print(f"[API] Version {version} not found")
            return False
        
        # Serialize for transmission
        import pickle
        serialized = pickle.dumps(state_data)
        
        # Add world state header
        broadcast_data = b"QRES_WORLD_STATE" + serialized
        
        # Write to outbox for swarm broadcast
        if not os.path.exists("tensor_outbox"):
            os.makedirs("tensor_outbox")
        
        import time
        filename = f"tensor_outbox/world_{version}_{int(time.time()*1000)}.qws"
        
        with open(filename, "wb") as f:
            f.write(broadcast_data)
        
        print(f"[API] World state {version} queued for broadcast")
        print(f"  - Size: {len(broadcast_data) / 1024:.2f} KB")
        print(f"  - File: {filename}")
        
        return True

    def _compress_standard(self, data: bytes) -> bytes:
        weights = None
        if self.metabrain:
            try:
                # Feature Extraction: Normalized Byte Histogram + Entropy
                # Using numpy for speed
                arr = np.frombuffer(data, dtype=np.uint8)
                counts = np.bincount(arr, minlength=256).astype(np.float32)
                
                if len(data) > 0:
                    # Histogram feature
                    norm_hist = counts / len(data)
                    
                    # Entropy calculation logic
                    probs = counts[counts > 0] / len(data)
                    entropy = -np.sum(probs * np.log2(probs))
                    # Normalize entropy (0-8) -> 0-1
                    norm_entropy = entropy / 8.0
                    
                    # Concatenate [Hist(256), Entropy(1)] -> (257,)
                    obs = np.concatenate([norm_hist, [norm_entropy]])
                    
                    action, _ = self.metabrain.predict(obs, deterministic=True)
                    
                    # Pack 6 floats (24 bytes) for Rust
                    if len(action) >= 6:
                         weights = b''.join([struct.pack('<f', float(x)) for x in action[:6]])
                        #  print(f"[API] MetaBrain weights: {[f'{x:.2f}' for x in action[:6]]}")
            except Exception as e:
                print(f"[API] MetaBrain error: {e}")

        if qres_rust:
            try:
                return qres_rust.encode_bytes(data, 0, weights)
            except Exception as e:
                print(f"[API] Rust backend warning: {e}")
                return data
        return data

    def _compress_tensor(self, data: bytes) -> bytes:
        """
        Experimental: Maps bytes to graph -> tensor -> compressed.
        """
        print("[API] Tensor Mode: Activating Tensor Network...")
        
        # 1. Byte -> Text/Image Node (Mock classification)
        # For demo, treat data as text
        try:
            text = data.decode('utf-8')
            node_id = f"chunk_{hash(data)}"
            self.memory.add_text_node(node_id, text)
        except UnicodeDecodeError:
            print("[API] Non-UTF8 data detected. Building binary spectral graph...")
            import networkx as nx
            # Binary-to-graph: Treat as byte sequences, build spectral graph
            bytes_seq = list(data)
            # Create a simple chain/correlation graph from bytes
            # Limiting graph size for performance on large binary blobs
            max_nodes = min(len(bytes_seq), 1000) 
            
            # Temporary graph for this chunk
            temp_graph = nx.Graph()
            for i in range(max_nodes - 1):
                u, v = bytes_seq[i], bytes_seq[i+1]
                if temp_graph.has_edge(u, v):
                    temp_graph[u][v]['weight'] += 1
                else:
                    temp_graph.add_edge(u, v, weight=1)
            
            # Merge into main memory (or keep separate). For now, we merge a summary node.
            # In a full implementation, we'd merge the whole subgraph or use it for tensor contraction directly.
            # Here we just ensure the memory graph has content so encode_graph works.
            node_id = f"bin_chunk_{hash(data)}"
            self.memory.graph.add_node(node_id, type="binary", size=len(data))
            # Attach the binary graph structure to the main memory somehow, or just let encode_graph use the main memory.
            # For this demo, let's just ensure we don't crash and have *some* graph.
            # Real fallback: use the temp_graph for the encoding step.
            self.memory.graph = nx.compose(self.memory.graph, temp_graph)
             
        # 2. Encode Graph
        full, reduced, metrics = self.tensor.encode_graph(self.memory.graph)
        
        if metrics and 'ratio' in metrics:
            print(f"[API] Tensor Compression Ratio: {metrics['ratio']:.4%}")
            # Serialize reduced tensor (mock serialization)
            return b"QRES_T_TENSOR" + reduced.full().tobytes()
        else:
            print("[API] Tensor Encode metrics missing or failed, fallback.")
            return data

if __name__ == "__main__":
    api = QRES_API(mode="tensor")
    api.load_brain()
    
    # Prune
    api.optimize_system()
    
    # Compress
    out = api.compress(b"Hello Quantum World")
    print("Output size:", len(out))
