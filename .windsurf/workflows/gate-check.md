---
description: BAM build stream gate enforcement — verify each G0→G7 acceptance gate is met before the next stream begins
---

# /gate-check — BAM Stream Gate Enforcement

Run this workflow at each stream transition to prevent work starting before prerequisites are met.
The user is the orchestrator — run the appropriate step manually at each transition.

## Gate Map

```
G0 — Contract Sign-Off        → unlocks Stream 2 (Infrastructure)
G1 — Infrastructure Ready     → unlocks Stream 3 (Model 1) + Stream 4 (Model 2)
G2 — Model 1 Baseline         → unlocks Stream 5 (Model 3)
G3 — All Models Running       → unlocks Stream 6 (Testing)
G4 — Testing Complete         → unlocks Stream 7 (Measurement)
G5 — Measurement Dashboard    → unlocks Stream 8 (UX + Launch)
G6 — Winner Selected          → unlocks blue-green deploy
G7 — Production Live          → milestone complete
```

---

## G0 — Contract Sign-Off Gate

**Blocks**: All implementation coding until this passes.

Checklist (from `01_INTEGRATION_CONTRACTS.md`):
- [ ] Contract 1: Signal Router → OMS message schema frozen
- [ ] Contract 2: OMS → Risk Bus interface frozen
- [ ] Contract 3: Market Data → SHM bridge wire format frozen
- [ ] Contract 4: Model output → Scoring API schema frozen
- [ ] Contract 5: Scoring API → Dashboard WebSocket schema frozen
- [ ] Contract 6: Deploy harness → blue-green cutover protocol frozen
- [ ] CCB sign-off recorded with date and reviewer names

Verify:
```bash
grep -c "SIGNED" .planning/01_INTEGRATION_CONTRACTS.md
# Must return 6
```

---

## G1 — Infrastructure Ready Gate

**Requires G0 passed.**

Checklist (from `02_STREAM_INFRASTRUCTURE.md`):
- [ ] Polygon.io WebSocket batch ingest running — 5 symbols confirmed
- [ ] QuestDB receiving tick data — verify with `questdb-client` query
- [ ] Redis cache layer operational
- [ ] GitHub Actions CI pipeline green on `main`
- [ ] Paper trading mode active (`PAPER_TRADING=true` in env)
- [ ] `cargo check --package oms-engine` error count not increased

Verify:
```bash
cargo check --package oms-engine 2>&1 | grep "^error" | wc -l
# Must equal pre-infrastructure baseline or lower
git log --oneline -5
```

---

## G2 — Model 1 Baseline Gate

**Requires G1 passed.**

Checklist (from `03_STREAM_MODEL1.md`):
- [ ] BAM grid deterministic signals firing on live data
- [ ] p50 signal latency < 500µs measured
- [ ] p99 signal latency < 2ms measured
- [ ] Sharpe ratio baseline recorded in `07_STREAM_MEASUREMENT.md`
- [ ] Zero RiskBus bypass incidents in logs
- [ ] All Model 1 unit tests passing

Verify:
```bash
cargo test --package oms-engine -- --nocapture 2>&1 | grep -E "(PASS|FAIL|test result)"
```

---

## G3 — All Models Running Gate

**Requires G2 passed.**

Checklist (from `04_STREAM_MODEL2.md` + `05_STREAM_MODEL3.md`):
- [ ] Model 2 physics-ML producing signals (even if unoptimized)
- [ ] Model 3 binary assembly stub executing (FPGA sim acceptable if hardware unavailable)
- [ ] All three models writing to scoring API endpoint
- [ ] No model is bypassing RiskBus
- [ ] Paper trading running all three models simultaneously

---

## G4 — Testing Complete Gate

**Requires G3 passed.**

Checklist (from `06_STREAM_TESTING.md`):
- [ ] Level 1 unit tests: 100% pass rate
- [ ] Level 2 integration tests: SHM bridge, OMS→RiskBus, scoring API
- [ ] Level 3 backtest: minimum 90-day window run for each model
- [ ] Level 4 paper trading: minimum 5-day live run for each model
- [ ] Level 5 Monte Carlo: 10,000 scenario runs completed
- [ ] Statistical significance: Bonferroni-corrected p < 0.05 for winner

---

## G5 — Measurement Dashboard Gate

**Requires G4 passed.**

Checklist (from `07_STREAM_MEASUREMENT.md`):
- [ ] 12-dimension scoring matrix populated for all 3 models
- [ ] Executive dashboard rendering live data
- [ ] Winner model identified with statistical backing
- [ ] Loser model rollback plan documented

---

## G6 — Winner Selected Gate

**Requires G5 passed.**

Checklist (from `08_STREAM_UX.md`):
- [ ] Blue-green deployment config validated in staging
- [ ] Rollback procedure tested (can revert in < 5 minutes)
- [ ] UX elevated for winner model's dashboard
- [ ] legal-compliance-checker sign-off obtained
- [ ] Monitoring alerts configured for production

---

## G7 — Production Live Gate

**Requires G6 passed.**

Checklist:
- [ ] Blue-green cutover executed
- [ ] 15-minute monitoring window clean (zero anomalous fills)
- [ ] All agents notified of live status
- [ ] MASTER_THREE_MODEL_BUILD_PLAN.md updated to COMPLETE
- [ ] Milestone retrospective scheduled

---

## Usage

At each gate transition, paste the relevant checklist section into Windsurf and ask:

```
Run /gate-check for G[N].
Read each checklist item, verify it against the current codebase and logs,
and report PASS or FAIL with evidence for each item.
Do not proceed to the next stream until all items pass.
```
