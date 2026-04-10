# Core Engine Governance

## Specialized Laws for Trading Engine

### BANNED Patterns
- Synchronous exchange calls in event loop
- Order execution without risk validation
- Direct database access (use DataStorage)
- Unhandled exceptions in main trading loop
- State mutations without persistence

### REQUIRED Patterns
```python
# All exchange operations must be async
await exchange.submit_order(order)

# Risk validation before every order
if not await risk_manager.validate_order(order, positions):
    return False

# State persistence after changes
await data_storage.save_order(order)
await data_storage.save_position(position)
```

### Performance Requirements
- Order processing < 10ms average
- Position updates < 5ms average
- Memory usage < 1GB for 1000 active positions
- Zero message loss in order queue

### Error Handling
- All exchange errors logged with order ID
- Failed orders marked REJECTED immediately
- Circuit breaker on >5 consecutive failures
- Automatic retry with exponential backoff

### State Management
- Single source of truth: engine.positions
- All state changes through engine methods
- Atomic updates for position/order pairs
- Recovery from persisted state on restart
