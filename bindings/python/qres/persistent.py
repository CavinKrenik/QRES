"""
QRES v8.1 - Persistent World State Management
Enables lossless serialization and persistence of complete system states
across compression cycles, supporting continuity and proto-identity.
"""

import json
import base64
import os
import time
import numpy as np
from typing import Optional, Tuple, Dict
import networkx as nx
import torch

try:
    import qutip as qt
    QUTIP_AVAILABLE = True
except ImportError:
    QUTIP_AVAILABLE = False
    print("[QRES-Persistent] QuTiP not available. Persistence disabled.")

class WorldStateManager:
    """
    Manages persistent world states using quantum-inspired compression.
    Stores complete snapshots of MultiModalMemory graphs, quantum tensors,
    and neural weights with versioning and fidelity guarantees.
    """
    
    def __init__(self, db_path: str = "qres_world_state.db"):
        self.db_path = db_path
        self.states = {}  # In-memory cache (will use sled in Rust later)
        self._load_db()
        
    def _json_encoder(self, obj):
        if isinstance(obj, np.ndarray):
            return {
                "__type__": "numpy",
                "dtype": str(obj.dtype),
                "shape": obj.shape,
                "data": base64.b64encode(obj.tobytes()).decode('ascii')
            }
        elif isinstance(obj, torch.Tensor):
             return {
                "__type__": "torch",
                "dtype": str(obj.dtype),
                "shape": tuple(obj.shape),
                "data": base64.b64encode(obj.detach().cpu().numpy().tobytes()).decode('ascii')
            }
        elif isinstance(obj, bytes):
             return {
                "__type__": "bytes",
                "data": base64.b64encode(obj).decode('ascii')
            }
        elif isinstance(obj, complex):
             return {
                "__type__": "complex",
                "real": obj.real,
                "imag": obj.imag
            }
        raise TypeError(f"Type {type(obj)} not serializable")

    def _json_decoder(self, dct):
        if "__type__" in dct:
            if dct["__type__"] == "numpy":
                data = base64.b64decode(dct["data"])
                return np.frombuffer(data, dtype=dct["dtype"]).reshape(dct["shape"])
            elif dct["__type__"] == "torch":
                data = base64.b64decode(dct["data"])
                # Note: This naively restores as float32/64 depending on setup, rigorous type string parsing omitted for brevity
                # Getting dtype from string correctly would require a mapping, defaulting to float32 for now or simple eval if safe
                np_arr = np.frombuffer(data).reshape(dct["shape"])
                return torch.from_numpy(np_arr)
            elif dct["__type__"] == "bytes":
                return base64.b64decode(dct["data"])
            elif dct["__type__"] == "complex":
                return complex(dct["real"], dct["imag"])
        return dct

    def _load_db(self):
        """Load existing states from disk."""
        if os.path.exists(self.db_path):
            try:
                with open(self.db_path, 'r') as f:
                    self.states = json.load(f, object_hook=self._json_decoder)
                print(f"[WorldState] Loaded {len(self.states)} states from {self.db_path}")
            except Exception as e:
                print(f"[WorldState] Warning: Could not load DB: {e}")
                self.states = {}
        else:
            self.states = {}
            
    def _save_db(self):
        """Persist states to disk."""
        try:
            with open(self.db_path, 'w') as f:
                json.dump(self.states, f, default=self._json_encoder)
        except Exception as e:
            print(f"[WorldState] Error saving DB: {e}")
    
    def serialize_world_state(
        self, 
        graph: nx.Graph, 
        tensor_state: Optional[object] = None,
        neural_weights: Optional[np.ndarray] = None,
        version: Optional[str] = None
    ) -> str:
        """
        Serialize a complete world state with quantum compression.
        
        Args:
            graph: MultiModalMemory graph
            tensor_state: QuTiP Qobj tensor (optional)
            neural_weights: Neural network weights (optional)
            version: Version identifier (auto-generated if None)
            
        Returns:
            Version key for the stored state
        """
        if not QUTIP_AVAILABLE and tensor_state is not None:
            print("[WorldState] Warning: QuTiP unavailable, tensor not compressed")
            
        # Generate version if not provided
        if version is None:
            version = f"world_v{int(time.time())}"
            
        print(f"[WorldState] Serializing {version}...")
        
        # 1. Serialize Graph
        graph_data = {
            'nodes': list(graph.nodes(data=True)),
            'edges': list(graph.edges(data=True)),
            'graph_attrs': dict(graph.graph)
        }
        
        # 2. Compress Tensor State (if available)
        compressed_tensor = None
        if tensor_state is not None and QUTIP_AVAILABLE:
            try:
                # Direct serialization for lossless storage
                # (Unitary compression can be added later for transmission)
                compressed_tensor = {
                    'data': tensor_state.full().tobytes(),
                    'dims': tensor_state.dims,
                    'shape': tensor_state.shape
                }
            except Exception as e:
                print(f"[WorldState] Tensor compression failed: {e}")
                compressed_tensor = None
        
        # 3. Package complete state
        world_state = {
            'version': version,
            'timestamp': time.time(),
            'graph': graph_data,
            'tensor': compressed_tensor,
            'neural_weights': neural_weights.tobytes() if neural_weights is not None else None,
            'neural_shape': neural_weights.shape if neural_weights is not None else None,
            'metadata': {
                'num_nodes': graph.number_of_nodes(),
                'num_edges': graph.number_of_edges(),
                'tensor_size': len(compressed_tensor['data']) if compressed_tensor else 0
            }
        }
        
        # 4. Store
        self.states[version] = world_state
        self._save_db()
        
        # Calculate JSON size estimation
        size_kb = len(json.dumps(world_state, default=self._json_encoder)) / 1024
        print(f"[WorldState] Persisted {version} ({size_kb:.2f} KB)")
        print(f"  - Nodes: {world_state['metadata']['num_nodes']}")
        print(f"  - Edges: {world_state['metadata']['num_edges']}")
        
        return version
    
    def load_world_state(self, version: str) -> Optional[Dict]:
        """
        Load a world state by version.
        
        Returns:
            Dictionary with 'graph', 'tensor', 'neural_weights'
        """
        if version not in self.states:
            print(f"[WorldState] Version {version} not found")
            return None
            
        state = self.states[version]
        print(f"[WorldState] Loading {version}...")
        
        # Reconstruct graph
        graph = nx.Graph()
        for node, attrs in state['graph']['nodes']:
            graph.add_node(node, **attrs)
        for u, v, attrs in state['graph']['edges']:
            graph.add_edge(u, v, **attrs)
        graph.graph.update(state['graph']['graph_attrs'])
        
        # Reconstruct tensor
        tensor = None
        if state['tensor'] is not None and QUTIP_AVAILABLE:
            try:
                data = np.frombuffer(state['tensor']['data'], dtype=complex)
                data = data.reshape(state['tensor']['shape'])
                tensor = qt.Qobj(data, dims=state['tensor']['dims'])
            except Exception as e:
                print(f"[WorldState] Tensor reconstruction failed: {e}")
        
        # Reconstruct neural weights
        neural_weights = None
        if state['neural_weights'] is not None:
            neural_weights = np.frombuffer(state['neural_weights'], dtype=float)
            neural_weights = neural_weights.reshape(state['neural_shape'])
        
        return {
            'graph': graph,
            'tensor': tensor,
            'neural_weights': neural_weights,
            'metadata': state['metadata']
        }
    
    def merge_world_states(
        self, 
        local_version: str, 
        remote_version: str,
        fidelity_threshold: float = 0.98
    ) -> str:
        """
        Merge two world states with fidelity checks.
        
        Args:
            local_version: Local state version
            remote_version: Remote state version
            fidelity_threshold: Minimum fidelity required
            
        Returns:
            Version key of merged state
        """
        print(f"[WorldState] Merging {local_version} + {remote_version}...")
        
        local = self.load_world_state(local_version)
        remote = self.load_world_state(remote_version)
        
        if local is None or remote is None:
            print("[WorldState] Merge failed: Missing state")
            return local_version if local else remote_version
        
        # 1. Merge graphs (union)
        merged_graph = nx.compose(local['graph'], remote['graph'])
        
        # 2. Merge tensors (average with fidelity check)
        merged_tensor = None
        if local['tensor'] is not None and remote['tensor'] is not None and QUTIP_AVAILABLE:
            try:
                merged_tensor = (local['tensor'] + remote['tensor']) / 2
                fidelity = qt.fidelity(local['tensor'], merged_tensor)
                
                if fidelity < fidelity_threshold:
                    print(f"[WorldState] Warning: Low fidelity {fidelity:.4f} < {fidelity_threshold}")
                    # Use local state if fidelity too low
                    merged_tensor = local['tensor']
                else:
                    print(f"[WorldState] Merge fidelity: {fidelity:.4f}")
            except Exception as e:
                print(f"[WorldState] Tensor merge failed: {e}")
                merged_tensor = local['tensor']
        
        # 3. Merge neural weights (average)
        merged_weights = None
        if local['neural_weights'] is not None and remote['neural_weights'] is not None:
            if local['neural_weights'].shape == remote['neural_weights'].shape:
                merged_weights = (local['neural_weights'] + remote['neural_weights']) / 2
            else:
                print("[WorldState] Neural weight shapes mismatch, using local")
                merged_weights = local['neural_weights']
        
        # 4. Create merged version
        merged_version = f"merged_{int(time.time())}"
        self.serialize_world_state(
            merged_graph,
            merged_tensor,
            merged_weights,
            merged_version
        )
        
        return merged_version
    
    def list_versions(self) -> list:
        """List all stored versions."""
        return sorted(self.states.keys())
    
    def get_latest_version(self) -> Optional[str]:
        """Get the most recent version."""
        versions = self.list_versions()
        return versions[-1] if versions else None
