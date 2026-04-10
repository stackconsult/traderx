# Strategy Framework Governance

## Specialized Laws for Trading Strategies

### BANNED Patterns
- Direct exchange API calls in strategies
- Hardcoded symbols or parameters
- State persistence (handled by engine)
- Blocking operations in generate_signals
- Access to order book directly (use normalized data)

### REQUIRED Patterns
```python
# Inherit from BaseStrategy
class MyStrategy(BaseStrategy):
    async def generate_signals(self, symbol: str, data: List[List[float]]) -> List[Signal]:
        # Must return Signal objects
        return signals
    
    async def calculate_indicators(self, symbol: str, data: List[List[float]]) -> Dict[str, float]:
        # Calculate technical indicators
        return indicators
```

### Performance Requirements
- Signal generation < 1ms per symbol
- Indicator calculation using vectorized operations
- Memory usage < 100MB per strategy
- No external API calls in hot path

### Data Access
- Market data only through engine-provided OHLCV
- Position state through engine.positions
- No direct database access
- All timestamps in UTC

### Risk Integration
- All signals include position size calculation
- Stop-loss/take-profit in Signal metadata
- Respect position limits from risk manager
- Paper trading validation before live

### Testing Requirements
- Backtest with 6 months historical data
- Sharpe ratio > 0.5 required
- Maximum drawdown < 20%
- Win rate > 40%
