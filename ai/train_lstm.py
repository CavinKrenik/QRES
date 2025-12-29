import torch
import torch.nn as nn
import torch.optim as optim
import numpy as np
import struct
import math

# --- Config ---
EPOCHS = 2000
HIDDEN_SIZE = 8
INPUT_SIZE = 1
SEQ_LEN = 10 

# --- 1. Dataset Generation ---
def generate_complex_wave(size=10000):
   """
   Generates a complex modulated wave (Sine * Cosine) + Noise.
   Hard for linear predictors, learnable for LSTM.
   """
   t = np.linspace(0, 100 * np.pi, size)
   # Modulated wave: sin(t) * cos(t/3)
   wave = np.sin(t) * np.cos(t / 3.0) 
   # Normalize to 0-255
   wave = ((wave + 1.0) / 2.0 * 255.0).astype(np.uint8)
   
   data = []
   labels = []
   
   # Prepare sequence data
   # Predict x[t] given x[t-1] (Stateful)
   # Actually, for training stability, we train on short sequences.
   # For inference, QRES feeds 1 byte at a time.
   
   normalized = wave.astype(np.float32) / 255.0
   
   # Input: Current Byte. Target: Next Byte.
   # LSTM maintains state implicitly.
   # For training, we provide sequences [t, t+1... t+n] -> [t+1, ... t+n+1]
   
   X = []
   Y = []
   
   for i in range(len(normalized) - SEQ_LEN - 1):
       x_seq = normalized[i : i+SEQ_LEN]
       y_seq = normalized[i+1 : i+SEQ_LEN+1]
       X.append(x_seq.reshape(-1, 1))
       Y.append(y_seq.reshape(-1, 1))
       
   return np.array(X), np.array(Y)

# --- 2. MicroLSTM Model ---
class MicroLSTM(nn.Module):
    def __init__(self):
        super().__init__()
        # Input: 1, Hidden: 8
        self.lstm = nn.LSTM(input_size=INPUT_SIZE, hidden_size=HIDDEN_SIZE, batch_first=True)
        # Project hidden (8) to output (1)
        self.fc = nn.Linear(HIDDEN_SIZE, 1)
        
    def forward(self, x, hidden=None):
        out, (h, c) = self.lstm(x, hidden)
        out = self.fc(out)
        return out, (h, c)

# --- 3. Training Loop ---
def train():
    print("🧠 Generating Complex Wave Data...")
    X, y = generate_complex_wave()
    X = torch.tensor(X, dtype=torch.float32)
    y = torch.tensor(y, dtype=torch.float32)
    
    model = MicroLSTM()
    optimizer = optim.Adam(model.parameters(), lr=0.01)
    criterion = nn.MSELoss()
    
    print(f"🧠 Training MicroLSTM ({EPOCHS} Epochs)...")
    for epoch in range(EPOCHS):
        optimizer.zero_grad()
        output, _ = model(X)
        loss = criterion(output, y)
        loss.backward()
        optimizer.step()
        
        if epoch % 500 == 0:
            print(f"Epoch {epoch}: Loss = {loss.item():.6f}")
            
    print("✅ Training Complete.")
    return model

# --- 4. Export to Binary (.qnn) ---
def export_brain(model, filename="lstm.qnn"):
    print(f"💾 Exporting to {filename}...")
    
    # Extract Weights
    # PyTorch LSTM weights are packed:
    # weight_ih_l0: (4*hidden_size, input_size) -> (32, 1)
    # weight_hh_l0: (4*hidden_size, hidden_size) -> (32, 8)
    # bias_ih_l0:   (32)
    # bias_hh_l0:   (32)
    
    # Gates order in PyTorch: Input, Forget, Cell, Output (IFCO)
    # Rust implementation needs to match this.
    
    w_ih = model.lstm.weight_ih_l0.detach().numpy().T # Transpose for consistency? 
    # Let's keep PyTorch layout (Rows=Gates) for easier loading if we read sequentially.
    # Actually, Rust usually does `Row * Vec`.
    # PyTorch does `x @ W_ih.T` internally or `W_ih @ x`.
    # PyTorch `Linear` is `x @ W.T + b`. `LSTM` is similar.
    # Let's write them exactly as PyTorch stores them, and handle the math in Rust to match.
    # PyTorch storage: [32, 1] and [32, 8].
    
    # Using .detach().numpy() (No Transpose)
    w_ih = model.lstm.weight_ih_l0.detach().numpy() # [32, 1]
    w_hh = model.lstm.weight_hh_l0.detach().numpy() # [32, 8]
    b_ih = model.lstm.bias_ih_l0.detach().numpy()   # [32]
    b_hh = model.lstm.bias_hh_l0.detach().numpy()   # [32]
    
    # Linear Layer
    w_fc = model.fc.weight.detach().numpy()         # [1, 8]
    b_fc = model.fc.bias.detach().numpy()           # [1]
    
    with open(filename, "wb") as f:
        # Header (Magic + Version)
        f.write(b"LSTM") 
        
        # LSTM Weights
        f.write(w_ih.tobytes())
        f.write(w_hh.tobytes())
        f.write(b_ih.tobytes())
        f.write(b_hh.tobytes())
        
        # FC Weights
        f.write(w_fc.tobytes())
        f.write(b_fc.tobytes())
        
    calc_size = w_ih.nbytes + w_hh.nbytes + b_ih.nbytes + b_hh.nbytes + w_fc.nbytes + b_fc.nbytes
    print(f"✅ Exported {filename} ({calc_size} bytes of weights)")

if __name__ == "__main__":
    brain = train()
    export_brain(brain)
