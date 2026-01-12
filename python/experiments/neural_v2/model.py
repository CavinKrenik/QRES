"""
TinyPredictor: A lightweight Transformer-based time series predictor.

Architecture:
- 1D CNN embedding layer
- Small Transformer Encoder (2 layers, d_model=32, nhead=4)
- Linear output layer

Target: <50KB ONNX file size for edge deployment.
"""

import torch
import torch.nn as nn
from typing import Optional


class TinyPredictor(nn.Module):
    """
    A tiny Transformer-based predictor for time series forecasting.
    
    Args:
        window_size: Number of input time steps (default: 32).
        d_model: Model dimension (default: 32).
        nhead: Number of attention heads (default: 4).
        num_layers: Number of transformer layers (default: 2).
    """
    
    def __init__(
        self,
        window_size: int = 32,
        d_model: int = 32,
        nhead: int = 4,
        num_layers: int = 2,
    ):
        super().__init__()
        
        self.window_size = window_size
        self.d_model = d_model
        
        # 1D CNN Embedding: Convert raw values to d_model dimension
        self.embedding = nn.Sequential(
            nn.Conv1d(1, d_model // 2, kernel_size=3, padding=1),
            nn.ReLU(),
            nn.Conv1d(d_model // 2, d_model, kernel_size=3, padding=1),
            nn.ReLU(),
        )
        
        # Positional encoding (learnable)
        self.pos_encoding = nn.Parameter(torch.randn(1, window_size, d_model) * 0.02)
        
        # Transformer Encoder
        encoder_layer = nn.TransformerEncoderLayer(
            d_model=d_model,
            nhead=nhead,
            dim_feedforward=d_model * 2,
            dropout=0.1,
            batch_first=True,
        )
        self.transformer = nn.TransformerEncoder(encoder_layer, num_layers=num_layers)
        
        # Output projection: predict next value
        self.output = nn.Sequential(
            nn.Linear(d_model, d_model // 2),
            nn.ReLU(),
            nn.Linear(d_model // 2, 1),
        )
    
    def forward(self, x: torch.Tensor) -> torch.Tensor:
        """
        Forward pass.
        
        Args:
            x: Input tensor of shape (batch, window_size) or (batch, 1, window_size).
        
        Returns:
            Predicted next value of shape (batch, 1).
        """
        # Handle input shape
        if x.dim() == 2:
            x = x.unsqueeze(1)  # (batch, 1, window_size)
        
        # CNN Embedding: (batch, 1, window_size) -> (batch, d_model, window_size)
        x = self.embedding(x)
        
        # Transpose for transformer: (batch, d_model, window_size) -> (batch, window_size, d_model)
        x = x.transpose(1, 2)
        
        # Add positional encoding
        x = x + self.pos_encoding
        
        # Transformer Encoder
        x = self.transformer(x)
        
        # Take the last position's representation
        x = x[:, -1, :]  # (batch, d_model)
        
        # Output projection
        return self.output(x)


def count_parameters(model: nn.Module) -> int:
    """Count trainable parameters."""
    return sum(p.numel() for p in model.parameters() if p.requires_grad)


if __name__ == "__main__":
    # Test the model
    model = TinyPredictor()
    print(f"TinyPredictor Parameters: {count_parameters(model):,}")
    
    # Test forward pass
    x = torch.randn(4, 32)  # Batch of 4, window size 32
    y = model(x)
    print(f"Input shape: {x.shape}")
    print(f"Output shape: {y.shape}")
