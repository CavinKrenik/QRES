import gymnasium as gym
from gymnasium import spaces
import numpy as np
import struct
import qres
import random

# --- Data Generators (Simplified from train_transformer.py) ---
def gen_sine(size):
    x = np.linspace(0, 100 * np.pi, size)
    y = np.sin(x) * 100 + 128
    return y.astype(np.uint8).tobytes()

def gen_text(size):
    chars = np.random.choice(list(range(32, 127)) + [10, 13, 32] * 10, size=size)
    return chars.astype(np.uint8).tobytes()

def gen_noise(size):
    return np.random.randint(0, 256, size, dtype=np.uint8).tobytes()

def gen_linear(size):
    return np.arange(size, dtype=np.uint8).tobytes()

class CompressionMixingEnv(gym.Env):
    """
    QRES v7.0 RL Environment for Adaptive Compression Mixing.
    """
    metadata = {"render_modes": ["human"]}

    def __init__(self, chunk_size=4096):
        super().__init__()
        self.chunk_size = chunk_size
        
        # Action space: 4 continuous values [0, 1] representing weights
        self.action_space = spaces.Box(low=0, high=1, shape=(4,), dtype=np.float32)
        
        # Observation space: 5 features
        self.observation_space = spaces.Box(low=-np.inf, high=np.inf, shape=(5,), dtype=np.float32)
        
        self.current_chunk = None
        self.last_ratio = 1.0
        
        # Generators: Include Linear for easy wins
        self.generators = [gen_sine, gen_text, gen_noise, gen_linear]

    def _get_obs(self, chunk):
        # Calculate simple features
        data = np.frombuffer(chunk, dtype=np.uint8)
        probs = np.bincount(data, minlength=256) / len(data)
        entropy = -np.sum(probs * np.log2(probs + 1e-9))
        mean = np.mean(data) / 255.0
        var = np.var(data) / (255.0**2)
        
        # Simple autocorrelation (lag 1)
        if len(data) > 1:
            ac = np.corrcoef(data[:-1], data[1:])[0, 1]
            if np.isnan(ac): ac = 0
        else:
            ac = 0
            
        return np.array([entropy, mean, var, ac, self.last_ratio], dtype=np.float32)

    def reset(self, seed=None, options=None):
        super().reset(seed=seed)
        
        # Generate new chunk
        gen = random.choice(self.generators)
        self.current_chunk = gen(self.chunk_size)
        self.last_ratio = 1.0 # Reset baseline
        
        return self._get_obs(self.current_chunk), {}

    def step(self, action):
        # 1. Normalize Action
        # Clip to safe range [0.01, 10.0] for Mixers
        weights = np.clip(action, 0.01, 10.0)
        
        # 2. Construct Weight Vector (24 Bytes)
        # We need 6 weights: [Linear, Simple, Graph, Spectral, LZ, Transformer]
        # The Action space provides 4 weights. We append 2 placeholders.
        # This matches the WEIGHTS_LEN = 24 in Rust (6 floats * 4 bytes).
        full_action = np.concatenate([weights, [0.1, 0.1]]).astype(np.float32)
        
        weight_bytes = struct.pack('6f', *full_action)
        
        # 3. Run QRES Compression
        try:
            # We pass predictor_id=0, but weights trigger the Custom/Neural path
            compressed = qres.encode_bytes(self.current_chunk, 0, weight_bytes)
            
            # 4. Calculate Reward
            # If compression fails (returns larger than original), QRES returns zstd fallback.
            # We want to reward the RAW predictive performance.
            
            ratio = len(compressed) / len(self.current_chunk)
            
            # Reward:
            # Baseline is 1.0 (uncompressed). 
            # We want ratio < 1.0. 
            # New Reward: (1.0 - Ratio) * 100. Higher compression = Higher reward.
            # This gives absolute performance feedback rather than delta-based.
            reward = (1.0 - ratio) * 100.0
            
            self.last_ratio = ratio
            
            # Terminated: True (Contextual Bandit Mode)
            # Since Rust backend is currently stateless per call, we cannot benefit 
            # from multi-step episodes yet.
            terminated = True
            truncated = False
            
            info = {"ratio": ratio, "size": len(compressed)}
            
            return self._get_obs(self.current_chunk), reward, terminated, truncated, info
            
        except Exception as e:
            print(f"Compression failed: {e}")
            # Heavy penalty for crash
            return self._get_obs(self.current_chunk), -100.0, True, False, {}

if __name__ == "__main__":
    print("Testing CompressionMixingEnv...")
    env = CompressionMixingEnv()
    obs, _ = env.reset()
    print("Obs:", obs)
    
    action = env.action_space.sample()
    print("Action:", action)
    
    obs, reward, done, _, info = env.step(action)
    print(f"Step Result: Reward={reward:.4f}, Ratio={info['ratio']:.4f}")
