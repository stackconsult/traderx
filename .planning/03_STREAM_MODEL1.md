# 03 — STREAM: Model 1 — Base Foundation (Hardening)
**Workstream Owner**: AI Engineer  
**Support**: Performance Benchmarker, Test Writer Fixer  
**Depends On**: G0 (Stream 2 complete) — specifically P0-1 (ingest) and P0-3 (harness)  
**Sprint**: Week 1 Days 4-5  
**Status**: PLANNING — awaiting G0 gate

---

## STREAM GOAL
Model 1 already exists (`multi_market_grid.rs`, `funnel_index.rs`, `pattern_matcher.rs`). This stream is **hardening only** — no logic changes. Goals:
1. Wrap existing decision logic to emit `ModelDecision` (Contract 1)
2. Add per-component latency probes to establish baseline measurement
3. Property-based tests to lock in invariants before comparison testing
4. Stress test: flash crash scenarios must not exceed 200μs or crash the process

**Guarded Line**: ZERO changes to Model 1 trading logic. Instrument only. All changes must be additive wrappers.

---

## TASK LIST

### P1-1 — Benchmark Harness Adapter
**File**: `tests/model1_adapter.rs`  
**Specialist**: AI Engineer  
**Time**: 4h  
**Depends On**: P0-3 (harness), Contract 1 (ModelDecision format)

**Spec**:
- Implement `ModelRunner` trait for Model 1
- Map existing Model 1 output to `ModelDecision` struct field-by-field
- Symbol ID lookup from `SymbolRegistry`
- Confidence: map existing conviction score (0.0-1.0 f64) to u8 via `(conviction * 255.0) as u8`
- Pattern signature: xxHash64 of the pattern vector that triggered the decision
- Timestamp: read from DPDK hardware clock, NOT `SystemTime::now()`

**File structure**:
```rust
// tests/model1_adapter.rs
pub struct Model1Runner {
    grid: Arc<MultiMarketGrid>,
    registry: Arc<SymbolRegistry>,
}

impl ModelRunner for Model1Runner {
    fn run(&self, tick: &Tick) -> ModelDecision {
        // ... adapter logic only, no trading logic here
    }
    fn model_id(&self) -> u8 { 1 }
}
```

**Acceptance Gate**:
- [ ] Produces valid `ModelDecision` for 10K random ticks without panic
- [ ] `model_id` always returns 1
- [ ] Pattern signature is deterministic: same pattern vector = same hash
- [ ] File ≤150 lines

---

### P1-2 — Latency Probes (Per-Component)
**File**: `src/infra/latency_probe.rs`  
**Specialist**: Performance Benchmarker  
**Time**: 3h  
**Depends On**: P0-1 (ingest running to generate real ticks)

**Spec**:
- Instrument these existing Model 1 components with nanosecond timestamps:
  - `funnel_index.rs` → `FunnelIndex::lookup()` entry/exit
  - `pattern_matcher.rs` → `PatternMatcher::scan()` entry/exit
  - `multi_market_grid.rs` → `Grid::evaluate()` entry/exit
- Use `std::time::Instant` (nanosecond) NOT `SystemTime`
- Emit histograms via Prometheus: `model1_funnel_ns`, `model1_pattern_ns`, `model1_evaluate_ns`
- Zero overhead when probes disabled: compile-time feature flag `latency-probes`

**Acceptance Gate**:
- [ ] With probes enabled: histogram metrics appear in Prometheus scrape
- [ ] With probes disabled (`--no-default-features`): zero timing code compiled in (verified by `cargo expand`)
- [ ] Per-component p50/p99 breakdown visible in Grafana baseline dashboard
- [ ] `latency_probe.rs` ≤120 lines

---

### P1-3 — Property-Based Invariant Tests
**File**: `tests/model1_invariants.rs`  
**Specialist**: Test Writer Fixer  
**Time**: 4h  
**Depends On**: P1-1 (adapter for standardized input/output)

**Spec**:
- Use `proptest` crate for random input generation
- Invariants to test (all must hold for any valid tick):
  1. `ModelDecision::side` is always 0, 1, 2, or 3 — never other values
  2. `ModelDecision::confidence` ≥ 0 and ≤ 255
  3. If `regime_tag == 3` (Halted), then `side` must be 0 (NoOp) — cannot trade during halt
  4. `ModelDecision::timestamp_ns` is always > 0
  5. Pattern signature is never all-zeros for a triggered decision
- Run with `PROPTEST_CASES=1000` minimum

**Acceptance Gate**:
- [ ] All 5 invariants pass for 1000 random inputs
- [ ] Any invariant failure produces a minimal reproducer (proptest shrinking)
- [ ] Tests run in <30 seconds
- [ ] File ≤120 lines

---

### P1-4 — Flash Crash Stress Test
**File**: `tests/model1_stress.rs`  
**Specialist**: Performance Benchmarker  
**Time**: 3h  
**Depends On**: P1-1, P1-2

**Spec**:
- Replay 3 historical crash scenarios using pre-recorded tick data:
  - 1987 Black Monday proxy (15% single-day drop in 30 minutes of ticks)
  - 2010 Flash Crash proxy (deep 5-minute drop then full recovery)
  - 2020 COVID crash (sustained 34% drawdown, multiple circuit breaker triggers)
- Synthetic data if historical unavailable: parameterized step-down tick generator
- Measure during replay:
  - p99 latency must stay ≤200μs
  - Process must not crash (no panic, no OOM)
  - Circuit breaker must trigger in ≤1ms when drawdown >5%
  - Halted regime correctly sets `ModelDecision::regime_tag = 3`

**Acceptance Gate (Gate G1)**:
- [ ] All 3 scenarios complete without process crash
- [ ] p99 latency ≤200μs during flash crash (measured by latency probes from P1-2)
- [ ] Circuit breaker fires within 1ms on >5% drawdown
- [ ] `regime_tag == 3` on all decisions during circuit breaker active period
- [ ] File ≤150 lines

---

## STREAM DEPENDENCY CHAIN

```
Day 4:  P1-1 (Adapter) ──→ P1-3 (Invariants)
        P1-2 (Probes)  ──→ P1-4 (Stress)

Day 5:  G1 gate check (Performance Benchmarker + Test Writer Fixer review results)

Gate G1: ✅ All 4 tasks complete and accepting
         ✅ Model 1 latency baseline documented in Prometheus/Grafana
         ✅ No cargo check errors introduced
         → Model 1 locked. No further changes until after Phase 4 comparative testing.
```

---

## ROLLBACK PROCEDURES

| Failure | Rollback Action |
|---------|----------------|
| Adapter changes break existing Model 1 behavior | Revert adapter; investigate mapping logic |
| Latency probe overhead >1μs (measured) | Disable probe on that component; use sampling (1-in-100) |
| Invariant test reveals Model 1 logic bug | FILE BUG REPORT. Do NOT fix Model 1 logic — mark model "under investigation" |
| Flash crash causes p99 >200μs | Document as known baseline; does NOT block stream; flag for Phase 7 optimization |

---

## FILE MANIFEST

| File | Owner | Max Lines | Depends On |
|------|-------|-----------|------------|
| `tests/model1_adapter.rs` | AI Engineer | 150 | Contracts 1, 6 |
| `src/infra/latency_probe.rs` | Performance Benchmarker | 120 | None |
| `tests/model1_invariants.rs` | Test Writer Fixer | 120 | P1-1 |
| `tests/model1_stress.rs` | Performance Benchmarker | 150 | P1-1, P1-2 |

---

## GUARDED LINES (Coding Agents Must Not Cross)

1. **Zero changes to Model 1 trading logic** — `multi_market_grid.rs`, `funnel_index.rs`, `pattern_matcher.rs` are READ-ONLY in this stream
2. **No mock storage** in stress tests — use real Hot tier data (NVMe pool from P0-2)
3. **No `SystemTime::now()`** for latency measurement — use `std::time::Instant` exclusively
4. **Invariant failures are bugs, not test failures** — if proptest finds a failing invariant, stop and escalate before fixing
5. **Latency probe feature flag must default to OFF** in release builds
