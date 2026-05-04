# Phase 1: Fix Foundation - Detailed Spec

## Objective
Establish a stable, warning-free foundation before building the full-stack trading system. All tests must pass, all warnings eliminated, architecture quality baseline established.

## Success Criteria
- [ ] `cargo test --package oms-engine` passes with **0 errors**
- [ ] `cargo check --package oms-engine --lib` shows **0 errors, ≤10 warnings**
- [ ] Sentrux installed and baseline quality score ≥6500 established
- [ ] All 5 module splits (engineering_orchestra, llm_message_bus, reliability_assessment, self_healing, agents) verified working

---

## Task 1.1: Fix 22 Test Errors in cross_market/

### Errors Identified
```
error[E0433]: cannot find type `Duration` in this scope
  --> metrics.rs:325, 343
  
error[E0603]: struct import `AssetFabricState` is private
  --> fabric_orchestrator.rs:370
  
error[E0433]: cannot find type `BottomLayerPattern` in this scope
  --> cross_layer_fusion.rs:199, 209, 220
  
error[E0433]: cannot find type `TopLayerPattern` in this scope
  --> cross_layer_fusion.rs:210, 221
  
error[E0433]: cannot find type `MarketRegime` in this scope
  --> noise_filter.rs:369
```

### Fix Plan

#### 1.1.1 Fix Duration in metrics.rs
**File**: `packages/oms-engine/src/metrics.rs:325,343`
**Issue**: Missing `use std::time::Duration;` in test module
**Fix**: Add import or use `std::time::Duration` directly

#### 1.1.2 Fix AssetFabricState visibility
**File**: `packages/oms-engine/src/cross_market/fabric_orchestrator.rs:370`
**Issue**: `AssetFabricState` is private
**Fix**: Make it `pub` in its definition module

#### 1.1.3 Fix pattern types in cross_layer_fusion.rs
**File**: `packages/oms-engine/src/cross_market/cross_layer_fusion.rs:199,209,210,220,221`
**Issue**: `BottomLayerPattern` and `TopLayerPattern` not found
**Fix**: Import from appropriate module or define if missing

#### 1.1.4 Fix MarketRegime in noise_filter.rs
**File**: `packages/oms-engine/src/cross_market/noise_filter.rs:369`
**Issue**: `MarketRegime` not in scope
**Fix**: Import from `regime_detection` module

---

## Task 1.2: Fix 26 Cargo Warnings

### Warning Categories
1. **Unused variables** (backtest, portfolio, adapters)
2. **Naming conventions** (`AT_OPEN`, `AT_CLOSE` → `AtOpen`, `AtClose`)
3. **Unused doc comments** on macro invocations
4. **Unused imports** (chrono::Utc, uuid::Uuid, rand::random)
5. **Dead code** (unused methods, fields)

### Fix Plan

#### 1.2.1 Fix naming conventions
**Files**: `orders/advanced.rs:36,38`
**Change**: `AT_OPEN` → `AtOpen`, `AT_CLOSE` → `AtClose`

#### 1.2.2 Remove unused imports
**Files**:
- `middleware/function_orchestrator.rs:518-519` (chrono::Utc, uuid::Uuid)
- `risk_bus.rs:306` (rand::random)

#### 1.2.3 Fix unused variables
**Files**:
- `bin/backtest.rs:816-817` (high, low) → prefix with `_`
- `stability/reliability_assessment/mod.rs:45` (assessment) → prefix with `_`

#### 1.2.4 Fix dead code warnings
**Files**:
- `portfolio.rs:36` (initial_cash) - check if truly unused
- `metrics_server.rs:40` (rate_limiter) - verify usage
- `engineering_orchestra/mod.rs:236` (main) - test-only function

---

## Task 1.3: Install Sentrux & Establish Baseline

### Installation
```bash
# Try brew first (may fail on non-Tier 1)
brew install sentrux/tap/sentrux

# Fallback: Install from source
git clone https://github.com/sentrux/sentrux.git
cd sentrux && cargo build --release
sudo cp target/release/sentrux /usr/local/bin/
```

### Baseline Commands
```bash
cd packages/oms-engine
sentrux check . --format json > ../../.sentrux/baseline.json
sentrux gate --save .
```

### Expected Output
- Quality score: target ≥6500
- Files scanned: ~200
- Bottleneck identified (likely "modularity" or "depth")

---

## Execution Order

```
Step 1: Diagnose exact test errors
  └─ cargo test 2>&1 | tee /tmp/test_errors.txt

Step 2: Fix cross_market type errors (in order)
  ├─ 2.1 Fix Duration imports
  ├─ 2.2 Fix AssetFabricState visibility
  ├─ 2.3 Fix pattern types (BottomLayerPattern, TopLayerPattern)
  └─ 2.4 Fix MarketRegime import

Step 3: Verify tests pass
  └─ cargo test --lib (should show 0 errors)

Step 4: Fix warnings (batch by type)
  ├─ 4.1 Fix naming conventions
  ├─ 4.2 Remove unused imports
  ├─ 4.3 Fix unused variables
  └─ 4.4 Address dead code

Step 5: Verify clean build
  └─ cargo check --lib (should show ≤10 warnings)

Step 6: Install sentrux
  └─ brew install OR build from source

Step 7: Establish baseline
  └─ sentrux check . > baseline.json

Step 8: Commit all changes
  └─ git add -A && git commit -m "fix(oms-engine): Phase 1 foundation fixes"
```

---

## Rollback Plan

If any fix introduces new errors:
1. `git stash` current changes
2. Fix one file at a time
3. `cargo check` after each file
4. Commit incremental fixes

---

## Verification Commands

```bash
# Quick check
cargo check --package oms-engine --lib 2>&1 | grep -E "^error" | wc -l
# Expected: 0

# Full test
cargo test --package oms-engine --lib 2>&1 | tail -20
# Expected: "test result: ok. X passed; 0 failed"

# Warning count
cargo check --package oms-engine --lib 2>&1 | grep -E "^warning:" | wc -l
# Expected: ≤10

# Sentrux quality
cd packages/oms-engine && sentrux check . 2>&1 | jq '.quality_signal'
# Expected: ≥6500
```

---

## Definition of Done

- [ ] All checkboxes in Success Criteria marked
- [ ] Commit hash recorded: `____________`
- [ ] Sentrux baseline saved: `.sentrux/baseline.json`
- [ ] No blocking issues for Phase 2 (Backend API)
- [ ] Team notified: Foundation stable
