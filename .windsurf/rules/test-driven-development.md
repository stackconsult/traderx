---
trigger: always_on
description: "Test-driven development - write tests before infrastructure"
---

# Test-Driven Development for Trading Systems

## 🎯 Core Principle
Write the TEST that proves functionality BEFORE writing the functionality. Tests must use REAL components, not mocks.

## 📋 TDD Workflow

### 1. Red: Write a Failing Test
```rust
#[tokio::test]
async fn test_signal_to_order_conversion() {
    // Arrange
    let risk_bus = RiskBus::new(1_000_000.0, -2000);
    let (oms_tx, _) = mpsc::channel(100);
    let router = SignalRouter::new(config, risk_bus, oms_tx);
    
    let signal = AgentSignal {
        symbol: "AAPL".to_string(),
        direction: "long".to_string(),
        conviction: 0.7,
        max_notional: 10_000.0,
        // ...
    };
    
    // Act
    let outcome = router.route_signal(signal);
    
    // Assert
    assert_eq!(outcome.status, RouteStatus::Submitted);
    assert!(outcome.order_id.is_some());
}
```

### 2. Green: Make Test Pass
- Write MINIMAL code to make test pass
- NO extra features
- NO optimizations

### 3. Refactor
- Improve code while keeping test green
- Remove duplication
- Maintain functionality

## 🚫 FORBIDDEN PATTERNS

- ❌ Writing production code before test
- ❌ Using mocks for core trading logic
- ❌ Testing infrastructure instead of functionality
- ❌ Writing tests that always pass

## ✅ REQUIRED PATTERNS

### 1. Integration Tests First
```rust
// Test REAL signal flow, not individual components
#[tokio::test]
async fn test_complete_trading_flow() {
    // 1. Create all components
    // 2. Wire them together
    // 3. Send real signal
    // 4. Verify order created
    // 5. Simulate fill
    // 6. Verify P&L updated
}
```

### 2. Performance Tests in TDD
```rust
#[tokio::test]
async fn test_risk_check_performance() {
    let start = Instant::now();
    
    // Run 10,000 risk checks
    for _ in 0..10_000 {
        risk_bus.check_symbol("AAPL", 1000.0);
    }
    
    let duration = start.elapsed();
    assert!(duration.as_nanos() / 10_000 < 100); // <100ns per check
}
```

### 3. Error Case Testing
```rust
#[tokio::test]
async fn test_risk_limit_enforcement() {
    // Exceed position limit
    let result = risk_bus.check_symbol("AAPL", 2_000_000.0);
    assert!(result.is_err());
    
    // Verify specific error
    assert_eq!(result.unwrap_err(), "Position limit exceeded");
}
```

## 🔄 Trading System TDD Rules

### 1. Test the Money Flow
- Every test must move value through the system
- Verify P&L calculations
- Check risk enforcement

### 2. Test Failure Modes
- Market data disconnect
- Order rejection
- Risk limit breach
- Network partition

### 3. Test Performance Continuously
- Every PR must include performance regression test
- Latency SLOs are part of test suite
- Throughput validated under load

## 📊 Test Organization

```
tests/
├── integration/
│   ├── trading_flow.rs      # Complete signal → P&L
│   ├── risk_enforcement.rs  # Risk limit tests
│   └── recovery.rs         # Crash recovery tests
├── performance/
│   ├── latency.rs          # SLO validation
│   └── throughput.rs       # Load tests
└── unit/
    ├── risk_bus.rs         # Individual component tests
    └── portfolio.rs        # P&L calculation tests
```

## 🎯 Success Criteria

Before adding ANY infrastructure:
- [ ] Core trading flow has passing integration test
- [ ] Performance tests meet SLOs
- [ ] Error cases are covered
- [ ] Tests run in CI/CD pipeline

## 📋 Pre-Infrastructure Checklist

- [ ] Can I run `cargo test` and see the trading flow work?
- [ ] Do performance tests pass?
- [ ] Are all failure modes tested?
- [ ] Does the test use REAL components?

If NO to any question → NO INFRASTRUCTURE until tests pass!
