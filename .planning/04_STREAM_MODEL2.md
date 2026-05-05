# 04 — STREAM: Model 2 — Advanced Physics-ML
**Workstream Owner**: AI Engineer  
**Support**: Backend Architect (SHM bridge), Test Writer Fixer (adapter + tests)  
**Depends On**: G0 (Stream 2 complete), Contract 3 (SHM Bridge), Contract 5 (StorageTrait)  
**Sprint**: Weeks 2–3  
**Status**: PLANNING — awaiting G0 gate

---

## STREAM GOAL
Build the physics-informed PyTorch model that runs in parallel with Model 1. It must:
- Accept BAMContainer inputs via zero-copy SHM bridge (Contract 3)
- Emit `ModelDecision` (Contract 1) within 2ms p99
- Run LSTM/Transformer for variance prediction with Takens embedding + Fisher metric
- Independently retrain weekly without disrupting live inference
- Fall back silently to Model 1 if inference exceeds 3ms

**Architecture Decision**: Python inference process runs in a separate OS process. Rust writes containers to SHM ring, Python reads and writes decisions back via a second SHM ring. No subprocess blocking, no serialization overhead.

---

## TASK LIST

### P2-1 — Physics Encoder
**File**: `ml/physics_encoder.py`  
**Specialist**: AI Engineer  
**Time**: 10h  
**Depends On**: G0 (need historical data from QuestDB Warm tier)

**Spec**:
- Input: `BAMContainer` read from SHM (as numpy structured array, zero-copy via `np.frombuffer`)
- Takens delay embedding: reconstruct attractor from close price time series, lag=3, dim=4
- Fisher Information Metric: compute geometric distance between consecutive attractors
- Hurst exponent: direct read from `BAMContainer.hurst_exponent` field (pre-computed)
- VPIN toxicity: direct read from `BAMContainer.vpin_estimate` field (pre-computed)
- Output: `PhysicsFeatures` dataclass with 12 float32 fields

```python
# ml/physics_encoder.py — max 200 lines
@dataclass
class PhysicsFeatures:
    takens_dim: float          # Attractor dimension
    fisher_distance: float     # Geometric change rate
    hurst: float               # 0.0-2.0 regime indicator
    vpin: float                # 0.0-1.0 toxicity
    rsi_norm: float            # RSI / 100
    macd_norm: float           # MACD signal normalized
    bollinger_z: float         # Current position in sigma units
    atr_norm: float            # ATR normalized by price
    volume_ratio: float        # Volume vs 20-bar moving average
    corr_strength: float       # Cross-market correlation 0.0-1.0
    corr_lag_norm: float       # Lag / 1000ms normalized
    ripple_score: float        # Derived from ripple_signature hash entropy
```

**Acceptance Gate**:
- [ ] `PhysicsFeatures` computed in <500μs for single container (Python profiler)
- [ ] Takens embedding matches reference implementation (unit test with known attractor)
- [ ] All 12 output fields are finite floats (no NaN, no Inf) — validated via property test
- [ ] File ≤200 lines

---

### P2-2 — LSTM Variance Predictor
**File**: `ml/variance_predictor.py`  
**Specialist**: AI Engineer  
**Time**: 8h  
**Depends On**: P2-1 (needs PhysicsFeatures as input)

**Spec**:
- Architecture: LSTM (hidden=128, layers=2) → Linear → Softmax over [Long, Short, NoOp, Close]
- Input sequence: 20 consecutive `PhysicsFeatures` vectors (sliding window)
- Output: `(side: int, confidence: float, regime: int)` — maps directly to ModelDecision fields
- Batch inference: collect 10 containers, infer together, return in order (amortize CUDA launch overhead)
- Fallback: if GPU unavailable, fall back to CPU (slower but functional)
- Model checkpoint: `ml/checkpoints/model2_v{version}.pt`, versioned with git SHA

```python
# ml/variance_predictor.py — max 200 lines
class VariancePredictor(nn.Module):
    def __init__(self):
        super().__init__()
        self.lstm = nn.LSTM(input_size=12, hidden_size=128, num_layers=2, batch_first=True)
        self.classifier = nn.Linear(128, 4)
```

**Acceptance Gate**:
- [ ] Batched inference (batch=10) in <1ms on GPU (criterion-equivalent: timeit 100 runs)
- [ ] Single inference <2ms on CPU (fallback path validated)
- [ ] Output `side` always in {0, 1, 2, 3}
- [ ] Checkpoint save/load round-trip produces identical outputs
- [ ] File ≤200 lines

---

### P2-3 — Rust ↔ PyTorch SHM Bridge (Rust Side)
**File**: `src/infra/pytorch_bridge.rs`  
**Specialist**: Backend Architect  
**Time**: 8h  
**Depends On**: Contract 3 (SHM protocol), G0 complete

**Spec**:
- On startup: `mmap` SHM segment at `SHM_BRIDGE_PATH`, verify magic `0xBAD_BAM2`, write heartbeat
- Producer loop: atomic write `BAMContainer` at `producer_seq % SLOTS`, then increment `producer_seq` (SeqCst store)
- Overflow handling: if `producer_seq - consumer_seq >= SHM_SLOTS`, busy-wait 10μs then retry (max 3 retries, then drop + log)
- Decision readback: second SHM segment `/dev/shm/traderx_model2_decisions` — Python writes `ModelDecision`, Rust reads
- Watchdog: if Python `heartbeat_ns` older than 5 seconds, set shutdown flag, trigger Model 1 fallback

```rust
// src/infra/pytorch_bridge.rs — max 200 lines
pub struct PytorchBridge {
    container_ring: *mut SHMBridgeHeader,
    decision_ring: *mut SHMBridgeHeader,
    fallback_model: Arc<dyn ModelRunner>,
}

impl PytorchBridge {
    pub fn push_container(&self, c: &BAMContainer) -> BridgeResult<()> { ... }
    pub fn poll_decision(&self) -> Option<ModelDecision> { ... }
    pub fn is_python_alive(&self) -> bool { ... }
}
```

**Acceptance Gate**:
- [ ] Container write-to-SHM overhead <1μs (measured with `std::time::Instant`)
- [ ] Python process disconnect detected within 5 seconds, fallback activates
- [ ] Zero data races: validated by running with ThreadSanitizer (`RUSTFLAGS=-Zsanitizer=thread cargo test`)
- [ ] File ≤200 lines

---

### P2-4 — SHM Bridge Reader (Python Side)
**File**: `ml/bridge_reader.py`  
**Specialist**: AI Engineer  
**Time**: 4h  
**Depends On**: P2-3 (needs Rust side to read against)

**Spec**:
- Validate magic `0xBAD_BAM2` on attach. Hard abort if mismatch (with error log).
- Read loop: `consumer_seq` advances only after successful `PhysicsFeatures` extraction
- Write decisions back to decision ring at `decision_seq % SLOTS`
- Update own heartbeat every 100ms
- Graceful shutdown: on `SIGTERM`, drain remaining containers, flush decisions, unmap

```python
# ml/bridge_reader.py — max 150 lines
class BridgeReader:
    def attach(self) -> None: ...
    def read_next(self) -> Optional[np.ndarray]: ...  # zero-copy numpy view
    def write_decision(self, decision: ModelDecision) -> None: ...
```

**Acceptance Gate**:
- [ ] Attaches and reads 10K containers in <5 seconds (throughput test)
- [ ] Heartbeat updated correctly (Rust watchdog test from P2-3 validates this)
- [ ] File ≤150 lines

---

### P2-5 — Online Correlation Updater
**File**: `src/fabric/correlation_updater.rs`  
**Specialist**: AI Engineer  
**Time**: 5h  
**Depends On**: P0-1 (live tick stream), Contract 2 (BAMContainer cross-market fields)

**Spec**:
- Exponential moving average (EMA) correlation: update every 1000 ticks per symbol pair
- Lead-lag detection: cross-correlation function over 20-bar sliding window, argmax = lag
- Write updated `correlation_leader`, `lag_ms`, `correlation_strength` back into live BAMContainer before writing to SHM
- Regime-aware: pause updates during `regime_tag == 3` (Halted)

**Acceptance Gate**:
- [ ] Correlation update for 10 symbol pairs in <100μs (per-call benchmark)
- [ ] Lead-lag detection unit tested with synthetic sinusoidal pairs at known lags
- [ ] File ≤150 lines

---

### P2-6 — Pre-Training Pipeline
**File**: `ml/training/train_model2.py`  
**Specialist**: AI Engineer  
**Time**: 12h  
**Depends On**: P0-2 (Warm tier QuestDB with ≥90 days of data)  
**Parallel Schedule**: Start Day 1 of Week 2 (runs overnight); does not block P2-1 through P2-5

**Spec**:
- Load 1 year of historical containers from QuestDB via JDBC
- Train-val-test split: 70/15/15 by time (no lookahead)
- Loss: cross-entropy on side prediction + MSE on confidence
- Early stopping: validation loss plateau for 5 epochs
- Save checkpoint to `ml/checkpoints/model2_v1.pt` with metadata JSON
- Weekly retraining: cron job, diff checkpoint from previous, log improvement

**Acceptance Gate**:
- [ ] Training completes in <8 hours (GPU required; document GPU spec)
- [ ] Validation accuracy >60% on side prediction (better than random 25%)
- [ ] Checkpoint metadata JSON contains: git_sha, training_date, val_accuracy, epoch_count
- [ ] File ≤200 lines (split train loop into `train_model2.py` + `dataset.py` if needed)

---

### P2-7 — Benchmark Adapter
**File**: `tests/model2_adapter.rs`  
**Specialist**: Test Writer Fixer  
**Time**: 4h  
**Depends On**: P2-3, P2-4 (bridge both sides working), Contract 1

**Spec**:
- Implement `ModelRunner` trait for Model 2
- Adapter starts Python subprocess (`ml/bridge_reader.py`) as a child process
- Feeds containers via SHM, reads decisions back with 5ms timeout
- If timeout: emit `ModelDecision` with `side=0` (NoOp) and `confidence=0`
- Shutdown: send SIGTERM to Python on `Drop`

**Acceptance Gate**:
- [ ] End-to-end Rust → SHM → Python → SHM → Rust round trip in <2ms p99
- [ ] Timeout correctly produces NoOp decision (not a panic)
- [ ] Python process cleaned up on `Drop` (no zombie processes)
- [ ] File ≤150 lines

---

## STREAM DEPENDENCY CHAIN

```
Week 2 Day 1:  P2-1 (Encoder) ──→ P2-2 (LSTM)          [sequential, Day 1-3]
               P2-6 (Training) ──────────────────────→   [parallel, runs overnight]

Week 2 Day 2:  P2-3 (Rust SHM) ──→ P2-4 (Python SHM)   [sequential, Day 2-3]
               P2-5 (Correlation) ──────────────────────→ [parallel with P2-2]

Week 3 Day 1:  P2-7 (Adapter) — needs P2-3 + P2-4 + P2-2 complete

Gate G2 (end of Week 3):
  ✅ P2-7 adapter end-to-end <2ms p99
  ✅ P2-6 training checkpoint saved with >60% val accuracy
  ✅ P2-3 watchdog tested (Python kill → Rust fallback)
  ✅ No new cargo check errors
  → Model 2 locked for comparative testing
```

---

## ROLLBACK PROCEDURES

| Failure | Rollback Action |
|---------|----------------|
| PyTorch inference >2ms p99 | Reduce batch to 5; reduce LSTM hidden to 64; re-benchmark |
| SHM bridge data race | Switch to mutex-guarded write (accept latency hit), file CCB ticket |
| Training accuracy <60% | Extend training data to 2 years; increase hidden units; do NOT increase sequence length >20 |
| Python process crash | Watchdog detects in ≤5s; Model 1 takes all decisions; alert fired |
| GPU unavailable in CI | CPU fallback path tested separately; GPU tests marked `#[ignore]` in CI, run manually |

---

## FILE MANIFEST

| File | Owner | Max Lines | Depends On |
|------|-------|-----------|------------|
| `ml/physics_encoder.py` | AI Engineer | 200 | Contract 2 |
| `ml/variance_predictor.py` | AI Engineer | 200 | P2-1 |
| `src/infra/pytorch_bridge.rs` | Backend Architect | 200 | Contract 3 |
| `ml/bridge_reader.py` | AI Engineer | 150 | P2-3 |
| `src/fabric/correlation_updater.rs` | AI Engineer | 150 | Contract 2 |
| `ml/training/train_model2.py` | AI Engineer | 200 | P0-2 |
| `ml/training/dataset.py` | AI Engineer | 150 | P0-2 |
| `tests/model2_adapter.rs` | Test Writer Fixer | 150 | Contracts 1, 3 |

---

## GUARDED LINES (Coding Agents Must Not Cross)

1. **No serialization** in the hot path — SHM is zero-copy; `pickle`, `json`, `msgpack` are FORBIDDEN between Rust and Python
2. **No `time.sleep()` in read loop** — use `os.sched_yield()` or spin with backoff
3. **No mutable global state** in Python inference — all state inside `BridgeReader` and `VariancePredictor` instances
4. **Weekly retraining must use a COPY of live data** — never retrain against the live SHM ring
5. **Model 1 fallback is MANDATORY** — the bridge must always have a fallback path; removing it requires CCB approval
