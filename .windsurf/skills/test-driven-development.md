# Test-Driven Development (TDD) Skill

## Description

Systematic TDD workflow for Rust/Python trading systems with red-green-refactor cycles, property-based testing, and comprehensive coverage analysis. Ensures code reliability and maintainability in high-frequency trading environments.

**Source**: Adapted from `impeccable` testing patterns and Rust best practices  
**Repository**: Internal TDD workflow for TraderX  
**Applies to**: Rust (cargo test), Python (pytest), HFT systems

---

## Why TDD for Trading Systems

### **Critical Requirements**
1. **Zero-defect tolerance**: Trading bugs cost money immediately
2. **Regression safety**: Changes must not break existing strategies
3. **Refactoring confidence**: Clean code evolution without fear
4. **Documentation**: Tests serve as executable specifications
5. **Performance validation**: Latency requirements must be verified

### **Benefits for HFT**
- 🎯 **Catch bugs before they cost money**
- 🎯 **Ensure latency guarantees are met**
- 🎯 **Validate risk management logic**
- 🎯 **Document trading rules in code**

---

## Core TDD Workflow

### **The Red-Green-Refactor Cycle**

```
┌─────────┐    ┌─────────┐    ┌─────────┐
│   RED   │ → │  GREEN  │ → │ REFACTOR│
│ (write  │    │ (make   │    │ (clean  │
│  test)  │    │  it     │    │  up)    │
│         │    │  pass)  │    │         │
└─────────┘    └─────────┘    └─────────┘
      ↑                              ↓
      └──────────────────────────────┘
```

---

## Phase 1: Red (Write Failing Test)

### **Step 1: Define the Behavior**

**Before writing code**, clearly specify:
- What input should the component receive?
- What output should it produce?
- What are the error conditions?
- What are the performance requirements?

### **Step 2: Write the Test**

#### **Rust Example: Risk Manager**

```rust
// tests/risk_manager_test.rs
#[cfg(test)]
mod risk_manager_tests {
    use traderx::risk::{RiskManager, RiskCheck, RiskResult};
    use rust_decimal::Decimal;

    #[test]
    fn should_reject_order_exceeding_position_limit() {
        // Arrange
        let risk_manager = RiskManager::new()
            .with_position_limit("BTC-USD", Decimal::from(10)); // 10 BTC max
        
        let order = Order {
            symbol: "BTC-USD",
            side: Side::Buy,
            quantity: Decimal::from(15), // Exceeds 10 BTC limit
            price: Decimal::from(50000),
        };

        // Act
        let result = risk_manager.check_order(&order);

        // Assert
        assert_eq!(result, RiskResult::Reject("Position limit exceeded".into()));
    }

    #[test]
    fn should_accept_order_within_position_limit() {
        // Arrange
        let risk_manager = RiskManager::new()
            .with_position_limit("BTC-USD", Decimal::from(10));
        
        let order = Order {
            symbol: "BTC-USD",
            side: Side::Buy,
            quantity: Decimal::from(5), // Within limit
            price: Decimal::from(50000),
        };

        // Act
        let result = risk_manager.check_order(&order);

        // Assert
        assert_eq!(result, RiskResult::Accept);
    }

    #[test]
    fn should_track_running_position() {
        // Arrange
        let mut risk_manager = RiskManager::new()
            .with_position_limit("BTC-USD", Decimal::from(10));
        
        // Act - first order
        let order1 = Order {
            symbol: "BTC-USD",
            side: Side::Buy,
            quantity: Decimal::from(6),
            price: Decimal::from(50000),
        };
        risk_manager.check_order(&order1);

        // Second order that would exceed limit when combined
        let order2 = Order {
            symbol: "BTC-USD",
            side: Side::Buy,
            quantity: Decimal::from(5), // 6 + 5 = 11 > 10
            price: Decimal::from(51000),
        };
        let result = risk_manager.check_order(&order2);

        // Assert
        assert_eq!(result, RiskResult::Reject("Position limit exceeded".into()));
    }
}
```

#### **Python Example: Market Data Interpreter**

```python
# tests/test_market_data_interpreter.py
import pytest
from traderx.ai_agents import MarketDataInterpreter, MarketInsight

class TestMarketDataInterpreter:
    """TDD test suite for LLM market data interpretation"""

    def test_should_generate_bullish_signal_on_positive_volume_price_divergence(self):
        """Test that positive volume/price divergence generates bullish insight"""
        # Arrange
        interpreter = MarketDataInterpreter()
        market_data = {
            "symbol": "AAPL",
            "price_change": +2.5,  # +2.5%
            "volume_change": +150,  # 150% of average
            "rsi": 65,
            "timestamp": "2026-04-15T14:30:00Z"
        }

        # Act
        insight: MarketInsight = interpreter.interpret(market_data)

        # Assert
        assert insight.sentiment == "bullish"
        assert insight.confidence > 0.7
        assert "volume" in insight.reasoning.lower()
        assert "price" in insight.reasoning.lower()

    def test_should_generate_bearish_signal_on_high_volume_sell_pressure(self):
        """Test that high volume with negative price = bearish"""
        # Arrange
        interpreter = MarketDataInterpreter()
        market_data = {
            "symbol": "TSLA",
            "price_change": -3.2,
            "volume_change": +200,  # 200% volume
            "rsi": 35,
            "timestamp": "2026-04-15T14:30:00Z"
        }

        # Act
        insight = interpreter.interpret(market_data)

        # Assert
        assert insight.sentiment == "bearish"
        assert insight.confidence > 0.6
        assert "sell pressure" in insight.reasoning.lower()

    def test_should_return_low_confidence_on_insufficient_data(self):
        """Test that incomplete data results in low confidence"""
        # Arrange
        interpreter = MarketDataInterpreter()
        incomplete_data = {
            "symbol": "INVALID",
            # Missing price_change, volume_change
        }

        # Act
        insight = interpreter.interpret(incomplete_data)

        # Assert
        assert insight.confidence < 0.3
        assert "insufficient data" in insight.reasoning.lower()
```

### **Step 3: Run the Test (Watch it FAIL)**

```bash
# Rust
cargo test --test risk_manager_test

# Expected output:
# running 3 tests
# test should_reject_order_exceeding_position_limit ... FAILED
# test should_accept_order_within_position_limit ... FAILED
# test should_track_running_position ... FAILED
#
# failures:
#   thread 'risk_manager_tests::should_reject_order_exceeding_position_limit' panicked at 
#   'RiskManager does not exist'
```

✅ **RED phase complete** - Test fails as expected

---

## Phase 2: Green (Make it Pass)

### **Step 4: Write Minimal Implementation**

**Only write enough code to make the test pass.**

#### **Rust: Risk Manager Implementation**

```rust
// src/risk.rs
use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum RiskResult {
    Accept,
    Reject(String),
}

pub struct RiskManager {
    position_limits: HashMap<String, Decimal>,
    current_positions: HashMap<String, Decimal>,
}

impl RiskManager {
    pub fn new() -> Self {
        RiskManager {
            position_limits: HashMap::new(),
            current_positions: HashMap::new(),
        }
    }

    pub fn with_position_limit(mut self, symbol: &str, limit: Decimal) -> Self {
        self.position_limits.insert(symbol.to_string(), limit);
        self
    }

    pub fn check_order(&mut self, order: &Order) -> RiskResult {
        let symbol = &order.symbol;
        let limit = match self.position_limits.get(symbol) {
            Some(l) => l,
            None => return RiskResult::Accept, // No limit set
        };

        let current = self.current_positions.get(symbol).unwrap_or(&Decimal::ZERO);
        let new_position = current + order.quantity;

        if new_position > *limit {
            return RiskResult::Reject("Position limit exceeded".into());
        }

        // Update position tracking
        self.current_positions.insert(symbol.clone(), new_position);
        
        RiskResult::Accept
    }
}
```

#### **Python: Market Data Interpreter**

```python
# src/ai_agents/market_data_interpreter.py
from dataclasses import dataclass
from typing import Dict

@dataclass
class MarketInsight:
    sentiment: str  # "bullish", "bearish", "neutral"
    confidence: float  # 0.0 - 1.0
    reasoning: str
    symbol: str

class MarketDataInterpreter:
    """LLM-based market data interpretation agent"""

    def __init__(self):
        self.min_confidence_threshold = 0.3

    def interpret(self, market_data: Dict) -> MarketInsight:
        """Generate market insight from raw data"""
        
        # Check for required fields
        required_fields = ["price_change", "volume_change", "symbol"]
        missing = [f for f in required_fields if f not in market_data]
        
        if missing:
            return MarketInsight(
                sentiment="neutral",
                confidence=0.1,
                reasoning=f"Insufficient data: missing {missing}",
                symbol=market_data.get("symbol", "UNKNOWN")
            )

        price_change = market_data["price_change"]
        volume_change = market_data["volume_change"]
        symbol = market_data["symbol"]

        # Bullish: positive price + high volume
        if price_change > 0 and volume_change > 100:
            return MarketInsight(
                sentiment="bullish",
                confidence=min(0.9, 0.5 + (volume_change / 1000)),
                reasoning=f"Strong bullish signal: {price_change}% price increase with {volume_change}% volume surge",
                symbol=symbol
            )

        # Bearish: negative price + high volume
        if price_change < 0 and volume_change > 100:
            return MarketInsight(
                sentiment="bearish",
                confidence=min(0.8, 0.5 + (abs(price_change) / 10)),
                reasoning=f"Sell pressure detected: {price_change}% price drop with {volume_change}% volume surge",
                symbol=symbol
            )

        # Default: neutral
        return MarketInsight(
            sentiment="neutral",
            confidence=0.4,
            reasoning=f"No clear signal: {price_change}% price change, {volume_change}% volume change",
            symbol=symbol
        )
```

### **Step 5: Run Tests (Watch them PASS)**

```bash
# Rust
cargo test --test risk_manager_test

# Expected output:
# running 3 tests
# test should_reject_order_exceeding_position_limit ... ok
# test should_accept_order_within_position_limit ... ok
# test should_track_running_position ... ok
#
# test result: ok. 3 passed; 0 failed
```

✅ **GREEN phase complete** - All tests pass

---

## Phase 3: Refactor (Clean Up)

### **Step 6: Improve Without Breaking Tests**

#### **Refactoring Principles**

1. **DRY**: Don't Repeat Yourself
2. **Single Responsibility**: Each function does one thing
3. **Clear Names**: Variables and functions explain themselves
4. **Remove Duplication**: Extract common patterns
5. **Add Documentation**: Document WHY, not WHAT

#### **Refactored Rust Example**

```rust
// src/risk.rs - Refactored with better structure
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Result of a risk check on an order
#[derive(Debug, PartialEq, Clone)]
pub enum RiskResult {
    /// Order passes all risk checks
    Accept,
    /// Order rejected with reason
    Reject(String),
}

/// Manages position limits and risk checks for trading
pub struct RiskManager {
    /// Maximum allowed position per symbol
    position_limits: HashMap<String, Decimal>,
    /// Current tracked positions
    current_positions: HashMap<String, Decimal>,
}

impl RiskManager {
    /// Create new RiskManager with empty limits
    pub fn new() -> Self {
        RiskManager {
            position_limits: HashMap::new(),
            current_positions: HashMap::new(),
        }
    }

    /// Set position limit for a symbol (builder pattern)
    pub fn with_position_limit(mut self, symbol: &str, limit: Decimal) -> Self {
        self.position_limits.insert(symbol.to_string(), limit);
        self
    }

    /// Check if order passes risk criteria
    /// 
    /// # Arguments
    /// * `order` - The order to validate
    /// 
    /// # Returns
    /// * `RiskResult::Accept` if order passes
    /// * `RiskResult::Reject(reason)` if order fails
    pub fn check_order(&mut self, order: &Order) -> RiskResult {
        if let Some(limit) = self.position_limits.get(&order.symbol) {
            let current = self.current_position(&order.symbol);
            let projected = current + order.quantity;

            if projected > *limit {
                return RiskResult::Reject(
                    format!("Position limit exceeded: {} > {}", projected, limit)
                );
            }

            self.update_position(&order.symbol, projected);
        }

        RiskResult::Accept
    }

    /// Get current position for symbol
    fn current_position(&self, symbol: &str) -> Decimal {
        self.current_positions
            .get(symbol)
            .copied()
            .unwrap_or(Decimal::ZERO)
    }

    /// Update position for symbol
    fn update_position(&mut self, symbol: &str, new_position: Decimal) {
        self.current_positions.insert(symbol.to_string(), new_position);
    }
}

/// Trading order structure
#[derive(Debug, Clone)]
pub struct Order {
    pub symbol: String,
    pub side: Side,
    pub quantity: Decimal,
    pub price: Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}
```

### **Step 7: Run Tests Again (Verify Still Pass)**

```bash
cargo test --test risk_manager_test

# Expected: All tests still pass after refactoring
```

✅ **REFACTOR phase complete** - Code is cleaner, tests still pass

---

## Advanced TDD Techniques

### **1. Property-Based Testing**

```rust
// tests/property_based_tests.rs
use proptest::prelude::*;

proptest! {
    #[test]
    fn risk_manager_never_accepts_beyond_limit(
        initial_position in 0.0..100.0f64,
        order_size in 0.0..200.0f64,
        limit in 10.0..100.0f64
    ) {
        let mut manager = RiskManager::new()
            .with_position_limit("TEST", Decimal::from_f64(limit).unwrap());
        
        // Setup initial position
        manager.current_positions.insert(
            "TEST".to_string(), 
            Decimal::from_f64(initial_position).unwrap()
        );

        let order = Order {
            symbol: "TEST".to_string(),
            side: Side::Buy,
            quantity: Decimal::from_f64(order_size).unwrap(),
            price: Decimal::from(100),
        };

        let result = manager.check_order(&order);

        // Property: If accepted, position must be ≤ limit
        if result == RiskResult::Accept {
            let final_position = manager.current_positions.get("TEST").unwrap();
            prop_assert!(final_position <= &Decimal::from_f64(limit).unwrap());
        }
    }
}
```

### **2. Test Coverage Requirements**

```bash
# Rust with tarpaulin
cargo tarpaulin --out Html

# Python with pytest-cov
pytest --cov=traderx --cov-report=html --cov-fail-under=80
```

**Coverage Targets**:
- Core trading logic: **95%+**
- Risk management: **100%** (critical)
- AI agents: **80%+**
- Infrastructure: **70%+**

### **3. Performance Testing in TDD**

```rust
// tests/performance_tests.rs
use std::time::{Duration, Instant};

#[test]
fn risk_check_latency_under_1_microsecond() {
    let mut manager = create_test_manager();
    let order = create_test_order();

    let start = Instant::now();
    let iterations = 100_000;
    
    for _ in 0..iterations {
        let _ = manager.check_order(&order);
    }

    let elapsed = start.elapsed();
    let avg_latency = elapsed / iterations;

    assert!(
        avg_latency < Duration::from_micros(1),
        "Average latency {:?} exceeds 1μs limit",
        avg_latency
    );
}
```

---

## TDD Workflow Integration

### **Session-Start Workflow Integration**

Update `.windsurf/workflows/session-start.md`:

```markdown
### TDD Pre-flight Check
- [ ] Run `cargo test` - all tests pass?
- [ ] Check coverage report - above thresholds?
- [ ] Identify test gaps from recent changes
- [ ] Write failing test before new feature
```

### **Quality Guardian Integration**

Update `.windsurf/workflows/quality-guardian.md`:

```markdown
### Test Quality Gates
1. **Coverage Gate**: Minimum 80% coverage
2. **Performance Gate**: Critical paths < 1μs
3. **Regression Gate**: All tests must pass
4. **Property Gate**: Property-based tests for core logic
```

---

## Common TDD Anti-Patterns to Avoid

### ❌ **Testing Implementation Details**
```rust
// BAD - Testing internal state
test_risk_manager_hashmap_is_empty()

// GOOD - Testing behavior
test_new_order_within_limit_is_accepted()
```

### ❌ **Mocking Everything**
```rust
// BAD - All mocked, no integration
test_with_all_mocks()

// GOOD - Test real components, mock only external services
test_with_real_database_mock_external_api()
```

### ❌ **Skipping the Red Phase**
```rust
// BAD - Writing implementation before test
fn complex_function() { /* lots of code */ }

// Write test first, see it fail!
```

---

## Success Metrics

### **After Adopting TDD Skill**:

| Metric | Before TDD | After TDD | Target |
|--------|-----------|-----------|--------|
| **Bug rate in production** | High | Low | **< 1 critical/month** |
| **Code coverage** | 40% | 85%+ | **> 80%** |
| **Time to add feature** | Variable | Predictable | **< 2 days** |
| **Refactoring confidence** | Low | High | **Anytime** |
| **Documentation quality** | Poor | Good | **Self-documenting** |
| **Onboarding time** | Long | Short | **< 1 week** |

---

## Integration with Existing Skills

### **Works With**:
- ✅ `audit-compliance.md` - Tests serve as compliance evidence
- ✅ `harden.md` - Security tests embedded in TDD
- ✅ `optimize.md` - Performance tests part of TDD cycle
- ✅ `hexagonal-adapters.md` - Port interfaces tested first

### **Enables**:
- 🚀 Safe refactoring of critical trading code
- 🚀 Confident deployment of new strategies
- 🚀 Fast iteration on AI agent improvements
- 🚀 Reliable backtesting validation

---

## Getting Started

### **For Your Next Task**:

1. **Identify the feature** to implement
2. **Write ONE failing test** describing the behavior
3. **Run the test** - watch it fail (RED)
4. **Write minimal code** to make it pass (GREEN)
5. **Refactor** the code (REFACTOR)
6. **Repeat** for next behavior

### **First Week Goals**:
- [ ] Write 10 tests for existing untested code
- [ ] Achieve 60% coverage on one module
- [ ] Practice red-green-refactor 3 times daily
- [ ] Review test quality with team

---

**This skill is critical for TraderX engineering quality. Apply it to every code change.**
