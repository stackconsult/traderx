---
trigger: always_on
description: "Code simplification - prevent over-engineering"
---

# Code Simplification

## 🎯 Core Principle
Choose the simplest implementation that works. Complexity is the enemy of reliability in trading systems.

## 📋 Simplicity Checklist

Before writing code, ask:
- Can this be done with a function instead of a class?
- Can I use a primitive instead of a custom type?
- Do I need this abstraction RIGHT NOW?
- Would a junior developer understand this immediately?

## 🚫 FORBIDDEN PATTERNS

- ❌ Generic frameworks for specific problems
- ❌ Abstract factories for 2-3 instances
- ❌ Event systems for direct function calls
- ❌ Configuration-driven code for fixed logic
- ❌ Premature optimization

## ✅ REQUIRED PATTERNS

### 1. Prefer Concrete Types
```rust
// Bad: Generic abstraction
pub trait ExchangeAdapter {
    fn send_order(&self, order: GenericOrder) -> Result<GenericFill>;
}

// Good: Concrete implementation
pub struct NasdaqAdapter {
    endpoint: String,
}

impl NasdaqAdapter {
    pub fn send_order(&self, order: NasdaqOrder) -> Result<NasdaqFill> {
        // Direct implementation
    }
}
```

### 2. Use Functions Over Classes
```rust
// Bad: Unnecessary class
pub class OrderValidator {
    public function validate(order: Order): bool {
        return order.quantity > 0 && order.price > 0;
    }
}

// Good: Simple function
pub fn validate_order(order: &Order) -> bool {
    order.quantity > Decimal::ZERO && order.price > Some(Decimal::ZERO)
}
```

### 3. Inline Simple Logic
```rust
// Bad: Strategy pattern for two cases
pub enum PricingStrategy {
    Standard,
    Emergency,
}

impl PricingStrategy {
    pub fn calculate(&self, base: f64) -> f64 {
        match self {
            Self::Standard => base * 1.001,
            Self::Emergency => base * 1.0005,
        }
    }
}

// Good: Simple function with parameter
pub fn calculate_price(base: f64, emergency: bool) -> f64 {
    if emergency {
        base * 1.0005
    } else {
        base * 1.001
    }
}
```

## 🔄 Trading System Simplicity Rules

### 1. Direct Data Flow
```rust
// Bad: Complex event bus
pub event_bus.subscribe(OrderEvent::Submitted, |event| {
    risk_bus.process(event);
    portfolio.update(event);
    metrics.record(event);
});

// Good: Direct function calls
pub fn submit_order(&self, order: Order) -> Result<()> {
    self.risk_bus.check(&order)?;
    self.portfolio.add_position(&order)?;
    self.metrics.record_order(&order);
    Ok(())
}
```

### 2. Simple State Management
```rust
// Bad: Complex state machine
pub enum OrderState {
    Created { timestamp: u64 },
    Validated { timestamp: u64, validator: String },
    RiskChecked { timestamp: u64, risk_score: f64 },
    Submitted { timestamp: u64, exchange: String },
}

// Good: Essential states only
pub enum OrderState {
    Pending,
    Submitted,
    Filled,
    Cancelled,
}
```

### 3. Minimal Configuration
```rust
// Bad: Complex config
#[derive(Deserialize)]
pub struct RiskConfig {
    pub position_limits: HashMap<String, f64>,
    pub var_calculations: VarConfig,
    pub stress_scenarios: Vec<StressTest>,
    pub limits_by_time: TimeBasedLimits,
}

// Good: Essential config only
#[derive(Deserialize)]
pub struct RiskConfig {
    pub max_position_usd: f64,
    pub max_drawdown_bps: i32,
}
```

## 📊 Complexity Metrics

### Before Committing:
- [ ] Can I explain this in 30 seconds?
- [ ] Does it have <3 levels of nesting?
- [ ] Does it use <3 abstractions?
- [ ] Would I write this the same way again?

### Red Flags:
- More than 5 parameters in a function
- More than 3 levels of inheritance/traits
- Configuration for behavior not yet needed
- Generic types with only one instantiation

## 🔄 Refactoring to Simplicity

### 1. Extract Only When Needed
```rust
// Start with this
pub fn process_signal(signal: AgentSignal) -> Result<Order> {
    if signal.conviction < 0.5 {
        return Err("Low conviction".into());
    }
    // ... rest of logic
}

// ONLY extract if you need it elsewhere
pub fn validate_conviction(conviction: f64) -> Result<()> {
    if conviction < 0.5 {
        return Err("Low conviction".into());
    }
    Ok(())
}
```

### 2. Remove Unused Abstractions
```rust
// If you see this, question it:
pub trait OrderExecutor {
    fn execute(&self, order: Order) -> Result<Fill>;
}

// And only ONE implementation:
pub struct SimpleExecutor;

impl OrderExecutor for SimpleExecutor {
    fn execute(&self, order: Order) -> Result<Fill> {
        // Just use the function directly
    }
}

// Replace with:
pub fn execute_order(order: Order) -> Result<Fill> {
    // Direct implementation
}
```

## 🎯 Success Criteria

- New team members understand code in <1 hour
- No "magic" abstractions
- Direct execution paths
- Minimal configuration
- Tests are straightforward

## 📋 Code Review Checklist

For Every PR:
- [ ] Is there a simpler way to do this?
- [ ] Are all abstractions necessary?
- [ ] Can we remove any configuration?
- [ ] Is this over-engineered for the current need?
- [ ] Would a junior developer understand this?
