"""
VariancePredictor LSTM
Phase B: Advanced Model - Task B2: VariancePredictor LSTM
"""

import torch
import torch.nn as nn
import numpy as np


class VariancePredictor(nn.Module):
    """
    LSTM-based variance predictor for market data.
    Predicts future variance from historical patterns.
    """
    
    def __init__(self, input_dim: int = 64, hidden_dim: int = 128, num_layers: int = 2):
        super().__init__()
        self.input_dim = input_dim
        self.hidden_dim = hidden_dim
        self.num_layers = num_layers
        
        # LSTM layers
        self.lstm = nn.LSTM(
            input_size=input_dim,
            hidden_size=hidden_dim,
            num_layers=num_layers,
            batch_first=True,
        )
        
        # Variance prediction head
        self.variance_head = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim // 2),
            nn.ReLU(),
            nn.Linear(hidden_dim // 2, 1),
            nn.Softplus(),  # Ensure positive variance
        )
    
    def forward(self, x: torch.Tensor) -> torch.Tensor:
        """
        Forward pass through the LSTM.
        
        Args:
            x: Input tensor of shape (batch_size, seq_len, input_dim)
        
        Returns:
            Predicted variance of shape (batch_size, 1)
        """
        lstm_out, _ = self.lstm(x)
        # Use the last timestep output
        last_output = lstm_out[:, -1, :]
        variance = self.variance_head(last_output)
        return variance
    
    def predict_variance(self, encoded_sequence: np.ndarray) -> float:
        """
        Predict variance from encoded sequence.
        
        Args:
            encoded_sequence: Encoded sequence as numpy array of shape (seq_len, input_dim)
        
        Returns:
            Predicted variance as float
        """
        seq_tensor = torch.from_numpy(encoded_sequence.astype(np.float32)).unsqueeze(0)
        
        with torch.no_grad():
            variance = self.forward(seq_tensor)
        
        return variance.squeeze(0).item()


if __name__ == "__main__":
    # Test the predictor
    predictor = VariancePredictor()
    test_sequence = np.random.randn(10, 64).astype(np.float32)
    variance = predictor.predict_variance(test_sequence)
    print(f"Predicted variance: {variance}")
