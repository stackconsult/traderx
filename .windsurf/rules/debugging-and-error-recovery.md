---
trigger: always_on
description: "Systematic debugging and error recovery for trading systems"
---

# Debugging and Error Recovery

## 🎯 Core Principle
Every failure must have a systematic debugging approach and documented recovery procedure.

## 🔄 Debugging Workflow

### 1. Reproduce Consistently
```bash
# Save exact state
git stash
git checkout <commit-hash>
cargo run --bin main -- --test-scenario signal_rejection

# Capture logs
RUST_LOG=debug cargo run 2>&1 | tee debug.log
```

### 2. Isolate the Component
```rust
// Test each component in isolation
#[tokio::test]
async fn debug_risk_bus_isolation() {
    let risk_bus = RiskBus::new(1_000_000.0, -2000);
    
    // Test 1: Basic check
    assert!(risk_bus.check_symbol("AAPL", 1000.0).is_ok());
    
    // Test 2: Boundary condition
    assert!(risk_bus.check_symbol("AAPL", 2_000_000.0).is_err());
    
    // Test 3: State consistency
    assert!(!risk_bus.is_halted());
}
```

### 3. Trace Data Flow
```rust
// Add tracing at each boundary
#[tracing::instrument(level = "debug", skip(self))]
pub fn route_signal(&self, signal: AgentSignal) -> RouteOutcome {
    debug!("Routing signal: {} {}", signal.symbol, signal.direction);
    
    // Trace each step
    let validated = self.validate_signal(&signal)?;
    debug!("Signal validated: {:?}", validated);
    
    let risk_result = self.risk_bus.check_symbol(&signal.symbol, notional)?;
    debug!("Risk check result: {:?}", risk_result);
    
    // ...
}
```

## 🚫 FORBIDDEN DEBUGGING PATTERNS

- ❌ Adding debug prints without tracing
- ❌ Ignoring "impossible" errors
- ❌ Assuming state without verification
- ❌ Fixing symptoms without root cause

## ✅ REQUIRED DEBUGGING PATTERNS

### 1. Structured Error Handling
```rust
#[derive(Error, Debug)]
pub enum TradingError {
    #[error("Risk check failed for {symbol}: {reason}")]
    RiskCheckFailed { symbol: String, reason: String },
    
    #[error("Order validation failed: {field} = {value}")]
    ValidationFailed { field: String, value: String },
    
    #[error("Portfolio update failed: {0}")]
    PortfolioUpdate(#[from] PortfolioError),
}

// Always log with context
match result {
    Ok(_) => info!("Operation completed successfully"),
    Err(e) => error!(error = %e, "Operation failed"),
}
```

### 2. State Verification Checkpoints
```rust
impl OmsEngine {
    pub fn verify_state_consistency(&self) -> StateReport {
        StateReport {
            total_orders: self.orders.len(),
            pending_orders: self.count_orders_by_state(OrderState::Pending),
            risk_bus_halted: self.risk_bus.is_halted(),
            portfolio_nav: self.portfolio.get_total_nav(),
        }
    }
}
```

### 3. Recovery Procedures
```rust
// Automatic recovery from journal
pub async fn recover_from_journal(&self) -> Result<RecoveryReport> {
    info!("Starting journal recovery");
    
    let entries = self.journal.read_all()?;
    let mut recovered = 0;
    let mut failed = 0;
    
    for entry in entries {
        match self.apply_journal_entry(entry).await {
            Ok(_) => recovered += 1,
            Err(e) => {
                error!(error = %e, "Failed to apply journal entry");
                failed += 1;
            }
        }
    }
    
    info!("Recovery complete: {} recovered, {} failed", recovered, failed);
    Ok(RecoveryReport { recovered, failed })
}
```

## 📊 Trading System Debugging Rules

### 1. Money Flow Verification
Always trace:
- Signal → Order conversion
- Order → Fill execution
- Fill → Position update
- Position → P&L calculation

### 2. Performance Debugging
```rust
// Measure each critical path
#[tracing::instrument(level = "trace")]
pub fn check_symbol(&self, symbol: &str, notional: f64) -> Result<()> {
    let start = Instant::now();
    
    let result = self.internal_check(symbol, notional);
    
    let duration = start.elapsed();
    if duration.as_nanos() > 100 {
        warn!(
            symbol = %symbol,
            notional,
            duration_ns = duration.as_nanos(),
            "Risk check took longer than 100ns"
        );
    }
    
    result
}
```

### 3. Concurrent Debugging
```rust
// Use atomic operations for debugging
static DEBUG_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn process_signal(&self, signal: AgentSignal) {
    let id = DEBUG_COUNTER.fetch_add(1, Ordering::SeqCst);
    debug!(signal_id = id, "Processing signal");
    
    // Process...
    
    debug!(signal_id = id, "Signal processing complete");
}
```

## 🔄 Error Recovery Strategies

### 1. Circuit Breaker Pattern
```rust
pub struct CircuitBreaker {
    failure_count: AtomicU64,
    last_failure: AtomicU64,
    state: AtomicU8, // 0=Closed, 1=Open, 2=Half-Open
}

impl CircuitBreaker {
    pub async fn call<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        if self.is_open() {
            return Err(Error::CircuitBreakerOpen);
        }
        
        match f() {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(e) => {
                self.on_failure();
                Err(e)
            }
        }
    }
}
```

### 2. Graceful Degradation
```rust
pub fn route_signal(&self, signal: AgentSignal) -> RouteOutcome {
    // Try full risk check
    match self.risk_bus.check_symbol(&signal.symbol, notional) {
        Ok(_) => {
            // Full processing
            self.full_route(signal)
        }
        Err(_) if self.emergency_mode => {
            // Emergency mode: simple validation only
            warn!("Operating in emergency mode - simplified routing");
            self.emergency_route(signal)
        }
        Err(e) => {
            // Normal failure
            RouteOutcome {
                status: RouteStatus::Rejected,
                reason: format!("Risk check failed: {}", e),
                order_id: None,
            }
        }
    }
}
```

## 📋 Debugging Checklist

For Every Bug:
- [ ] Can I reproduce it consistently?
- [ ] Do I have logs with tracing enabled?
- [ ] Have I isolated the failing component?
- [ ] Is the data flow documented?
- [ ] Do I have a recovery test case?

## 🎯 Success Metrics

- Mean Time to Detection (MTTD) < 5 minutes
- Mean Time to Recovery (MTTR) < 15 minutes
- 100% of failures have documented recovery
- All critical paths have tracing
