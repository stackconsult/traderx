# Phase 8B: Multi-Market BAM Grid & Comparative Container Architecture Analysis

**Status**: ANALYSIS — Pre-Implementation  
**Derived From**: `JOURNAL_PATTERN_DISCOVERIES.md` (9-layer patterns), `MARKET_FABRIC_SPEC.md` (BAM integration), `PHASE_8_MARKET_FABRIC_CYPHER.md` (dot-cypher + grid)  
**Agent Council**: AI Engineer, Backend Architect, Performance Benchmarker, Tool Evaluator, Test Writer Fixer

---

## 1. Executive Summary: Multiple BAM Grids — YES, with Unified Cypher Overlay

| Market Class | Grid Dimensions | Cell Structure | Why Separate |
|--------------|-----------------|---------------|------------|
| **Equities** | 10×60 | 26-byte BamCell | Market hours, SEC rules, NBBO |
| **FX** | 10×60 | 26-byte + pip_precision u8 | 24h, no central exchange, pip-based |
| **Metals (XAU/USD)** | 10×60 | 26-byte + lot_size u16 | OTC, London fix, different tick size |
| **Commodities (CL, GC)** | 10×60 | 26-byte + contract_month u8 | Futures expiry, contango/backwardation |
| **Crypto (BTC, ETH)** | 10×60 | 26-byte + venue_bitmap u8 | Multi-venue, 24/7, DEX/CEX fragmentation |
| **Indices (SPX, NDX)** | 10×60 | 26-byte + constituent_count u16 | Derived price, arb vs constituents |

**Unification layer**: `CrossMarketCypher` queries across all grids via `symbol_id` namespace partitioning.

---

## 2. Comparative Container Architecture: Flash Funnel Assemblies

### 2.1 Concept: Mathematical Pattern Inference Containers

Instead of carrying raw BAM cells into comparables, we **pre-compute pattern fingerprints** and store them in flash-addressable containers:

```
┌─────────────────────────────────────────────────────────────────┐
│ FLASH FUNNEL CONTAINER (per market class, per 1s snapshot)     │
├─────────────────────────────────────────────────────────────────┤
│ Header (16 bytes)                                               │
│  ├── container_id: u64           — nanosecond timestamp         │
│  ├── market_class: u8            — 0=EQ, 1=FX, 2=MT, 3=CM, 4=CR│
│  ├── pattern_count: u16          — number of patterns in batch│
│  └── checksum: u32               — CRC32 for integrity         │
├─────────────────────────────────────────────────────────────────┤
│ Pattern Imprint 1 (32 bytes)                                    │
│  ├── pattern_type: u8            — 9-layer encoded              │
│  ├── symbol_id: u16              — asset identifier             │
│  ├── confidence: u8              — 0-255 scaled                 │
│  ├── predictability: u8          — 0-255 scaled               │
│  ├── direction: i8               — -127 short, +127 long        │
│  ├── magnitude: u16              — projected ticks/pips         │
│  ├── timeframe: u8               — encoded TF (1m, 5m, 1h...) │
│  ├── cross_asset_edges: [u16; 4] — correlated symbol_ids      │
│  └── edge_weights: [i8; 4]       — correlation strength        │
│ ... (repeating for each detected pattern)                      │
├─────────────────────────────────────────────────────────────────┤
│ Dot-Cypher Imprint (variable)                                   │
│  ├── graph_hash: [u8; 32]        — SHA-256 of fabric topology   │
│  ├── query_signature: [u8; 16]   — pre-computed cypher match   │
│  └── bloom_filter: [u8; 16]      — fast negative lookup        │
└─────────────────────────────────────────────────────────────────┘
```

**Container generation rate**: 1 per second per market class = ~6 containers/sec total.
**Container size**: ~2KB average (64 patterns × 32 bytes + overhead).
**Total throughput**: ~12KB/sec — negligible even on single core.

### 2.2 Comparative Matching Engine

When a new pattern emerges in Market A (e.g., FX), the system queries:

```cypher
// Find similar patterns across all market classes in last 60s
MATCH (p:PatternImprint)
WHERE p.pattern_type = $pattern_type
  AND p.predictability > 0.7
  AND p.timestamp > now() - 60s
  AND p.market_class != $current_market_class
WITH p, similarity(p.signature, $query_signature) AS sim
WHERE sim > 0.85
RETURN p.market_class, p.symbol_id, p.direction, p.magnitude, sim
ORDER BY sim DESC, p.predictability DESC
LIMIT 10
```

**This is the "comparables" concept** — historical patterns that rhyme across market classes, weighted by similarity + predictability.

---

## 3. Toolage for Efficient Scan/Filter — Technology Assessment

### 3.1 Options Evaluated (Tool Evaluator)

| Technology | Scan Speed | Memory | Complexity | Verdict |
|------------|-----------|--------|------------|---------|
| **Linear scan (Rust Vec)** | ~10M patterns/sec | Low | Minimal | ⚠️ Fine for <1M patterns |
| **Roaring Bitmap** | ~100M patterns/sec | Very Low | Low | ✅ **Best for category filtering** |
| **SIMD (AVX-512)** | ~500M patterns/sec | Low | Medium | ✅ **Best for signature comparison** |
| **Vector DB (Qdrant)** | ~50K patterns/sec | High | High | ❌ Too slow for HFT |
| **GPU (CUDA/Metal)** | ~10B patterns/sec | Very High | Very High | ❌ Overkill, transfer latency |
| **Custom FPGA** | ~100B patterns/sec | Fixed | Extreme | ❌ Not portable |
| **Bloom Filter + Linear** | ~50M patterns/sec | Very Low | Low | ✅ **Best for negative lookup** |

### 3.2 Recommended Stack: Hybrid

```rust
/// Flash Funnel Index — tiered lookup
pub struct FunnelIndex {
    /// Layer 1: Bloom filter — instant negative lookup
    /// "Is there ANY pattern like this in last 60s?"
    bloom: BloomFilter<PatternSignature>,

    /// Layer 2: Roaring bitmap — market_class × pattern_type × direction
    /// "Which containers have head-and-shoulders + bullish in FX?"
    category_index: HashMap<(u8, u8, i8), RoaringBitmap>,

    /// Layer 3: SIMD linear scan — exact signature similarity
    /// "Find patterns with >85% signature match"
    recent_containers: VecDeque<FlashContainer>, // 60-second ring buffer

    /// Layer 4: PyTorch tensor — cross-market correlation inference
    /// "Given FX pattern, what's the probability of similar move in crypto?"
    correlation_model: Arc<Mutex<Tensor>>, // Updated every 60s
}

impl FunnelIndex {
    /// Total lookup: <50µs for 1M patterns
    pub fn find_comparables(&self, query: &PatternImprint) -> Vec<ComparableMatch> {
        // Layer 1: Bloom filter (~100ns)
        if !self.bloom.may_contain(&query.signature) {
            return vec![]; // Instant negative
        }

        // Layer 2: Roaring bitmap (~1µs)
        let candidates = self.category_index
            .get(&(query.market_class, query.pattern_type, query.direction))
            .map(|bm| bm.to_vec())
            .unwrap_or_default();

        // Layer 3: SIMD scan (~10-50µs for 10K candidates)
        let matches = candidates.iter()
            .filter_map(|idx| {
                let container = &self.recent_containers[*idx as usize % self.recent_containers.len()];
                let sim = simd_signature_similarity(&query.signature, &container.signature);
                if sim > 0.85 {
                    Some(ComparableMatch { container_idx: *idx, similarity: sim })
                } else {
                    None
                }
            })
            .collect();

        matches
    }
}
```

---

## 4. Base Foundation (Simple) vs Advanced Model — Comparative Testing Framework

### 4.1 Two-Model Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    BASE MODEL (Deterministic)                   │
│  • Rule-based pattern matching (9-layer from existing code)      │
│  • Fixed weights (Top=0.85, Indicative=0.9, etc.)              │
│  • No ML inference — pure combinatorial logic                   │
│  • Latency: <200µs end-to-end                                   │
│  • Use: Live paper trading, guard rail validation               │
├─────────────────────────────────────────────────────────────────┤
│                    ADVANCED MODEL (ML-Physics)                  │
│  • PyTorch neural net: LSTM/Transformer on BAM grid sequences   │
│  • Physics-informed: Taken's embedding + Fisher info metric     │
│  • Online learning: Weight updates every 60s from backtest        │
│  • Latency: <2ms end-to-end (batch inference)                   │
│  • Use: Shadow trading, weight adaptation, regime detection     │
├─────────────────────────────────────────────────────────────────┤
│                    COMPARATIVE ARBITRATOR                       │
│  • Both models emit trade decisions independently                │
│  • If both agree → execute with 2x confidence                   │
│  • If disagree → advanced model wins IF predictability > 0.8    │
│  • If base says no → ALWAYS block (never override guard)        │
│  • Weekly: Retrain advanced model if base outperforms           │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 Divergence Detection & Auto-Correction

```rust
pub struct ModelDivergenceMonitor {
    pub base_decisions: VecDeque<TradeDecision>,
    pub advanced_decisions: VecDeque<TradeDecision>,
    pub divergence_threshold: f64, // 0.3 = 30% disagreement rate triggers review
}

impl ModelDivergenceMonitor {
    pub fn check_divergence(&self) -> DivergenceReport {
        let total = self.base_decisions.len();
        let disagreements = self.base_decisions.iter()
            .zip(self.advanced_decisions.iter())
            .filter(|(b, a)| b.action != a.action)
            .count();

        let rate = disagreements as f64 / total as f64;

        if rate > self.divergence_threshold {
            // Trigger: Retrain advanced model or alert human
            DivergenceReport::RetrainAdvanced { rate, samples: total }
        } else {
            DivergenceReport::Normal { rate }
        }
    }
}
```

### 4.3 Visualization: Side-by-Side Dashboard

```typescript
// React component: ComparativeModelView
interface ModelComparison {
  timestamp: string;
  symbol: string;
  baseSignal: {
    action: 'buy' | 'sell' | 'hold';
    confidence: number;
    patternLayers: string[]; // which 9 layers fired
    latency_us: number;
  };
  advancedSignal: {
    action: 'buy' | 'sell' | 'hold';
    confidence: number;
    modelPrediction: number;
    physicsMetrics: {
      takensDimension: number;
      fisherInfo: number;
      lyapunovExponent: number;
    };
    latency_us: number;
  };
  agreement: boolean;
  executed: 'base' | 'advanced' | 'none' | 'both';
  pnl: number; // after 1-hour hold
}
```

---

## 5. PyTorch + Physics Modeling Integration

### 5.1 Physics-Informed Market State Embedding

Based on existing `ml-physics-modeling.md` skills (Takens embedding, Fisher metrics, renormalization):

```python
# PyTorch model — runs in separate process, communicates via shared memory
import torch
import torch.nn as nn

class MarketPhysicsEncoder(nn.Module):
    """
    Encodes BAM grid into physics-informed latent space.
    Input: (batch, markets=6, levels=10, time=60, features=5)
    Output: (batch, latent_dim=256)
    """
    def __init__(self):
        super().__init__()
        # Temporal convolution across 60s window
        self.temporal_conv = nn.Conv1d(50, 64, kernel_size=5, stride=1)
        # Cross-market attention (which markets influence each other)
        self.cross_market_attn = nn.MultiheadAttention(64, num_heads=4)
        # Physics-informed regularization: Fisher information metric
        self.fisher_layer = FisherInformationLayer(dim=64)
        # Latent compression
        self.latent = nn.Linear(64 * 6, 256)

    def forward(self, bam_grids: torch.Tensor) -> torch.Tensor:
        # bam_grids: (B, 6, 10, 60, 5) — 6 markets, 10 levels, 60s, 5 features
        B = bam_grids.shape[0]

        # Flatten levels × features → 50 dims per (market, time)
        x = bam_grids.view(B, 6, 60, 50).permute(0, 1, 3, 2)  # (B, 6, 50, 60)
        x = x.reshape(B * 6, 50, 60)

        # Temporal convolution
        x = torch.relu(self.temporal_conv(x))  # (B*6, 64, 56)
        x = nn.AdaptiveAvgPool1d(1)(x).squeeze(-1)  # (B*6, 64)
        x = x.view(B, 6, 64).permute(1, 0, 2)  # (6, B, 64)

        # Cross-market attention: "If FX moves, what happens to crypto?"
        attn_out, _ = self.cross_market_attn(x, x, x)
        x = attn_out.permute(1, 0, 2).reshape(B, 6 * 64)

        # Physics regularization
        x = self.fisher_layer(x)

        return torch.tanh(self.latent(x))

class FisherInformationLayer(nn.Module):
    """
    Penalizes trajectories with low Fisher information
    (i.e., states where prediction is inherently uncertain).
    """
    def forward(self, x: torch.Tensor) -> torch.Tensor:
        # Approximate Fisher info as gradient magnitude squared
        # In practice: L2 norm with learned scaling
        return x / (1 + torch.norm(x, dim=-1, keepdim=True))
```

### 5.2 Pre-Loading Variance Prediction ("Predictable Pre-Loading")

```python
class VariancePredictor(nn.Module):
    """
    Predicts WHICH container will experience high-predictability events
    BEFORE they fully materialize. Uses "pre-loading" concept:
    market states that historically precede strong moves.
    """
    def __init__(self, latent_dim=256):
        super().__init__()
        self.lstm = nn.LSTM(latent_dim, 128, num_layers=2, batch_first=True)
        self.variance_head = nn.Linear(128, 6)  # variance per market class
        self.timing_head = nn.Linear(128, 6)    # expected timing (seconds)

    def forward(self, latent_sequence: torch.Tensor) -> dict:
        # latent_sequence: (B, T, 256) — T=60s of historical states
        lstm_out, _ = self.lstm(latent_sequence)
        last = lstm_out[:, -1, :]  # (B, 128)

        return {
            'variance_forecast': torch.sigmoid(self.variance_head(last)),
            'timing_forecast': torch.relu(self.timing_head(last)) * 60,
            'confidence': torch.sigmoid(torch.norm(last, dim=-1))
        }
```

### 5.3 Rust ↔ PyTorch Bridge (Zero-Copy)

```rust
/// Shared memory ring buffer for BAM grids → PyTorch
/// Avoids serialization overhead entirely
pub struct BamGridShm {
    pub buffer: SharedMemory,
    pub write_idx: AtomicU64,
    pub read_idx: AtomicU64,
}

impl BamGridShm {
    /// Write 6-market BAM grid to shared memory (~15KB)
    /// Time: ~1µs (memcpy into mmap)
    pub fn publish(&self, grid: &MultiMarketBamGrid) {
        let offset = self.write_idx.fetch_add(1, Ordering::SeqCst) % RING_SIZE;
        let ptr = unsafe { self.buffer.as_ptr().add(offset * GRID_SIZE) };
        unsafe {
            std::ptr::copy_nonoverlapping(
                grid as *const _ as *const u8,
                ptr,
                GRID_SIZE
            );
        }
    }
}

/// PyTorch side (Python) reads via numpy from shared memory
/// import numpy as np; grid = np.frombuffer(shm, dtype=np.float32).reshape(6,10,60,5)
```

---

## 6. Binary Lex Cypher Transfer — Pre-Labeled Pattern Matching

### 6.1 Concept

Instead of parsing text cypher queries at runtime, we **pre-compile** common pattern queries into binary signatures:

```rust
/// Pre-labeled binary cypher pattern
/// Loaded at startup from `patterns/compiled_cyphers.bin`
#[repr(C, packed)]
pub struct CompiledCypherPattern {
    /// Pattern ID — human-readable name hash
    pub pattern_id: [u8; 16], // e.g., SHA-256("head_and_shoulders_cross_market")

    /// Market classes this pattern applies to (bitmask)
    pub market_mask: u8, // bit 0 = EQ, 1 = FX, 2 = MT, 3 = CM, 4 = CR

    /// Required pattern types from 9-layer detection (bitmask)
    pub layer_mask: u16, // bits 0-8 = Top, Bottom, Cross, Vertical, Horizontal, Matching, Squeeze, Indicative

    /// Direction filter: -1 = short only, 0 = both, 1 = long only
    pub direction_filter: i8,

    /// Minimum predictability threshold (0-255 scaled)
    pub min_predictability: u8,

    /// Minimum confidence threshold (0-255 scaled)
    pub min_confidence: u8,

    /// Cross-market correlation requirement
    pub require_cross_correlation: bool,
    pub min_cross_correlation: u8, // 0-255 scaled

    /// Timeframe filter: encoded buckets (1m, 5m, 15m, 1h, 4h, D)
    pub timeframe_mask: u8,

    /// Pre-computed bloom filter for fast negative lookup
    pub signature_bloom: [u8; 16],
}
```

### 6.2 Pattern Matching at Wire Speed

```rust
/// Incoming BAM grid → match against ALL compiled patterns in <10µs
pub struct CypherPatternMatcher {
    /// ~100 pre-compiled patterns loaded at startup
    patterns: Vec<CompiledCypherPattern>,

    /// SIMD-accelerated pattern filter
    /// First pass: check market_mask + layer_mask + direction (vectorized)
    fast_filter: Vec<u8>, // packed bitmask array
}

impl CypherPatternMatcher {
    /// Match incoming grid against all patterns
    /// Returns: Vec<(pattern_id, match_strength)>
    pub fn match_grid(&self, grid: &MultiMarketBamGrid) -> Vec<PatternMatch> {
        // SIMD pass 1: Filter by market class + layers (AVX-256, ~100ns for 100 patterns)
        let candidates = self.simd_fast_filter(grid);

        // SIMD pass 2: Signature bloom filter match (~500ns)
        let bloom_pass = candidates.iter()
            .filter(|idx| self.bloom_match(grid.signature, idx))
            .collect();

        // Scalar pass 3: Full threshold comparison (~5µs for ~20 candidates)
        bloom_pass.iter()
            .filter_map(|idx| {
                let p = &self.patterns[*idx];
                let strength = self.compute_match_strength(grid, p);
                if strength > p.min_predictability as f64 / 255.0 {
                    Some(PatternMatch { pattern_id: p.pattern_id, strength })
                } else {
                    None
                }
            })
            .collect()
    }
}
```

---

## 7. BAM Schema Registry — Market-Specific Data Variance

### 7.1 Registry Structure

```rust
/// Schema registry — loaded once at startup, versioned
pub struct BamSchemaRegistry {
    /// Schema version (e.g., "2025.05.04-v1")
    pub version: String,

    /// Per-market-class schema definitions
    pub market_schemas: HashMap<MarketClass, MarketSchema>,

    /// Cross-market correlation schema (how grids relate)
    pub correlation_schema: CorrelationSchema,
}

pub struct MarketSchema {
    pub market_class: MarketClass,
    pub tick_size: f64,           // Minimum price increment
    pub lot_size: f64,            // Minimum quantity
    pub price_decimal_places: u8,
    pub qty_decimal_places: u8,
    pub trading_hours: TradingHours,
    pub venue_count: u8,
    pub special_flags: u16,       // bitflags for market-specific behavior
}

pub enum MarketClass {
    Equities = 0,
    FX = 1,
    Metals = 2,
    Commodities = 3,
    Crypto = 4,
    Indices = 5,
}

pub struct CorrelationSchema {
    /// Pre-computed lead-lag matrix (from historical analysis)
    /// Rows: source market class, Cols: target market class
    /// Values: average lag in microseconds
    pub lead_lag_matrix: [[i64; 6]; 6],

    /// Correlation strength matrix (0.0 - 1.0)
    pub correlation_matrix: [[f64; 6]; 6],

    /// Update frequency: how often to recompute (seconds)
    pub recompute_interval: u64,
}
```

### 7.2 Market-Specific Cell Variance

Same `BamCell` struct, different interpretation per schema:

| Market | `bid_price` Interpretation | `quantity` Unit | Special Flags |
|--------|---------------------------|-----------------|---------------|
| Equities | Cents (×10000) | Shares | AfterHours, Halted |
| FX | Pips (×10000) | Standard lots (100K) | Rollover, Fix |
| Metals | Cents per oz (×100) | Troy oz | LondonFix, Comex |
| Commodities | Cents per unit (×100) | Contracts | ExpiryWeek, Roll |
| Crypto | Satoshi/wei (×1e8) | Base units | DEX, CEX, StablePair |
| Indices | Index points (×100) | Notional | CashSettle, Future |

**Registry ensures**: All downstream code (pattern detection, cypher queries, dashboard) interprets cells correctly.

---

## 8. Ripple Pattern Compounding — Cross-Market Interference

### 8.1 Mathematical Model

When a pattern fires in Market A, the probability of a correlated pattern firing in Market B is NOT independent — it's a **compound ripple**:

```
P(pattern_B | pattern_A) = base_rate_B + ripple_coefficient(A→B) × predictability_A × time_decay(lag)

Where:
- base_rate_B = historical frequency of pattern_B (e.g., 0.12 for H&S in FX)
- ripple_coefficient(A→B) = from CorrelationSchema (e.g., 0.34 for EQ→FX)
- predictability_A = confidence of detected pattern in A (0.0-1.0)
- time_decay(lag) = exp(-lag / τ) where τ = typical propagation time (e.g., 500ms for EQ→FX)
```

### 8.2 Compound Ripple Detection

```rust
pub struct RippleCompounder {
    pub schema: Arc<BamSchemaRegistry>,
    pub recent_patterns: VecDeque<DetectedPattern>, // 60-second window
}

impl RippleCompounder {
    /// When pattern detected in Market A, compute compound probability for all markets
    pub fn compute_ripple_effects(&self, pattern: &DetectedPattern) -> Vec<RippleForecast> {
        let source_class = pattern.market_class as usize;
        let mut forecasts = vec![];

        for target_class in 0..6 {
            if target_class == source_class { continue; }

            let corr = self.schema.correlation_schema.correlation_matrix[source_class][target_class];
            let lag = self.schema.correlation_schema.lead_lag_matrix[source_class][target_class];

            // Compound probability
            let base_rate = self.historical_base_rate(target_class, pattern.pattern_type);
            let ripple = corr * pattern.predictability * (-(lag as f64) / 500.0).exp();
            let compound_prob = (base_rate + ripple).min(1.0);

            if compound_prob > 0.6 { // Threshold for alert
                forecasts.push(RippleForecast {
                    target_market: MarketClass::from(target_class),
                    compound_probability: compound_prob,
                    expected_lag_ms: lag.abs() as u64,
                    confidence: pattern.predictability,
                });
            }
        }

        forecasts.sort_by(|a, b| b.compound_probability.partial_cmp(&a.compound_probability).unwrap());
        forecasts
    }
}
```

---

## 9. Portfolio Fund Management — Cross-Market Benchmark Stewardship

### 9.1 Multi-Asset Allocation via BAM Grid State

```rust
pub struct PortfolioFabricAllocator {
    /// Current allocation across 6 market classes
    pub allocation: [f64; 6],

    /// Risk budget per class (from Kelly Criterion + correlation adjustment)
    pub risk_budget: [f64; 6],

    /// Benchmark: target Sharpe ratio
    pub target_sharpe: f64,

    /// Rebalancing threshold: drift > 5% triggers rebalance
    pub rebalance_threshold: f64,
}

impl PortfolioFabricAllocator {
    /// Recompute allocation based on current BAM grid predictability
    /// Run every 60 seconds
    pub fn rebalance(&mut self, grids: &MultiMarketBamGrids) -> Vec<RebalanceOrder> {
        // 1. Compute predictability per market class from BAM grids
        let predictability: [f64; 6] = grids.markets
            .iter()
            .map(|g| g.overall_predictability())
            .collect::<Vec<_>>()
            .try_into().unwrap();

        // 2. Kelly Criterion: f* = (p×b - q) / b, adjusted for correlation
        let kelly_fractions: [f64; 6] = predictability
            .iter()
            .map(|p| (p * 2.0 - (1.0 - p)) / 2.0) // simplified: b=2 (2:1 payoff)
            .collect::<Vec<_>>()
            .try_into().unwrap();

        // 3. Correlation adjustment: reduce allocation to correlated markets
        let corr_adjusted = self.apply_correlation_penalty(&kelly_fractions);

        // 4. Normalize to 100% allocation
        let sum: f64 = corr_adjusted.iter().sum();
        let target: Vec<f64> = corr_adjusted.iter().map(|x| x / sum).collect();

        // 5. Generate rebalance orders if drift > threshold
        target.iter()
            .zip(self.allocation.iter())
            .enumerate()
            .filter(|(_, (t, a))| (t - a).abs() > self.rebalance_threshold)
            .map(|(i, (t, a))| RebalanceOrder {
                market_class: MarketClass::from(i as u8),
                target_weight: *t,
                current_weight: *a,
                delta: t - a,
            })
            .collect()
    }
}
```

### 9.2 Benchmark Stewardship Metrics

```rust
pub struct BenchmarkStewardshipReport {
    /// Performance vs benchmark
    pub alpha: f64,                // excess return
    pub beta: f64,                 // market sensitivity
    pub information_ratio: f64,    // alpha / tracking error

    /// Risk-adjusted
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub max_drawdown: f64,
    pub calmar_ratio: f64,         // return / max_drawdown

    /// Predictability attribution
    pub pattern_attribution: HashMap<String, f64>, // which patterns contributed to alpha
    pub market_attribution: [f64; 6],              // which market classes contributed

    /// Cross-market efficiency
    pub correlation_exploited: f64,    // how much alpha came from cross-market ripple
    pub timing_accuracy: f64,          // % of ripple forecasts that were correct
}
```

---

## 10. Implementation Roadmap — Base vs Advanced

### Phase A: Base Foundation (24 hours)

| Task | Owner | File | Time |
|------|-------|------|------|
| Multi-market BAM grid allocator | Backend Architect | `mesh/multi_market_grid.rs` | 4h |
| BamSchemaRegistry loader | Backend Architect | `mesh/schema_registry.rs` | 3h |
| Flash Funnel container + index | Backend Architect | `fabric/funnel_index.rs` | 4h |
| Compiled cypher pattern matcher | Rapid Prototyper | `fabric/pattern_matcher.rs` | 3h |
| Ripple compounder | AI Engineer | `fabric/ripple_compounder.rs` | 3h |
| Base model integration (existing 9-layer) | AI Engineer | `fabric/base_model.rs` | 2h |
| Comparative dashboard component | Frontend Developer | `components/comparative-view.tsx` | 3h |
| Property-based tests | Test Writer Fixer | `tests/multi_market_invariants.rs` | 2h |

### Phase B: Advanced Model (24 hours)

| Task | Owner | File | Time |
|------|-------|------|------|
| PyTorch MarketPhysicsEncoder | AI Engineer | `ml/physics_encoder.py` | 4h |
| VariancePredictor LSTM | AI Engineer | `ml/variance_predictor.py` | 3h |
| Rust ↔ PyTorch SHM bridge | Backend Architect | `mesh/pytorch_bridge.rs` | 3h |
| Advanced model inference pipeline | AI Engineer | `fabric/advanced_model.rs` | 4h |
| Model divergence monitor | Test Writer Fixer | `fabric/divergence_monitor.rs` | 2h |
| Correlation schema updater (online) | AI Engineer | `fabric/correlation_updater.rs` | 3h |
| Portfolio fabric allocator | Backend Architect | `portfolio/fabric_allocator.rs` | 3h |
| Benchmark stewardship reporter | Frontend Developer | `components/stewardship-report.tsx` | 2h |

### Phase C: Comparative Testing (16 hours)

| Task | Owner | File | Time |
|------|-------|------|------|
| Side-by-side backtest harness | Test Writer Fixer | `tests/comparative_backtest.rs` | 4h |
| Statistical significance tests | Test Writer Fixer | `tests/significance_tests.rs` | 2h |
| Model A/B testing framework | Test Writer Fixer | `tests/ab_test_framework.rs` | 3h |
| Performance regression suite | Performance Benchmarker | `benches/multi_market_latency.rs` | 3h |
| Dashboard comparative visualization | Frontend Developer | `components/model-comparison.tsx` | 4h |

**Total: 64 hours (2 weeks @ 80% allocation)**

---

## 11. Key Technology Decisions Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Multiple grids? | **YES** — 6 market classes | Different microstructure, tick sizes, hours |
| Unification? | Cross-market cypher overlay | `symbol_id` namespace + correlation schema |
| Comparables storage? | Flash Funnel containers (2KB/s) | Pre-computed pattern fingerprints, fast scan |
| Scan toolage? | Roaring Bitmap + SIMD + Bloom | <50µs for 1M patterns, zero allocation |
| Base model? | Existing 9-layer deterministic | Proven, <200µs, rule-based guard rail |
| Advanced model? | PyTorch physics-informed NN | LSTM + Fisher info + cross-market attention |
| Comparative test? | Side-by-side shadow trading | Both emit decisions, arbitrator decides |
| Wire format? | FlatBuffers (same as before) | 10ns decode, proven in existing system |
| Rust↔PyTorch? | Shared memory ring buffer | Zero serialization, ~1µs handoff |
| Schema registry? | Compile-time loaded HashMap | Market-specific cell interpretation |
| Ripple compounding? | Lead-lag matrix + time decay | Mathematically grounded, updatable |
| Portfolio allocation? | Kelly + correlation penalty | Maximizes growth, limits drawdown |

---

## 12. Risk Assessment

| Risk | Mitigation |
|------|-----------|
| PyTorch inference >2ms latency | Batch inference every 100ms, async to trade path |
| Advanced model overfits | Weekly retrain, base model as guard, divergence monitor |
| Cross-market correlation breaks | Online updater, fallback to uncorrelated allocation |
| Schema registry out of sync | Versioned, checksum, panic on mismatch |
| Memory pressure (6 grids × 60s) | Each grid = 6 markets × 10 levels × 60s × 26B = ~94KB. Total: ~564KB. Trivial. |

---

**Ready for Phase A implementation? Confirm and I'll create `mesh/multi_market_grid.rs` + `fabric/funnel_index.rs` + schema registry as the foundation.**
