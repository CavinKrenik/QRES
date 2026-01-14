import torch
import torch.nn as nn
import torch.optim as optim
import numpy as np
import struct
import math

# --- Config ---
BATCH_SIZE = 64
EPOCHS = 5000
HIDDEN_SIZE = 8
CONTEXT_SIZE = 3

# --- 1. Dataset Generation ---
def generate_data(size=10000):
   """
   Generates a mix of Sine Wave (predictable) and Text-like patterns.
   """
   data = []
   labels = []
   
   # Sine Wave
   x = np.linspace(0, 100 * np.pi, size // 2)
   sine = (np.sin(x) * 100 + 128).astype(np.uint8)
   
   for i in range(CONTEXT_SIZE, len(sine)):
       ctx = sine[i-CONTEXT_SIZE:i] / 255.0
       target = sine[i] / 255.0
       data.append(ctx)
       labels.append(target)
       
   # Synthetic Text (Repeating patterns)
   text = b"The quick brown fox jumps over the lazy dog. " * (size // 100)
   text_bytes = np.frombuffer(text, dtype=np.uint8)
   
   for i in range(CONTEXT_SIZE, len(text_bytes)):
       ctx = text_bytes[i-CONTEXT_SIZE:i] / 255.0
       target = text_bytes[i] / 255.0
       data.append(ctx)
       labels.append(target)
       
   return np.array(data, dtype=np.float32), np.array(labels, dtype=np.float32).reshape(-1, 1)

# --- 2. Tiny Brain Model ---
class TinyBrain(nn.Module):
    def __init__(self):
        super().__init__()
        self.fc1 = nn.Linear(CONTEXT_SIZE, HIDDEN_SIZE)
        self.relu = nn.ReLU()
        self.fc2 = nn.Linear(HIDDEN_SIZE, 1) # Output is 0-1 float
        
    def forward(self, x):
        x = self.fc1(x)
        x = self.relu(x)
        x = self.fc2(x)
        return x

# --- 3. Training Loop ---
def train():
    print("🧠 Generating Training Data...")
    X, y = generate_data()
    X = torch.tensor(X)
    y = torch.tensor(y)
    
    model = TinyBrain()
    optimizer = optim.Adam(model.parameters(), lr=0.01)
    criterion = nn.MSELoss()
    
    print(f"🧠 Training TinyBrain ({EPOCHS} Epochs)...")
    for epoch in range(EPOCHS):
        optimizer.zero_grad()
        output = model(X)
        loss = criterion(output, y)
        loss.backward()
        optimizer.step()
        
        if epoch % 1000 == 0:
            print(f"Epoch {epoch}: Loss = {loss.item():.6f}")
            
    print("✅ Training Complete.")
    return model

# --- 4. Export to Binary (.qnn) ---
def export_brain(model, filename="brain.qnn"):
    print(f"💾 Exporting to {filename}...")
    
    weights = []
    
    # Layer 1 Weights (3 -> 8)
    # Transpose to match Rust's expected linear memory layout if needed?
    # PyTorch Linear weight is (out_features, in_features).
    # Rust manual logic usually does: input (1x3) * W (3x8). 
    # So we need W to be (3, 8). PyTorch stores (8, 3). So we transpose.
    w1 = model.fc1.weight.detach().numpy().T 
    b1 = model.fc1.bias.detach().numpy()
    
    # Layer 2 Weights (8 -> 1)
    w2 = model.fc2.weight.detach().numpy().T
    b2 = model.fc2.bias.detach().numpy()
    
    with open(filename, "wb") as f:
        # Header (Magic + Version)
        f.write(b"QNN1") 
        
        # W1 (3x8 floats)
        f.write(w1.tobytes())
        # b1 (8 floats)
        f.write(b1.tobytes())
        
        # W2 (8x1 floats)
        f.write(w2.tobytes())
        # b2 (1 float)
        f.write(b2.tobytes())
        
    print(f"✅ Exported {filename} ({w1.nbytes + b1.nbytes + w2.nbytes + b2.nbytes} bytes of weights)")

if __name__ == "__main__":
    brain = train()
    export_brain(brain)
