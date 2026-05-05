# 01 — INTEGRATION CONTRACTS v1.0
**Status**: FROZEN — No stream starts implementation until this file has CCB sign-off.  
**CCB (Change Control Board)**: Backend Architect + AI Engineer + Performance Benchmarker (unanimous vote required for layout changes; 2/3 for additive).  
**Purpose**: Every boundary between workstreams is defined here. This prevents interface drift that causes integration failure in Phases 4-5.

---

## CONTRACT 1 — ModelDecision (Decision Output)

**Owner**: Backend Architect  
**Consumers**: Benchmark harness, Divergence monitor, Paper trading engine, Dashboard API, all 3 models emit this.

```rust
// src/contracts/decision.rs
#[repr(C, align(64))]
pub struct ModelDecision {
    pub model_id: u8,              // 1=Base, 2=Advanced, 3=Binary
    pub symbol_id: u16,            // Index into SymbolRegistry
    pub side: u8,                  // 0=NoOp, 1=Long, 2=Short, 3=Close
    pub confidence: u8,            // 0-255 maps to 0.0-1.0
    pub quantity_lots: u32,        // Standardized lot units
    pub price_limit: u64,          // Fixed-point 10^8 (e.g. 150_25000000 = $150.25)
    pub pattern_signature: [u8; 8],// xxHash64 of triggering pattern
    pub timestamp_ns: u64,         // DPDK hardware clock, Unix nanoseconds
    pub regime_tag: u8,            // 0=Choppy, 1=Trending, 2=Volatile, 3=Halted
    pub _pad: [u8; 11],            // Pad to 48 bytes total
}
// CONSTRAINT: size_of::<ModelDecision>() == 48. Never change without version bump.
pub const DECISION_SCHEMA_VERSION: u8 = 1;
```

**Guardrail**: Any struct size change requires all 5 consumers to update simultaneously. Gate: `static_assert!(size_of::<ModelDecision>() == 48)`.

---

## CONTRACT 2 — BAMContainer (256-byte Grid Cell)

**Owner**: Rapid Prototyper (layout), Performance Benchmarker (alignment verification)  
**Consumers**: Model 1 (header + price fields only), Model 2 (derived metrics block), Model 3 (full container + model_3_reserved).

```rust
// src/contracts/bam_container.rs
#[repr(C, align(64))]
pub struct BAMContainer {
    // Block A: Header — 8 bytes
    pub version: u8,
    pub symbol_id: u16,
    pub flags: u8,                 // Bit 0=halt, 1=anomaly, 2=cross-market-corr
    pub timestamp_ms: u32,         // Unix ms (sufficient for >=1s bar)

    // Block B: Price — 32 bytes
    pub open: u64,                 // Fixed-point 10^8
    pub high: u64,
    pub low: u64,
    pub close: u64,

    // Block C: Volume — 16 bytes
    pub volume: u64,
    pub vwap: u64,                 // Fixed-point 10^8

    // Block D: Derived Metrics — 32 bytes
    pub rsi_14: u16,               // 0-10000 = 0.00-100.00
    pub macd_signal: i16,          // Fixed-point 10^4
    pub bollinger_position: i8,    // -4 to +4 (sigma units)
    pub hurst_exponent: u8,        // 0-200 = 0.00-2.00
    pub atr_14: u32,               // Fixed-point 10^4
    pub vpin_estimate: u16,        // 0-10000 = 0.00-1.00
    pub pattern_hash_16x16: u64,   // Hierarchical fingerprint tier-1
    pub pattern_hash_8x8: u32,     // Tier-2
    pub pattern_hash_4x4: u16,     // Tier-3
    pub pattern_hash_2x2: u8,      // Tier-4
    pub _pad_d: u8,

    // Block E: Cross-Market Correlation — 24 bytes
    pub correlation_leader: u16,   // Symbol ID of lead market
    pub lag_ms: i16,               // Detected lag in milliseconds
    pub correlation_strength: u8,  // 0-255 = 0.0-1.0
    pub ripple_signature: [u8; 8], // xxHash64 cross-market fingerprint
    pub _pad_e: [u8; 11],

    // Block F: Model-Specific Extensions — 144 bytes (48 per model)
    pub model_1_reserved: [u8; 48],
    pub model_2_reserved: [u8; 48],
    pub model_3_reserved: [u8; 48],
}
// CONSTRAINT: size_of::<BAMContainer>() == 256. Layout change invalidates NVMe storage.
pub const CONTAINER_SCHEMA_VERSION: u8 = 1;
```

---

## CONTRACT 3 — SHM Bridge Protocol (Rust → Python)

**Owner**: Backend Architect  
**Producer**: `src/infra/pytorch_bridge.rs`  
**Consumer**: `ml/bridge_reader.py`

```
Path:        /dev/shm/traderx_model2_bridge
Size:        256 MiB  (268_435_456 bytes)
Slots:       1_048_576  (256 MiB / 256 bytes)
Magic:       0xBAD_BAM2  (u32, identifies valid segment)
Header:      4096 bytes at offset 0 (page-aligned)
Ring start:  offset 4096

Header layout:
  [u32]  magic
  [u16]  version
  [u64]  producer_seq   (only Rust writes — atomic, relaxed store)
  [u64]  consumer_seq   (only Python writes — atomic, relaxed store)
  [u64]  heartbeat_ns   (producer updates every 100ms)
  [u32]  flags          (bit 0 = Python alive, bit 1 = shutdown requested)

Overflow rule: if (producer_seq - consumer_seq) >= SLOTS, producer stalls 10µs then retries.
Shutdown: Rust sets flag bit 1, waits for consumer_seq to catch up, then unmaps.
```

**Guardrail**: Python must check `magic == 0xBAD_BAM2` and `version == 1` on startup. Mismatch = hard abort with error log.

---

## CONTRACT 4 — SBE Wire Format (Market Data Ingestion)

**Owner**: Backend Architect  
**Producers**: Polygon adapter, Twelve Data adapter  
**Consumers**: All 3 models (via storage), NVMe hot-tier writer, QuestDB loader

```
Tick message layout (18 bytes, little-endian):
  [u16]  symbol_id
  [u64]  timestamp_ns   (DPDK hardware clock)
  [u64]  price          (fixed-point 10^8)
  [u32]  size           (share/contract count)
  [u8]   flags          (bit 0=trade, 1=bid, 2=ask, 3=halt)

Validation rules (enforced by SBE parser before storage):
  - symbol_id must exist in SymbolRegistry
  - timestamp_ns must be >= last seen timestamp for that symbol (monotonic)
  - price must be > 0
  - flags must not be 0x00 (at least one type bit set)
  Failure: emit ParseError metric, drop tick, continue (never crash ingestion)
```

---

## CONTRACT 5 — Storage Trait

**Owner**: Backend Architect  
**Implementations**: `infra/nvme_pool.rs` (Hot), QuestDB adapter (Warm), S3 Parquet loader (Cold)  
**Consumers**: All 3 models, backtest engine, training pipeline

```rust
// src/contracts/storage.rs
#[async_trait]
pub trait TickStorage: Send + Sync + 'static {
    async fn append(&self, tick: &Tick) -> Result<(), StorageError>;
    async fn read_range(&self, symbol_id: u16, start_ns: u64, end_ns: u64) -> Result<Vec<Tick>>;
    async fn latest(&self, symbol_id: u16) -> Result<Option<Tick>>;
}

#[async_trait]
pub trait ContainerStorage: Send + Sync + 'static {
    async fn write_batch(&self, containers: &[BAMContainer]) -> Result<(), StorageError>;
    async fn read_batch(&self, keys: &[ContainerKey]) -> Result<Vec<BAMContainer>>;
    async fn scan_by_pattern(&self, hash_64: u64) -> Result<Vec<ContainerKey>>;
}

// ContainerKey — 16-byte composite, sortable, O(1) hashable
// Layout: [symbol_id: 2B][timestamp_ms: 6B][layer: 1B][version: 1B][reserved: 6B]
#[repr(C)]
pub struct ContainerKey(pub u128);
```

**Rule**: Models call the trait. They never import concrete storage types directly.

---

## CONTRACT 6 — ModelMetricsReport (12-Dimension Scoring)

**Owner**: Analytics Reporter  
**Producers**: Each model's metrics adapter  
**Consumers**: Scoring engine (`tests/scoring_engine.rs`), Dashboard API, Executive report

```rust
// src/contracts/metrics.rs
pub struct ModelMetricsReport {
    pub model_id: u8,
    pub report_ns: u64,
    pub window_start_ns: u64,
    pub window_end_ns: u64,

    // D1: Latency (microseconds)
    pub lat_p50_us: u32,
    pub lat_p99_us: u32,
    pub lat_p999_us: u32,

    // D2: Throughput
    pub events_per_sec: u32,
    pub trades_per_sec: u32,

    // D3: Predictive Accuracy (basis points: 8000 = 80.00%)
    pub precision_bps: u16,
    pub recall_bps: u16,

    // D4: Robustness — Sharpe × 100 per regime
    pub sharpe_choppy: i16,
    pub sharpe_trending: i16,
    pub sharpe_volatile: i16,

    // D5: Cross-Market Alpha (bps of P&L from correlation signals)
    pub cross_market_pnl_bps: i16,

    // D6: Tail Risk
    pub max_drawdown_bps: u16,
    pub cvar_95_bps: u16,

    // D7: Resilience
    pub uptime_bps: u16,           // 9999 = 99.99%
    pub cb_triggers: u32,

    // D8: Risk-Adjusted Return
    pub sortino_x100: i16,
    pub calmar_x100: i16,

    // D9: Cognitive Load
    pub human_interventions: u16,

    // D10: Code Complexity
    pub avg_cyclomatic: u8,
    pub max_cyclomatic: u8,

    // D11: UX Impact (0-500 = 0.0-5.0 user trust score)
    pub user_trust_score: u16,
}
```

**Guardrail**: Scoring engine weights sum = 100. Any new dimension requires scoring weight redistribution and CCB approval.

---

## SIGN-OFF TRACKER

| Stream | Representative | Status |
|--------|-------------|--------|
| Infrastructure (Stream 2) | Backend Architect | ☐ Pending |
| Model 1 (Stream 3) | AI Engineer | ☐ Pending |
| Model 2 (Stream 4) | AI Engineer | ☐ Pending |
| Model 3 (Stream 5) | Rapid Prototyper | ☐ Pending |
| Testing (Stream 6) | Test Writer Fixer | ☐ Pending |
| Measurement (Stream 7) | Analytics Reporter | ☐ Pending |
| UX (Stream 8) | Frontend Developer | ☐ Pending |

**Rule**: No stream writes a single line of implementation code until this table is fully checked off.
