# MASTER PLAN: Three-Model BAM Grid HFT System

**Status**: DECOMPOSED — 8 workstream files generated, 3 commits on `feature/github-mcp-setup`  
**Target**: Build, test, measure, deploy 3 models with recursive parallel testing + UX optimization from winners.

---

## ENGINEERING DECOMPOSITION INDEX

This monolithic plan has been vertically sliced into 8 independent, commit-verified workstream files. Each file contains: spec, acceptance gates, dependency chain, rollback procedures, file manifest, and guarded lines.

**Read before building**: `01_INTEGRATION_CONTRACTS.md` must be signed off by CCB before any stream starts.

| File | Workstream | Sprint | Gate | Owner |
|------|-----------|--------|------|-------|
| [01_INTEGRATION_CONTRACTS.md](01_INTEGRATION_CONTRACTS.md) | Frozen interface contracts (all 6) | Pre-start | CCB sign-off | Backend Arch + AI Eng + Perf Bench |
| [02_STREAM_INFRASTRUCTURE.md](02_STREAM_INFRASTRUCTURE.md) | Ingest, storage, harness, CI/CD | Week 1 | G0: 10K ticks/sec + CI green | Backend Architect |
| [03_STREAM_MODEL1.md](03_STREAM_MODEL1.md) | Model 1 hardening (instrument only) | Week 1 D4-5 | G1: stress test + <200μs | AI Engineer |
| [04_STREAM_MODEL2.md](04_STREAM_MODEL2.md) | Model 2: PyTorch physics-ML + SHM bridge | Weeks 2-3 | G2: <2ms p99 E2E | AI Engineer |
| [05_STREAM_MODEL3.md](05_STREAM_MODEL3.md) | Model 3: u8 binary assembly + FPGA stub | Weeks 3-5 | G3: <350μs + no heap alloc | Rapid Prototyper |
| [06_STREAM_TESTING.md](06_STREAM_TESTING.md) | 5-level recursive parallel testing | Weeks 5-6 | G4/G5: backtest + paper trading | Test Writer Fixer |
| [07_STREAM_MEASUREMENT.md](07_STREAM_MEASUREMENT.md) | 12-dimension scoring + executive dashboard | Week 6 (parallel) | G6: winner declared | Analytics Reporter |
| [08_STREAM_UX.md](08_STREAM_UX.md) | Winner optimization + UX elevation | Week 7 | G7: trust score >4.2 + deploy | Project Shipper |

### Root Issues Fixed by Decomposition

1. **Monolithic coordination bottleneck** → 8 owned files, each specialist reads only theirs
2. **Sequential chains throttling parallel work** → streams start simultaneously from Day 1
3. **No interface contracts** → `01_INTEGRATION_CONTRACTS.md` frozen before any code
4. **Rigid phase walls** → each stream has its own rollback procedure and short-circuit
5. **Undefined rollback paths** → explicit failure → recovery table in every stream file
6. **Undefined review process** → CCB defined (3 members, quorum rules, 4h SLA)

---

## 1. MODEL DEFINITIONS

| # | Name | Type | Latency | Key Components |
|---|------|------|---------|---------------|
| 1 | Base Foundation | Rule-based 9-layer deterministic | <200μs | `multi_market_grid.rs`, `schema_registry.rs`, `funnel_index.rs`, `pattern_matcher.rs` |
| 2 | Advanced Physics-ML | PyTorch LSTM/Transformer + Takens + Fisher | <2ms | `physics_encoder.py`, `variance_predictor.py`, `pytorch_bridge.rs` |
| 3 | Hyper-Static Binary | u8-level assembly + FPGA acceleration | 200-350μs | `bam_container.rs`, `native_ternary.rs`, `sbe_parser.rs`, `nvme_pool.rs`, `bidir_matcher.rs`, `elastic_detector.rs`, `fpga/pattern_match.vhd` |

---

## 2. PHASE MAP

### PHASE 0: Foundation (Week 1)

| Task | Specialist | File | Time | Depends On | Gate |
|------|-----------|------|------|------------|------|
| Batch data ingestion pipeline | Backend Architect | `infra/batch_ingest.rs` | 8h | None | Ingests 10K ticks/sec; SBE validation <5μs |
| Three-tier storage (Hot/Warm/Cold) | Backend Architect | `infra/data_cache.rs`, `infra/nvme_pool.rs` | 10h | P0-1 | NVMe 5GB/s; QuestDB 90 days; S3 5+ years |
| Benchmark harness | Test Writer Fixer | `tests/benchmark_harness.rs` | 6h | None | Runs all 3 models simultaneously |
| Model registry | Backend Architect | `infra/model_registry.rs` | 4h | None | Git-tag versions; reproducible builds |
| Comparative metrics (12 dimensions) | Analytics Reporter | `tests/comparative_metrics.rs` | 6h | P0-3 | Measures latency, throughput, Sharpe, drawdown, predictability, robustness, cross-market alpha, tail risk, resilience, cognitive load, code complexity, UX impact |
| CI/CD pipeline | DevOps Automator | `.github/workflows/three-model-ci.yml` | 6h | P0-4 | <10 min build; cargo check gates |
| Monitoring baseline | DevOps Automator | `infra/monitoring.rs` | 4h | P0-6 | Prometheus + Grafana; p99 latency alerts |

**Short-Circuit**: If P0-1 fails 10K ticks/sec, HALT all downstream.

---

### PHASE 1: Model 1 Hardening (Week 1, Days 4-5)

| Task | Specialist | File | Time | Gate |
|------|-----------|------|------|------|
| Integrate into benchmark | AI Engineer | `tests/model1_adapter.rs` | 4h | Standardized decision format |
| Latency probes (DPDK) | Performance Benchmarker | `infra/latency_probe.rs` | 3h | Per-component breakdown |
| Property-based tests | Test Writer Fixer | `tests/model1_invariants.rs` | 4h | 1000+ random inputs; deterministic |
| Stress: flash crash | Performance Benchmarker | `tests/model1_stress.rs` | 3h | Survives 1987/2010/2020 replay; CB <1ms |

---

### PHASE 2: Model 2 Build (Weeks 2-3)

| Task | Specialist | File | Time | Depends On |
|------|-----------|------|------|------------|
| Physics encoder | AI Engineer | `ml/physics_encoder.py` | 10h | P0-1 |
| LSTM variance predictor | AI Engineer | `ml/variance_predictor.py` | 8h | P2-1 |
| Rust ↔ PyTorch SHM bridge | Backend Architect | `infra/pytorch_bridge.rs` | 8h | P2-2 |
| Online correlation updater | AI Engineer | `fabric/correlation_updater.rs` | 5h | P2-3 |
| Inference pipeline | AI Engineer | `fabric/model2_pipeline.rs` | 6h | P2-4 |
| Benchmark adapter | Test Writer Fixer | `tests/model2_adapter.rs` | 4h | P2-5 |
| Pre-train (1 year historical) | AI Engineer | `ml/training/` | 12h | P0-2 |

**Guarded Line**: MUST use zero-copy SHM bridge — NO serialization overhead.

---

### PHASE 3: Model 3 Build (Weeks 3-5)

| Task | Specialist | File | Time | Depends On |
|------|-----------|------|------|------------|
| 256-byte BAM container | Rapid Prototyper | `fabric/bam_container.rs` | 6h | P0-1 |
| NativeTernary encoder | Rapid Prototyper | `fabric/native_ternary.rs` | 5h | P3-1 |
| SBE parser | Backend Architect | `infra/sbe_parser.rs` | 6h | P3-2 |
| Immutable container + SHA-256 | Backend Architect | `fabric/immutable_container.rs` | 6h | P3-1 |
| NVMe flash pool | Backend Architect | `infra/nvme_pool.rs` | 8h | P3-4 |
| Bidirectional matcher | Rapid Prototyper | `fabric/bidir_matcher.rs` | 8h | P3-1 |
| Multi-level dictionary | Rapid Prototyper | `fabric/pattern_dict.rs` | 6h | P3-6 |
| Elastic extreme detector | AI Engineer | `fabric/elastic_detector.rs` | 8h | P3-7 |
| x86-64 ASM detection | Performance Benchmarker | `asm/elastic_detect.s` | 6h | P3-8 |
| Multi-timeframe stream | Backend Architect | `fabric/mtf_stream.rs` | 8h | P3-5 |
| Pattern state machine (6 states) | AI Engineer | `fabric/pattern_fsm.rs` | 6h | P3-10 |
| FPGA matcher stub | Rapid Prototyper | `fpga/pattern_match.vhd` | 6h | P3-7 |
| Benchmark adapter | Test Writer Fixer | `tests/model3_adapter.rs` | 4h | P3-11 |

**Guarded Line**: NO heap allocation during trading hours. All operations at u8 level.

---

### PHASE 4: Comparative Testing (Week 5)

| Task | Specialist | File | Time | Gate |
|------|-----------|------|------|------|
| Side-by-side backtest | Test Writer Fixer | `tests/comparative_backtest.rs` | 8h | Same data, all 3 models, identical conditions |
| Statistical significance | Test Results Analyzer | `tests/significance_tests.rs` | 6h | Paired t-tests; p<0.05; Bonferroni correction |
| A/B framework | Experiment Tracker | `tests/ab_test_framework.rs` | 6h | Random assignment; 95% CI; early stopping |
| Performance regression | Performance Benchmarker | `benches/latency_regression.rs` | 6h | p50/p99/p999 per model |
| Divergence monitor | AI Engineer | `fabric/divergence_monitor.rs` | 4h | Flags >30% disagreement |
| Comparative dashboard API | Backend Architect | `api/comparative_dashboard.rs` | 6h | <50ms response |

---

### PHASE 5: Recursive Parallel Testing (Week 6)

| Level | Name | Data | Parallelism | Trigger | Duration | Gate |
|-------|------|------|-------------|---------|----------|------|
| L1 | Unit | Synthetic 1M ticks | Components | Every commit | <5 min | Pass rate >95% |
| L2 | Integration | 1 day historical | All 3 models | Every 4h | 1 hour | Agreement on obvious patterns |
| L3 | Backtest | 90 days (QuestDB) | Models parallel | Nightly | 4 hours | Sharpe, drawdown, win rate |
| L4 | Live Paper | Live tick stream | All 3 models | Market hours | Continuous | Latency, slippage, accuracy |
| L5 | Monte Carlo | 10K stress scenarios | 30K runs | Weekly | 8 hours | Tail risk, CB effectiveness |

**Short-Circuit**: If L4 shows >5% drawdown for ANY model, auto-halt and alert.

---

### PHASE 6: Measurement Framework (Week 6, parallel)

| Dimension | Metric | Target | Weight |
|-----------|--------|--------|--------|
| Latency | p50/p99/p999 E2E | <200μs/<350μs/<500μs | 15% |
| Throughput | Events/sec, trades/sec | >100K trades/sec | 10% |
| Predictive Accuracy | Precision/recall | >80%/>75% | 15% |
| Robustness | Sharpe across 3 regimes | >1.5/>2.0/>1.2 | 15% |
| Cross-Market Alpha | % P&L from correlation | >20% | 10% |
| Tail Risk | Max drawdown, CVaR 95% | <8%, <5% | 10% |
| Resilience | Uptime, CB trigger | >99.99%, <1ms | 5% |
| Risk-Adjusted Return | Sharpe/Sortino/Calmar | >2.0/>2.5/>3.0 | 10% |
| Cognitive Load | Human interventions/day | <5 | 3% |
| Code Complexity | CC per function | <10 avg | 4% |
| UX Impact | User trust score | >4.2/5.0 | 3% |

**Guarded Line**: NO model declared winner on fewer than 6 dimensions.

---

### PHASE 7: Winner Optimization & UX Elevation (Week 7)

| Task | Specialist | File | Time | Depends On |
|------|-----------|------|------|------------|
| Hot-path optimization | Performance Benchmarker | `optimizations/winner_hotpath.rs` | 8h | P6 scoring |
| FPGA hardware path | Rapid Prototyper | `fpga/winner_fpga.vhd` | 10h | P7-1 (if Model 3 wins) |
| User trust visualization | Visual Storyteller | `dashboards/trust-score.tsx` | 6h | P6-4 |
| Preemptive alerts | UX Researcher | `components/preemptive-alerts.tsx` | 6h | P7-3 |
| Cross-market ripple viz | UI Designer | `dashboards/ripple-viz.tsx` | 6h | P7-3 |
| Personalized tuning | Frontend Developer | `components/model-tuning.tsx` | 6h | P7-5 |
| "Why This Trade" explainer | Visual Storyteller | `components/trade-explainer.tsx` | 4h | P7-4 |
| Gamification: streaks | UI Designer | `components/prediction-streaks.tsx` | 4h | P7-6 |
| Mobile companion app | Mobile App Builder | `mobile/traderx-metrics/` | 10h | P7-7 |

---

## 3. SPECIALIST AGENT ASSIGNMENT MATRIX

| Specialist | Primary Phase(s) | Key Contribution | Audit Notes |
|------------|-----------------|------------------|-------------|
| **Backend Architect** | 0, 2, 3, 4, 5 | Infrastructure, bridges, storage, APIs | Max 200 lines/file; SOLID; strong typing |
| **AI Engineer** | 1, 2, 3, 4, 5 | ML models, inference pipelines, SHM bridge | <200 lines/file; typed interfaces; no hardcoded prompts |
| **Rapid Prototyper** | 3, 7 | Binary containers, FPGA stubs, hot-path asm | Type-safe; document TODOs; 6-day sprint discipline |
| **Performance Benchmarker** | 1, 3, 4, 5, 7 | DPDK probes, SIMD optimization, FPGA timing | Web Vitals + backend latency targets; profile before optimize |
| **Test Writer Fixer** | 0, 1, 2, 3, 4, 5 | All adapters, invariants, integration harness | Property-based; no mocks for core trading; REAL components |
| **DevOps Automator** | 0, 5 | CI/CD, monitoring, deployment, IaC | <200 lines/IaC file; secrets in vault; blue-green/canary |
| **Analytics Reporter** | 0, 5, 6 | Metrics collection, scoring engine, reports | Cohort analysis; confidence intervals; no vanity metrics |
| **Experiment Tracker** | 4, 6 | A/B framework, statistical rigor, feature flags | p<0.05, power>0.80, sample>1000, min 1 week runtime |
| **UX Researcher** | 7 | Preemptive alerts, trust scoring, user journeys | 5-second tests; 5-user minimum; action-oriented insights |
| **UI Designer** | 7 | Ripple viz, tuning interface, streaks, mobile | Tailwind-first; <200 lines/component; 8px grid; dark mode |
| **Visual Storyteller** | 6, 7 | Comparison reports, trade explainers, trust viz | 5-act structure; D3.js/Flourish; data-to-insight in <5s |
| **Frontend Developer** | 6, 7 | Dashboards, components, React/Next.js | Code splitting; TanStack Query; Zod validation; <200 lines |
| **Mobile App Builder** | 7 | React Native companion app | Offline cache; push notifications; <150MB; 60fps |
| **Project Shipper** | 7 | Launch coordination, GTM, release management | Checklist-driven; rollback plan; post-launch monitoring |
| **Sprint Prioritizer** | All | RICE scoring, capacity planning, scope management | Value vs effort matrix; 70-20-10 rule; buffer for issues |
| **Studio Producer** | All | Cross-team coordination, resource allocation | Daily standups 15min; blocked → escalate in 2h; no silos |
| **Tool Evaluator** | 0, 3 | Technology selection, POC validation | Speed to market 40% weight; <2h setup; 6-day trial |
| **Trend Researcher** | 7 | Viral features, market timing, competitive intel | 1-4 week momentum = perfect; >8 weeks = saturated |
| **Brand Guardian** | 7 | Design tokens, consistency, cross-platform harmony | 4px grid; WCAG 4.5:1; component library; versioned assets |
| **Workflow Optimizer** | All | Process efficiency, human-AI collaboration | Batch similar; pipeline parallel; cache reuse; fail fast; prefetch next |

---

## 4. DEPENDENCY GRAPH

```
PHASE 0 (Week 1)
  ├─ P0-1 (Ingestion) ──→ P0-2 (Storage)
  ├─ P0-3 (Harness) ────→ P0-5 (Metrics)
  ├─ P0-4 (Registry) ──→ P0-6 (CI/CD) ──→ P0-7 (Monitoring)

PHASE 1 (Days 4-5)
  └─ All depend on P0-3

PHASE 2 (Weeks 2-3)
  ├─ P2-1 (Encoder) ──→ P2-2 (LSTM)
  ├─ P2-3 (Bridge) ────→ P2-4 (Updater) ──→ P2-5 (Pipeline)
  └─ P2-5 ──→ P2-6 (Adapter); P2-7 (Training) starts Day 1, parallel

PHASE 3 (Weeks 3-5)
  ├─ P3-1 (Container) ──→ P3-2 (Ternary) ──→ P3-3 (SBE)
  ├─ P3-1 ──→ P3-4 (Immutable) ──→ P3-5 (NVMe)
  ├─ P3-1 ──→ P3-6 (Bidir) ──→ P3-7 (Dict) ──→ P3-8 (Elastic) ──→ P3-9 (ASM)
  ├─ P3-7 ──→ P3-12 (FPGA)
  ├─ P3-5 ──→ P3-10 (MTF)
  └─ P3-10 + P3-8 ──→ P3-11 (FSM) ──→ P3-13 (Adapter)

PHASE 4 (Week 5)
  └─ P4-1 (Backtest) ──→ [P4-2, P4-3, P4-4, P4-5, P4-6] all parallel

PHASE 5 (Week 6)
  └─ All depend on P4-1; [P5-1..P5-6] parallel after

PHASE 6 (Week 6, parallel with 5)
  ├─ P6-1 (Collector) ──→ P6-2 (Scoring) ──→ P6-3 (Report) ──→ P6-4 (Dashboard)
  └─ P6-4 ──→ P6-5 (Mobile)

PHASE 7 (Week 7)
  ├─ P7-1 (Optimization) ──→ P7-2 (FPGA hardware)
  └─ P6-4 ──→ [P7-3..P7-9] all parallel
```

---

## 5. WORKFLOW GUARDRAILS

### Batching

- **Week 1**: All infrastructure tasks batched (P0-1..P0-7)
- **Week 3 Days 1-2**: All container core tasks batched (P3-1..P3-3)
- **Week 7**: All UX elevation tasks batched (P7-3..P7-9)

### Pipelining

- P2-7 (training) prefetched during P2-1..P2-3 build
- P3-12 (FPGA synthesis) prefetched during P3-8..P3-11
- P6-5 (mobile) prefetched during P6-1..P6-4

### Caching

- Pre-compute xxHash64 for all historical containers (one-time, reuse forever)
- Cache correlation matrices daily snapshots
- Cache backtest results keyed by (model_version, date_range, params_hash)

### Short-Circuiting

- If base model says NO → immediately block, skip Models 2/3 inference
- If FPGA hash not in dictionary → skip elastic detection (16ns)
- If any model latency >2× budget → fall back to base model
- If L4 paper trading drawdown >5% → auto-halt that model

---

## 6. DATA ARCHITECTURE: THREE-TIER STORAGE

| Tier | Technology | Retention | Use Case | Performance |
|------|-----------|-----------|----------|-------------|
| Hot | NVMe SSD Array (mmap) | 7 days | Real-time testing, latency benchmarks | 5-7 GB/s sequential |
| Warm | QuestDB | 90 days | Backtesting, pattern dictionary training | 12-36x InfluxDB speed |
| Cold | S3 + Parquet | 5+ years | Initial training, stress testing, Monte Carlo | Compressed, columnar |

---

## 7. ACCEPTANCE CRITERIA: GO/NO-GO GATES

| Gate | Criteria | Phase |
|------|----------|-------|
| G0 | P0-1 ingests 10K ticks/sec; P0-6 CI <10 min | Phase 0 |
| G1 | Model 1 passes stress test (flash crash); latency <200μs | Phase 1 |
| G2 | Model 2 inference <2ms p99; SHM bridge <1μs | Phase 2 |
| G3 | Model 3 WORM seal verified; FPGA stub compiles; no heap alloc | Phase 3 |
| G4 | All 3 models produce standardized decisions; statistical significance p<0.05 | Phase 4 |
| G5 | L1-L4 all passing; L5 Monte Carlo VaR <5% | Phase 5 |
| G6 | Winner scored on ≥6 dimensions; confidence interval excludes others | Phase 6 |
| G7 | UX improvements A/B tested; trust score >4.2; mobile app live | Phase 7 |

---

## 8. RISK MITIGATION

| Risk | Mitigation | Owner |
|------|------------|-------|
| PyTorch inference >2ms | Batch every 100ms; async to trade path; base model guard | AI Engineer |
| Advanced model overfits | Weekly retrain; divergence monitor; base model veto | AI Engineer |
| Cross-market correlation breaks | Online updater; fallback to uncorrelated; regime detection | Backend Architect |
| FPGA unavailable | x86-64 ASM fallback; still <500μs | Performance Benchmarker |
| NVMe failure | RAID 10; hot spare; automatic failover to warm tier | DevOps Automator |
| Model 3 complexity exceeds team | Rapid Prototyper + Performance Benchmarker pair; weekly code review | Studio Producer |

---

## 9. TECHNOLOGY STACK

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| Core Language | Rust | Zero-cost abstractions; memory safety; <200μs targets |
| ML | PyTorch (Python) | LSTM/Transformer ecosystem; JAX migration path |
| Bridge | POSIX SHM + lock-free ring buffer | Zero-copy; <1μs handoff |
| Database (Warm) | QuestDB | 12-36x InfluxDB; SQL-native; time-series optimized |
| Storage (Hot) | NVMe Gen4 + mmap | 5-7 GB/s; kernel bypass potential |
| Storage (Cold) | S3 + Parquet | Cost-effective; columnar analytics |
| Frontend | React + Next.js + Tailwind + D3.js | Component reuse; rapid iteration; data viz |
| Mobile | React Native | Cross-platform; shared logic; push notifications |
| CI/CD | GitHub Actions | Native Rust support; matrix builds; artifact caching |
| Monitoring | Prometheus + Grafana | Industry standard; p99 alerting; custom metrics |
| Testing | cargo test + criterion + proptest | Property-based; benchmark regression; fuzzing |

---

## 10. SPECIALIST AUDIT SUMMARY

Each of the 20 specialists reviewed this plan and appended the following concerns/requirements:

1. **Backend Architect**: All files <200 lines; SOLID principles; dependency injection; no N+1 queries
2. **AI Engineer**: Typed interfaces for all ML inputs/outputs; <200 lines per Python file; retry logic with backoff; cost tracking per inference
3. **Rapid Prototyper**: Document all TODOs in TECHNICAL_DEBT.md; type-safe even in prototypes; 6-day sprint discipline
4. **Performance Benchmarker**: Profile BEFORE optimizing; DPDK for hardware timestamping; Web Vitals + backend metrics; power consumption tracking
5. **Test Writer Fixer**: Property-based tests for all invariants; REAL components (no mocks) for trading flow; <10 min test suite
6. **DevOps Automator**: IaC <200 lines; secrets in vault (never .env); blue-green deployment; <10 min CI builds
7. **Analytics Reporter**: Confidence intervals on ALL metrics; cohort analysis; no vanity metrics; automated weekly reports
8. **Experiment Tracker**: p<0.05 with Bonferroni; minimum 1000 users per variant; 1-4 week runtime; early stopping rules
9. **UX Researcher**: 5-second tests for all new UX; 5-user minimum; action-oriented insights; synthesize within 24h
10. **UI Designer**: Tailwind-first; <200 lines/component; 8px spacing grid; dark mode; WCAG 4.5:1 contrast
11. **Visual Storyteller**: 5-act narrative structure; D3.js for data viz; <5 seconds to insight; mobile-optimized
12. **Frontend Developer**: Code splitting; TanStack Query; Zod validation; <200KB initial bundle; <1.8s FCP
13. **Mobile App Builder**: React Native; offline cache last 24h; push alerts; <150MB; 60fps; crash rate <0.1%
14. **Project Shipper**: Launch checklist; rollback plan; post-launch monitoring T+0 to T+30 days
15. **Sprint Prioritizer**: RICE scoring every task; 70-20-10 capacity rule; buffer 20% for unexpected issues
16. **Studio Producer**: Daily 15-min standups; blocked → escalate in 2h; feature teams; no silos; WIP limits
17. **Tool Evaluator**: POC before adoption; speed to market 40% weight; community size matters; avoid vendor lock-in
18. **Trend Researcher**: 1-4 week momentum = build now; >8 weeks = find differentiator; TikTok/App Store monitoring
19. **Brand Guardian**: Design tokens (CSS custom properties); versioned assets; consistent across web + mobile
20. **Workflow Optimizer**: Batch similar tasks; pipeline parallel work; cache previous results; fail fast; prefetch next steps

---

**Next Step**: Review this master plan with Strategy Team + PM + Engineering Leads. Once approved, each specialist receives their assigned task files with guarded lines and begins systematic execution.
