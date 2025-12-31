"""
Phase 1: Real Supervised Learning for Meta-Selector
Trains a lightweight transformer on actual compression race results.
Exports to safetensors for Rust/Candle inference.
"""
import numpy as np
import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import Dataset, DataLoader
import os
import sys

# Import QRES for compression racing
try:
    import qres
except ImportError:
    print("ERROR: qres package not installed. Run: maturin develop --release")
    sys.exit(1)

# --- CONFIG ---
SAMPLES_PER_TYPE = 100  # More samples for better learning
CHUNK_SIZE = 4096
FULL_SIZE = 64 * 1024
EMBED_DIM = 128
HIDDEN_DIM = 256
HEADS = 4
LAYERS = 2
SEQ_LEN = 256  # Analyze first 256 bytes
EPOCHS = 100
BATCH_SIZE = 32
LEARNING_RATE = 0.001

# --- DATA GENERATORS ---
def gen_sine(size):
    """Smooth periodic signal - good for Linear predictor"""
    x = np.linspace(0, 100 * np.pi, size)
    y = np.sin(x) * 100 + 128
    return y.astype(np.uint8).tobytes()

def gen_linear(size):
    """Linear ramp - perfect for Linear predictor"""
    return np.arange(size, dtype=np.uint8).tobytes()

def gen_noise(size):
    """Random noise - good for Zstd"""
    return np.random.randint(0, 256, size, dtype=np.uint8).tobytes()

def gen_text(size):
    """Text-like data - good for Semantic/LSTM"""
    # ASCII printable range with realistic distribution
    chars = np.random.choice(list(range(32, 127)) + [10, 13, 32] * 10, size=size)
    return chars.astype(np.uint8).tobytes()

def gen_sparse(size):
    """Sparse data - excellent for Linear/RLE"""
    data = np.zeros(size, dtype=np.uint8)
    # Add occasional spikes
    spikes = np.random.randint(0, size, size // 100)
    data[spikes] = np.random.randint(1, 256, len(spikes))
    return data.tobytes()

def gen_structured(size):
    """Structured patterns - good for Tensor predictor"""
    # Repeating blocks
    block = np.random.randint(0, 256, 256, dtype=np.uint8)
    repeats = size // 256 + 1
    return np.tile(block, repeats)[:size].tobytes()

# --- LOAD WEIGHTS ---
def load_weights():
    """Load pre-trained LSTM and Tensor weights"""
    base = os.path.dirname(os.path.abspath(__file__))
    assets = os.path.join(base, "../qres_rust/assets")
    
    lstm_path = os.path.join(assets, "lstm.qnn")
    tensor_path = os.path.join(assets, "tensor.qnn")
    
    if not os.path.exists(lstm_path) or not os.path.exists(tensor_path):
        print(f"WARNING: Weight files not found in {assets}")
        print("Using None for LSTM/Tensor weights (will use default initialization)")
        return None, None
    
    with open(lstm_path, "rb") as f:
        lstm_w = f.read()
    with open(tensor_path, "rb") as f:
        tensor_w = f.read()
    return lstm_w, tensor_w

lstm_weights, tensor_weights = load_weights()

# --- COMPRESSION RACE ---
def race_engines(data):
    """
    Race all compression engines and return winner + compression ratios.
    Returns: (winner_id, ratios_dict)
    """
    original_size = len(data)
    results = {}
    
    try:
        # ID 1: Linear Predictor
        c1 = qres.encode_bytes(data, 1, None)
        results[1] = len(c1) / original_size
    except Exception as e:
        print(f"Linear failed: {e}")
        results[1] = 1.5  # Penalty
    
    try:
        # ID 3: LSTM Predictor
        c3 = qres.encode_bytes(data, 3, lstm_weights)
        results[3] = len(c3) / original_size
    except Exception as e:
        print(f"LSTM failed: {e}")
        results[3] = 1.5
    
    try:
        # ID 4: Tensor Predictor
        c4 = qres.encode_bytes(data, 4, tensor_weights)
        results[4] = len(c4) / original_size
    except Exception as e:
        print(f"Tensor failed: {e}")
        results[4] = 1.5
    
    try:
        # ID 6: Zstd Fallback
        c6 = qres.encode_bytes(data, 6, None)
        results[6] = len(c6) / original_size
    except Exception as e:
        print(f"Zstd failed: {e}")
        results[6] = 1.5
    
    # Find winner (lowest ratio)
    winner = min(results, key=results.get)
    return winner, results

# --- DATASET ---
class CompressionDataset(Dataset):
    def __init__(self, samples):
        self.samples = samples
        # Map engine IDs to class indices
        self.id_to_class = {1: 0, 3: 1, 4: 2, 6: 3}
        
    def __len__(self):
        return len(self.samples)
    
    def __getitem__(self, idx):
        data, winner = self.samples[idx]
        # Take first SEQ_LEN bytes
        chunk = np.frombuffer(data[:SEQ_LEN], dtype=np.uint8)
        if len(chunk) < SEQ_LEN:
            # Pad with zeros
            chunk = np.pad(chunk, (0, SEQ_LEN - len(chunk)), 'constant')
        
        # Convert to tensor
        x = torch.from_numpy(chunk).long()
        y = torch.tensor(self.id_to_class[winner], dtype=torch.long)
        return x, y

# --- TRANSFORMER MODEL ---
class TransformerBlock(nn.Module):
    def __init__(self, embed_dim, heads, hidden_dim):
        super().__init__()
        self.attention = nn.MultiheadAttention(embed_dim, heads, batch_first=True)
        self.ff = nn.Sequential(
            nn.Linear(embed_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, embed_dim)
        )
        self.ln1 = nn.LayerNorm(embed_dim)
        self.ln2 = nn.LayerNorm(embed_dim)
        
    def forward(self, x):
        # Self-attention with residual
        attn_out, _ = self.attention(x, x, x)
        x = self.ln1(x + attn_out)
        
        # Feed-forward with residual
        ff_out = self.ff(x)
        x = self.ln2(x + ff_out)
        return x

class MetaTransformer(nn.Module):
    def __init__(self, vocab_size=256, embed_dim=EMBED_DIM, heads=HEADS, 
                 hidden_dim=HIDDEN_DIM, layers=LAYERS, num_classes=4):
        super().__init__()
        self.embed = nn.Embedding(vocab_size, embed_dim)
        self.blocks = nn.ModuleList([
            TransformerBlock(embed_dim, heads, hidden_dim) 
            for _ in range(layers)
        ])
        self.head = nn.Linear(embed_dim, num_classes)
        
    def forward(self, x):
        # x: [batch, seq_len]
        x = self.embed(x)  # [batch, seq_len, embed_dim]
        
        for block in self.blocks:
            x = block(x)
        
        # Pool: mean over sequence
        x = x.mean(dim=1)  # [batch, embed_dim]
        
        # Classification head
        logits = self.head(x)  # [batch, num_classes]
        return logits

# --- TRAINING ---
def train_model():
    print("🤖 Generating training data from real compression races...")
    
    generators = [gen_sine, gen_linear, gen_noise, gen_text, gen_sparse, gen_structured]
    samples = []
    
    for i, gen in enumerate(generators):
        print(f"  Generating {SAMPLES_PER_TYPE} samples for generator {i+1}/{len(generators)}...")
        for j in range(SAMPLES_PER_TYPE):
            data = gen(FULL_SIZE)
            winner, ratios = race_engines(data)
            samples.append((data, winner))
            
            if (j + 1) % 20 == 0:
                print(f"    Progress: {j+1}/{SAMPLES_PER_TYPE} - Last winner: Engine {winner}")
    
    print(f"\n✅ Collected {len(samples)} training samples")
    
    # Create dataset and dataloader
    dataset = CompressionDataset(samples)
    dataloader = DataLoader(dataset, batch_size=BATCH_SIZE, shuffle=True)
    
    # Initialize model
    device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
    print(f"🔧 Training on: {device}")
    
    model = MetaTransformer().to(device)
    criterion = nn.CrossEntropyLoss()
    optimizer = optim.Adam(model.parameters(), lr=LEARNING_RATE)
    
    print(f"\n🧠 Training MetaTransformer ({LAYERS} layers, {EMBED_DIM}d, {HEADS} heads)...")
    
    for epoch in range(EPOCHS):
        model.train()
        total_loss = 0
        correct = 0
        total = 0
        
        for batch_x, batch_y in dataloader:
            batch_x, batch_y = batch_x.to(device), batch_y.to(device)
            
            optimizer.zero_grad()
            outputs = model(batch_x)
            loss = criterion(outputs, batch_y)
            loss.backward()
            optimizer.step()
            
            total_loss += loss.item()
            _, predicted = outputs.max(1)
            total += batch_y.size(0)
            correct += predicted.eq(batch_y).sum().item()
        
        avg_loss = total_loss / len(dataloader)
        accuracy = 100. * correct / total
        
        if epoch % 10 == 0 or epoch == EPOCHS - 1:
            print(f"Epoch {epoch:3d}: Loss {avg_loss:.4f}, Accuracy {accuracy:.2f}%")
    
    print(f"\n✅ Training complete! Final accuracy: {accuracy:.2f}%")
    return model

# --- EXPORT ---
def export_model(model):
    print("\n💾 Exporting model to safetensors...")
    
    # Save state dict
    state_dict = model.state_dict()
    
    # Try safetensors first
    try:
        from safetensors.torch import save_file
        output_path = "ai/meta_brain.safetensors"
        save_file(state_dict, output_path)
        print(f"✅ Exported to {output_path} (safetensors format)")
    except ImportError:
        # Fallback to PyTorch format
        output_path = "ai/meta_brain.pth"
        torch.save(state_dict, output_path)
        print(f"✅ Exported to {output_path} (PyTorch format)")
        print("⚠️  Install safetensors for Rust compatibility: pip install safetensors")
    
    # Print model info
    total_params = sum(p.numel() for p in model.parameters())
    print(f"📊 Model size: {total_params:,} parameters (~{total_params * 4 / 1024:.1f} KB)")
    
    return output_path

# --- MAIN ---
if __name__ == "__main__":
    print("=" * 60)
    print("QRES Meta-Selector Training Pipeline")
    print("Phase 1: Supervised Learning from Compression Races")
    print("=" * 60)
    
    model = train_model()
    output_path = export_model(model)
    
    print("\n🎯 Next steps:")
    print("1. Install safetensors if needed: pip install safetensors")
    print("2. Update Rust code to load from:", output_path)
    print("3. Test inference in qres_rust/src/meta_brain.rs")
    print("\n✨ Ready for Rust integration!")
