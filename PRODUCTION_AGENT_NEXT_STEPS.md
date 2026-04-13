# Production Agent Next Steps - Immediate Execution

## Current Status: 4/15 Build Steps Complete
✅ Security Fixes (Steps 1-4) - DONE
⏳ Critical Validation (Steps 5-7) - NEXT
⏳ High Priority Tests (Steps 8-9)
⏳ Integration & Deployment (Steps 10-15)

## Execution Plan - Next 24 Hours

### STEP 5: Risk Bus Atomicity Test (IMMEDIATE)
- File: `packages/oms-engine/tests/risk_bus_atomicity.rs`
- Action: Create test with 100 concurrent threads, 1000 ops each
- Validate: <100ns latency, no data corruption
- Dependencies: ✅ Security fixes complete

### STEP 6: Signal Router Load Test (AFTER STEP 5)
- File: `packages/oms-engine/tests/signal_router_load.rs`
- Action: Test 10k signals/sec with <5μs latency
- Validate: Throughput and latency targets
- Dependencies: ✅ Input validation complete

### STEP 7: OMS Journal Recovery Test (AFTER STEP 6)
- File: `packages/oms-engine/tests/journal_recovery.rs`
- Action: Test crash recovery with 1M orders
- Validate: Complete state recovery
- Dependencies: ✅ Risk Bus atomicity validated

## Execution Sequence:
1. Create Risk Bus test → Run → Validate
2. Create Signal Router test → Run → Validate  
3. Create OMS recovery test → Run → Validate
4. Commit all tests
5. Continue to Steps 8-9

Each step must pass before proceeding to next.
