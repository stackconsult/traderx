# Machine Fabric Predictability System — Architecture Spec

## Vision
Read the entire market as a unified signal fabric. Distinguish noise (volatility spikes, illiquidity gaps) from deterministic, machine-tradeable patterns. Execute via ripple-synced synchronous/asynchronous combinations across all asset classes with guard-guided hardened architecture.

---

## Team Assignment (Agentic Coding Hoard)

### Team Alpha — Fabric Signal Intelligence
- **Role**: Unified market fabric reader and noise discriminator
- **Components**:
  - `MarketFabric` — Single-pane-of-glass view across all markets
  - `NoiseFilter` — Real-time volatility/illiquidity noise gate
  - `SignalQualityIndex` — Per-signal confidence scoring (0.0–1.0)
  - `FabricState` — Immutable snapshot of current fabric state

### Team Beta — Pattern Synthesis Engine
- **Role**: Detect and exploit predictable price patterns
- **Components**:
  - `PatternDetector` — Multi-scale pattern recognition (tick → macro)
  - `RippleSyncEngine` — Cross-market lead-lag ripple detection
  - `DeterministicModel` — Repeatable, backtested pattern → trade mapping
  - `PredictabilityScore` — Mathematical predictability metric per pattern

### Team Gamma — Execution Fabric Router
- **Role**: Route tradeable signals via time-bounded paths
- **Components**:
  - `FastPathRouter` — <100µs HFT path (sync)
  - `MediumPathRouter` — <1ms swing path (async)
  - `SlowPathRouter` — <100ms trend path (async)
  - `BackgroundPathRouter` — Rebalancing/portfolio (async)

### Team Delta — Guard & Hardening
- **Role**: Safety, audit, deterministic validation
- **Components**:
  - `FabricGuard` — Pre-trade blocking with human override
  - `DeterministicValidator` — Hash-chain validation of all decisions
  - `CircuitBreaker` — Auto-halt on anomaly detection
  - `AuditTrail` — Immutable cryptographically chained log

---

## Architecture: Market Fabric Layers

```
┌─────────────────────────────────────────────────────────────┐
│                    MARKET FABRIC (Layer 0)                   │
│  All asset classes unified into single signal topology      │
│  Equity │ FI │ FX │ Crypto │ Commodity │ Macro │ Sentiment  │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│              NOISE FILTER (Layer 1)                          │
│  • Volatility spike gate (ignore if σ > 3x)                │
│  • Illiquidity gate (ignore if spread > 2x avg)             │
│  • Flash crash detector (volume anomaly + price shock)      │
│  • Fat-finger filter (single-tick 5σ moves)                 │
└────────────────────┬────────────────────────────────────────┘
                     │ Clean Signals Only
┌────────────────────▼────────────────────────────────────────┐
│           PATTERN DETECTION (Layer 2)                        │
│  • Ripple Sync: Lead-lag cross-market correlation            │
│  • Harmonic Patterns: Repeating multi-timeframe structures   │
│  • Regime-Conditional: Pattern validity per market regime    │
│  • Statistical Arbitrage: Mean-reversion / momentum          │
└────────────────────┬────────────────────────────────────────┘
                     │ Predictable Patterns Only
┌────────────────────▼────────────────────────────────────────┐
│        DETERMINISTIC PROFIT ENGINE (Layer 3)                 │
│  • Deterministic hash of signal → decision → execution       │
│  • Bayesian weight fusion with confidence intervals            │
│  • Attribution tracking (which signal contributed how much)   │
│  • Predictability score > 0.7 threshold for execution        │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│         TIME-BOUNDED ROUTER (Layer 4)                        │
│  Fast (<100µs) ──→ HFT scalping, cross-exchange arb          │
│  Medium (<1ms) ──→ Swing, intraday momentum                  │
│  Slow (<100ms) ──→ Trend following, macro                    │
│  Background ────→ Portfolio rebalancing, risk adj            │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│              GUARD & AUDIT (Layer 5)                         │
│  • RiskBus check (never bypass)                              │
│  • Deterministic validator (hash chain)                      │
│  • Circuit breaker (auto-halt on anomaly)                  │
│  • Immutable audit trail (cryptographically chained)         │
└─────────────────────────────────────────────────────────────┘
```

---

## Core Data Structures

### FabricState (Immutable Snapshot)
```rust
pub struct FabricState {
    pub timestamp: DateTime<Utc>,
    pub asset_states: HashMap<String, AssetFabricState>,
    pub global_regime: MarketRegime,
    pub noise_level: f64,          // 0.0 = clean, 1.0 = maximum noise
    pub predictability_index: f64,  // 0.0 = random, 1.0 = perfectly predictable
    pub deterministic_hash: String,
}
```

### AssetFabricState
```rust
pub struct AssetFabricState {
    pub symbol: String,
    pub price: f64,
    pub volume_24h: f64,
    pub volatility_short: f64,     // 1-min realized vol
    pub volatility_long: f64,      // 1-day realized vol
    pub liquidity_score: f64,      // 0.0 = frozen, 1.0 = deep liquid
    pub spread_bps: f64,
    pub regime: MarketRegime,
    pub signal_quality: f64,       // Post-noise-filter quality score
    pub predictability_score: f64, // Pattern-based predictability
    pub bam_address: u16,          // 10-bit BAM address
}
```

### NoiseFilterResult
```rust
pub struct NoiseFilterResult {
    pub is_noise: bool,
    pub noise_type: NoiseType,     // VolatilitySpike, Illiquidity, FlashCrash, FatFinger
    pub confidence: f64,
    pub recommended_action: FilterAction, // Ignore, ReduceSize, Pause, Proceed
}
```

### RipplePattern
```rust
pub struct RipplePattern {
    pub source_market: String,     // Lead market
    pub target_market: String,     // Lag market
    pub correlation_strength: f64,
    pub lag_microseconds: u64,
    pub pattern_type: PatternType, // MomentumRipple, MeanReversionRipple, VolatilitySpillover
    pub predictability_score: f64,
    pub confidence_interval: (f64, f64),
}
```

### DeterministicTradeDecision
```rust
pub struct DeterministicTradeDecision {
    pub decision_hash: String,      // SHA-256 of inputs
    pub action: TradeAction,        // Buy, Sell, Hold, Close
    pub size: f64,
    confidence: f64,
    pub predictability_score: f64,
    pub attribution: HashMap<String, f64>,
    pub bam_route: Vec<BamSignal>,
    pub timestamp: DateTime<Utc>,
}
```

---

## Execution Flow (Ripple Sync)

```
1. FABRIC READ
   → Capture all market states simultaneously
   → Compute unified FabricState hash

2. NOISE FILTER (parallel per asset)
   → Volatility spike? → Mark as noise
   → Illiquidity? → Mark as noise
   → Flash crash? → Trigger circuit breaker
   → Pass clean signals to Layer 2

3. PATTERN DETECTION (ripple-synced)
   → Synchronous: Cross-market lead-lag within <1ms
   → Asynchronous: Multi-timeframe pattern stacking
   → Compute predictability score per pattern
   → Filter: predictability > 0.7

4. DETERMINISTIC PROFIT ENGINE
   → Bayesian fusion of all passing patterns
   → Compute deterministic hash of inputs
   → Generate confidence interval
   → If confidence > threshold → generate trade decision

5. TIME-BOUNDED ROUTER
   → Fast path: <100µs, HFT signals
   → Medium path: <1ms, swing signals
   → Slow path: <100ms, trend signals
   → Background: Rebalancing

6. GUARD & EXECUTE
   → RiskBus check (MUST pass)
   → Deterministic validator (hash match)
   → Circuit breaker (if anomaly detected)
   → Execute via selected router
   → Append to immutable audit trail
```

---

## Performance Targets

| Component | Latency Target | Throughput |
|-----------|---------------|------------|
| Fabric Read | <50µs | 10K assets/sec |
| Noise Filter | <10µs per asset | 100K assets/sec |
| Ripple Sync | <100µs cross-market | 1K pairs/sec |
| Pattern Detection | <500µs | 10K patterns/sec |
| Deterministic Engine | <100µs | 1K decisions/sec |
| Fast Router | <100µs end-to-end | 10K trades/sec |
| Guard Check | <50µs | 20K checks/sec |

---

## BAM Integration

Every component produces BAM signals for cross-market routing:

- **Layer 0 (Fabric)**: `BamDomain::CrossAsset` + `BamLayer::Prim`
- **Layer 1 (Noise Filter)**: `BamDomain::*` + `BamLayer::Sub` + noise_type_flag
- **Layer 2 (Pattern)**: `BamDomain::*` + `BamLayer::Type` + pattern_type_flag
- **Layer 3 (Profit Engine)**: `BamDomain::*` + `BamLayer::Axis` + decision_flag
- **Layer 4 (Router)**: `BamDomain::HftSignals` + path_latency_flag
- **Layer 5 (Guard)**: `BamDomain::Portfolio` + validation_flag

---

## Implementation Phases (Mini-Chunk Execution)

### Chunk 1: Market Fabric Reader + Noise Filter
- `market_fabric.rs` — Unified fabric state capture
- `noise_filter.rs` — Real-time noise discrimination
- Tests + QA + Security + Commit

### Chunk 2: Ripple Sync + Pattern Detection
- `ripple_sync.rs` — Cross-market lead-lag detection
- `pattern_detector.rs` — Multi-scale pattern recognition
- Tests + QA + Security + Commit

### Chunk 3: Deterministic Profit Engine
- `deterministic_engine.rs` — Hash-based deterministic trading decisions
- Integration with existing Bayesian fusion
- Tests + QA + Security + Commit

### Chunk 4: Time-Bounded Router + Guard
- `time_bounded_router.rs` — Fast/Medium/Slow/Background paths
- `fabric_guard.rs` — Circuit breaker + deterministic validator
- Tests + QA + Security + Commit

### Chunk 5: Integration + Handoff Controller
- `fabric_orchestrator.rs` — End-to-end workflow orchestration
- Immutable audit trail integration
- Full integration tests + Commit

---

## Success Criteria

- [ ] Market fabric reads all asset classes simultaneously
- [ ] Noise filter correctly identifies >95% of volatility spikes and illiquidity events
- [ ] Pattern detector finds predictable patterns with >0.7 predictability score
- [ ] Ripple sync detects cross-market lead-lag within <100µs
- [ ] Deterministic engine produces identical decisions for identical inputs (hash-verified)
- [ ] All trade decisions pass through RiskBus (no bypass)
- [ ] Circuit breaker halts within <1ms of anomaly detection
- [ ] Audit trail is immutable and cryptographically chained
- [ ] All components emit BAM signals for cross-market routing
- [ ] End-to-end latency <200µs for fast path
