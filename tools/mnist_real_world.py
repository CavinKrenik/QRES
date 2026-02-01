"""
QRES v19.0 - MNIST Real-World Validation

Validates BFP-16 arithmetic against real neural network gradients from MNIST.
Trains a simple MLP and compares convergence between Float32 and BFP-16
quantized gradient updates.

Success Criterion: BFP-16 accuracy within +/- 3% of Float32 baseline.

Usage: python tools/mnist_real_world.py
"""

import numpy as np
import sys
from datetime import datetime

import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import DataLoader, Subset
from torchvision import datasets, transforms

# --- BFP-16 Quantizer ---
BFP_MANTISSA_MAX = 32767
BFP_MANTISSA_MIN = -32768
BFP_EXPONENT_MIN = -128
BFP_EXPONENT_MAX = 127


def quantize_bfp(tensor):
    """Quantize a PyTorch tensor to BFP-16 representation and back."""
    x = tensor.detach().cpu().numpy().flatten()
    max_abs = np.max(np.abs(x))
    if max_abs == 0:
        return tensor.clone()
    raw_exp = np.ceil(np.log2(max_abs / BFP_MANTISSA_MAX))
    shared_exp = int(np.clip(raw_exp, BFP_EXPONENT_MIN, BFP_EXPONENT_MAX))
    scale = 2.0 ** shared_exp
    mantissas = np.round(x / scale)
    mantissas = np.clip(mantissas, BFP_MANTISSA_MIN, BFP_MANTISSA_MAX)
    reconstructed = mantissas * scale
    result = torch.from_numpy(reconstructed.reshape(tensor.shape)).float()
    return result.to(tensor.device)


class SimpleMLP(nn.Module):
    """Two-layer MLP for MNIST classification."""
    def __init__(self):
        super().__init__()
        self.fc1 = nn.Linear(784, 128)
        self.fc2 = nn.Linear(128, 10)

    def forward(self, x):
        x = x.view(-1, 784)
        x = torch.relu(self.fc1(x))
        x = self.fc2(x)
        return x


def train_epoch(model, loader, optimizer, criterion, use_bfp=False):
    """Train one epoch, optionally quantizing gradients via BFP-16."""
    model.train()
    total_loss = 0.0
    correct = 0
    total = 0
    zero_grad_count = 0
    total_grad_count = 0

    for data, target in loader:
        optimizer.zero_grad()
        output = model(data)
        loss = criterion(output, target)
        loss.backward()

        if use_bfp:
            # Quantize gradients through BFP-16 before optimizer step
            for param in model.parameters():
                if param.grad is not None:
                    original = param.grad.data.clone()
                    param.grad.data = quantize_bfp(param.grad.data)
                    # Track zero-update rate
                    non_zero = (original.abs() > 1e-30).sum().item()
                    zeros = ((param.grad.data.abs() == 0) & (original.abs() > 1e-30)).sum().item()
                    total_grad_count += non_zero
                    zero_grad_count += zeros

        optimizer.step()
        total_loss += loss.item() * data.size(0)
        pred = output.argmax(dim=1)
        correct += pred.eq(target).sum().item()
        total += data.size(0)

    avg_loss = total_loss / total
    accuracy = correct / total
    zero_rate = zero_grad_count / total_grad_count if total_grad_count > 0 else 0.0
    return avg_loss, accuracy, zero_rate


def evaluate(model, loader):
    """Evaluate model accuracy on a dataset."""
    model.eval()
    correct = 0
    total = 0
    with torch.no_grad():
        for data, target in loader:
            output = model(data)
            pred = output.argmax(dim=1)
            correct += pred.eq(target).sum().item()
            total += data.size(0)
    return correct / total


def run_training(use_bfp, lr, epochs, train_loader, test_loader, seed):
    """Run full training loop and return per-epoch metrics."""
    torch.manual_seed(seed)
    np.random.seed(seed)

    model = SimpleMLP()
    optimizer = optim.SGD(model.parameters(), lr=lr)
    criterion = nn.CrossEntropyLoss()

    format_name = "BFP-16" if use_bfp else "Float32"
    metrics = []

    for epoch in range(epochs):
        train_loss, train_acc, zero_rate = train_epoch(
            model, train_loader, optimizer, criterion, use_bfp=use_bfp
        )
        test_acc = evaluate(model, test_loader)
        metrics.append({
            'epoch': epoch + 1,
            'train_loss': train_loss,
            'train_acc': train_acc,
            'test_acc': test_acc,
            'zero_rate': zero_rate
        })
        print(f"   [{format_name}] Epoch {epoch+1}: loss={train_loss:.4f}, "
              f"train_acc={train_acc*100:.1f}%, test_acc={test_acc*100:.1f}%"
              + (f", bfp_zero={zero_rate*100:.2f}%" if use_bfp else ""))

    return metrics


def main():
    print("=" * 70)
    print("QRES v19.0 - MNIST Real-World Validation")
    print("=" * 70)
    print(f"Timestamp: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")

    # Configuration
    lr = 0.01
    epochs = 5
    batch_size = 64
    seed = 42
    subset_size = 10000  # Use subset for speed

    print(f"Model: 2-layer MLP (784 -> 128 -> 10)")
    print(f"LR: {lr}, Epochs: {epochs}, Batch Size: {batch_size}")
    print(f"Training subset: {subset_size} samples")
    print("=" * 70)

    # Load MNIST
    transform = transforms.Compose([
        transforms.ToTensor(),
        transforms.Normalize((0.1307,), (0.3081,))
    ])

    print("\nLoading MNIST...")
    train_dataset = datasets.MNIST('./data', train=True, download=True, transform=transform)
    test_dataset = datasets.MNIST('./data', train=False, transform=transform)

    # Use subset for faster iteration
    rng = np.random.default_rng(seed)
    train_indices = rng.choice(len(train_dataset), subset_size, replace=False)
    train_subset = Subset(train_dataset, train_indices)

    train_loader = DataLoader(train_subset, batch_size=batch_size, shuffle=True)
    test_loader = DataLoader(test_dataset, batch_size=256, shuffle=False)

    # Run Float32 baseline
    print("\n--- Float32 Baseline ---")
    float32_metrics = run_training(False, lr, epochs, train_loader, test_loader, seed)

    # Run BFP-16
    print("\n--- BFP-16 ---")
    bfp_metrics = run_training(True, lr, epochs, train_loader, test_loader, seed)

    # Compare
    float32_final = float32_metrics[-1]['test_acc']
    bfp_final = bfp_metrics[-1]['test_acc']
    delta = abs(bfp_final - float32_final)

    print()
    print("=" * 70)
    print("MNIST VALIDATION RESULTS")
    print("=" * 70)
    print(f"Float32 Final Test Accuracy: {float32_final*100:.2f}%")
    print(f"BFP-16  Final Test Accuracy: {bfp_final*100:.2f}%")
    print(f"Delta: {delta*100:.2f}%")
    print(f"BFP-16 Zero Update Rate (final epoch): {bfp_metrics[-1]['zero_rate']*100:.4f}%")
    print()

    convergence_table = []
    print("Epoch-by-Epoch Comparison:")
    print(f"{'Epoch':<8} {'Float32 Acc':<15} {'BFP-16 Acc':<15} {'Delta':<10}")
    for f_m, b_m in zip(float32_metrics, bfp_metrics):
        d = abs(f_m['test_acc'] - b_m['test_acc'])
        print(f"{f_m['epoch']:<8} {f_m['test_acc']*100:<15.2f} {b_m['test_acc']*100:<15.2f} {d*100:<10.2f}")
        convergence_table.append({
            'epoch': f_m['epoch'],
            'float32': f_m['test_acc'],
            'bfp16': b_m['test_acc'],
            'delta': d
        })

    passed = delta <= 0.03
    print()
    print(f"CRITERION: BFP-16 within +/-3% of Float32: [{'PASS' if passed else 'FAIL'}] (delta={delta*100:.2f}%)")
    print("=" * 70)

    return {
        'passed': passed,
        'float32_acc': float32_final,
        'bfp_acc': bfp_final,
        'delta': delta,
        'convergence': convergence_table
    }


if __name__ == "__main__":
    result = main()
    sys.exit(0 if result['passed'] else 1)
