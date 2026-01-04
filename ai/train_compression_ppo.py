import gymnasium as gym
from gymnasium import spaces
import numpy as np
import os
import sys
import struct
import math
import random
from PIL import Image
try:
    from sentence_transformers import SentenceTransformer
    EMBED_AVAILABLE = True
except ImportError:
    EMBED_AVAILABLE = False
    
from snn_predictor import SNNPredictor  # Breakthrough 1: SNN Integration
from stable_baselines3 import PPO
from stable_baselines3.common.vec_env import DummyVecEnv
from stable_baselines3.common.callbacks import CheckpointCallback

# Patch sys.path to find qres
sys.path.append(os.path.join(os.getcwd(), 'python'))

# Try to import qres_rust directly
try:
    from qres import qres_rust
except ImportError:
    try:
        import qres_rust
    except ImportError:
        print("Could not import qres_rust for training environment")
        sys.exit(1)

def generate_synthetic_data(size=1024, mode='random'):
    if mode == 'text':
        # Simulated text (ASCII range)
        return bytes([random.randint(32, 126) for _ in range(size)])
    elif mode == 'sine':
        # Structured sine wave
        x = np.linspace(0, 100, size)
        y = (np.sin(x) * 127 + 128).astype(np.uint8)
        return y.tobytes()
    elif mode == 'constant':
        # Low entropy
        return bytes([random.randint(0, 255)] * size)
    else:
        # Random high entropy
        return os.urandom(size)

def load_real_chunks(data_dir='data/', chunk_size=1024):
    all_chunks = []
    print(f"Loading real training data from {data_dir}...")
    
    for root, _, files in os.walk(data_dir):
        for f in files:
            full_path = os.path.join(root, f)
            try:
                # Handle Images via raw bytes (Simulating pure compression first)
                if f.lower().endswith(('.jpg', '.png', '.jpeg')):
                    with Image.open(full_path) as img:
                        # Convert to RGB to standardize
                        img = img.convert('RGB')
                        # Get raw bytes
                        raw = img.tobytes()
                        # Chunk it
                        for i in range(0, len(raw), chunk_size):
                            all_chunks.append(raw[i:i+chunk_size])
                            
                # Handle Text/Binary/Other
                else:
                    with open(full_path, 'rb') as f_obj:
                        raw = f_obj.read()
                        
                        # Special handling for "other" edge cases (PDF/WAV/GZ logic)
                        if f.lower().endswith('.pdf') or f.lower().endswith('.wav') or f.lower().endswith('.gz'):
                            # Treat as opaque bytes but ensure they get into the mix
                            # Potentially add metadata tag in future
                            pass
                            
                        for i in range(0, len(raw), chunk_size):
                            all_chunks.append(raw[i:i+chunk_size])
            except Exception as e:
                pass # Skip bad files
                
    print(f"Loaded {len(all_chunks)} chunks from real files.")
    return all_chunks

class CompressionEnv(gym.Env):
    def __init__(self, data_path=None, data_dir='data/', chunk_size=1024):
        super(CompressionEnv, self).__init__()
        
        self.chunk_size = chunk_size
        self.data_buffer = []

        # 1. Load Real Data from data/ folder
        real_chunks = load_real_chunks(data_dir, chunk_size)
        self.data_buffer.extend(real_chunks)

        # 2. Load specific large file if needed (legacy support)
        if data_path and os.path.exists(data_path):
            with open(data_path, 'rb') as f:
                # Read up to 5MB 
                real_data = f.read(5 * 1024 * 1024)
                for i in range(0, len(real_data), chunk_size):
                    self.data_buffer.append(real_data[i:i+chunk_size])
        
        # 3. Fill remaining with synthetic if needed
        # We want a mix, so let's guarantee at least 500 synthetic chunks too
        for _ in range(500):
            modes = ['text', 'sine', 'constant', 'random']
            mode = random.choice(modes)
            self.data_buffer.append(generate_synthetic_data(chunk_size, mode))
            
        random.shuffle(self.data_buffer)
            
        random.shuffle(self.data_buffer)
        self.num_chunks = len(self.data_buffer)
        self.current_step = 0
        
        # Observation: Normalized Byte Histogram (256 floats) + Entropy (1 float)
        self.observation_space = spaces.Box(low=0, high=1.0, shape=(257,), dtype=np.float32)
        
        # Action: 6 continuous weights for the Mixer [0, 1]
        self.action_space = spaces.Box(low=0, high=1.0, shape=(6,), dtype=np.float32)
        
        # SNN for sparsity reward (simulated integration)
        self.snn = SNNPredictor() 

        
    def reset(self, seed=None, options=None):
        super().reset(seed=seed)
        # Shuffle buffer on reset for randomness
        random.shuffle(self.data_buffer)
        self.current_step = 0
        return self._get_obs(self.data_buffer[0]), {}
    
    def _calculate_entropy(self, data):
        if not data:
            return 0.0
        counts = np.bincount(np.frombuffer(data, dtype=np.uint8), minlength=256)
        probs = counts[counts > 0] / len(data)
        return -np.sum(probs * np.log2(probs))

    def _get_obs(self, chunk):
        # Pad if needed
        if len(chunk) < self.chunk_size:
            chunk = chunk + b'\0' * (self.chunk_size - len(chunk))
            
        # Histogram
        arr = np.frombuffer(chunk, dtype=np.uint8)
        counts = np.bincount(arr, minlength=256).astype(np.float32)
        norm_hist = counts / self.chunk_size
        
        # Entropy (normalized 0-1, assuming max entropy is 8 bits)
        entropy = self._calculate_entropy(chunk) / 8.0
        
        return np.concatenate([norm_hist, [entropy]])
        
    def step(self, action):
        if self.current_step >= len(self.data_buffer):
            # End of epoch
            return np.zeros(257, dtype=np.float32), 0, True, False, {}

        chunk = self.data_buffer[self.current_step]
        
        # Convert action to bytes
        weights_bytes = b''.join([struct.pack('<f', w) for w in action])
        
        reward = 0.0
        try:
           # data, predictor_id, weights
           # Use qres_rust encode
           compressed = qres_rust.encode_bytes(chunk, 0, weights_bytes)
           ratio = len(compressed) / len(chunk)
           
           # Reward Function Optimization
           # Base reward: Compression efficiency
           # (1 - ratio) gives raw savings. 
           # Scaling: 20x to make it significant vs noise
           reward = (1.0 - ratio) * 20.0
           
           # Bonus for "Great" compression (<0.5)
           if ratio < 0.5:
               reward += 5.0
               
           # Penalty for expansion
           if ratio > 1.05:
               reward -= 10.0
               
           # --- Breakthrough 1: SNN Sparsity Reward ---
           # Reward confident/sparse SNN predictions
           if len(chunk) > 10:
              probs = self.snn.predict_next(chunk[:10]) 
              if np.max(probs) > 0.5:
                  reward += 1.0
        except Exception as e:
           print(f"Error in compression: {e}")
           reward = -10.0
           
        self.current_step += 1
        done = self.current_step >= self.num_chunks
        
        if not done:
            next_obs = self._get_obs(self.data_buffer[self.current_step])
        else:
            next_obs = np.zeros(257, dtype=np.float32)
             
        return next_obs, reward, done, False, {}

def make_env():
    # Helper for VecEnv
    return CompressionEnv(data_path="iot_telemetry.dat", data_dir="data/")

if __name__ == "__main__":
    print("="*60)
    print("QRES MetaBrain Trainer - Advanced PPO")
    print("="*60)

    # 1. Vectorized Environment (Batch Size boost)
    # Using DummyVecEnv for Windows compatibility (safer than Subproc)
    num_envs = 4
    env = DummyVecEnv([make_env for _ in range(num_envs)])
    
    # 2. Hyperparameters (Tuned)
    model = PPO(
        "MlpPolicy", 
        env, 
        learning_rate=3e-4,
        batch_size=128,      # Larger batch for stability
        n_epochs=10, 
        gamma=0.99,
        gae_lambda=0.95,
        clip_range=0.2,
        ent_coef=0.01,       # Encourage exploration
        verbose=1
    )
    
    print(f"Starting training on {num_envs} vectorized environments...")
    print("Target: 20,000 timesteps")
    
    # Checkpoint every 5k steps
    checkpoint_callback = CheckpointCallback(save_freq=5000, save_path='./ai/logs/', name_prefix='metabrain_v4')
    
    # Load previous model if available to continue learning
    if os.path.exists("ai/metabrain_ppo_v3.zip"):
         print("Resuming from v3 model...")
         model = PPO.load("ai/metabrain_ppo_v3.zip", env=env)
         model.set_parameters("ai/metabrain_ppo_v3.zip")
    
    model.learn(total_timesteps=20000, callback=checkpoint_callback)
    
    output_path = "ai/metabrain_ppo_v4.zip"
    model.save(output_path)
    print(f"\n✅ Training Complete. Model saved to {output_path}")
