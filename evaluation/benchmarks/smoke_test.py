import torch
import networkx as nx
import gymnasium as gym
import qutip
import sentence_transformers
import open_clip

print("Smoke Test: Foundations v7.0")
print(f"PyTorch version: {torch.__version__}")
print(f"CUDA Available: {torch.cuda.is_available()}")
print(f"NetworkX version: {nx.__version__}")
print(f"Gymnasium version: {gym.__version__}")
print(f"QuTiP version: {qutip.__version__}")
print(f"Sentence-Transformers: Loaded")
print(f"OpenCLIP: Loaded")

# Simple Graph Test
G = nx.Graph()
G.add_edge(1, 2)
print("Graph edge added.")

# Simple RL Env Test
env = gym.make("CartPole-v1", render_mode=None)
obs, info = env.reset()
print(f"Gym Env Reset: {obs}")

# Simple Quantum Test
psi = qutip.basis(2, 0)
print(f"QuTiP State: {psi}")
