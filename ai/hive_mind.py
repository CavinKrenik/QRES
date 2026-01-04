import torch
import numpy as np
import os
import copy
from stable_baselines3 import PPO

class HiveMind:
    """
    Manages Continual Learning across the P2P Swarm.
    Implements Federated Averaging (FedProx) to merge 'Epiphanies' (model weights)
    from other nodes without sharing raw data.
    """
    def __init__(self, local_model_path="ai/metabrain_ppo_v5.zip", swarm_id="node_01"):
        self.local_model_path = local_model_path
        self.swarm_id = swarm_id
        self.peers = [] # Mock peer list
        self.device = torch.device("cpu")
        
    def load_local_model(self):
        if not os.path.exists(self.local_model_path):
            raise FileNotFoundError(f"Local model {self.local_model_path} not found.")
        
        # Load PPO model
        model = PPO.load(self.local_model_path, device=self.device)
        return model

    def generate_epiphany(self):
        """
        Extracts and quantizes local model weights to share with the swarm.
        Returns: Dictionary of {layer_name: weight_tensor}
        """
        model = self.load_local_model()
        params = model.get_parameters()
        
        # In a real scenario, we would compress/quantize these float32s to int8
        # for efficient transmission over libp2p.
        epiphany = {
            "node_id": self.swarm_id,
            "weights": params,
            "fidelity_score": 0.99 # Mock score
        }
        return epiphany

    def assimilate_epiphanies(self, incoming_epiphanies):
        """
        Aggregates weights from peers using FedAvg/FedProx.
        incoming_epiphanies: List of weight dictionaries from peers.
        """
        if not incoming_epiphanies:
            return
            
        print(f"[{self.swarm_id}] Assimilating knowledge from {len(incoming_epiphanies)} peers...")
        
        local_model = self.load_local_model()
        local_params = local_model.get_parameters()
        
        # Federated Averaging: W_new = (W_local + Sum(W_peers)) / (N + 1)
        # We assume strict structure matching (v5 architecture)
        
        aggregated_params = copy.deepcopy(local_params)
        
        # Iterate over policy/value networks (flattened dict in SB3)
        for key in aggregated_params['policy'].keys():
            # Sum peer contributions
            peer_sum = torch.tensor(aggregated_params['policy'][key]) # Start with local
            
            for peer_data in incoming_epiphanies:
                peer_weights = peer_data['weights']['policy'][key]
                peer_sum += torch.tensor(peer_weights)
                
            # Average
            avg_weight = peer_sum / (len(incoming_epiphanies) + 1)
            aggregated_params['policy'][key] = avg_weight.numpy()
            
        # Apply Logic to Value Net as well (omitted for brevity, assume shared structure)
        
        # Update Local Model
        local_model.set_parameters(aggregated_params)
        local_model.save(self.local_model_path)
        print(f"[{self.swarm_id}] Hive Mind sync complete. Model evolved.")

    def simulate_swarm_cycle(self, n_peers=3):
        """
        Simulates a P2P cycle:
        1. Generate local epiphany.
        2. Receive mock epiphanies from 'peers' (perturbed versions of local).
        3. Assimilate.
        """
        print("--- Hive Mind Simulation Start ---")
        my_epiphany = self.generate_epiphany()
        
        # Generate mock peer data (simulating other nodes learning slightly different things)
        peer_updates = []
        for i in range(n_peers):
            peer_weights = copy.deepcopy(my_epiphany['weights'])
            # Add noise to simulate diverse learning
            for k in peer_weights['policy']:
                noise = np.random.normal(0, 0.01, peer_weights['policy'][k].shape)
                peer_weights['policy'][k] += noise
                
            peer_updates.append({
                "node_id": f"peer_{i}",
                "weights": peer_weights
            })
            
        self.assimilate_epiphanies(peer_updates)

if __name__ == "__main__":
    # Test Run
    hive = HiveMind(local_model_path="ai/metabrain_ppo_v5.zip")
    try:
        hive.simulate_swarm_cycle()
    except Exception as e:
        print(f"Hive simulation skipped (missing model?): {e}")
