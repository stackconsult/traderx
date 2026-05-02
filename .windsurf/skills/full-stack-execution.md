# Skill: Full Stack Execution Architecture

## Role
End-to-end execution platform architect. Designs time-bounded routing, guard systems, and orchestration layers.

## Execution Paths
- **Fast Path**: <100µs, sync, HFT scalping, cross-exchange arb
- **Medium Path**: <1ms, async, swing, intraday momentum
- **Slow Path**: <100ms, async, trend following, macro
- **Background**: Portfolio rebalancing, risk adjustment

## Guard Systems
- `FabricGuard`: Pre-trade blocking with human override capability
- `DeterministicValidator`: Hash-chain validation of all decisions
- `CircuitBreaker`: Auto-halt on anomaly detection (>3σ deviation)
- `AuditTrail`: Immutable cryptographically chained log (append-only)

## Orchestration Requirements
- End-to-end workflow from signal ingestion to execution confirmation
- Every trade decision passes RiskBus (no bypass)
- All decisions hash-verifiable and reproducible
- Circuit breaker halts within <1ms of anomaly detection
- Audit trail captures full provenance chain

## References
- `packages/oms-engine/src/signal_router.rs`
- `packages/oms-engine/src/aeron_journal.rs`
- `packages/oms-engine/src/disruptor.rs`
- `MARKET_FABRIC_SPEC.md`
