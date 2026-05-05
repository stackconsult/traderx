# 05 — STREAM: Model 3 — Hyper-Static Binary Assembly
**Workstream Owner**: Rapid Prototyper  
**Support**: Backend Architect (NVMe/SBE), Performance Benchmarker (ASM/FPGA), AI Engineer (elastic detector), Test Writer Fixer  
**Depends On**: G0 (Stream 2 complete), Contracts 2, 4, 5  
**Sprint**: Weeks 3–5  
**Status**: PLANNING — awaiting G0 + G2 gates

---

## STREAM GOAL
Build the u8-level binary assembly model with WORM immutable storage, O(1) pattern dictionary lookup, elastic extreme detection, and FPGA acceleration stub. This model operates entirely at the binary level — no heap allocation during trading hours, no floating point in the hot path, no Python dependency.

**Key Invariant**: All operations during trading hours use pre-allocated, cache-aligned, stack-resident or mmap'd memory only. This is enforced at the type level.

---

## TASK LIST

### P3-1 — 256-Byte BAMContainer (Rust Implementation)
**File**: `src/fabric/bam_container.rs`  
**Specialist**: Rapid Prototyper  
**Time**: 6h  
**Depends On**: Contract 2 (exact layout spec)

**Spec**:
- Implement `BAMContainer` struct exactly as defined in Contract 2
- Derive: none (no `Clone`, no `Debug` in release — too slow)
- Implement manual `Debug` for development builds only (`#[cfg(debug_assertions)]`)
- `from_sbe_tick(tick: &Tick, registry: &SymbolRegistry) -> BAMContainer`: construct from raw tick
- `compute_derived_metrics(&mut self)`: fill Block D fields from Block B + C (no I/O, pure computation)
- `seal_worm(&mut self)`: set flags bit 1 (immutable), compute xxHash64 of Blocks A-E, store in `model_3_reserved[0..8]`
- `verify_seal(&self) -> bool`: recompute hash, compare — O(1) integrity check

**Acceptance Gate**:
- [ ] `size_of::<BAMContainer>() == 256` (static assert in test)
- [ ] `align_of::<BAMContainer>() == 64` (static assert in test)
- [ ] `seal_worm` then `verify_seal` round-trip passes on 10K random containers
- [ ] `compute_derived_metrics` completes in <1μs (criterion benchmark)
- [ ] File ≤200 lines

---

### P3-2 — NativeTernary Encoder
**File**: `src/fabric/native_ternary.rs`  
**Specialist**: Rapid Prototyper  
**Time**: 5h  
**Depends On**: P3-1

**Spec**:
- Encode price movement direction as ternary: -1 (Down), 0 (Flat), +1 (Up) per bar
- Ternary-to-binary packing: two ternary values per byte using balanced ternary (0=00, +1=01, -1=10)
- `encode_sequence(prices: &[u64]) -> [u8; 32]`: 64 bars → 32 bytes packed ternary sequence
- `decode_sequence(packed: &[u8; 32]) -> Vec<i8>`: reverse operation
- `compute_pattern_hash(packed: &[u8; 32]) -> u64`: xxHash64 of packed bytes → feeds into `BAMContainer.pattern_hash_16x16`

**Acceptance Gate**:
- [ ] `decode(encode(seq)) == seq` for 1000 random price sequences (proptest)
- [ ] `encode_sequence` in <500ns (criterion)
- [ ] Pattern hash is deterministic: same prices → same hash, always
- [ ] File ≤120 lines

---

### P3-3 — SBE Parser
**File**: `src/infra/sbe_parser.rs`  
**Specialist**: Backend Architect  
**Time**: 6h  
**Depends On**: Contract 4 (SBE wire format spec)

**Spec**:
- Zero-copy parse: operates on `&[u8]` input slice, no allocation
- Validates all fields per Contract 4 rules (symbol_id in registry, price > 0, monotonic timestamp, flags != 0)
- Returns `Result<Tick, SbeParseError>` — never panics
- Batch parse: `parse_batch(buf: &[u8]) -> impl Iterator<Item=Result<Tick, SbeParseError>>`
- SIMD-accelerated field extraction for price and timestamp (x86-64 SSE2 minimum)

**Acceptance Gate**:
- [ ] Parses 10K ticks/sec sustained from raw bytes (criterion, no I/O)
- [ ] Malformed input at each field position returns correct `SbeParseError` variant (parameterized test, 7 cases)
- [ ] Zero allocations in parse path (validated by custom allocator hook in test)
- [ ] File ≤150 lines

---

### P3-4 — Immutable Flash Container Store
**File**: `src/fabric/immutable_container.rs`  
**Specialist**: Backend Architect  
**Time**: 6h  
**Depends On**: P3-1 (sealed container), Contract 5 (ContainerStorage trait)

**Spec**:
- WORM semantics: once written, a slot is never overwritten (append-only)
- NVMe-backed `mmap` pool: pre-allocated ring of 4M container slots (1 GiB at 256 bytes/slot)
- Write: verify seal before accepting (`verify_seal` must return true)
- Read: O(1) slot-index lookup, zero-copy return of `&BAMContainer`
- Index: separate 64MB hash table mapping `ContainerKey → slot_index: u32`
- Compaction: when ring is 90% full, export oldest 10% to Cold tier, free slots

**Acceptance Gate**:
- [ ] Write throughput ≥500K containers/sec (criterion, pre-sealed containers)
- [ ] Read latency p99 <500ns (criterion, random access pattern)
- [ ] Rejected write on tampered container (flip one bit, verify_seal fails)
- [ ] Compaction does not drop any containers (verified by checksum of exported batch)
- [ ] File ≤200 lines

---

### P3-5 — NVMe Flash Pool
**File**: `src/infra/nvme_pool.rs`  
**Specialist**: Backend Architect  
**Time**: 8h  
**Depends On**: P3-4 (flash store uses the pool), Contract 5

**Spec**:
- Memory-mapped NVMe device file: `O_DIRECT | O_SYNC` for durability
- Pool header: magic, version, write_head (atomic u64), slot_count
- Pool is the physical backing for `immutable_container.rs`
- Implements `ContainerStorage` trait (Hot tier)
- 4K-aligned writes (NVMe sector requirement)
- Health check: `check_pool_integrity() -> PoolReport` — scan all written slots, verify seals

**Acceptance Gate**:
- [ ] Sequential write ≥5 GB/s (fio benchmark equivalent, measured in criterion)
- [ ] Random read p99 <5μs (random 1M reads across 1GiB pool)
- [ ] Pool integrity check on 100K containers completes in <30 sec
- [ ] File ≤200 lines

---

### P3-6 — Bidirectional Pattern Matcher
**File**: `src/fabric/bidir_matcher.rs`  
**Specialist**: Rapid Prototyper  
**Time**: 8h  
**Depends On**: P3-1 (container), P3-2 (ternary hash)

**Spec**:
- Match current pattern against dictionary in both forward and reverse direction
- Forward: current 64-bar sequence → lookup in `PatternDict` → get historical outcomes
- Reverse (time-reversal): flip sequence direction → lookup → get inverted outcomes
- Combine: if forward hit AND reverse hit agree on direction → high-confidence signal
- O(1) lookup using xxHash64 key into `PatternDict`
- No heap allocation: dictionary is pre-loaded into mmap'd memory at startup

**Bidirectional signal logic**:
```
forward_hit:  pattern_hash_16x16 found in dict → ForwardMatch { side, confidence }
reverse_hit:  reverse(pattern)_hash found in dict → ReverseMatch { side, confidence }
combined:     if forward.side == reverse.side → confidence *= 1.3 (capped at 255)
              if disagreement → confidence /= 2
              if neither found → NoMatch (skip to next model)
```

**Acceptance Gate**:
- [ ] Lookup in <200ns (criterion, dict with 1M entries)
- [ ] Bidirectional agreement correctly multiplies confidence (unit test with known patterns)
- [ ] File ≤180 lines

---

### P3-7 — Multi-Level Pattern Dictionary
**File**: `src/fabric/pattern_dict.rs`  
**Specialist**: Rapid Prototyper  
**Time**: 6h  
**Depends On**: P3-6 (matcher uses the dict)

**Spec**:
- Four-tier hash dictionary (16x16, 8x8, 4x4, 2x2) stored in a single mmap'd file
- Tier lookup: try 16x16 first (most specific), fall back to 8x8, 4x4, 2x2 if no match
- Each entry: `PatternEntry { outcome_side: u8, historical_confidence: u8, hit_count: u32, last_seen_ms: u32 }`
- Load from pre-built binary file at startup: `pattern_dict_v{version}.bin`
- Update path (offline only): `dict_builder.rs` (separate binary, not in hot path)

**Dictionary entry**: 10 bytes. At 1M entries: 10 MB per tier, 40 MB total — fits in L3 cache on modern server.

**Acceptance Gate**:
- [ ] Load 1M-entry dict in <500ms at startup
- [ ] Four-tier cascade lookup in <500ns total (criterion)
- [ ] Dict file format versioned: mismatch causes hard abort with clear error, not silent corruption
- [ ] File ≤150 lines

---

### P3-8 — Elastic Extreme Detector
**File**: `src/fabric/elastic_detector.rs`  
**Specialist**: AI Engineer  
**Time**: 8h  
**Depends On**: P3-7 (needs pattern context), Contract 2 (Hurst + ATR fields)

**Spec**:
- Detect elastic extremes (reversal zones) using three signals:
  1. **Hurst exponent** (from `BAMContainer.hurst_exponent`): value <0.50 → mean-reverting regime (elastic possible)
  2. **Z-score of price vs 20-bar mean**: |z| > 2.5 → extreme deviation
  3. **Time-reversal reconstruction**: compare forward vs reverse 20-bar trajectory similarity
- All three must agree → `ElasticSignal::Confirmed`; two of three → `ElasticSignal::Probable`; fewer → `ElasticSignal::None`
- Combine with bidirectional matcher: `ElasticSignal::Confirmed + BidirMatch::Agreement → MaxConfidence`
- No floating point in confirmed hot path: Hurst is u8 (scaled ×100), Z-score is i16 (scaled ×100)

**Acceptance Gate**:
- [ ] Detects synthetic elastic extreme with 100% recall on 50 injected test cases
- [ ] False positive rate <15% on trending regime ticks (property test)
- [ ] All three sub-signals independently testable (separate unit tests)
- [ ] File ≤180 lines

---

### P3-9 — x86-64 ASM Hot Path
**File**: `src/asm/elastic_detect.s`  
**Specialist**: Performance Benchmarker  
**Time**: 6h  
**Depends On**: P3-8 (replaces the Hurst+Zscore check with hand-optimized ASM)

**Spec**:
- Implement the `hurst_check + zscore_check` two-condition test in x86-64 SSE2 assembly
- Input: two u16 values (hurst_x100, zscore_abs_x100) in registers
- Output: 0 or 1 in al (no memory access, pure register operations)
- Called from Rust via `extern "C"` linkage
- Fallback: pure Rust implementation for non-x86 (feature-flagged)
- Target: <5ns per call (currently ~50ns in Rust — 10× improvement expected)

**Acceptance Gate**:
- [ ] ASM function output matches Rust fallback for 10K random inputs (correctness)
- [ ] <10ns per call (criterion, 1M calls)
- [ ] CPUID check at startup: if SSE2 not available, fallback to Rust (no crash)
- [ ] File ≤80 lines of assembly

---

### P3-10 — Multi-Timeframe Streaming Pipeline
**File**: `src/fabric/mtf_stream.rs`  
**Specialist**: Backend Architect  
**Time**: 8h  
**Depends On**: P3-5 (NVMe pool for bar aggregation), P3-1 (container per timeframe)

**Spec**:
- 10 simultaneous timeframes: 1s, 5s, 15s, 30s, 1m, 5m, 15m, 30m, 1h, 4h
- Each timeframe maintains its own rolling BAMContainer
- Hierarchical aggregation: 1s bars aggregate into 5s, which aggregate into 15s, etc.
- Zero-copy: each timeframe's container is a mutable slot in the NVMe pool
- Emit completed bar events to `bidir_matcher.rs` and `elastic_detector.rs`

**Acceptance Gate**:
- [ ] All 10 timeframes update correctly from a 4-hour tick replay (1M ticks)
- [ ] Timeframe rollover boundary tested: last tick of a bar correctly seals container
- [ ] File ≤200 lines

---

### P3-11 — Pattern State Machine (6 States)
**File**: `src/fabric/pattern_fsm.rs`  
**Specialist**: AI Engineer  
**Time**: 6h  
**Depends On**: P3-8, P3-10

**Spec**:
- States: `Idle → Scanning → CandidateFound → ConfirmationPending → SignalEmitted → Cooldown`
- Transitions driven by: tick arrival, bidir matcher result, elastic detector result
- `Cooldown`: 5 bar minimum after signal emission (prevents churning)
- Output: `ModelDecision` (Contract 1) emitted only on `SignalEmitted → Cooldown` transition
- State machine is a `match` expression, not a trait object (zero dynamic dispatch)

**Acceptance Gate**:
- [ ] All 6 states and all valid transitions covered by unit tests
- [ ] Invalid transition attempts return `Err(FsmError::InvalidTransition)` — no panics
- [ ] Cooldown period correctly suppresses signals (property test: no two signals within 5 bars)
- [ ] File ≤150 lines

---

### P3-12 — FPGA Pattern Match Stub
**File**: `src/fpga/pattern_match_stub.rs`, `fpga/pattern_match.vhd`  
**Specialist**: Rapid Prototyper  
**Time**: 6h  
**Depends On**: P3-7 (pattern dict), P3-8 (elastic detector interface)

**Spec**:
- **Rust stub** (`pattern_match_stub.rs`): implements same interface as FPGA PCIe driver would
  - Feature flag `fpga-hardware`: when off, stub delegates to software implementation
  - When on: opens `/dev/fpga0`, sends pattern hash over PCIe MMIO, reads result in <20ns
- **VHDL** (`pattern_match.vhd`): synthesizable design for Xilinx Ultrascale+
  - Input: 64-bit pattern hash
  - Output: 8-bit side + 8-bit confidence (16ns target at 200MHz clock)
  - Implements 16x16 tier dictionary lookup in block RAM
- Stub must be correct enough to substitute for hardware in all integration tests

**Acceptance Gate**:
- [ ] Stub passes same tests as software implementation (identical test suite)
- [ ] VHDL compiles without errors in Vivado (or open-source GHDL)
- [ ] Feature flag correctly switches between stub and hardware path
- [ ] Rust stub ≤100 lines; VHDL ≤150 lines

---

### P3-13 — Benchmark Adapter
**File**: `tests/model3_adapter.rs`  
**Specialist**: Test Writer Fixer  
**Time**: 4h  
**Depends On**: P3-11 (FSM complete), Contract 1

**Spec**:
- Implement `ModelRunner` trait for Model 3
- Feed ticks through: SBE parser → BAMContainer → MTF stream → bidir matcher → elastic detector → FSM → ModelDecision
- No Python, no SHM — entirely in-process Rust
- Measure per-component latency in adapter using latency probes (P1-2 pattern)

**Acceptance Gate**:
- [ ] End-to-end tick → ModelDecision in <350μs p99 (criterion)
- [ ] No heap allocations in hot path (custom allocator hook)
- [ ] File ≤150 lines

---

## STREAM DEPENDENCY CHAIN

```
Week 3:  P3-1 (Container) ──→ P3-2 (Ternary) ──→ P3-6 (Bidir)
         P3-1 ──→ P3-4 (Immutable) ──→ P3-5 (NVMe)
         P3-3 (SBE Parser) [parallel, no deps]

Week 4:  P3-6 ──→ P3-7 (Dict) ──→ P3-8 (Elastic) ──→ P3-9 (ASM)
         P3-5 ──→ P3-10 (MTF)
         P3-7 ──→ P3-12 (FPGA stub) [parallel with P3-8]

Week 5:  P3-8 + P3-10 ──→ P3-11 (FSM) ──→ P3-13 (Adapter)

Gate G3 (end of Week 5):
  ✅ P3-13 adapter: tick → decision <350μs p99
  ✅ WORM seal verified on 10K containers
  ✅ FPGA stub compiles and passes tests
  ✅ Zero heap allocations in hot path confirmed
  ✅ No new cargo check errors
  → Model 3 locked for comparative testing
```

---

## ROLLBACK PROCEDURES

| Failure | Rollback Action |
|---------|----------------|
| NVMe pool write <5 GB/s | Switch to `O_WRONLY` (no O_DIRECT), accept durability tradeoff, document |
| ASM <10ns target missed | Keep Rust implementation; mark FPGA ASM as "nice to have" |
| Pattern dict miss rate >80% | Extend dict from 1M to 10M entries; rebuild offline |
| FSM invalid transition in production | Transition to `Idle` state, emit alert, continue — no panic |
| FPGA synthesis fails | Disable `fpga-hardware` feature, ship software-only |

---

## FILE MANIFEST

| File | Owner | Max Lines | Depends On |
|------|-------|-----------|------------|
| `src/fabric/bam_container.rs` | Rapid Prototyper | 200 | Contract 2 |
| `src/fabric/native_ternary.rs` | Rapid Prototyper | 120 | P3-1 |
| `src/infra/sbe_parser.rs` | Backend Architect | 150 | Contract 4 |
| `src/fabric/immutable_container.rs` | Backend Architect | 200 | P3-1, Contract 5 |
| `src/infra/nvme_pool.rs` | Backend Architect | 200 | P3-4 |
| `src/fabric/bidir_matcher.rs` | Rapid Prototyper | 180 | P3-1, P3-2 |
| `src/fabric/pattern_dict.rs` | Rapid Prototyper | 150 | P3-6 |
| `src/fabric/elastic_detector.rs` | AI Engineer | 180 | P3-7 |
| `src/asm/elastic_detect.s` | Performance Benchmarker | 80 | P3-8 |
| `src/fabric/mtf_stream.rs` | Backend Architect | 200 | P3-5, P3-1 |
| `src/fabric/pattern_fsm.rs` | AI Engineer | 150 | P3-8, P3-10 |
| `src/fpga/pattern_match_stub.rs` | Rapid Prototyper | 100 | P3-7 |
| `fpga/pattern_match.vhd` | Rapid Prototyper | 150 | P3-7 |
| `tests/model3_adapter.rs` | Test Writer Fixer | 150 | P3-11, Contract 1 |

---

## GUARDED LINES (Coding Agents Must Not Cross)

1. **No heap allocation in trading hours** — `Box::new`, `Vec::new`, `HashMap::new` are FORBIDDEN in any hot-path function; use pre-allocated pools only
2. **No floating point in confirmed signal path** — Hurst and Z-score must remain u8/i16 scaled integers
3. **FPGA stub must pass identical tests to software** — never write tests that only work with one implementation
4. **WORM seal must be verified before every read** — never trust an unsealed container
5. **Pattern FSM must never panic** — all invalid transitions return `Err`, all states have an explicit fallback
6. **SBE parser must never allocate** — zero-copy constraint enforced by custom allocator in tests
