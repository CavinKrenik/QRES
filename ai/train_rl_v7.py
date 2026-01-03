import os
import sys
import numpy as np
from stable_baselines3 import PPO
from stable_baselines3.common.callbacks import BaseCallback
from stable_baselines3.common.env_util import make_vec_env

# Ensure qres available
import qres

# Import the environment
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
from rl_mixer_env import CompressionMixingEnv

class TensorboardCallback(BaseCallback):
    """
    Custom callback for plotting additional values in tensorboard.
    """
    def __init__(self, verbose=0):
        super().__init__(verbose)
        self.avg_ratio = 0
        self.window = []

    def _on_step(self) -> bool:
        # Access info from the environment
        infos = self.locals.get("infos", [{}])
        for info in infos:
            if "ratio" in info:
                self.window.append(info["ratio"])
                if len(self.window) > 100:
                    self.window.pop(0)
        
        if self.window:
            self.logger.record("custom/compression_ratio", np.mean(self.window))
            
        return True

def train_agent():
    print("="*60)
    print("🤖 QRES v7.0 RL Training: Adaptive Mixer Agent")
    print("="*60)

    # 1. Create Environment
    # We use vector environment for stability, though single env is fine for this lightweight task
    env = make_vec_env(lambda: CompressionMixingEnv(), n_envs=4)
    print("✅ Environment created (4 parallel envs)")

    # 2. Define Model (PPO is a safe default for continuous action spaces)
    model = PPO(
        "MlpPolicy", 
        env, 
        verbose=1, 
        tensorboard_log="./ai/rl_logs/",
        learning_rate=3e-4,
        n_steps=2048,
        batch_size=64,
        n_epochs=10,
        gamma=0.99,
        gae_lambda=0.95,
        clip_range=0.2,
        ent_coef=0.01 # Encourages exploration
    )
    
    print("🧠 Model initialized (PPO-MlpPolicy)")

    # 3. Train
    total_timesteps = 25000 # Short run for Phase 1 proof
    print(f"🏋️ Starting training for {total_timesteps} timesteps...")
    
    model.learn(total_timesteps=total_timesteps, callback=TensorboardCallback())
    
    print("\n✅ Training complete.")

    # 4. Save
    save_path = "ai/ppo_mixer_v7"
    model.save(save_path)
    print(f"💾 Model saved to {save_path}.zip")
    
    # 5. Plot
    try:
        import matplotlib.pyplot as plt
        plt.figure(figsize=(10, 6))
        # Use the callback window we collected (requires modifying callback to be accessible)
        # Actually, let's just return the callback
        return model, model.env 
    except ImportError:
        pass
        
    return model

def plot_results(log_dir):
    try:
        import matplotlib.pyplot as plt
        import pandas as pd
        # Parse tensorboard logs if possible, or just skip for now as we don't have a CSV capability easily without TB parsing lib.
        # Alternatively, we can use the monitor csv from SB3 if we wrapped the env.
        print("skipped plotting (logs in tensorboard)")
    except:
        pass

def test_agent(model_path="ai/ppo_mixer_v7"):
    print("\n🧪 Testing Trained Agent...")
    env = CompressionMixingEnv()
    model = PPO.load(model_path)
    
    obs, _ = env.reset()
    total_reward = 0
    steps = 100
    
    # Track statistics
    ratios = []
    
    print(f"{'Step':<5} | {'Action (Lin, Sim, Grp, Spc)':<30} | {'Ratio':<6} | {'Chunk Type'}")
    print("-" * 65)
    
    for i in range(steps):
        action, _ = model.predict(obs, deterministic=True)
        obs, reward, done, _, info = env.step(action)
        total_reward += reward
        ratios.append(info['ratio'])
        
        if done:
            obs, _ = env.reset()
        
        if i % 10 == 0:
            act_str = f"[{action[0]:.2f}, {action[1]:.2f}, {action[2]:.2f}, {action[3]:.2f}]"
            # Simple heuristic to identify chunk
            entropy = obs[0]
            ctype = "Unknown"
            if entropy < 1.0: ctype = "Linear/Sine"
            elif entropy > 7.0: ctype = "Noise"
            else: ctype = "Text"
            
            print(f"{i:<5} | {act_str:<30} | {info['ratio']:.2%} | {ctype}")
            
    avg_reward = total_reward / steps
    avg_ratio = sum(ratios) / len(ratios)
    print(f"\nAverage Reward: {avg_reward:.4f}")
    print(f"Average Compression Ratio: {avg_ratio:.2%}")
    
    # Generate Hero Image for XAI
    try:
        import matplotlib.pyplot as plt
        plt.figure(figsize=(10, 6))
        plt.plot(ratios, label='Comp Ratio')
        plt.title('Agent Performance on Mixed Stream')
        plt.xlabel('Step')
        plt.ylabel('Ratio')
        plt.legend()
        plt.savefig('benchmarks/results/rl_agent_performance.png')
        print("✅ Saved performance plot to benchmarks/results/rl_agent_performance.png")
    except Exception as e:
        print(f"Failed to plot: {e}")

if __name__ == "__main__":
    train_agent()
    test_agent()
