---
trigger: always_on
description: "API and interface design - define clean component boundaries"
---

# API and Interface Design

## 🎯 Core Principle
Design interfaces that are:
1. Simple to understand
2. Hard to misuse
3. Easy to test
4. Minimal but complete

## 📋 Interface Design Checklist

For Every Public Interface:
- [ ] Does it have a single responsibility?
- [ ] Are all parameters required?
- [ ] Are error cases explicit?
- [ ] Is the return type unambiguous?
- [ ] Can I test it in isolation?

## 🚫 FORBIDDEN PATTERNS

- ❌ Optional parameters for core functionality
- ❌ Boolean flags that change behavior
- ❌ String-typed enums
- ❌ Methods that do multiple things
- ❌ Callback hell

## ✅ REQUIRED PATTERNS

### 1. Explicit Input Types
```rust
// Bad: Generic parameters
pub fn process_order(data: &serde_json::Value) -> Result<serde_json::Value>;

// Good: Specific types
pub fn process_order(order: SubmitOrderRequest) -> Result<OrderResponse>;

#[derive(Debug, Deserialize)]
pub struct SubmitOrderRequest {
    pub symbol: String,
    pub side: Side,
    pub quantity: Decimal,
    pub order_type: OrderType,
    pub time_in_force: TimeInForce,
}
```

### 2. Result Types for Fallible Operations
```rust
// Bad: Panic on error
pub fn get_position(symbol: &str) -> Position {
    positions.get(symbol).unwrap()
}

// Good: Explicit error handling
pub fn get_position(symbol: &str) -> Result<Position, PositionError> {
    positions
        .get(symbol)
        .ok_or(PositionError::NotFound(symbol.to_string()))
}
```

### 3. Builder Pattern for Complex Objects
```rust
// Bad: Many parameters
pub fn new_order(
    symbol: String,
    side: Side,
    quantity: Decimal,
    price: Option<Decimal>,
    order_type: OrderType,
    time_in_force: TimeInForce,
    account_id: Uuid,
    strategy_id: String,
) -> Order;

// Good: Builder pattern
pub struct OrderBuilder {
    symbol: Option<String>,
    side: Option<Side>,
    quantity: Option<Decimal>,
    // ...
}

impl OrderBuilder {
    pub fn new() -> Self { /* ... */ }
    pub fn symbol(mut self, symbol: String) -> Self { /* ... */ }
    pub fn side(mut self, side: Side) -> Self { /* ... */ }
    pub fn build(self) -> Result<Order, BuildError> { /* ... */ }
}
```

## 🔄 Trading System Interface Rules

### 1. Channel-Based Communication
```rust
// Define clear message types
#[derive(Debug, Clone)]
pub enum OmsMessage {
    SubmitOrder { order: Order, response: oneshot::Sender<OrderResponse> },
    CancelOrder { order_id: Uuid, response: oneshot::Sender<CancelResponse> },
    GetOrder { order_id: Uuid, response: oneshot::Sender<Option<Order>> },
}

// Single handle_message method
impl OmsEngine {
    pub async fn handle_message(&self, message: OmsMessage) {
        match message {
            OmsMessage::SubmitOrder { order, response } => {
                let result = self.submit_order_internal(order);
                let _ = response.send(result);
            }
            // ... other messages
        }
    }
}
```

### 2. Trait Boundaries for Testing
```rust
// Define behavior, not implementation
#[async_trait]
pub trait ExecutionGateway {
    async fn submit_order(&self, order: &Order) -> Result<Fill, ExecutionError>;
    async fn cancel_order(&self, order_id: Uuid) -> Result<(), ExecutionError>;
    async fn get_position(&self, symbol: &str) -> Result<Position, ExecutionError>;
}

// Mock for testing
pub struct MockExecutionGateway {
    fills: Arc<Mutex<HashMap<Uuid, Fill>>>,
}

#[async_trait]
impl ExecutionGateway for MockExecutionGateway {
    async fn submit_order(&self, order: &Order) -> Result<Fill, ExecutionError> {
        // Mock implementation
    }
}
```

### 3. Event-Driven Architecture
```rust
// Clear event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradingEvent {
    OrderSubmitted { order: Order, timestamp: DateTime<Utc> },
    OrderFilled { order_id: Uuid, fill: Fill, timestamp: DateTime<Utc> },
    PositionUpdated { symbol: String, position: Decimal, timestamp: DateTime<Utc> },
    RiskLimitBreached { limit_type: String, current_value: f64, timestamp: DateTime<Utc> },
}

// Event handler trait
#[async_trait]
pub trait EventHandler {
    async fn handle(&self, event: TradingEvent) -> Result<(), HandlerError>;
}
```

## 📊 Interface Validation

### 1. Compile-Time Guarantees
```rust
// Use type system to prevent errors
pub struct Quantity(pub Decimal);

impl Quantity {
    pub fn new(value: Decimal) -> Result<Self, QuantityError> {
        if value <= Decimal::ZERO {
            return Err(QuantityError::MustBePositive);
        }
        if value.fract_digits() > 8 {
            return Err(QuantityError::TooPrecise);
        }
        Ok(Quantity(value))
    }
}

// Now you can't create invalid quantities
```

### 2. State Machine Types
```rust
// Prevent invalid state transitions
pub struct SubmittedOrder(Order);
pub struct FilledOrder(Order);

impl SubmittedOrder {
    pub fn fill(self, fill: Fill) -> FilledOrder {
        // Transition only allowed this way
        FilledOrder(self.0)
    }
}

// Can't accidentally fill an already filled order
```

### 3. Protocol Buffers for External APIs
```protobuf
syntax = "proto3";

message SubmitOrderRequest {
    string symbol = 1;
    Side side = 2;
    fixed64 quantity = 3;  // Use fixed-point for money
    oneof price {
        fixed64 limit_price = 4;
        MarketOrder market = 5;
    }
    TimeInForce time_in_force = 6;
}

enum Side {
    BUY = 0;
    SELL = 1;
}

enum TimeInForce {
    DAY = 0;
    IOC = 1;
    FOK = 2;
}
```

## 🔄 API Evolution Strategy

### 1. Versioning
```rust
// Version your APIs
pub mod v1 {
    pub fn submit_order(request: SubmitOrderRequestV1) -> Result<OrderResponseV1>;
}

pub mod v2 {
    pub fn submit_order(request: SubmitOrderRequestV2) -> Result<OrderResponseV2>;
    
    // Provide migration path
    pub fn from_v1(request: SubmitOrderRequestV1) -> SubmitOrderRequestV2 {
        // Convert v1 to v2
    }
}
```

### 2. Backward Compatibility
```rust
// Never remove fields, only add
#[derive(Serialize, Deserialize)]
pub struct OrderResponseV2 {
    pub order_id: Uuid,
    pub status: OrderStatus,
    // New field - optional for old clients
    pub execution_venue: Option<String>,
    pub created_at: DateTime<Utc>,
}
```

## 📋 Interface Review Checklist

For Every Public API:
- [ ] Is the purpose clear from name?
- [ ] Are all parameters validated?
- [ ] Are error cases documented?
- [ ] Is it thread-safe if needed?
- [ ] Can it be easily mocked?
- [ ] Does it follow naming conventions?
- [ ] Is it versioned for external use?

## 🎯 Success Metrics

- Zero ambiguity in parameter types
- All error cases are explicit
- Interfaces can be understood in <30 seconds
- No need to read implementation to use
- Easy to write tests for
