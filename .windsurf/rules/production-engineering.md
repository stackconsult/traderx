---
trigger: always_on
description: "Production engineering workflow - functional completeness first"
---

# Production Engineering Workflow Rules

## 🚨 MANDATORY PRE-FLIGHT CHECKLIST

Before ANY infrastructure step (metrics, K8s, CI, etc.), you MUST verify:

### 1. Core Trading Functionality Gate
- [ ] Does `src/bin/main.rs` exist and run without errors?
- [ ] Can ONE signal flow through: Signal → Order → Fill → P&L?
- [ ] Are all components wired together (not just standalone)?
- [ ] Does the system actually trade when executed?

### 2. Integration Validation Gate
- [ ] Can I run `cargo run` and see the system process a trade?
- [ ] Do integration tests use REAL components, not mocks?
- [ ] Is there a working end-to-end trading flow?

### 3. Vertical Slice Completion Gate
- [ ] Have I implemented ONE complete trading scenario?
- [ ] Does the vertical slice pass through ALL layers?
- [ ] Can I demonstrate the system making money (or losing it)?

## 🔄 WORKFLOW SEQUENCE

### Phase 0: Vertical Slice (MUST BE FIRST)
1. Create `src/bin/main.rs` that wires all components
2. Implement ONE complete trading flow
3. Test with real market data
4. Verify P&L calculation accuracy

### Phase 1: Add Observability (AFTER vertical slice works)
- Add metrics to working trading flow
- Add health checks to running system
- Add logging to functional components

### Phase 2: Add Deployment (AFTER observability works)
- Containerize working system
- Create K8s manifests
- Add CI/CD around functional tests

### Phase 3: Add Scale (AFTER deployment works)
- Add HPA
- Add multiple instances
- Add chaos engineering

## 🛡️ SELF-CORRECTION MECHANISMS

### After Each Step, Ask:
1. "Did I just add FUNCTIONALITY or SCAFFOLDING?"
2. "Can the system TRADE better than before?"
3. "Did I validate with a REAL trading scenario?"

### If Answer is "Scaffolding":
- STOP immediately
- Return to Phase 0
- Add functional trading logic first

## 📋 VALIDATION CRITERIA

### Functional Completeness:
- Signal processing works
- Order lifecycle complete
- Fill handling accurate
- P&L calculation correct
- Risk enforcement active

### Integration Validation:
- All components communicate
- No stub implementations
- Real data flows through
- Error handling works

### Performance Validation:
- Meets latency SLOs
- Handles required throughput
- Resource usage acceptable

## 🚫 FORBIDDEN PATTERNS

- ❌ Building infrastructure before functionality
- ❌ Adding metrics to non-working system
- ❌ Creating K8s manifests for code that doesn't run
- ❌ Writing tests that only mock components
- ❌ Optimizing code that doesn't work

## ✅ REQUIRED PATTERNS

- ✅ Always have a runnable `main.rs`
- ✅ Test with real data, not mocks
- ✅ Implement complete flows, not pieces
- ✅ Validate end-to-end before optimizing
- ✅ Add infrastructure ONLY to working systems
