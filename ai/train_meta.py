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

def train_and_export():
    data, labels = generate_structured_data(4000)

    # Batching for efficiency
    dataset = TensorDataset(data, labels)
    loader = DataLoader(dataset, batch_size=512, shuffle=True)

    model = MetaTransformer()
    optimizer = optim.Adam(model.parameters(), lr=0.001)
    criterion = nn.CrossEntropyLoss()

    print("🧠 Training MetaTransformer...")
    for epoch in range(50):
        epoch_loss = 0.0
        for batch_data, batch_labels in loader:
            optimizer.zero_grad()
            out = model(batch_data)
            loss = criterion(out, batch_labels)
            loss.backward()
            optimizer.step()
            epoch_loss += loss.item()
        if epoch % 10 == 0:
            print(f"Epoch {epoch}: Avg Loss {epoch_loss / len(loader):.4f}")

    print("💾 Exporting to meta_brain.safetensors...")
    tensors = {k: v for k, v in model.state_dict().items()}
    save_file(tensors, "qres_rust/assets/meta_brain.safetensors")
    print("✅ Done.")

if __name__ == "__main__":
    train_and_export()
