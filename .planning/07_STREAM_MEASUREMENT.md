# 07 — STREAM: 12-Dimension Measurement & Scoring
**Workstream Owner**: Analytics Reporter  
**Support**: Experiment Tracker, Visual Storyteller, Frontend Developer  
**Depends On**: P4-1 (backtest engine), P4-4 (latency regression), P4-5 (divergence monitor)  
**Runs Parallel With**: Stream 6 (Testing) during Week 6  
**Status**: PLANNING — awaiting P4-1 gate

---

## STREAM GOAL
Produce a statistically defensible, weighted score for each model across 11 technical + 1 UX dimension. The scoring engine must reach a conclusion with confidence intervals that exclude the runner-up before any model is declared "winner." No model wins on fewer than 6 dimensions.

---

## SCORING WEIGHTS

| # | Dimension | Weight | Rationale |
|---|-----------|--------|-----------|
| D1 | Latency | 15% | HFT primary constraint |
| D2 | Throughput | 10% | Capacity headroom |
| D3 | Predictive Accuracy | 15% | Core signal quality |
| D4 | Robustness (multi-regime Sharpe) | 15% | Works in all conditions |
| D5 | Cross-Market Alpha | 10% | Unique BAM value-add |
| D6 | Tail Risk | 10% | Drawdown + CVaR control |
| D7 | Resilience | 5% | Uptime, recovery speed |
| D8 | Risk-Adjusted Return | 10% | Sharpe/Sortino/Calmar |
| D9 | Cognitive Load | 3% | Human maintenance cost |
| D10 | Code Complexity | 4% | Long-term velocity |
| D11 | UX Impact | 3% | Trust score (user survey) |

**Total**: 100%. Weights are fixed — changing requires CCB + PM approval.

---

## SCORING ALGORITHM

```
For each model m in {1, 2, 3}:
  For each dimension d in {D1..D11}:
    raw_score[m][d] = normalize(metric[m][d], target[d])
      where normalize maps [0, target] → [0, 100] linearly
      capped at 120 (20% bonus for beating target)

  dimension_score[m][d] = raw_score[m][d] × weight[d]
  total_score[m] = sum(dimension_score[m][d] for d in D1..D11)

Winner criteria:
  1. total_score[winner] - total_score[runner_up] > margin_of_error (95% CI)
  2. winner leads on >= 6 individual dimensions
  3. winner Sharpe > runner_up Sharpe in all 3 regimes (robustness dominance)

If no model meets all 3 criteria: flag "No Clear Winner" — escalate to engineering leads.
```

---

## TASK LIST

### P6-1 — Metrics Collector
**File**: `tests/metrics_collector.rs`  
**Specialist**: Analytics Reporter  
**Time**: 6h  
**Depends On**: P4-1, P4-4, Contract 6 (ModelMetricsReport)

**Spec**:
- Subscribes to metric events from all 3 model adapters
- Aggregates into time-windowed `ModelMetricsReport` (1-min, 5-min, 1-hr windows)
- Persists snapshots to QuestDB Warm tier (`metrics` table, one row per window per model)
- Streams live metrics to `comparative_dashboard.rs` API via in-memory channel
- Confidence intervals: 95% CI computed for each metric using bootstrap resampling (1000 resamples)

**Acceptance Gate**:
- [ ] Collects metrics from 3 concurrent runners without dropping any window
- [ ] Bootstrap CI correctly computed (unit test with known distribution)
- [ ] QuestDB write latency <10ms per batch
- [ ] File ≤150 lines

---

### P6-2 — Scoring Engine
**File**: `tests/scoring_engine.rs`  
**Specialist**: Analytics Reporter  
**Time**: 6h  
**Depends On**: P6-1

**Spec**:
- Loads latest `ModelMetricsReport` for each model from QuestDB
- Applies normalization and weighting per the scoring algorithm above
- Outputs `ScoringResult { model_id, total_score, dimension_scores: [f64; 11], winner: Option<u8>, confidence: f64 }`
- Re-runs every 4 hours during testing week; final run after 90-day backtest complete
- Audit trail: every scoring run logged to `scoring_runs.jsonl` with full input data

**Acceptance Gate**:
- [ ] Correctly identifies winner in synthetic dataset (Model A injected with 20% better metrics)
- [ ] Correctly outputs "No Clear Winner" when models differ by <5% on all dimensions
- [ ] `scoring_runs.jsonl` grows monotonically, never truncated
- [ ] File ≤150 lines

---

### P6-3 — Comparative Report Generator
**File**: `tests/report_generator.rs`  
**Specialist**: Visual Storyteller  
**Time**: 6h  
**Depends On**: P6-2

**Spec**:
- Generates HTML + PDF report from `ScoringResult`
- Structure (5-act narrative):
  1. **Executive Summary**: winner, total scores, confidence
  2. **Dimension Breakdown**: radar chart (D3.js), bar chart per dimension
  3. **Regime Analysis**: Sharpe by regime, heatmap
  4. **Risk Profile**: drawdown curves, CVaR comparison
  5. **Recommendation**: winner rationale, deployment conditions, monitoring requirements
- Mobile-optimized: reports must render correctly on 375px viewport
- Auto-generated weekly; manually triggered after final backtest

**Acceptance Gate**:
- [ ] Report renders without JavaScript errors in Chrome + Safari
- [ ] Radar chart correctly shows all 11 dimensions
- [ ] Report generated in <30 seconds from `ScoringResult` input
- [ ] Mobile viewport renders all charts without horizontal scroll
- [ ] File ≤150 lines

---

### P6-4 — Executive Dashboard (Frontend)
**File**: `src/dashboards/comparison-dashboard.tsx`  
**Specialist**: Frontend Developer  
**Time**: 8h  
**Depends On**: P4-6 (API), P6-3 (report generator)

**Spec**:
- React + Next.js + Tailwind
- 4 panels visible above the fold:
  1. Live score leaderboard (3 models, total score + trend)
  2. Latency p50/p99 per model (real-time line chart, 60-second window)
  3. Divergence heatmap (3×3 agreement matrix, color-coded)
  4. Regime indicator (current regime tag, last changed)
- Data refresh: TanStack Query, 5-second polling against `GET /api/v1/models/comparison`
- Dark mode default (trading terminal UX convention)
- WCAG 4.5:1 contrast for all text

**Acceptance Gate**:
- [ ] All 4 panels load with stub data in <1.8 seconds FCP (Lighthouse)
- [ ] Real-time updates visible within 6 seconds of a model metric change
- [ ] Dark mode confirmed across Chrome, Safari, Firefox
- [ ] Initial bundle <200KB (webpack-bundle-analyzer)
- [ ] File ≤200 lines (split into sub-components if needed)

---

### P6-5 — Mobile Companion App (React Native)
**File**: `mobile/traderx-metrics/`  
**Specialist**: Mobile App Builder  
**Time**: 10h  
**Depends On**: P4-6 (API), P6-4 (design reference)

**Spec**:
- React Native (Expo managed workflow)
- Screens: Home (leaderboard + regime), Model Detail (full 11-dimension scorecard), Alerts
- Push notifications via Expo Notifications: L4 auto-halt events, divergence alerts, scoring winner declared
- Offline cache: last 24 hours of metrics stored in AsyncStorage (read without network)
- Performance: 60fps scroll, <150MB memory, crash rate <0.1%

**Acceptance Gate**:
- [ ] App launches in <3 seconds on iPhone 12 (cold start)
- [ ] Push notification received within 15 seconds of L4 halt event
- [ ] Offline cache serves stale data correctly when network unavailable
- [ ] All 3 screens pass accessibility scan (no critical issues)

---

## MEASUREMENT TIMELINE

```
Week 6 Day 1-2:  P6-1 (Collector) running alongside L2/L3 tests
Week 6 Day 2-3:  P6-2 (Scoring engine) first run after 24h of data
Week 6 Day 3-4:  P6-3 (Report) — interim report generated
Week 6 Day 4:    P6-4 (Dashboard) deployed, team reviews live
Week 6 Day 5:    P6-5 (Mobile) build submitted to TestFlight

Gate G6 (end of Week 6):
  ✅ Scoring engine identifies winner or flags "No Clear Winner"
  ✅ Winner leads ≥6 dimensions
  ✅ 95% CI excludes runner-up total score
  ✅ Dashboard deployed and team-reviewed
  → If No Clear Winner: extend L3 backtest to 180 days and re-run scoring
  → If Winner: proceed to Stream 8 (UX Elevation) with winner identified
```

---

## GUARDED LINES (Coding Agents Must Not Cross)

1. **No vanity metrics** — metrics not in `ModelMetricsReport` schema are not scored; no ad-hoc additions
2. **Confidence intervals are mandatory** — no dimension score reported without 95% CI
3. **Winner declaration requires all 3 criteria** — total score margin + 6-dimension lead + regime dominance; all three, never just one
4. **Scoring weights are frozen** — `weights: [f64; 11]` is a compile-time constant array; no runtime override without CCB
5. **Report audit trail is append-only** — `scoring_runs.jsonl` may never be truncated or modified; only appended

---

## FILE MANIFEST

| File | Owner | Max Lines | Depends On |
|------|-------|-----------|------------|
| `tests/metrics_collector.rs` | Analytics Reporter | 150 | P4-1, P4-4, Contract 6 |
| `tests/scoring_engine.rs` | Analytics Reporter | 150 | P6-1 |
| `tests/report_generator.rs` | Visual Storyteller | 150 | P6-2 |
| `src/dashboards/comparison-dashboard.tsx` | Frontend Developer | 200 | P4-6, P6-3 |
| `mobile/traderx-metrics/` | Mobile App Builder | N/A | P4-6 |
