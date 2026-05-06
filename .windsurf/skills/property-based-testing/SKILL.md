# Property-Based Testing Skill

---

name: property-based-testing
description: Writes property-based tests using proptest to verify invariants across all inputs. Use when testing components with complex input spaces or when you need to verify invariants hold for all possible inputs.

---

## When to Activate

Use when:
- Testing components with complex input spaces
- Verifying invariants that should hold for all inputs
- Testing edge cases that are hard to enumerate
- Writing tests for data structures or algorithms
- Testing state machine transitions

## Core Principles

### What is Property-Based Testing?
Instead of writing specific test cases (unit tests), you write properties (invariants) that should hold for ALL possible inputs. The testing framework generates thousands of random inputs to verify these properties.

### Common Properties
1. **Idempotency**: Applying the operation twice yields same result
   ```rust
   prop_assert_eq!(f(f(x)), f(x))
   ```

2. **Commutativity**: Order of operations doesn't matter
   ```rust
   prop_assert_eq!(f(x, y), f(y, x))
   ```

3. **Associativity**: Grouping doesn't matter
   ```rust
   prop_assert_eq!(f(f(x, y), z), f(x, f(y, z)))
   ```

4. **Identity**: There's an element that doesn't change others
   ```rust
   prop_assert_eq!(f(x, identity), x)
   ```

5. **Round-trip**: Encoding then decoding yields original
   ```rust
   prop_assert_eq!(decode(encode(x)), x)
   ```

### TraderX-Specific Properties

#### Risk Bus Properties
- **Monotonicity**: More notional = higher risk score
- **Boundedness**: Risk score always in [0, 1]
- **Additivity**: Risk of combined orders ≤ sum of individual risks

#### BAM Grid Properties
- **Sparsity**: BAM grid is mostly zeros (market is quiet)
- **Temporal Consistency**: Patterns don't change arbitrarily
- **Cross-Market Correlation**: BAM grids correlate across markets

#### Portfolio Properties
- **Conservation**: Total position = sum of individual positions
- **No-Negative**: Position can't be negative (short positions tracked separately)
- **P&L Linearity**: P&L scales linearly with position size

## Implementation Checklist

- [ ] Identify invariants for the component
- [ ] Write property-based tests using proptest
- [ ] Test with thousands of random inputs
- [ ] Add shrinking for failing cases
- [ ] Verify properties hold for edge cases
- [ ] Document properties in test comments

## Common Pitfalls

- ❌ Testing implementation details instead of invariants → test the WHAT, not the HOW
- ❌ Too many properties → focus on the most important invariants
- ❌ Not shrinking failing cases → always add shrinking to find minimal counterexample
- ❌ Properties that are too specific → keep properties general

## TraderX-Specific Adaptations

### Using proptest in Rust
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_risk_score_bounded(notional in 0.0..1_000_000.0) {
        let risk_score = calculate_risk_score(notional);
        prop_assert!(risk_score >= 0.0 && risk_score <= 1.0);
    }
}

proptest! {
    #[test]
    fn prop_bam_grid_sparsity(grid in prop::collection::vec(vec![0u8..=1u8], 600)) {
        let zeros = grid.iter().filter(|&&x| x == 0).count();
        let sparsity = zeros as f64 / grid.len() as f64;
        prop_assert!(sparsity > 0.9); // At least 90% zeros
    }
}
```

### Property-Based Testing for Invariants
```rust
// Round-trip property for serialization
proptest! {
    #[test]
    fn prop_order_serialization_roundtrip(order in any::<Order>()) {
        let serialized = bincode::serialize(&order).unwrap();
        let deserialized: Order = bincode::deserialize(&serialized).unwrap();
        prop_assert_eq!(order, deserialized);
    }
}

// Idempotency property for id generation
proptest! {
    #[test]
    fn prop_uuid_generation_is_unique() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        prop_assert_ne!(id1, id2);
    }
}
```

## Verification

After writing property-based tests:
- [ ] All properties pass with thousands of random inputs
- [ ] Shrinking finds minimal counterexamples for failures
- [ ] Properties are documented and justified
- [ ] Tests run in CI/CD pipeline
- [ ] Test coverage for invariants is complete
