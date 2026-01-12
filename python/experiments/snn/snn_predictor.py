import torch
import torch.nn as nn
import snntorch as snn
from snntorch import surrogate
import numpy as np
import os

def create_dataset(seq_len=1000, window_size=32):
    """
    Creates a simple synthetic dataset (Sine wave + noise)
    Returns: (X, y) tensors
        X: [seq_len - window_size, window_size, 1] (Time, Window, Channels)
        y: [seq_len - window_size, 1]
    """
    x_vals = np.linspace(0, 100, seq_len)
    data = np.sin(x_vals) + 0.1 * np.random.randn(seq_len)
    
    X = []
    y = []
    
    for i in range(len(data) - window_size):
        window = data[i:i+window_size]
        target = data[i+window_size]
        X.append(window.reshape(-1, 1)) # (Time, Channels)
        y.append(target)
        
    return torch.FloatTensor(np.array(X)), torch.FloatTensor(np.array(y)).unsqueeze(1)

dataset_X, dataset_y = create_dataset()

# Split Train/Test
split = int(0.8 * len(dataset_X))
train_X, test_X = dataset_X[:split], dataset_X[split:]
train_y, test_y = dataset_y[:split], dataset_y[split:]

# Hyperparameters
beta = 0.95 # Decay rate
num_steps = 32 # Time steps (matches window size in this temporal coding setup)
batch_size = 32
lr = 1e-3
num_epochs = 50

# Network Architecture
# Input (1) -> Hidden LIF (128) -> Output LIF (1)
class SNNPredictor(nn.Module):
    def __init__(self):
        super().__init__()
        
        # Initialize layers
        self.fc1 = nn.Linear(1, 128)
        self.lif1 = snn.Leaky(beta=beta, spike_grad=surrogate.fast_sigmoid())
        self.fc2 = nn.Linear(128, 1)
        self.lif2 = snn.Leaky(beta=beta, spike_grad=surrogate.fast_sigmoid(), output=True)

    def forward(self, x):
        # x shape: [Batch, Window, Features]
        # We treat 'Window' as 'Time Steps' for the SNN
        
        mem1 = self.lif1.init_leaky()
        mem2 = self.lif2.init_leaky()
        
        # Record the final output
        spk2_rec = []
        mem2_rec = []
        spk1_count = 0 # Track total spikes for energy estimation
        
        # Time loop (processing the window sequentially)
        for step in range(x.size(1)):
            cur_input = x[:, step, :]
            
            # Layer 1
            cur1 = self.fc1(cur_input)
            spk1, mem1 = self.lif1(cur1, mem1)
            spk1_count += spk1.sum().item()
            
            # Layer 2
            cur2 = self.fc2(spk1)
            spk2, mem2 = self.lif2(cur2, mem2)
            
            spk2_rec.append(spk2)
            mem2_rec.append(mem2)
            
        return torch.stack(spk2_rec, dim=0), torch.stack(mem2_rec, dim=0), spk1_count

def train():
    device = torch.device("cuda") if torch.cuda.is_available() else torch.device("cpu")
    model = SNNPredictor().to(device)
    optimizer = torch.optim.Adam(model.parameters(), lr=lr)
    loss_fn = nn.MSELoss()
    
    print(f"Training SNN on {device}...")
    
    for epoch in range(num_epochs):
        model.train()
        train_loss = 0
        total_spikes = 0
        
        # Simple full batch for demo
        inputs = train_X.to(device)
        targets = train_y.to(device)
        
        optimizer.zero_grad()
        spk_rec, mem_rec, spk_count = model(inputs)
        
        # Use the membrane potential of the last step as the continuous prediction
        # (Regression with SNN usually uses membrane potential)
        preds = mem_rec[-1] 
        loss = loss_fn(preds, targets)
        
        loss.backward()
        optimizer.step()
        
        if epoch % 10 == 0:
            print(f"Epoch {epoch}: Loss = {loss.item():.6f}")

    # Evaluate
    model.eval()
    with torch.no_grad():
        inputs = test_X.to(device)
        _, mem_rec, test_spikes = model(inputs)
        preds = mem_rec[-1]
        test_mse = loss_fn(preds, test_y.to(device)).item()
        
    print("="*40)
    print(f"Test MSE: {test_mse:.6f}")
    print(f"Total Spikes (Test Set): {test_spikes}")
    print("="*40)
    
    # Save stats for energy analysis
    with open("snn_stats.txt", "w") as f:
        f.write(f"{test_mse}\n")
        f.write(f"{test_spikes}\n")
        f.write(f"{len(test_X)}\n") # Num samples

if __name__ == "__main__":
    train()
