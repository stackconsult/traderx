# 06 — STREAM: Recursive Parallel Testing
**Workstream Owner**: Test Writer Fixer  
**Support**: Performance Benchmarker, Experiment Tracker, Analytics Reporter, DevOps Automator  
**Depends On**: G1 (Model 1 hardened), G2 (Model 2 built), G3 (Model 3 built)  
**Sprint**: Weeks 5–6  
**Status**: PLANNING — awaiting G1, G2, G3 gates

---

## STREAM GOAL
Run all three models against identical conditions at 5 levels of increasing rigor. Tests are parallel across models, sequential across levels. The output of this stream is statistical confidence that one model is measurably superior on the primary dimensions.

**Key Design Principle**: All 3 models receive IDENTICAL data. Randomness in model behavior comes from internal logic only, never from data differences.

---

## TASK LIST

### P4-1 — Comparative Backtest Engine
**File**: `tests/comparative_backtest.rs`  
**Specialist**: Test Writer Fixer  
**Time**: 8h  
**Depends On**: G1, G2, G3 (all three adapters working)

**Spec**:
- Deterministic tick replayer: reads from QuestDB Warm tier, same sequence every run
- Feeds identical `Tick` stream to all 3 `ModelRunner` instances simultaneously (via clone, not channel)
- Collects `ModelDecision` from each per tick into `BacktestRecord { tick, decisions: [ModelDecision; 3] }`
- Simulated execution: apply decisions to paper portfolio (no real orders), track P&L per model
- Paper portfolio: fixed $1M capital, 1× leverage max, slippage model: 0.5 bps per trade
- Output: `BacktestResult` per model containing full P&L series, trade log, and per-decision metadata
- Run 3 regimes: 30-day choppy period, 30-day trending period, 30-day volatile period (90 days total)

**Acceptance Gate**:
- [ ] Same seed → same decisions → same P&L (determinism test, run twice, diff output)
- [ ] All 3 models complete 90-day replay in <4 hours (time budget)
- [ ] Output CSV contains one row per tick per model (3M+ rows for 90-day 1s bars, 3 models)
- [ ] File ≤200 lines

---

### P4-2 — Statistical Significance Tests
**File**: `tests/significance_tests.rs`  
**Specialist**: Test Results Analyzer  
**Time**: 6h  
**Depends On**: P4-1 (needs BacktestResult data)

**Spec**:
- Paired t-test on daily Sharpe ratios (Model A vs B, A vs C, B vs C)
- Bonferroni correction for multiple comparisons (3 pairs → α = 0.05/3 ≈ 0.0167)
- Required: p < 0.0167 for a difference to be declared significant
- Wilcoxon signed-rank test for non-normal distributions (alternative to t-test)
- Effect size: Cohen's d (d > 0.5 = medium effect, d > 0.8 = large)
- Output: `SignificanceReport { pair, p_value, effect_size_d, test_type, conclusion }`

**Acceptance Gate**:
- [ ] Correctly rejects null for known synthetic dataset (injected 20% Sharpe difference)
- [ ] Does NOT reject null for two identical models (drift test)
- [ ] Report output matches expected schema exactly
- [ ] File ≤120 lines

---

### P4-3 — A/B Testing Framework
**File**: `tests/ab_test_framework.rs`  
**Specialist**: Experiment Tracker  
**Time**: 6h  
**Depends On**: P4-1

**Spec**:
- Random tick-level assignment: each tick routed to Model A or B (50/50, seeded RNG)
- 95% confidence interval on win rate difference
- Early stopping rules: O'Brien-Fleming spending function (stop early only if p < 0.001 at interim)
- Feature flags integration: `FeatureFlag::ModelBSelection` toggles which model is "B" in live paper trading
- Minimum runtime: 1 trading week (5 days) before any conclusion drawn

**Acceptance Gate**:
- [ ] Early stopping correctly triggers on injected 30% win rate difference at day 3
- [ ] Does NOT early-stop on injected 5% difference (within noise)
- [ ] Feature flag correctly routes decisions without data contamination
- [ ] File ≤150 lines

---

### P4-4 — Latency Regression Suite
**File**: `benches/latency_regression.rs`  
**Specialist**: Performance Benchmarker  
**Time**: 6h  
**Depends On**: G1, G2, G3 (all adapters)

**Spec**:
- Criterion benchmarks for all 3 models: p50, p99, p999 end-to-end tick → decision
- Regression detection: compare against stored baseline (JSON file per model version)
- Fail CI if p99 regresses >10% from baseline
- Report: per-component breakdown for each model (reuse latency probes from P1-2)
- Throughput: events/sec at saturation (producer fills as fast as possible)

**Latency Budgets**:
```
Model 1: p50 <100μs, p99 <200μs, p999 <500μs
Model 2: p50 <1ms,   p99 <2ms,   p999 <5ms
Model 3: p50 <150μs, p99 <350μs, p999 <750μs
```

**Acceptance Gate**:
- [ ] All 3 models within budget at current hardware
- [ ] Regression detection fires correctly (inject 50% slowdown, verify CI fails)
- [ ] Baseline JSON files committed to repo (versioned)
- [ ] File ≤150 lines

---

### P4-5 — Divergence Monitor
**File**: `src/fabric/divergence_monitor.rs`  
**Specialist**: AI Engineer  
**Time**: 4h  
**Depends On**: P4-1 (needs live decision stream)

**Spec**:
- Tracks agreement rate between all 3 model pairs per 5-minute window
- Agreement: Model A and B both emit same `side` for same `symbol_id` within 1 bar
- Alert threshold: if any pair disagrees >70% of the time → emit `DivergenceAlert`
- Stores rolling 24-hour divergence history in circular buffer (no DB dependency)
- Grafana panel: real-time divergence heatmap (3×3 grid, color-coded by agreement %)

**Acceptance Gate**:
- [ ] Alert fires when synthetic models injected with 80% disagreement
- [ ] No alert on 30% disagreement (within normal range)
- [ ] Circular buffer correctly evicts entries older than 24h
- [ ] File ≤120 lines

---

### P4-6 — Comparative Dashboard API
**File**: `src/api/comparative_dashboard.rs`  
**Specialist**: Backend Architect  
**Time**: 6h  
**Depends On**: P4-1, P4-4, P4-5

**Spec**:
- REST API (`axum`): `GET /api/v1/models/comparison` → returns current `ModelMetricsReport` for all 3
- `GET /api/v1/models/{id}/decisions?limit=100` → paginated recent decisions
- `GET /api/v1/divergence/current` → current divergence heatmap data
- Response time: <50ms p99 (served from in-memory cache, updated every 30 seconds)
- Auth: Bearer token (static for now, OAuth2 in Phase 7)

**Acceptance Gate**:
- [ ] All 3 endpoints return correct schema (contract test against OpenAPI spec)
- [ ] <50ms p99 under 100 concurrent clients (wrk benchmark)
- [ ] File ≤150 lines

---

## FIVE-LEVEL RECURSIVE PARALLEL TEST HIERARCHY

### Level 1 — Unit Tests
**Trigger**: Every commit to any stream branch  
**Data**: Synthetic 1M ticks (generated via seeded RNG)  
**Parallelism**: All 3 model adapters tested concurrently via `cargo test --jobs=3`  
**Duration**: <5 minutes  
**Gate**: Pass rate >95% (allow 5% flakiness budget); zero new compilation errors

### Level 2 — Integration Tests
**Trigger**: Every 4 hours during development; every PR merge  
**Data**: 1-day historical (QuestDB Warm tier, fixed date)  
**Parallelism**: All 3 adapters running simultaneously against same tick stream  
**Duration**: <1 hour  
**Gate**: All 3 models produce decisions for ≥90% of ticks; no panic/crash; divergence <70%

### Level 3 — Nightly Backtest
**Trigger**: Nightly at 00:00 UTC via CI cron  
**Data**: 90-day historical (3 regime periods)  
**Parallelism**: 3 models × 3 regimes = 9 parallel backtest processes (3× speedup)  
**Duration**: <4 hours  
**Gate**: Each model achieves Sharpe >0.5 in at least 1 regime; max drawdown <20%; no regime where all 3 Sharpe <0

**Batch strategy**: Split 90 days into 3 × 30-day regime chunks. Each chunk runs on separate CPU core. Results merged at end.

### Level 4 — Live Paper Trading
**Trigger**: Market open (9:30 AM ET weekdays)  
**Data**: Real-time tick stream (Polygon.io WebSocket)  
**Parallelism**: All 3 models consume same live feed simultaneously  
**Duration**: Continuous during market hours (6.5 hours/day)  
**Gate**: Any model with >5% intraday drawdown → auto-halt that model, emit alert, continue other two

**Short-circuit**: `DivergenceMonitor` triggers if agreement drops <30% — escalate to human review, do NOT auto-halt

### Level 5 — Monte Carlo Stress (Weekly)
**Trigger**: Saturday 08:00 UTC  
**Data**: 10,000 synthetic market scenarios (parameterized by regime, volatility, correlation)  
**Parallelism**: 30,000 total runs (10K scenarios × 3 models), distributed across all available CPU cores  
**Duration**: <8 hours  
**Gate**: VaR 95% <5% for all models; circuit breaker effectiveness >90% (triggers when it should)

**Monte Carlo parameters**:
```
Scenarios: 10,000
  - Regime distribution: 33% choppy, 33% trending, 34% volatile
  - Volatility: σ drawn from LogNormal(μ=0.01, σ=0.5)
  - Correlation shock: 20% of scenarios include sudden correlation breakdown
  - Flash crash: 5% of scenarios include 10%+ price drop in <5 minutes
  - Latency spikes: 10% of scenarios include 10× latency injection (network simulation)
```

---

## AUTOMATION WIRING

| Level | Trigger Mechanism | Result Destination | Failure Action |
|-------|------------------|-------------------|----------------|
| L1 | `git push` hook → GitHub Actions | PR check status | Block merge |
| L2 | CI cron every 4h | Grafana test dashboard | Slack alert |
| L3 | CI cron nightly | Comparative report email | PagerDuty alert |
| L4 | Market hours scheduler | Live Grafana dashboard | Auto-halt + alert |
| L5 | CI cron Saturday | Monte Carlo risk report | PagerDuty if VaR breach |

---

## ROLLBACK PROCEDURES

| Failure | Rollback Action |
|---------|----------------|
| L1 flaky test >5% | Mark test `#[ignore]` temporarily; file bug; fix within 1 sprint |
| L3 backtest OOM | Reduce parallelism from 9 → 3 processes; accept 3× longer runtime |
| L4 drawdown >5% on a model | Auto-halt model; continue with remaining two; human review before re-enabling |
| L5 Monte Carlo runtime >8h | Reduce scenarios from 10K → 5K; document tradeoff |
| P4-6 API >50ms p99 | Add Redis cache layer in front; API becomes read-through |

---

## FILE MANIFEST

| File | Owner | Max Lines | Depends On |
|------|-------|-----------|------------|
| `tests/comparative_backtest.rs` | Test Writer Fixer | 200 | Contracts 1, 6 |
| `tests/significance_tests.rs` | Test Results Analyzer | 120 | P4-1 |
| `tests/ab_test_framework.rs` | Experiment Tracker | 150 | P4-1 |
| `benches/latency_regression.rs` | Performance Benchmarker | 150 | Contracts 1, 6 |
| `src/fabric/divergence_monitor.rs` | AI Engineer | 120 | Contract 1 |
| `src/api/comparative_dashboard.rs` | Backend Architect | 150 | Contracts 1, 6 |

---

## GUARDED LINES (Coding Agents Must Not Cross)

1. **All 3 models MUST receive identical tick data** — no per-model filtering or preprocessing before the adapter boundary
2. **No mocks in L2-L5** — integration, backtest, paper, and Monte Carlo must use REAL model runners
3. **L4 auto-halt is non-negotiable** — removing or weakening the >5% drawdown halt requires CCB approval
4. **Statistical tests must use Bonferroni correction** — raw p-values without correction are never reported as conclusions
5. **Backtest determinism is a hard requirement** — any non-determinism (random seeds not fixed) blocks the stream
