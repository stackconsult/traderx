# DeltaLag Signal Logic Skill

## Description
Dynamic lead-lag detection using sparsified cross-attention mechanism. Discovers time-varying, non-linear interactions between markets with learnable time deltas (Δ).

## Source
- Repository: SaizhuoWang/quantbench
- File: q4l/model/zoo/spatial/base.py

## Implementation Pattern

### Core Architecture
```python
class DeltaLagModel:
    def __init__(self, max_lag_hours: int = 24):
        self.attention_weights = {}  # Learned cross-attention scores
        self.lag_deltas = {}        # Pair-specific lag values
        
    def forward(self, leader_features, follower_features):
        """Compute cross-attention with learnable lag"""
        # Apply temporal lag to leader features
        lagged_leader = self.apply_lag(leader_features, self.delta)
        # Compute sparse cross-attention
        attention = self.sparse_attention(lagged_leader, follower_features)
        return attention
```

### Activation Logic
1. **Calculate Information Coefficient (IC)**:
   ```
   IC = Correlation(Signal_t, Return_{t+21})
   ```

2. **Check Persistence Filter (H)**:
   - H > 0.5: Trending regime (activate)
   - H < 0.5: Mean-reverting (hibernate)
   - H ≈ 0.5: Random walk (monitor only)

3. **VPIN Toxicity Check**:
   - Spiking volume detected → Freeze all updates

4. **Deflated Sharpe Ratio (DSR)**:
   - Verify DSR > threshold before execution

### Market Regime Matrix
| Regime | Hurst (H) | System Action |
|--------|-----------|---------------|
| Persistent/Trending | H > 0.5 | Activate DeltaLag |
| Choppy/Mean-Reverting | H < 0.5 | Throttle execution |
| Random Walk | H ≈ 0.5 | Hibernate agents |
| Toxic Flow | VPIN spike | Immediate freeze |

### SugaFormer Integration
- Daily adaptive selection of leader-lagger pairs
- Dynamic attention weight updates
- Multi-head attention for multiple market pairs

### Performance Targets
- RankIC > 10% on NYSE backtest
- Sub-millisecond pattern matching
- Support for 12+ simultaneous market pairs

### Integration Points
- HSTR provides historical context
- VPIN monitors toxicity
- PTP ensures temporal alignment
- Strategy OS executes based on signals

## Usage Example
```python
deltalag = DeltaLagModel()
signal = deltalag.generate_signal("SPY", "DAX")
if signal.ic > 0.05 and signal.hurst > 0.5:
    execute_pair_trade(signal)
```

## Notes
- Replaces static cross-correlation with dynamic learning
- Critical for capturing evolving market relationships
- Must be paired with strict risk controls
