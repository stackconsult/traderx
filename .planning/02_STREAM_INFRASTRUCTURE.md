# 02 — STREAM: Infrastructure
**Workstream Owner**: Backend Architect  
**Support**: DevOps Automator, Tool Evaluator  
**Depends On**: 01_INTEGRATION_CONTRACTS.md (all 6 contracts must be signed off)  
**Enables**: All 3 model streams (they cannot start without this stream completing P0-1..P0-4)  
**Sprint**: Week 1 (Days 1-5)  
**Status**: PLANNING — awaiting contract sign-off

---

## STREAM GOAL
Deliver a running, instrumented, CI-gated shared foundation:
- Live market data in via SBE at ≥10K ticks/sec
- Three-tier storage operational (Hot / Warm / Cold)
- Benchmark harness running all 3 models simultaneously (stubs OK at this stage)
- Model registry with git-tag reproducibility
- CI pipeline gates enforcing `cargo check` clean before any merge

---

## TASK LIST (ordered by dependency, parallelism annotated)

### P0-1 — Batch Data Ingestion Pipeline
**File**: `src/infra/batch_ingest.rs`  
**Specialist**: Backend Architect  
**Time**: 8h  
**Depends On**: Contracts 1, 4 (SBE schema, StorageTrait)  
**Parallel With**: P0-3, P0-4 (no shared files)

**Spec**:
- Polygon.io WebSocket adapter + Twelve Data REST fallback
- Parse ticks via SBE (Contract 4) — validated before storage
- Write to Hot tier via `ContainerStorage::write_batch`
- Backpressure: if Hot tier full, drop ticks and emit `IngestOverflow` metric
- Emit `ticks_ingested_total`, `ticks_dropped_total`, `parse_errors_total` (Prometheus counters)

**Acceptance Gate** (verified by Test Writer Fixer):
- [ ] Ingests ≥10,000 ticks/sec sustained for 60 seconds (criterion benchmark)
- [ ] SBE validation rejects malformed ticks without crashing
- [ ] All 3 error metrics increment correctly on injected bad data
- [ ] `cargo check --package oms-engine` shows zero new errors after merge

**Short-Circuit Rule**: If this gate fails, ALL downstream tasks halt. Fix P0-1 before proceeding.

---

### P0-2 — Three-Tier Storage
**Files**: `src/infra/nvme_pool.rs`, `src/infra/questdb_adapter.rs`, `src/infra/s3_parquet_loader.rs`  
**Specialist**: Backend Architect  
**Time**: 10h  
**Depends On**: P0-1 (needs data to validate), Contract 5 (StorageTrait)  
**Parallel With**: P0-3, P0-4

**Spec**:
- **Hot** (`nvme_pool.rs`): memory-mapped NVMe ring buffer, 7-day rolling window, `mmap` + `msync`, 5–7 GB/s sequential throughput target
- **Warm** (`questdb_adapter.rs`): QuestDB ILP over TCP, 90-day retention, SQL query interface for backtesting, batched writes ≥1000 rows/batch
- **Cold** (`s3_parquet_loader.rs`): Arrow `RecordBatch` → Parquet, S3 multipart upload, compressed columnar, 5-year archive
- All three implement `ContainerStorage` trait (Contract 5) — no direct struct imports in callers

**File size rule**: Each file ≤200 lines. Split by tier. No shared state across tiers.

**Acceptance Gate**:
- [ ] Hot tier write ≥5 GB/s (criterion benchmark with 256-byte containers)
- [ ] QuestDB query returns 90-day slice in <2 sec for 1M rows
- [ ] S3 round-trip compress/decompress 1M containers in <30 sec
- [ ] StorageTrait mock passes in <1ms for unit tests

---

### P0-3 — Benchmark Harness
**File**: `tests/benchmark_harness.rs`  
**Specialist**: Test Writer Fixer  
**Time**: 6h  
**Depends On**: Contract 1 (ModelDecision), Contract 6 (ModelMetricsReport)  
**Parallel With**: P0-1, P0-2, P0-4

**Spec**:
- Accepts 3 pluggable `ModelRunner` trait objects (one per model)
- Feeds identical tick stream to all 3 simultaneously (deterministic replay)
- Collects `ModelDecision` from each via standardized channel
- Records `ModelMetricsReport` for each window (configurable: 1min / 5min / 1hr)
- Stub runners for Models 2 and 3 OK at this stage (emit random decisions)
- Output: CSV + JSON per run, git-tagged with model version hash

**Acceptance Gate**:
- [ ] Three stub runners produce output simultaneously without data races
- [ ] Results are deterministic: same seed = same output
- [ ] Harness completes 10K-tick replay in <30 sec wall time
- [ ] Output CSV schema matches `ModelMetricsReport` field names exactly

---

### P0-4 — Model Registry
**File**: `src/infra/model_registry.rs`  
**Specialist**: Backend Architect  
**Time**: 4h  
**Depends On**: None  
**Parallel With**: P0-1, P0-2, P0-3

**Spec**:
- Register model version as `(model_id: u8, git_sha: [u8; 20], params_hash: u64, built_at: u64)`
- Lookup by model_id returns latest registered version
- Reproducible build: same git SHA + same params_hash must produce identical decisions
- Storage: SQLite local file `registry.db` (no external dependency)

**Acceptance Gate**:
- [ ] Two registrations with same SHA + params_hash are idempotent (no duplicate)
- [ ] Lookup returns correct entry after restart (persistence verified)
- [ ] Registry file ≤100 lines

---

### P0-5 — CI/CD Pipeline
**File**: `.github/workflows/three-model-ci.yml`  
**Specialist**: DevOps Automator  
**Time**: 6h  
**Depends On**: P0-1, P0-3 (needs something to test)  
**Parallel With**: P0-4

**Spec**:
- Triggers: push to `main`, `feature/*`, PRs to `main`
- Steps in order:
  1. `cargo check --package oms-engine` — fail-fast on any new error
  2. `cargo test --package oms-engine` — unit tests only, no integration
  3. `cargo bench --bench benchmark_harness -- --sample-size 10` (smoke latency)
  4. Integration test: 60-second ingest replay against stub storage
- Artifact: upload criterion HTML reports per run
- Cache: `~/.cargo/registry` + `target/` keyed by `Cargo.lock` hash
- Budget: ≤10 min wall time total (fail build if exceeded)

**Acceptance Gate**:
- [ ] Pipeline passes green on clean repo state
- [ ] Any `cargo check` error causes immediate red (verified by injecting a compile error)
- [ ] Cache hit reduces total time to ≤4 min on repeat runs

---

### P0-6 — Monitoring Baseline
**Files**: `src/infra/metrics_server.rs`, `infra/grafana/dashboards/three-model-baseline.json`  
**Specialist**: DevOps Automator  
**Time**: 4h  
**Depends On**: P0-1 (needs real metrics to display)  
**Parallel With**: P0-5

**Spec**:
- Prometheus `/metrics` endpoint served from `metrics_server.rs` on port 9100
- Grafana dashboard: 4 panels — Ingest rate, Hot tier write rate, p99 latency by model, circuit breaker status
- Alerting rule: p99 latency for any model >2× budget → PagerDuty webhook
- Docker Compose for local: `docker compose -f infra/monitoring.yml up`

**Acceptance Gate**:
- [ ] Grafana dashboard loads without errors in local Docker
- [ ] Injecting a latency spike triggers alert within 15 seconds
- [ ] `metrics_server.rs` ≤150 lines

---

## STREAM DEPENDENCY CHAIN

```
Day 1-2:   P0-1 (Ingest) ──┬──→ P0-2 (Storage)    [starts Day 2]
           P0-3 (Harness)  │
           P0-4 (Registry) │

Day 3:     P0-2 ────────────┘
           P0-5 (CI) starts after P0-1 + P0-3 complete

Day 4-5:   P0-6 (Monitoring) after P0-1 stabilized
           Stream 3 (Model 1) can begin Day 4 if P0-3 ✅ and P0-1 ✅

Gate G0 (end of Day 5):
  ✅ Ingest ≥10K ticks/sec
  ✅ Hot/Warm/Cold all writing
  ✅ CI green on clean state
  ✅ Harness produces deterministic output
  → If G0 fails, freeze all model streams. Fix infra first.
```

---

## ROLLBACK PROCEDURES

| Failure | Rollback Action |
|---------|----------------|
| P0-1 ingestion crashes on malformed tick | Drop tick, increment counter, continue — NEVER crash |
| Hot tier disk full | Auto-rotate: delete oldest 10% of containers, emit alert |
| QuestDB connection lost | Buffer in memory ring (max 10K ticks), retry with exponential backoff |
| CI pipeline >10 min | Split: move integration tests to nightly schedule |
| Monitoring alert fires falsely | Adjust threshold with documented reason in PR |

---

## FILE MANIFEST

| File | Owner | Max Lines | Depends On |
|------|-------|-----------|------------|
| `src/infra/batch_ingest.rs` | Backend Architect | 200 | Contracts 1, 4 |
| `src/infra/nvme_pool.rs` | Backend Architect | 200 | Contract 5 |
| `src/infra/questdb_adapter.rs` | Backend Architect | 150 | Contract 5 |
| `src/infra/s3_parquet_loader.rs` | Backend Architect | 150 | Contract 5 |
| `src/infra/model_registry.rs` | Backend Architect | 100 | None |
| `src/infra/metrics_server.rs` | DevOps Automator | 150 | None |
| `tests/benchmark_harness.rs` | Test Writer Fixer | 200 | Contracts 1, 6 |
| `.github/workflows/three-model-ci.yml` | DevOps Automator | 100 | P0-1, P0-3 |
| `infra/grafana/dashboards/three-model-baseline.json` | DevOps Automator | N/A | P0-6 |

---

## GUARDED LINES (Coding Agents Must Not Cross)

1. **No heap allocation** in the hot ingest path (`batch_ingest.rs` inner loop)
2. **No direct struct imports** from `nvme_pool.rs`, `questdb_adapter.rs`, or `s3_parquet_loader.rs` in model code — always use `ContainerStorage` trait
3. **No `.unwrap()`** in any infra file — use `?` or explicit `match` with logged errors
4. **No blocking I/O** in tokio tasks — use `tokio::fs`, `tokio::net`
5. **No secrets in code** — API keys via environment variable, never hardcoded
