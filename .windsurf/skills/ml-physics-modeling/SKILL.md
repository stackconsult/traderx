# ML Physics-Informed Modeling Skill

---

name: ml-physics-modeling
description: Implements physics-informed ML models for trading. Use when building ML models that need to respect physical constraints, market microstructure, or time-series dynamics.

---

## When to Activate

Use when:
- Implementing PyTorch models with physics constraints
- Building models that need to respect market microstructure
- Designing models with time-series dynamics
- Implementing Fisher information regularization
- Building cross-market attention layers

## Core Principles

### Physics-Informed Design
1. **Identify Physical Constraints**: What laws govern the system?
   - Conservation laws (mass, momentum, energy)
   - Symmetry properties (time-invariance, scale-invariance)
   - Boundary conditions (no arbitrage, market closure)

2. **Encode Constraints in Model**:
   - Add penalty terms to loss function for constraint violations
   - Use architecture that respects symmetries
   - Initialize weights to satisfy constraints
   - Regularize to prevent constraint drift

3. **Validate Against Physics**:
   - Test model predictions against known physical limits
   - Verify conservation laws in outputs
   - Check for unphysical extrapolation
   - Validate against historical regimes

### Market Microstructure Constraints
- **No Arbitrage**: Model predictions should not create arbitrage opportunities
- **Market Impact**: Large orders move prices (Kyle lambda)
- **Bid-Ask Spread**: Prices must respect bid-ask spread
- **Time-of-Day Effects**: Volatility and volume vary intraday

### Time-Series Dynamics
- **Stationarity**: Use differencing or detrending for non-stationary series
- **Autocorrelation**: Model must capture temporal dependencies
- **Seasonality**: Account for intraday, weekly, monthly patterns
- **Regime Switching**: Model must handle different market regimes

## Implementation Checklist

- [ ] Identify physical constraints for the system
- [ ] Design loss function with constraint penalties
- [ ] Implement model architecture that respects symmetries
- [ ] Initialize weights to satisfy constraints
- [ ] Add regularization to prevent constraint drift
- [ ] Validate model against known physical limits
- [ ] Test for unphysical extrapolation
- [ ] Verify against historical regimes

## Common Pitfalls

- ❌ Ignoring physical constraints → add penalty terms to loss
- ❌ Using generic architectures → design physics-aware architectures
- ❌ Not validating against physics → add physics validation tests
- ❌ Overfitting to recent regime → add regime detection and adaptation

## TraderX-Specific Adaptations

### Market Physics
- **Kyle's Lambda**: Market impact scales with order size
- **Bid-Ask Bounce**: Prices revert to mid-quote after large moves
- **Volatility Clustering**: High volatility periods cluster together
- **Volume-Price Correlation**: Trading volume predicts price changes

### BAM Grid Constraints
- **10×60 Grid**: 10 time windows × 60 pattern dimensions
- **Binary Attributes**: Each cell is 0 or 1
- **Temporal Consistency**: Patterns must be temporally consistent
- **Cross-Market Correlation**: BAM grids across markets correlate

### Model Architecture
```python
# Physics-informed LSTM with Fisher information
class PhysicsInformedLSTM(nn.Module):
    def __init__(self, input_dim, hidden_dim):
        super().__init__()
        self.lstm = nn.LSTM(input_dim, hidden_dim)
        self.fisher_penalty = 0.0  # Fisher information regularization
        
    def forward(self, x):
        # LSTM forward pass
        output, (h_n, c_n) = self.lstm(x)
        
        # Apply physics constraints
        output = self.apply_no_arbitrage_constraint(output)
        output = self.apply_bid_ask_constraint(output)
        
        return output
    
    def apply_no_arbitrage_constraint(self, predictions):
        # Ensure predictions don't create arbitrage
        # Clamp to realistic ranges
        return torch.clamp(predictions, min=-1.0, max=1.0)
```

## Verification

After implementing a physics-informed model:
- [ ] Model respects all identified physical constraints
- [ ] Loss function includes constraint penalties
- [ ] Model tested against known physical limits
- [ ] No unphysical extrapolation observed
- [ ] Model validated against historical regimes
