"""
PyTorch MarketPhysicsEncoder
Phase B: Advanced Model - Task B1: PyTorch MarketPhysicsEncoder
"""

import torch
import torch.nn as nn
import numpy as np


class MarketPhysicsEncoder(nn.Module):
    """
    Physics-informed encoder for market data.
    Encodes market physics constraints into latent representation.
    """
    
    def __init__(self, input_dim: int = 600, hidden_dim: int = 128, output_dim: int = 64):
        super().__init__()
        self.input_dim = input_dim
        self.hidden_dim = hidden_dim
        self.output_dim = output_dim
        
        # Encoder layers
        self.encoder = nn.Sequential(
            nn.Linear(input_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, output_dim),
        )
        
        # Physics constraint layer
        self.physics_constraint = nn.Sequential(
            nn.Linear(output_dim, output_dim),
            nn.Tanh(),
        )
    
    def forward(self, x: torch.Tensor) -> torch.Tensor:
        """
        Forward pass through the encoder.
        
        Args:
            x: Input tensor of shape (batch_size, input_dim)
        
        Returns:
            Encoded tensor of shape (batch_size, output_dim)
        """
        encoded = self.encoder(x)
        constrained = self.physics_constraint(encoded)
        return constrained
    
    def encode_bam_grid(self, grid: np.ndarray) -> np.ndarray:
        """
        Encode a BAM grid (10×60 = 600 bytes).
        
        Args:
            grid: BAM grid as numpy array of shape (10, 60)
        
        Returns:
            Encoded representation as numpy array
        """
        grid_flat = grid.flatten().astype(np.float32) / 255.0
        grid_tensor = torch.from_numpy(grid_flat).unsqueeze(0)
        
        with torch.no_grad():
            encoded = self.forward(grid_tensor)
        
        return encoded.squeeze(0).numpy()


if __name__ == "__main__":
    # Test the encoder
    encoder = MarketPhysicsEncoder()
    test_grid = np.random.randint(0, 256, (10, 60), dtype=np.uint8)
    encoded = encoder.encode_bam_grid(test_grid)
    print(f"Encoded shape: {encoded.shape}")
    print(f"Encoded sample: {encoded[:5]}")
