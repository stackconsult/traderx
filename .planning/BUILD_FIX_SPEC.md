# Build Fix Specification

## Context
The backtest binary was recently added to `packages/oms-engine/src/bin/backtest.rs`. Context engineering assessment identified P0 violations of project rules that must be fixed.

## P0 Issues to Fix

### Issue 1: Error Handling Pattern Violation
**Location:** `packages/oms-engine/src/bin/backtest.rs`
**Lines:** 160-161, 212-213, 381-382
**Problem:** Uses `expect()` instead of `?` operator or explicit `match`
**Rule:** AGENTS.md states: "No `unwrap()` in production paths — use `?` or explicit `match`"

**Required Fix:**
- Replace `expect()` calls with proper error handling using `Result` types
- Change function signature to return `Result<(), Box<dyn Error>>` where appropriate
- Use `?` operator for propagating errors
- For truly unrecoverable errors, use `unwrap()` only with explicit comment explaining why

**Specific Changes:**
1. Line 160-161: `Utc.with_ymd_and_hms().expect()` → Handle invalid datetime error
2. Line 212-213: `price_series.get().expect()` → Handle missing key error
3. Line 381-382: `std::fs::write().expect()` → Handle write error

### Issue 2: Context Starvation - Missing Pattern Review
**Location:** `packages/oms-engine/src/bin/backtest.rs`
**Problem:** Implemented without reading existing backtest modules
**Files to Review:**
- `packages/oms-engine/src/cross_market/backtest_engine.rs`
- `packages/oms-engine/src/cross_market/backtest_universe.rs`

**Required Action:**
1. Read both existing backtest modules
2. Identify any duplicate patterns (price generation, simulation logic, reporting)
3. If duplication exists, refactor to use existing utilities
4. If no duplication, add comment explaining why new implementation was necessary

## Success Criteria

- [ ] All `expect()` calls replaced with proper error handling
- [ ] Function signatures updated to return `Result` where appropriate
- [ ] Existing backtest modules reviewed and documented
- [ ] Code compiles without errors: `cargo check --package oms-engine`
- [ ] Error count does not increase from baseline
- [ ] No new `unwrap()` or `expect()` calls added

## Constraints

- Follow AGENTS.md code style strictly
- No `unwrap()` in production paths
- No `// TODO` comments in committed code
- Use `thiserror::Error` for custom error types if needed
- Must pass `cargo check` before committing

## Priority
P0 - Violates project rules, must fix immediately
