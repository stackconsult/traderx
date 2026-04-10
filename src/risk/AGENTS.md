# Risk Management Governance

## Specialized Laws for Risk Manager

### BANNED Patterns
- Risk checks after order submission
- Silent risk limit breaches
- Manual override of circuit breaker
- Risk parameters in code (use configuration)
- Asynchronous risk validation

### REQUIRED Patterns
```python
# Synchronous validation before order submission
if not risk_manager.validate_order(order, positions):
    return False

# Immediate circuit breaker activation
if daily_pnl < -max_daily_loss:
    risk_manager.activate_circuit_breaker("Daily loss exceeded")

# Real-time position monitoring
await risk_manager.update_metrics(positions, balance)
```

### Critical Risk Limits
- Maximum position size per symbol
- Daily loss limit (hard stop)
- Maximum drawdown threshold
- Leverage ratio limits
- Order frequency limits

### Performance Requirements
- Order validation < 100 microseconds
- Position updates < 50 microseconds
- Risk metrics calculation < 10ms
- Circuit breaker activation < 1ms

### Monitoring Requirements
- Real-time P&L tracking
- Position exposure monitoring
- VaR calculation every 5 minutes
- Alert on limit breaches

### Compliance Rules
- All risk changes audited
- Circuit breaker events logged
- Risk parameters version controlled
- Daily risk reports generated
