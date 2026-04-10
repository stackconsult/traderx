# TraderX Global Governance Laws

## Meta-Coordinator (MCA) Directives

### Phase Loop Protocol
All development MUST follow: Discuss → Plan → Execute → Verify
- Every change requires JOURNAL.md entry
- All production code requires proof artifact in proofs/
- No "vibe-coding" - every line justified

### Memory & Context Management
- Maintain context window under 200k tokens
- Use JIT tool discovery - never load >15 tools at once
- Directory-scoped rules override global defaults

### Global Laws (Adapted for Python Trading System)

#### BANNED Patterns
- Direct database queries in hot paths (use repository pattern)
- Blocking I/O in trading engine (use async/await)
- Hardcoded API keys or secrets (use environment variables)
- Silent failures (all errors must be logged)
- Market data in strategy logic (use normalized data feeds)

#### REQUIRED Patterns
- All exchange interactions through base classes
- Risk validation before every order
- Comprehensive logging with structured format
- Binary milestone completion verification
- Health checks for all system components

### Cross-Platform Sync Tags
- Inbound: Wait for @step-triggered:ID before starting new phase
- Outbound: Emit @step-confirmed:ID to state-sync/confirmed.jsonl

### Proof Requirements
Every commit must include:
- proofs/test-[component].json (test results)
- proofs/perf-[component].json (performance metrics)
- JOURNAL.md entry explaining changes

### Risk Management Laws
- All orders pass through RiskManager
- Position limits enforced at engine level
- Circuit breaker stops all trading on breach
- Paper trading required before live deployment

### Data Integrity
- All market data timestamped and validated
- Order states persisted before execution
- Trade reconciliation on every restart
- Audit trail for all position changes
