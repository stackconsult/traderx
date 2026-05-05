# Performance Engineering Skill

## When to activate
Load when: latency regression, benchmark failures, HFT <100ns targets, memory profiling,
CPU cache tuning, throughput bottlenecks, or any task where "fast enough" must be proven.

## 2026 Evidence-Based Targets (HFT baseline)

| Operation | Target | Proof method |
|-----------|--------|-------------|
| Risk check per order | < 100ns | `criterion` benchmark |
| Signal → order routing | < 5μs | `tokio::time::Instant` |
| Fill processing | < 1μs | flamegraph + perf |
| WAL write | < 10μs | `tokio::fs` async |
| MCP round-trip (local) | < 2ms | `curl -w "%{time_total}"` |
| Ollama inference (local) | < 500ms | `/api/generate` timing |

## Profiling Toolkit

```bash
# Flamegraph (cargo-flamegraph)
cargo flamegraph --bin main -- --profile 30s

# Criterion benchmark
cargo bench --package oms-engine -- risk_check

# Heap profiling
cargo build --release && heaptrack ./target/release/main

# CPU cache misses
perf stat -e cache-misses,cache-references ./target/release/main

# Async task timing
TOKIO_CONSOLE=1 cargo run --features tokio-console

# Memory layout
cargo build --release && objdump -d target/release/main | head -200
```

## Lock-Free Patterns (Rust — always preferred in hot path)

```rust
// ✅ Lock-free counter
use std::sync::atomic::{AtomicI64, Ordering};
static FILLS: AtomicI64 = AtomicI64::new(0);
FILLS.fetch_add(1, Ordering::Relaxed);  // hot path: Relaxed OK

// ✅ Lock-free map (DashMap — already in portfolio-aggregation)
use dashmap::DashMap;
let positions: DashMap<String, AtomicI64> = DashMap::new();

// ❌ Never in hot path
use std::sync::Mutex;  // 40-100ns contention cost

// ✅ Channel for cross-thread communication
use tokio::sync::mpsc;  // bounded, backpressure-aware
```

## Benchmark Pattern (criterion — mandatory for any hot-path claim)

```rust
// benches/risk_check.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_risk_check(c: &mut Criterion) {
    let risk = RiskBus::new(1_000_000.0, -2000);
    c.bench_function("risk_check", |b| {
        b.iter(|| {
            risk.check_symbol(black_box("AAPL"), black_box(1000.0))
        })
    });
}

criterion_group!(benches, bench_risk_check);
criterion_main!(benches);
```

## Memory Optimisation Rules

```rust
// Prefer stack allocation
let buf = [0u8; 256];  // ✅ stack
let buf = vec![0u8; 256];  // ❌ heap — avoid in hot path

// Cache-line alignment for frequently accessed structs
#[repr(align(64))]
struct HotData { value: AtomicI64 }

// Small string optimisation — use SmolStr for symbols
use smol_str::SmolStr;
let sym: SmolStr = "AAPL".into();  // no heap alloc for ≤ 22 chars
```

## Async Performance Rules

```rust
// ✅ Spawn CPU-bound work on blocking thread pool
tokio::task::spawn_blocking(|| heavy_computation()).await?;

// ✅ Batch channel receives to reduce context switches  
while let Ok(msg) = rx.try_recv() { process(msg); }  // drain in loop

// ❌ Never block inside async
std::thread::sleep(Duration::from_millis(100));  // DEADLOCK risk

// ✅ Timeout all external calls
tokio::time::timeout(Duration::from_millis(50), ollama_call()).await?;
```

## Performance Regression Gate

Before every commit touching hot-path code:
```bash
cargo bench --package oms-engine 2>&1 | grep -E "time:|thrpt:" | tee /tmp/bench_result.txt
# Fail if any benchmark regressed > 10% vs baseline
```

Log result in `GENESIS_ROADMAP.md` under `## Performance Benchmarks`.
