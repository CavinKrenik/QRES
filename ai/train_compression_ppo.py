import gymnasium as gym
from gymnasium import spaces
import numpy as np
import os
import sys
import struct
from stable_baselines3 import PPO

# Patch sys.path to find qres
sys.path.append(os.path.join(os.getcwd(), 'python'))

# Try to import qres_rust directly
try:
    from qres import qres_rust
except ImportError:
    # Try direct import from pyd
    try:
        import qres_rust
    except ImportError:
        print("Could not import qres_rust for training environment")
        sys.exit(1)

class CompressionEnv(gym.Env):
    def __init__(self, data_path, chunk_size=1024):
        super(CompressionEnv, self).__init__()
        
        if not os.path.exists(data_path):
             raise FileNotFoundError(f"{data_path} not found")
             
        # Load sample data (limit to 1MB for training speed if file is huge)
        with open(data_path, 'rb') as f:
            self.full_data = f.read(1024 * 1024) 
            
        self.chunk_size = chunk_size
        self.num_chunks = max(1, len(self.full_data) // self.chunk_size)
        self.current_step = 0
        
        # Observation: Simple histogram of byte frequencies (normalized)
        self.observation_space = spaces.Box(low=0, high=1.0, shape=(256,), dtype=np.float32)
        
        # Action: 6 continuous weights for the Mixer [0, 1]
        self.action_space = spaces.Box(low=0, high=1.0, shape=(6,), dtype=np.float32)
        
    def reset(self, seed=None, options=None):
        super().reset(seed=seed)
        self.current_step = 0
        return self._get_obs(), {}
    
    def _get_obs(self):
        start = self.current_step * self.chunk_size
        end = start + self.chunk_size
        chunk = self.full_data[start:end]
        
        # Pad if needed
        if len(chunk) < self.chunk_size:
            chunk = chunk + b'\0' * (self.chunk_size - len(chunk))
            
        # Calc histogram
        # Fast histogram
        counts = np.zeros(256, dtype=np.float32)
        for b in chunk:
            counts[b] += 1.0
            
        return counts / self.chunk_size
        
    def step(self, action):
        start = self.current_step * self.chunk_size
        end = start + self.chunk_size
        chunk = self.full_data[start:end]
        
        if len(chunk) == 0:
            return self._get_obs(), 0, True, False, {}

        # Convert action to bytes (6 floats * 4 bytes = 24 bytes)
        # We need to match the WEIGHTS_LEN expected by Rust
        weights_bytes = b''.join([struct.pack('<f', w) for w in action])
        
        # Run compression with these weights
        try:
           # data, predictor_id, weights
           compressed = qres_rust.encode_bytes(chunk, 0, weights_bytes)
           ratio = len(compressed) / len(chunk)
           
           # Reward: Higher is better. 
           # If ratio < 1.0, we are compressing. Reward improvements.
           # Focus on beating a baseline? 
           # Let's just reward raw compression density.
           # (1 - ratio) -> 0 if no comp, 0.5 if 50% comp, 0.9 if 90% comp.
           # Scale it up
           reward = (1.0 - ratio) * 10
           
           # Penalty for expansion
           if ratio > 1.0:
               reward -= (ratio - 1.0) * 20
               
        except Exception as e:
           print(f"Error in compression: {e}")
           reward = -10.0
           
        self.current_step += 1
        done = self.current_step >= self.num_chunks
        
        next_obs = self._get_obs() if not done else np.zeros(256, dtype=np.float32)
             
        return next_obs, reward, done, False, {}

if __name__ == "__main__":
    # Ensure stable_baselines3 is installed
    try:
        from stable_baselines3 import PPO
    except ImportError:
        print("Please install stable-baselines3: pip install stable-baselines3 shimmy")
        sys.exit(1)

    print("Initializing QRES MetaBrain Trainer...")
    data_file = "iot_telemetry.dat"
    
    if os.path.exists(data_file):
        print(f"Training on {data_file}")
        
        env = CompressionEnv(data_file)
        model = PPO("MlpPolicy", env, verbose=1)
        
        print("Starting training (2000 timesteps)...")
        model.learn(total_timesteps=2000)
        
        output_path = "ai/metabrain_ppo_v2.zip"
        model.save(output_path)
        print(f"✅ Model saved to {output_path}")
    else:
        print("❌ iot_telemetry.dat not found. Skipping training.")
