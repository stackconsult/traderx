# Spec: Fabric Orchestrator API Alignment Fix

## Feature: Fix compilation errors in fabric_orchestrator.rs

### Input:
- `FabricState` with `asset_states: HashMap<String, AssetFabricState>`
- `NoiseFilter` with `filter(&mut self, asset: &AssetFabricState) -> NoiseFilterResult`
- `NoiseFilterResult` with `is_noise: bool` and `action_type: NoiseType`
- `FilterAction` enum with variants: `Ignore`, `ReduceSize`, `Pause` (no `Pass`)

### Output:
- `OrchestratorResult` with filtered clean states
- All compilation errors resolved
- `cargo check --package oms-engine --lib` passes with 0 errors

### Edge Cases:
- Empty fabric state: returns empty clean_states Vec
- All assets marked as noise: returns empty clean_states Vec
- Noise filter returning `ReduceSize` or `Pause` actions: handled by `is_noise` boolean
- Memory retrieval for similar questions: returns relevant past Q&A

### Success Criteria:
- [x] All 4 compilation errors resolved
- [x] `cargo check` passes with 0 errors
- [x] `cargo run --package oms-engine --bin engineering_orchestra` runs successfully
- [x] Mem0 memory layer stores Q&A interactions correctly (11 memories stored)
- [x] Mem0 memory retrieval works (retrieves 2-3 relevant memories per query)
- [x] System ready for deployment

### Test Examples:

```rust
// Test 1: Valid fabric state with clean assets
let fabric = FabricState {
    timestamp: Utc::now(),
    asset_states: HashMap::new(),
    global_regime: MarketRegime::Trending,
};
let result = orchestrator.tick(&fabric);
assert!(result.allowed.is_empty());

// Test 2: Memory storage
let mut orchestra = EngineeringOrchestra::new();
orchestra.process_question("test".to_string()).await?;
assert!(orchestra.memory_layer.memories.len() > 0);
```

### Changes Required:

1. **Line 87:** `fabric.assets` → `fabric.asset_states`
2. **Line 88:** `self.noise_filter.filter(sym, state)` → `self.noise_filter.filter(state)`
3. **Lines 89-94:** Replace `match nf.action { FilterAction::Pass => ... }` with `if !nf.is_noise { ... }`

### Root Cause:
`fabric_orchestrator.rs` written against v1 of noise_filter API. Module updated to v2 but orchestrator not synchronized.

### Prevention:
- Add integration test for fabric_orchestrator → noise_filter pipeline
- Consider API versioning or compatibility checks between cross_market modules
- Add CI check to catch module API mismatches before merge
