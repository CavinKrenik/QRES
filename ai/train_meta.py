# ai/train_meta.py
import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import DataLoader, TensorDataset
import numpy as np
import math
from safetensors.torch import save_file

# --- Config matches Rust ---
EMBED_DIM = 128
HIDDEN_DIM = 256
HEADS = 4
LAYERS = 2
SEQ_LEN = 256
CLASSES = 4
# Class 0: Linear (Arithmetic/Constant)
# Class 1: iPEPS (Complex periodic/Modulated)
# Class 2: Zstd (High Entropy/Noise)
# Class 3: Text (ASCII/Code)

class TransformerBlock(nn.Module):
    def __init__(self):
        super().__init__()
        self.ln1 = nn.LayerNorm(EMBED_DIM)
        self.ln2 = nn.LayerNorm(EMBED_DIM)
        self.attn = nn.MultiheadAttention(EMBED_DIM, HEADS, batch_first=True)
        self.ff = nn.Sequential(
            nn.Linear(EMBED_DIM, HIDDEN_DIM),
            nn.ReLU(),
            nn.Linear(HIDDEN_DIM, EMBED_DIM)
        )

    def forward(self, x):
        x_norm = self.ln1(x)
        attn_out, _ = self.attn(x_norm, x_norm, x_norm)
        x = x + attn_out
        x_norm = self.ln2(x)
        return x + self.ff(x_norm)

class MetaTransformer(nn.Module):
    def __init__(self):
        super().__init__()
        self.embed = nn.Embedding(256, EMBED_DIM)
        self.blocks = nn.ModuleList([TransformerBlock() for _ in range(LAYERS)])
        self.head = nn.Linear(EMBED_DIM, CLASSES)

    def forward(self, x):
        x = self.embed(x)
        for block in self.blocks:
            x = block(x)
        x = x.mean(dim=1)
        return self.head(x)

def generate_structured_data(num_samples=2000):
    data = []
    labels = []

    print(f"⚗️ Generating {num_samples} structured samples...")

    for i in range(num_samples):
        cls = i % CLASSES

        if cls == 0:  # Linear: Arithmetic progressions or constants
            start = np.random.randint(0, 256)
            step = np.random.randint(0, 5)
            seq = ((np.arange(SEQ_LEN) * step + start) % 256).astype(np.uint8)

        elif cls == 1:  # iPEPS: Complex Modulated Wave (Hard for Delta, learnable for Tensor)
            t = np.linspace(0, 4 * np.pi, SEQ_LEN)
            wave = np.sin(t) * np.cos(t * 3.0)
            seq = (((wave + 1.0) / 2.0 * 255.0).astype(np.uint8))

        elif cls == 2:  # Zstd: Pure Random Noise
            seq = np.random.randint(0, 256, SEQ_LEN).astype(np.uint8)

        elif cls == 3:  # Text: ASCII
            text_source = "def function(x): return x * 2; " * 20
            text_source = text_source * ((SEQ_LEN // len(text_source)) + 2)  # Loop to ensure length
            start_idx = np.random.randint(0, len(text_source) - SEQ_LEN)
            seq = np.array([ord(c) for c in text_source[start_idx : start_idx + SEQ_LEN]], dtype=np.uint8)

        data.append(seq)
        labels.append(cls)

    return torch.tensor(np.array(data), dtype=torch.long), torch.tensor(np.array(labels), dtype=torch.long)

import argparse
import os

def entropy(chunk):
    counts = np.bincount(chunk, minlength=256)
    probs = counts[counts > 0] / len(chunk)
    return -np.sum(probs * np.log2(probs))

def oracle_label_chunk(chunk):
    # Heuristic Oracle: Determine best engine based on statistical properties
    # 0: Linear, 1: iPEPS (Periodic), 2: Zstd (Random), 3: Text
    
    ent = entropy(chunk)
    
    # 1. Check for Text (Class 3)
    # Heuristic: mostly ASCII printable
    printable = np.sum((chunk >= 32) & (chunk <= 126))
    if printable / len(chunk) > 0.9:
        return 3 # Text
        
    # 2. Check for Low Entropy (Class 0 - Linear)
    if ent < 4.0:
        return 0 # Linear
        
    # 3. Check for Periodicity (Class 1 - iPEPS)
    # Simple check: strong auto-correlation at small lags?
    # FFT is better but expensive. Let's use simple diff check.
    # If 2nd derivative is small, it's smooth/periodic-ish.
    diff2 = np.diff(chunk, n=2)
    if np.mean(np.abs(diff2)) < 20: 
        return 1 # Periodic/Smooth
        
    # 4. Default to Zstd (Class 2 - High Entropy)
    return 2

def load_file_and_label(filepath, block_size=SEQ_LEN):
    print(f"📂 Loading {filepath}...")
    try:
        with open(filepath, "rb") as f:
            raw = f.read()
    except Exception as e:
        print(f"Error reading file: {e}")
        return generate_structured_data(1000) # Fallback

    data = []
    labels = []
    
    # Process chunks of file
    count = 0
    limit = 10000 # Max chunks to avoid OOM
    
    for i in range(0, len(raw) - block_size, block_size):
        if count >= limit: break
        
        chunk = np.frombuffer(raw[i:i+block_size], dtype=np.uint8)
        label = oracle_label_chunk(chunk)
        
        data.append(chunk)
        labels.append(label)
        count += 1
        
    if count == 0:
        return generate_structured_data(100) # Fallback for small files

    print(f"🏷️  Generated {count} labeled samples from file.")
    return torch.tensor(np.array(data), dtype=torch.long), torch.tensor(np.array(labels), dtype=torch.long)

def train_and_export():
    parser = argparse.ArgumentParser()
    parser.add_argument("--data_file", type=str, help="Path to data file for training")
    args = parser.parse_args()

    if args.data_file and os.path.exists(args.data_file):
        data, labels = load_file_and_label(args.data_file)
    else:
        data, labels = generate_structured_data(4000)

    # Batching for efficiency
    dataset = TensorDataset(data, labels)
    loader = DataLoader(dataset, batch_size=512, shuffle=True)

    model = MetaTransformer()
    optimizer = optim.Adam(model.parameters(), lr=0.001)
    criterion = nn.CrossEntropyLoss()

    print("🧠 Training MetaTransformer...")
    for epoch in range(10): # Reduced epochs for responsiveness
        epoch_loss = 0.0
        for batch_data, batch_labels in loader:
            optimizer.zero_grad()
            out = model(batch_data)
            loss = criterion(out, batch_labels)
            loss.backward()
            optimizer.step()
            epoch_loss += loss.item()
        
        # Simple progress output
        print(f"Epoch {epoch+1}/10: Loss {epoch_loss / len(loader):.4f}")

    print("💾 Exporting to meta_brain.safetensors...")
    tensors = {k: v for k, v in model.state_dict().items()}
    # Ensure directory exists
    os.makedirs("qres_rust/assets", exist_ok=True)
    save_file(tensors, "qres_rust/assets/meta_brain.safetensors")
    print("✅ Training Complete.")

if __name__ == "__main__":
    train_and_export()
