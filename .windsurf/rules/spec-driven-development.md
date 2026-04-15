---
trigger: always_on
description: "Spec-driven development - define what 'done' looks like before coding"
---

# Spec-Driven Development

## 🎯 Core Principle
Before writing ANY code, you must have a clear specification of:
1. What the feature does
2. What inputs it accepts
3. What outputs it produces
4. What edge cases exist
5. How success/failure is measured

## 📋 Pre-Coding Checklist

### For Every New Feature:
- [ ] Can I describe the feature in ONE sentence?
- [ ] Do I know the EXACT input format?
- [ ] Do I know the EXACT output format?
- [ ] Have I identified ALL error cases?
- [ ] Do I have concrete test examples?

### For Integration Tasks:
- [ ] Which components communicate?
- [ ] What message format do they use?
- [ ] What are the failure modes?
- [ ] How do I test the integration?

## 🚫 FORBIDDEN PATTERNS

- ❌ Writing code without clear inputs/outputs
- ❌ "I'll figure out the format later"
- ❌ Building generic solutions for specific problems
- ❌ Starting implementation before edge cases are known

## ✅ REQUIRED PATTERNS

### 1. Write the Spec First
```markdown
## Feature: Signal → Order Conversion

### Input:
- AgentSignal with fields: {agent_id, symbol, direction, conviction, max_notional}
- Validation rules: conviction ∈ [0,1], max_notional > 0

### Output:
- Order with fields: {order_id, account_id, symbol, side, quantity, price}
- Risk check result: PASS/FAIL with reason

### Edge Cases:
- Invalid conviction: REJECT
- Insufficient capital: REJECT
- Symbol not supported: REJECT
```

### 2. Test Examples in Spec
```rust
// Test case 1: Valid signal
let signal = AgentSignal {
    conviction: 0.7,
    max_notional: 10000.0,
    // ...
};
assert!(router.route_signal(signal).status == RouteStatus::Submitted);

// Test case 2: Invalid conviction
let signal = AgentSignal { conviction: 1.5, ... };
assert!(router.route_signal(signal).status == RouteStatus::Error);
```

### 3. Success Criteria
- [ ] All test examples pass
- [ ] Performance meets SLO (<5μs per signal)
- [ ] No memory leaks in integration
- [ ] Error handling covers all cases

## 🔄 Workflow Integration

1. **Before Coding**: Write spec in comment or doc
2. **During Coding**: Reference spec constantly
3. **After Coding**: Verify all spec requirements met
4. **In Review**: Check spec completeness

## 📊 TraderX-Specific Rules

### Trading System Specs Must Include:
- [ ] Latency requirements (e.g., <100ns risk check)
- [ ] Throughput requirements (e.g., 10k signals/sec)
- [ ] Failure modes (e.g., market data disconnect)
- [ ] Recovery procedures (e.g., journal replay)

### Integration Specs Must Include:
- [ ] Channel capacities and backpressure
- [ ] Serialization format (JSON, Protobuf, etc.)
- [ ] Error propagation paths
- [ ] Monitoring/metrics points
