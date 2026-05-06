# Integration Testing Skill

---

name: integration-testing
description: Writes integration tests that verify end-to-end workflows with REAL components, not mocks. Use when testing complete flows, cross-component communication, or system-wide behavior.

---

## When to Activate

Use when:
- Testing complete trading flows (Signal → Order → Fill → P&L)
- Testing cross-component communication
- Testing failure modes and recovery
- Testing system-wide behavior
- Verifying components work together in realistic scenarios

## Core Principles

### Real Components Only
Integration tests MUST use REAL components, not mocks. This is non-negotiable for TraderX.

- ✅ Use real RiskBus for risk checks
- ✅ Use real Portfolio for position tracking
- ✅ Use real OMS for order lifecycle
- ✅ Use real Exchange for order execution (paper trading mode)
- ❌ NO mocks for core trading logic

### End-to-End Testing
Test complete flows from input to output:

1. **Signal → Order → Fill → P&L**
   - Generate a trading signal
   - Route through RiskBus
   - Submit to OMS
   - Simulate fill
   - Verify P&L calculation

2. **Market Data → Signal Generation**
   - Ingest market data
   - Run signal generation
   - Verify signal quality
   - Check for signal failures

3. **Failure Mode Testing**
   - Simulate market data disconnect
   - Simulate order rejection
   - Simulate risk limit breach
   - Verify system recovers correctly

### TraderX-Specific Integration Tests

#### Signal Routing Integration
```rust
#[tokio::test]
async fn test_signal_to_order_integration() {
    // Arrange
    let risk_bus = RiskBus::new(1_000_000.0, -2000);
    let (oms_tx, mut oms_rx) = mpsc::channel(100);
    let router = SignalRouter::new(config, risk_bus, oms_tx);
    
    // Act
    let signal = AgentSignal {
        symbol: "AAPL".to_string(),
        direction: "long".to_string(),
        conviction: 0.7,
        max_notional: 10_000.0,
    };
    let outcome = router.route_signal(signal);
    
    // Assert
    assert_eq!(outcome.status, RouteStatus::Submitted);
    assert!(outcome.order_id.is_some());
    
    // Verify order was actually submitted to OMS
    let order = oms_rx.recv().await.unwrap();
    assert_eq!(order.symbol, "AAPL");
}
```

#### Risk Check Integration
```rust
#[tokio::test]
async fn test_risk_limit_enforcement_integration() {
    // Arrange
    let risk_bus = RiskBus::new(100_000.0, -2000); // Low capital
    
    // Act
    let result = risk_bus.check_symbol("AAPL", 200_000.0);
    
    // Assert
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Position limit exceeded");
}
```

## Implementation Checklist

- [ ] Identify complete flow to test
- [ ] Use REAL components (no mocks)
- [ ] Arrange-Act-Assert pattern
- [ ] Test success case
- [ ] Test failure cases
- [ ] Test edge cases
- [ ] Verify system recovers from failures

## Common Pitfalls

- ❌ Using mocks for core components → use REAL components only
- ❌ Testing in isolation → test complete flows
- ❌ Not testing failure modes → always test failures
- ❌ Hardcoding test data → use realistic market data

## TraderX-Specific Adaptations

### Complete Trading Flow Test
```rust
#[tokio::test]
async fn test_complete_trading_flow() {
    // 1. Generate signal
    let signal = generate_test_signal();
    
    // 2. Route through RiskBus
    let risk_bus = RiskBus::new(1_000_000.0, -2000);
    let router = SignalRouter::new(config, risk_bus, oms_tx);
    let outcome = router.route_signal(signal);
    assert_eq!(outcome.status, RouteStatus::Submitted);
    
    // 3. Submit to OMS
    let order = oms_rx.recv().await.unwrap();
    let order_id = order.id;
    
    // 4. Simulate fill
    let fill = Fill {
        order_id,
        price: 150.0,
        quantity: 100.0,
        timestamp: Utc::now(),
    };
    
    // 5. Update portfolio
    portfolio.add_fill(fill);
    
    // 6. Verify P&L
    let pnl = portfolio.calculate_pnl();
    assert!(pnl > 0.0); // Should be profitable for this test
}
```

### Failure Mode Testing
```rust
#[tokio::test]
async fn test_market_data_disconnect_recovery() {
    // Arrange
    let market_data = MarketDataClient::new();
    
    // Act: Simulate disconnect
    market_data.simulate_disconnect();
    
    // Assert: System should detect and handle
    let status = market_data.get_status();
    assert_eq!(status, ConnectionStatus::Disconnected);
    
    // Act: Reconnect
    market_data.simulate_reconnect();
    
    // Assert: System should recover
    let status = market_data.get_status();
    assert_eq!(status, ConnectionStatus::Connected);
}
```

## Verification

After writing integration tests:
- [ ] All tests use REAL components (no mocks)
- [ ] Complete flows tested end-to-end
- [ ] Failure modes tested
- [ ] System recovery verified
- [ ] Tests run in CI/CD pipeline
- [ ] Test coverage for integration is complete
