# Architectural Drift Analysis: Library vs Binary API Mismatch

**Date:** 2026-05-01
**Severity:** CRITICAL - Blocks all binary compilation
**Type:** API Drift / Integration Failure

---

## 1. ROOT FAILURE IDENTIFIED

**Failure Point:** `packages/oms-engine/src/bin/main.rs` and `minimal_main.rs`
**Nature:** Binary integration files written against OBSOLETE/DIVERGENT library API
**Impact:** 15+ compilation errors, blocking all binary builds and integration tests

---

## 2. ANALYSIS: WHY THIS FUNCTION IS CAUSING FAILURES

### 2.1 API Divergence Matrix

| Expected by Binary (main.rs) | Provided by Library | Status |
|------------------------------|---------------------|--------|
| `Order.quantity` | `Order.original_quantity` | ❌ MISMATCH |
| `SignalRouter.route_signal()` (sync) | `SignalRouter.route().await` (async) | ❌ MISMATCH |
| `OmsEvent::OrderFilled { fill_time: DateTime }` | `OmsEvent::OrderFilled { total_filled, order_state }` | ❌ MISMATCH |
| `portfolio_aggregation` crate | Not in Cargo.toml | ❌ MISSING |
| `order.price.unwrap().to_f64()` | `Decimal.to_f64()` requires `ToPrimitive` trait | ❌ MISMATCH |
| `OmsEngine::new()?.await` | `OmsEngine::new()` (sync, returns `Result`) | ❌ MISMATCH |

### 2.2 What the Binaries Assume vs Reality

**Binary `main.rs` assumes:**
```rust
// Synchronous signal routing
let outcome = signal_router.route_signal(test_signal);

// Order has 'quantity' field
order.quantity * order.price

// OmsEvent::OrderFilled has 'fill_time'  
OmsEvent::OrderFilled { fill_time: chrono::Utc::now() }

// OMS engine constructor is async
OmsEngine::new(...).await
```

**Library Reality:**
```rust
// Async signal routing
let outcome = signal_router.route(test_signal).await;

// Order has 'original_quantity' field
order.original_quantity * order.price

// OmsEvent::OrderFilled has 'total_filled' and 'order_state'
OmsEvent::OrderFilled { total_filled: ..., order_state: OrderState::Filled }

// OMS engine constructor is synchronous
OmsEngine::new(...)?  // Returns Result, not Future
```

---

## 3. ARCHITECTURAL ASSESSMENT: WHERE IT WENT WRONG

### 3.1 Design Anti-Pattern: Library Evolved, Binaries Fossilized

**What Should Have Happened:**
1. Library API changes → Binaries updated in SAME commit/PR
2. CI enforces `cargo check --bins` passes before merge
3. Public API has stability guarantees via lib.rs re-exports
4. Single canonical integration binary, not 3+ divergent copies

**What Actually Happened:**
1. Library modules (`state_machine.rs`, `signal_router.rs`, `oms.rs`) evolved independently
2. Binaries (`main.rs`, `minimal_main.rs`, `test_trading_flow.rs`) were written as "temporary bootstrap" code
3. No CI enforcement - binaries never compiled together with library changes
4. Three different binary files with THREE different API assumptions
5. `test_trading_flow.rs` doesn't even use the library - defines its own types!

### 3.2 Structural Problems

**Problem 1: Redundant Binary Targets**
```
src/bin/
  main.rs           - Tries to use library, but API is wrong
  minimal_main.rs    - Tries to use library, but API is wrong  
  test_trading_flow.rs - DOESN'T use library, defines own types
```

**Problem 2: No API Contract Enforcement**
- `lib.rs` re-exports public API but binaries use wrong field names/methods
- No compile-time verification that binaries match library

**Problem 3: Missing Integration Layer**
- Binaries try to wire everything together manually
- No integration module (`src/integration.rs`) that provides canonical wiring
- Each binary re-invents the wiring pattern

**Problem 4: Test Binary is Actually a Standalone Program**
- `test_trading_flow.rs` defines its own `RiskBus`, `SignalRouter`, `AgentSignal`
- This means it's NOT testing the actual library code
- It's testing a DUPLICATE implementation

---

## 4. REALIGNMENT PLAN

### 4.1 Immediate Fix (Correct Binaries to Match Library)

**Role: Integration Engineer**
- Fix `main.rs` to use actual `SignalRouter::route().await`
- Fix `main.rs` to use `Order.original_quantity`
- Fix `main.rs` to use correct `OmsEvent::OrderFilled` fields
- Fix `minimal_main.rs` with same corrections
- Delete or refactor `test_trading_flow.rs` to use library types

### 4.2 Structural Fix (Prevent Future Drift)

**Role: Architect**

**A. Create Integration Module**
```rust
// src/integration.rs
pub mod bootstrap;
pub mod trading_flow;
```

**B. Define Stable Bootstrap API**
```rust
// src/integration/bootstrap.rs
pub async fn create_trading_engine(
    capital: f64,
    drawdown_bps: i64,
) -> Result<(Arc<OmsEngine>, Arc<SignalRouter>), BootstrapError> {
    // Canonical wiring that ALL binaries use
}
```

**C. Single Binary Target**
```
[[bin]]
name = "traderx"
path = "src/bin/traderx.rs"
```

**D. CI Enforcement**
```yaml
# .github/workflows/ci.yml
- name: Check Binaries
  run: cargo check --bins
```

**E. Pre-commit Hook**
```yaml
# .pre-commit-config.yaml
- repo: local
  hooks:
  - id: cargo-check-bins
    name: Check binary compilation
    entry: cargo check --bins
    language: system
    pass_filenames: false
```

---

## 5. EXECUTION PLAN (Role-Based)

### Phase 1: Fix Current Binaries (Integration Engineer)

**Task 1.1:** Fix `main.rs` API mismatches
- `route_signal()` → `route().await`
- `order.quantity` → `order.original_quantity`
- `OmsEvent::OrderFilled` fields corrected
- `OmsEngine::new()` call fixed (remove .await)

**Task 1.2:** Fix `minimal_main.rs` with same pattern
**Task 1.3:** Refactor `test_trading_flow.rs` to use library types OR move to `examples/`

### Phase 2: Create Integration Module (System Architect)

**Task 2.1:** Create `src/integration/bootstrap.rs`
- Canonical engine creation function
- Type-safe wiring of RiskBus → OmsEngine → SignalRouter

**Task 2.2:** Create `src/integration/trading_flow.rs`
- Standard signal → order → fill flow
- All binaries use this instead of manual wiring

### Phase 3: CI/CD Enforcement (DevOps Engineer)

**Task 3.1:** Add `cargo check --bins` to GitHub Actions
**Task 3.2:** Add pre-commit hook for binary checks
**Task 3.3:** Add `cargo test --bins` to CI pipeline

---

## 6. WIRING TO PREVENT FUTURE FAILURES

### 6.1 API Stability Rules

```
RULE: When library public API changes:
  1. Update ALL binaries in same PR
  2. Run `cargo check --bins` before commit
  3. Update integration module if affected
  4. Update AGENTS.md API documentation
```

### 6.2 Binary Governance

```
RULE: Binary files MUST:
  1. Use integration module, not manual wiring
  2. Import types from library, never redefine
  3. Pass `cargo check --bins` before PR merge
  4. Have integration test verifying end-to-end flow
```

### 6.3 Test Strategy

```
TEST: Integration test in tests/integration/:
  1. Creates OmsEngine via bootstrap module
  2. Creates SignalRouter via bootstrap module  
  3. Sends test signal through router
  4. Verifies order appears in OMS
  5. Verifies risk checks pass/reject correctly
```

---

## 7. SUCCESS CRITERIA

- [ ] `cargo check --bins` passes with 0 errors
- [ ] `cargo test --bins` passes with all tests green
- [ ] Single canonical binary uses integration module
- [ ] CI enforces binary compilation on every PR
- [ ] No binary redefines types from library
- [ ] Integration test verifies end-to-end signal → order → fill flow

---

**Analysis by:** Root Cause Investigation Team
**Date:** 2026-05-01
**Status:** Analysis Complete - Ready for Execution
