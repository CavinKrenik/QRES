
import os
import numpy as np
import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import Dataset, DataLoader
from typing import Tuple

from model import TinyPredictor, count_parameters


# Configuration
WINDOW_SIZE = 32
BATCH_SIZE = 64
EPOCHS = 50
LEARNING_RATE = 0.001
DEVICE = "cuda" if torch.cuda.is_available() else "cpu"

# Output paths
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(SCRIPT_DIR, "..", "..", ".."))
ONNX_OUTPUT = os.path.join(PROJECT_ROOT, "qres_rust", "qres_core", "assets", "predictor_v2.onnx")


class SyntheticTimeSeriesDataset(Dataset):
    def __init__(self, num_samples: int = 10000, window_size: int = 32):
        self.num_samples = num_samples
        self.window_size = window_size
        self.data = []
        self.targets = []
        for _ in range(num_samples):
            freq = np.random.uniform(0.1, 0.5)
            phase = np.random.uniform(0, 2 * np.pi)
            amp = np.random.uniform(0.5, 2.0)
            offset = np.random.uniform(-0.5, 0.5)
            t = np.arange(window_size + 1) * freq + phase
            values = amp * np.sin(t) + offset + np.random.randn(window_size + 1) * 0.05
            self.data.append(values[:window_size].astype(np.float32))
            self.targets.append(values[window_size].astype(np.float32))
    
    def __len__(self) -> int:
        return self.num_samples
    
    def __getitem__(self, idx: int) -> Tuple[torch.Tensor, torch.Tensor]:
        return (torch.from_numpy(self.data[idx]), torch.tensor([self.targets[idx]]))


def train_model(model: nn.Module, train_loader: DataLoader, epochs: int) -> nn.Module:
    model.to(DEVICE)
    model.train()
    criterion = nn.MSELoss()
    optimizer = optim.Adam(model.parameters(), lr=LEARNING_RATE)
    scheduler = optim.lr_scheduler.StepLR(optimizer, step_size=20, gamma=0.5)
    
    for epoch in range(epochs):
        total_loss = 0.0
        for batch_x, batch_y in train_loader:
            batch_x = batch_x.to(DEVICE)
            batch_y = batch_y.to(DEVICE)
            optimizer.zero_grad()
            output = model(batch_x)
            loss = criterion(output, batch_y)
            loss.backward()
            optimizer.step()
            total_loss += loss.item()
        
        avg_loss = total_loss / len(train_loader)
        scheduler.step()
        if (epoch + 1) % 10 == 0 or epoch == 0:
            print(f"Epoch {epoch + 1}/{epochs}, Loss: {avg_loss:.6f}")
    return model


def export_to_onnx(model: nn.Module, output_path: str) -> None:
    """Export the model to ONNX format."""
    model.eval()
    model.to("cpu")
    
    dummy_input = torch.randn(1, WINDOW_SIZE)
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    
    import warnings
    warnings.filterwarnings("ignore")
    
    # Export DIRECTLY (No jit.trace)
    # The hotfix we applied to onnxscript will allow this to pass now
    print(f"Exporting model to {output_path}...")
    torch.onnx.export(
        model,
        dummy_input,
        output_path,
        export_params=True,
        opset_version=17, # Use a modern opset
        do_constant_folding=True,
        input_names=["input"],
        output_names=["output"],
        dynamic_axes=None,
        verbose=False,
    )
    
    file_size = os.path.getsize(output_path)
    print(f"ONNX model saved to: {output_path}")
    print(f"File size: {file_size / 1024:.2f} KB")


def main():
    print("=" * 50)
    print("TinyPredictor Training & Export")
    print("=" * 50)
    
    model = TinyPredictor(window_size=WINDOW_SIZE)
    print(f"Model parameters: {count_parameters(model):,}")
    
    print(f"\nGenerating training data...")
    dataset = SyntheticTimeSeriesDataset(num_samples=10000, window_size=WINDOW_SIZE)
    train_loader = DataLoader(dataset, batch_size=BATCH_SIZE, shuffle=True)
    
    print(f"\nTraining for {EPOCHS} epochs on {DEVICE}...")
    model = train_model(model, train_loader, EPOCHS)
    
    print(f"\nExporting to ONNX...")
    export_to_onnx(model, ONNX_OUTPUT)
    
    # Quick validation
    print(f"\nValidation:")
    model.eval()
    with torch.no_grad():
        test_input = torch.sin(torch.arange(WINDOW_SIZE).float() * 0.2).unsqueeze(0)
        prediction = model(test_input)
        expected = np.sin(WINDOW_SIZE * 0.2)
        print(f"  Input: sin(0.2 * t) for t in [0, {WINDOW_SIZE-1}]")
        print(f"  Predicted next: {prediction.item():.4f}")
        print(f"  Expected (approx): {expected:.4f}")
    
    print("\n" + "=" * 50)
    print("Training complete!")
    print("=" * 50)


if __name__ == "__main__":
    main()
