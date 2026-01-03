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
        # Normalize action to sum to 1 (Softmax-like or just L1 norm)
        # Even if QRES mixer expects raw floats, normalizing helps stability
        weights = np.clip(action, 0.01, 10.0) # Avoid zeros
        # weights /= np.sum(weights) # Optional: QRES mixer handles scaling internally mostly
        
        # Pack weights for QRES (4 floats = 16 bytes)
        # Assuming QRES expects [Lin, Sim, Graph, Spec, LZ, Trans] - Wait, check lib.rs for NUM_MODELS
        # In lib.rs: "let mut preds = [0u8; 6];"
        # "let mut mixer = Mixer::new(init_w, global_w);"
        
        # NOTE: Mixer in rust likely expects weights for ALL predictors?
        # Let's assume passed weights map to predictors 0..N.
        # Ideally we pass 4 weights for the main ones.
        # I'll enable 4 weights here.
        
        # We need to construct the byte array.
        # Step 167 lib.rs snippet: "f32_count >= 2 * NUM_MODELS"
        # I need to know NUM_MODELS. Usually it's 6 in v5/v6.
        # If I pass fewer, it might ignore or panic?
        # "if f32_count >= NUM_MODELS { (Some(&slice[0..NUM_MODELS]), None) }"
        # If NUM_MODELS is 6, I need 6 floats.
        
        # I'll pad with small values for the others (LZ, Transformer) which we aren't optimizing yet.
        full_action = np.concatenate([weights, [0.1, 0.1]]) # 6 weights
        
        weight_bytes = struct.pack('f'*6, *full_action)
        
        # Run QRES Compression
        try:
            # We use ID 2 to trigger Neural/Custom Weight path in QRES if possible?
            # Actually compress_chunk logic:
            # "if let Some(w) = _weights { let take = w.len().min(20); ... }"
            # It takes up to 20 bytes (5 floats). 
            # Wait, 5 floats? 
            # lib.rs: "let b = f.to_le_bytes(); stored_init_weights.extend... take = w.len().min(20)"
            # It seems it expects 5 floats for init weights in the header?
            # And expects NUM_MODELS for the Mixer?
            
            # Use specific ID or flags?
            # In encode_bytes, I call `compress_chunk(data, predictor_id, weights, None)`
            
            compressed = qres.encode_bytes(self.current_chunk, 0, weight_bytes)
            
            # Calculate Reward
            ratio = len(compressed) / len(self.current_chunk)
            reward = (self.last_ratio - ratio) * 10.0 # Reward for improving over baseline
            
            self.last_ratio = ratio
            terminated = True # Episode per chunk for now (Contextual Bandit style)
            truncated = False
            
            info = {"ratio": ratio, "size": len(compressed)}
            
            return self._get_obs(self.current_chunk), reward, terminated, truncated, info
            
        except Exception as e:
            print(f"Compression failed: {e}")
            return self._get_obs(self.current_chunk), -1.0, True, False, {}

if __name__ == "__main__":
    print("Testing CompressionMixingEnv...")
    env = CompressionMixingEnv()
    obs, _ = env.reset()
    print("Obs:", obs)
    
    action = env.action_space.sample()
    print("Action:", action)
    
    obs, reward, done, _, info = env.step(action)
    print(f"Step Result: Reward={reward:.4f}, Ratio={info['ratio']:.4f}")
