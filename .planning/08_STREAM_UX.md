# 08 — STREAM: Winner Optimization & UX Elevation
**Workstream Owner**: Project Shipper  
**Support**: Performance Benchmarker, UX Researcher, UI Designer, Visual Storyteller, Frontend Developer, Mobile App Builder, Brand Guardian, Trend Researcher  
**Depends On**: G6 (winner declared from Stream 7)  
**Sprint**: Week 7  
**Status**: PLANNING — awaiting G6 gate

---

## STREAM GOAL
Take the declared winner model and:
1. Optimize its hot path to maximum performance (FPGA if Model 3 wins)
2. Deliver UX features that give users an experience they cannot get anywhere else
3. Ship to production with full A/B validation, rollback plan, and post-launch monitoring

**Guarded Line**: This stream starts ONLY after G6. No UX work begins on a model that hasn't been declared winner.

---

## TASK LIST

### P7-1 — Winner Hot-Path Optimization
**File**: `src/optimizations/winner_hotpath.rs`  
**Specialist**: Performance Benchmarker  
**Time**: 8h  
**Depends On**: G6 (winner ID known), latency probes from P1-2 (baseline data)

**Spec**:
- Profile winner model with `perf record` + `cargo flamegraph` — identify top 3 bottlenecks
- Apply targeted optimizations (DO NOT change logic, only execution path):
  - SIMD vectorization for pattern hash comparison (AVX2 if available)
  - Branch prediction hints (`std::hint::likely/unlikely`) at regime check
  - Cache prefetch: `_mm_prefetch` on next container slot before processing current
- Re-benchmark after each optimization (criterion): confirm improvement before next
- Document each optimization with before/after nanosecond measurements

**Acceptance Gate**:
- [ ] Winner p99 latency improved ≥15% from G6 baseline (or document why not achievable)
- [ ] Zero logic changes verified: decisions on identical input unchanged (diff test)
- [ ] All 3 bottlenecks identified and addressed (even if only partially)
- [ ] File ≤150 lines

---

### P7-2 — FPGA Hardware Path (Model 3 Only)
**File**: `src/fpga/winner_fpga.rs`, `fpga/winner_pattern_match.vhd`  
**Specialist**: Rapid Prototyper + Performance Benchmarker  
**Time**: 10h  
**Depends On**: P7-1, G6 (only if Model 3 wins)  
**Skip condition**: If Model 1 or Model 2 wins, this task is CANCELLED

**Spec**:
- Upgrade stub (`pattern_match_stub.rs`) to real PCIe MMIO driver
- VHDL: expand from 16x16 single tier to all 4 tiers in parallel block RAM
- Target: <20ns pattern match (hardware clock cycle budget at 200MHz = 4 cycles)
- Rust driver: `open("/dev/fpga0")`, `mmap` MMIO region, write hash, read result, close on Drop
- Fallback: if `/dev/fpga0` not available at startup, silently fall back to software path

**Acceptance Gate**:
- [ ] Hardware path: <20ns per lookup (oscilloscope or PCIe latency counter)
- [ ] Software fallback: same test suite passes without FPGA present
- [ ] VHDL simulation: 10K pattern lookups produce correct results in ModelSim/GHDL
- [ ] Files ≤150 lines each

---

### P7-3 — User Trust Visualization
**File**: `src/components/trust-score.tsx`  
**Specialist**: Visual Storyteller  
**Time**: 6h  
**Depends On**: G6, P6-4 (dashboard deployed)

**Spec**:
- Animated gauge showing winner model's current "Trust Score" (0.0–5.0)
- Trust score composition visible on hover: 5 sub-scores (accuracy, consistency, latency, drawdown, user feedback)
- Color coding: <3.0 red, 3.0–4.0 amber, >4.0 green (WCAG-compliant palette)
- Trend sparkline: last 7-day trust score history
- Tooltip: "Why this score changed" — shows the sub-score that moved most in last 24h

**UX Research Gate (pre-build)**:
- 5-second test with 5 users: does the gauge communicate model confidence clearly?
- Minimum 80% correct interpretation before build begins

**Acceptance Gate**:
- [ ] Renders at 60fps on mobile (React DevTools profiler)
- [ ] Correct color at threshold boundaries (unit test: score=3.0 → amber, 3.01 → green)
- [ ] Tooltip correctly identifies largest sub-score delta
- [ ] File ≤150 lines

---

### P7-4 — Preemptive Alert System
**File**: `src/components/preemptive-alerts.tsx`, `src/api/alert_engine.rs`  
**Specialist**: UX Researcher (design), Backend Architect (engine)  
**Time**: 6h + 4h  
**Depends On**: P4-5 (divergence monitor), P4-6 (API), G6

**Spec**:
- Alert engine (`alert_engine.rs`): evaluates 4 alert conditions every 60 seconds:
  1. Model divergence >70% for >10 minutes → "Models Disagree — Manual Review Recommended"
  2. Regime change detected → "Market Regime Shift: [old] → [new]"
  3. Winner model latency >1.5× budget → "Performance Degradation Detected"
  4. Drawdown >3% intraday → "Risk Threshold Warning: [X]% Drawdown"
- Frontend component: non-intrusive banner (top of dashboard), dismissible per session
- Alert severity: `INFO` (blue), `WARNING` (amber), `CRITICAL` (red, non-dismissible)
- Push to mobile via Expo Notifications for WARNING+ severity

**Acceptance Gate**:
- [ ] All 4 alert conditions fire correctly on synthetic triggers (integration test)
- [ ] CRITICAL alert cannot be dismissed (UI test)
- [ ] Mobile push delivered within 15 seconds of engine trigger
- [ ] Alert engine ≤100 lines; component ≤120 lines

---

### P7-5 — Cross-Market Ripple Visualization
**File**: `src/dashboards/ripple-viz.tsx`  
**Specialist**: UI Designer  
**Time**: 6h  
**Depends On**: P4-5, P6-4

**Spec**:
- D3.js force-directed graph: nodes = markets, edges = correlation strength (thickness) + direction (arrow)
- Edge color: positive correlation = green, negative = red, neutral = grey
- Pulsing animation on active lead-lag relationship (opacity pulse 1.0→0.5→1.0, 2s period)
- Click on market node: side panel shows last 10 ripple events for that market
- Time slider: replay last 4 hours of correlation evolution
- Brand Guardian review: animation speed, color palette, typography must match design system

**Acceptance Gate**:
- [ ] 10-market graph renders at 60fps (no dropped frames during force simulation)
- [ ] Time slider correctly replays correlation data (integration test with known dataset)
- [ ] WCAG 4.5:1 contrast on all labels
- [ ] File ≤200 lines

---

### P7-6 — "Why This Trade" Explainer
**File**: `src/components/trade-explainer.tsx`  
**Specialist**: Visual Storyteller  
**Time**: 4h  
**Depends On**: G6, P4-6

**Spec**:
- Per-decision narrative panel: shown when user clicks any trade in the decision feed
- 3 elements per explanation:
  1. Pattern signature: visual representation of the triggering 64-bar ternary sequence (bar chart, 64 bars)
  2. Signal chain: which signals fired (Hurst? Bidir match? Elastic extreme?) with confidence contribution
  3. Outcome prediction: "Model expects [direction] move of ~[X]% within [timeframe]"
- Truncated to 3 key factors (not all 11 dimensions) — cognitive load constraint
- Mobile: swipe up to expand full explanation

**Acceptance Gate**:
- [ ] 5-second test: 5/5 users correctly identify trade direction from explainer
- [ ] Signal chain correctly attributed to winner model's actual decision logic (unit test)
- [ ] Mobile swipe-up works on iOS and Android (manual test)
- [ ] File ≤120 lines

---

### P7-7 — Personalized Model Tuning Interface
**File**: `src/components/model-tuning.tsx`  
**Specialist**: Frontend Developer  
**Time**: 6h  
**Depends On**: G6, P6-4

**Spec**:
- Sliders for 3 user-facing parameters (mapped to winner model internals):
  1. Aggression (0-10): scales confidence threshold for signal emission
  2. Risk Tolerance (0-10): scales drawdown halt threshold
  3. Regime Focus (Balanced / Trending / Mean-Revert): weights regime-specific Sharpe
- Preview panel: shows simulated effect on last 30-day backtest (recalculated client-side, pre-loaded data)
- Save: persists settings to user profile (localStorage + API sync)
- Reset: reverts to winner model defaults
- Validation: Zod schema on all inputs; no out-of-range values accepted

**Acceptance Gate**:
- [ ] Slider change triggers preview recalculation in <500ms (client-side)
- [ ] Zod validation rejects out-of-range input (property test: random values outside [0,10])
- [ ] Settings persist across page reload (localStorage round-trip)
- [ ] File ≤180 lines

---

### P7-8 — Gamification: Prediction Streaks
**File**: `src/components/prediction-streaks.tsx`  
**Specialist**: UI Designer + Trend Researcher  
**Time**: 4h  
**Depends On**: G6, P4-1 (decision accuracy history)

**Spec**:
- Show user's "streak": consecutive days the winning model's predictions were correct
- Visual: fire emoji animation at streak ≥7 days; lightning bolt at ≥14 days (brand-approved animations)
- Leaderboard: anonymous streak leaderboard (opt-in), top 10 users
- Trend Researcher gate: confirm streak gamification is in 1-4 week cultural momentum window before building
- Brand Guardian approval: animation style, color palette

**Acceptance Gate**:
- [ ] Streak count correct after synthetic win/loss injection (unit test)
- [ ] Animation fires at exactly day 7 and day 14 thresholds
- [ ] Leaderboard opt-in correctly excludes non-opted users
- [ ] File ≤100 lines

---

### P7-9 — Launch & Post-Launch Monitoring
**Specialist**: Project Shipper  
**Time**: 4h (coordination, not coding)  
**Depends On**: All P7-1..P7-8 complete

**Launch Checklist** (all must be ✅ before production deploy):
- [ ] Winner model passes G6 scoring gate
- [ ] P7-1 hot-path optimization committed and benchmarked
- [ ] All P7-3..P7-8 components A/B tested (min 48h, min 50 users per variant)
- [ ] Rollback plan documented: `git revert` to pre-Phase-7 tag restores previous behavior
- [ ] Monitoring dashboard T+0: all 4 panels live, alerts configured
- [ ] Post-launch monitoring schedule: T+1 day, T+3 days, T+7 days, T+30 days retrospective

**Blue-Green Deploy**:
```
1. Deploy winner model to Green environment
2. Route 5% of live traffic to Green (canary)
3. Monitor 4 hours: latency, drawdown, divergence
4. If all green: route 50%, monitor 2 hours
5. If all green: route 100%, retire Blue
6. If any red: instantly revert to Blue (single config change)
```

---

## STREAM DEPENDENCY CHAIN

```
Week 7 Day 1:  P7-1 (Hot-path) starts immediately on G6
               P7-3, P7-4, P7-5, P7-6, P7-7, P7-8 all start in parallel Day 1

Week 7 Day 2:  P7-2 (FPGA) starts if Model 3 won [parallel with others]

Week 7 Day 3:  All UX components in A/B test (staging environment)

Week 7 Day 4:  A/B results reviewed; UX components adjusted if needed

Week 7 Day 5:  P7-9 launch checklist run; blue-green deploy initiated

Gate G7 (end of Week 7):
  ✅ Winner model p99 latency improved ≥15%
  ✅ All UX components A/B positive (or neutral — no regression)
  ✅ User trust score >4.2/5.0 (user survey, n≥20)
  ✅ Mobile app live (TestFlight + Play Store beta)
  ✅ Blue-green deploy complete, all traffic on Green
```

---

## ROLLBACK PROCEDURES

| Failure | Rollback Action |
|---------|----------------|
| P7-1 optimization introduces logic error | `git revert` optimization commit; re-run diff test |
| FPGA driver crashes on `/dev/fpga0` open | Fallback to software in ≤1ms (watchdog in Rust Drop impl) |
| UX A/B test shows regression | Revert component to previous version; keep old variant live |
| Trust score <4.0 after launch | Disable gamification features (P7-8); investigate P7-6 explainer clarity |
| Blue-green canary >5% error rate | Immediate revert to Blue; post-mortem within 24h |

---

## FILE MANIFEST

| File | Owner | Max Lines | Depends On |
|------|-------|-----------|------------|
| `src/optimizations/winner_hotpath.rs` | Performance Benchmarker | 150 | G6 |
| `src/fpga/winner_fpga.rs` | Rapid Prototyper | 150 | P7-1 (Model 3 only) |
| `fpga/winner_pattern_match.vhd` | Rapid Prototyper | 150 | P7-1 (Model 3 only) |
| `src/components/trust-score.tsx` | Visual Storyteller | 150 | G6, P6-4 |
| `src/components/preemptive-alerts.tsx` | UX Researcher | 120 | P4-5, P4-6 |
| `src/api/alert_engine.rs` | Backend Architect | 100 | P4-5 |
| `src/dashboards/ripple-viz.tsx` | UI Designer | 200 | P4-5, P6-4 |
| `src/components/trade-explainer.tsx` | Visual Storyteller | 120 | G6, P4-6 |
| `src/components/model-tuning.tsx` | Frontend Developer | 180 | G6, P6-4 |
| `src/components/prediction-streaks.tsx` | UI Designer | 100 | G6, P4-1 |

---

## GUARDED LINES (Coding Agents Must Not Cross)

1. **No UX work before G6** — this is the hardest gate; Trend Researcher + Brand Guardian input required before building P7-8
2. **No logic changes in P7-1** — hot-path optimization is execution-path only; diff test must confirm zero decision changes
3. **FPGA task is Model 3-conditional** — if Model 1 or 2 wins, P7-2 is cancelled without replacement
4. **A/B test minimum runtime is 48h** — no UX component ships to production without meeting minimum exposure
5. **Blue-green rollback must remain live for 30 days** — Blue environment not retired until T+30 post-launch retrospective passes
