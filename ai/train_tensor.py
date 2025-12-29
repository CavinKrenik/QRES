import torch
import torch.nn as nn
import torch.optim as optim
import numpy as np
import struct

# --- Config ---
EPOCHS = 2000
BOND_DIM = 4
SEQ_LEN = 10 

# --- 1. Dataset Generation (Same as LSTM) ---
def generate_complex_wave(size=10000):
   t = np.linspace(0, 100 * np.pi, size)
   wave = np.sin(t) * np.cos(t / 3.0) 
   wave = ((wave + 1.0) / 2.0 * 255.0).astype(np.uint8)
   
   normalized = wave.astype(np.float32) / 255.0
   
   X = []
   Y = []
   for i in range(len(normalized) - SEQ_LEN - 1):
       x_seq = normalized[i : i+SEQ_LEN]
       y_seq = normalized[i+1 : i+SEQ_LEN+1]
       X.append(x_seq.reshape(-1, 1))
       Y.append(y_seq.reshape(-1, 1))
       
   return np.array(X), np.array(Y)

# --- 2. Tensor Network Model (Recurrent MPS) ---
class RecurrentMPS(nn.Module):
    def __init__(self):
        super().__init__()
        # State: psi [1, D]
        # Matrices: A0 [D, D], A1 [D, D]
        # Readout: C [D, 1]
        
        self.D = BOND_DIM
        
        # We model: psi_t = (A0 + x*A1) @ psi_{t-1}
        # But PyTorch Linear is x@A.T.
        # Let's define weights directly as parameters to control layout explicitly.
        
        self.A0 = nn.Parameter(torch.randn(self.D, self.D) * 0.1)
        self.A1 = nn.Parameter(torch.randn(self.D, self.D) * 0.1)
        
        self.C = nn.Parameter(torch.randn(self.D, 1) * 0.1)
        self.bias = nn.Parameter(torch.zeros(1))
        
    def forward(self, x, psi=None):
        # x: [batch, seq_len, 1]
        batch_size = x.size(0)
        seq_len = x.size(1)
        
        if psi is None:
            psi = torch.zeros(batch_size, self.D) # Start state 0
            # Or learned start state, but 0 is safe if we have bias
            # Actually, let's learn a start state or just fix it to [1,0,0,0]
            psi[:, 0] = 1.0 
            
        outputs = []
        
        for t in range(seq_len):
            xt = x[:, t, 0] # [batch]
            
            # Transition: A_eff = A0 + xt * A1
            # Batch optimized approach:
            # A0: [D, D]
            # A1: [D, D]
            # xt: [B]
            # A_eff: [B, D, D]
            
            # psi: [B, D]
            
            # psi_new = psi @ A_eff.T ? 
            # Let's keep math simple: psi_new = A_eff @ psi 
            # (row vector convention? psi_new = psi @ A_eff)
            
            # Let's use psi as row vector [1, D].
            # psi_new = psi @ (A0 + x*A1)
            
            A_eff = self.A0.unsqueeze(0) + xt.view(-1, 1, 1) * self.A1.unsqueeze(0) # [B, D, D]
            
            # bmm: [B, 1, D] @ [B, D, D] -> [B, 1, D]
            psi = torch.bmm(psi.unsqueeze(1), A_eff).squeeze(1) # [B, D]
            
            # Normalize to prevent explosion?
            # LayerNorm style?
            # For this simple task, maybe simple tanh or clamp on state?
            # BUT we want Linear Tensor. Pure Linear.
            # So rely on small initialization weights.
            
            # Readout
            # y = psi @ C + b
            y = psi @ self.C + self.bias
            outputs.append(y)
            
        return torch.stack(outputs, dim=1), psi

# --- 3. Training Loop ---
def train():
    print("⚛️ Generating Quantum Wave Data...")
    X, y = generate_complex_wave()
    X = torch.tensor(X, dtype=torch.float32)
    y = torch.tensor(y, dtype=torch.float32)
    
    model = RecurrentMPS()
    optimizer = optim.Adam(model.parameters(), lr=0.005)
    criterion = nn.MSELoss()
    
    print(f"⚛️ Training Tensor Network ({EPOCHS} Epochs)...")
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
def export_brain(model, filename="tensor.qnn"):
    print(f"💾 Exporting to {filename}...")
    
    # Export Layout:
    # A0 (D*D)
    # A1 (D*D)
    # C  (D*1)
    # Bias (1) -- Actually lets make it 4 to align C? No, C is 4x1, Basis 1.
    
    # Weights need to be flattened.
    # Note: Our inference engine in Rust will use [D, D] array.
    # We used `psi @ A`. So Rust side: `psi_new[j] = sum(psi[i] * A[i][j])`.
    
    a0 = model.A0.detach().numpy() # [4, 4]
    a1 = model.A1.detach().numpy() # [4, 4]
    c  = model.C.detach().numpy()  # [4, 1]
    b  = model.bias.detach().numpy() # [1]
    
    with open(filename, "wb") as f:
        f.write(b"MPS1") # Magic
        f.write(a0.tobytes())
        f.write(a1.tobytes())
        f.write(c.tobytes())
        f.write(b.tobytes())
        
    calc_size = a0.nbytes + a1.nbytes + c.nbytes + b.nbytes
    print(f"✅ Exported {filename} ({calc_size} bytes of weights)")

if __name__ == "__main__":
    brain = train()
    export_brain(brain)
