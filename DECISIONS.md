# TraderX Architectural Decisions

## Decision Record Format
- **Status**: Proposed | Accepted | Deprecated | Superseded
- **Date**: YYYY-MM-DD
- **Decision**: Brief description
- **Rationale**: Why we made this decision
- **Consequences**: What this means for the system

---

## ADR-001: Event-Driven Async Architecture
**Status**: Accepted  
**Date**: 2026-04-09  
**Decision**: Use Python asyncio with event-driven architecture for the trading engine.  

**Rationale**:  
- Trading systems handle many concurrent operations (market data, orders, positions)
- Async/await provides non-blocking I/O for exchange APIs
- Event-driven pattern allows loose coupling between components  

**Consequences**:  
- All exchange operations must be async
- No blocking calls in the main trading loop
- Requires careful error handling to prevent event loop stalls  

---

## ADR-002: Separation of Concerns with Base Classes
**Status**: Accepted  
**Date**: 2026-04-09  
**Decision**: Create abstract base classes for exchanges, strategies, and data storage.  

**Rationale**:  
- Enables easy addition of new exchanges and strategies
- Ensures consistent interfaces across implementations
- Facilitates testing with mock implementations  

**Consequences**:  
- All exchanges must inherit from BaseExchange
- All strategies must inherit from BaseStrategy
- Interface changes require updating all implementations  

---

## ADR-003: Risk Management as Separate Layer
**Status**: Accepted  
**Date**: 2026-04-09  
**Decision**: Implement risk management as independent layer that validates all orders.  

**Rationale**:  
- Risk rules must be enforced consistently
- Centralized risk management prevents rule evasion
- Circuit breaker provides emergency stop mechanism  

**Consequences**:  
- No orders can bypass risk validation
- Risk manager must be highly performant (<100μs)
- All position changes must update risk metrics  

---

## ADR-004: PostgreSQL + Redis for Data Storage
**Status**: Accepted  
**Date**: 2026-04-09  
**Decision**: Use PostgreSQL for persistent storage and Redis for caching/pub-sub.  

**Rationale**:  
- PostgreSQL provides ACID compliance for financial data
- Redis offers sub-millisecond access for hot data
- Combination balances durability and performance  

**Consequences**:  
- System requires both PostgreSQL and Redis
- Must handle Redis persistence failures
- Database migrations required for schema changes  

---

## ADR-005: Paper Trading for Strategy Testing
**Status**: Accepted  
**Date**: 2026-04-09  
**Decision**: Implement PaperExchange for safe strategy testing without real money.  

**Rationale**:  
- Strategies must be validated before live trading
- Paper trading simulates real market conditions
- Allows infinite testing with no financial risk  

**Consequences**:  
- Paper exchange must simulate realistic behavior
- Slippage and latency must be configurable
- Paper results may differ from live performance  

---

## ADR-006: Directory-Scoped Governance Rules
**Status**: Accepted  
**Date**: 2026-04-09  
**Decision**: Place AGENTS.md files in subdirectories for context-aware development rules.  

**Rationale**:  
- Different modules have different requirements
- Context-aware assistance improves developer productivity
- Prevents information overload with irrelevant rules  

**Consequences**:  
- Must maintain AGENTS.md in each directory
- IDE must support directory-scoped rules
- Rule conflicts must be resolved systematically  

---

## ADR-007: Binary Milestone Tracking
**Status**: Accepted  
**Date**: 2026-04-09  
**Decision**: Track progress through binary milestones with proof artifacts.  

**Rationale**:  
- Clear success criteria prevent ambiguity
- Proof artifacts provide verifiable evidence
- Enables reliable progress tracking  

**Consequences**:  
- Every milestone requires testable outcome
- Must maintain proofs/ directory
- Cannot proceed without milestone verification  

---

## ADR-008: Configuration via Environment Variables
**Status**: Accepted  
**Date**: 2026-04-09  
**Decision**: Use environment variables and .env files for all configuration.  

**Rationale**:  
- Separates configuration from code
- Enables different configs for dev/staging/prod
- Prevents accidental commit of secrets  

**Consequences**:  
- All sensitive data must be in environment
- Must document all configuration options
- Default values must be sensible  

---

## Pending Decisions

### PD-001: WebSocket vs REST for Market Data
**Status**: Proposed  
**Date**: 2026-04-09  
**Decision**: Choose between WebSocket streaming or REST polling for market data.  

**Options**:  
1. WebSocket - Real-time updates, lower latency
2. REST - Simpler implementation, reliable  

**R TBD**: Need to evaluate exchange support and reliability requirements.

### PD-002: Message Queue for Internal Communication
**Status**: Proposed  
**Date**: 2026-04-09  
**Decision**: Select message queue for inter-component communication.  

**Options**:  
1. Redis Pub/Sub - Already using Redis
2. RabbitMQ - More features, additional dependency
3. In-memory queues - Fastest, no persistence  

**R TBD**: Need to evaluate persistence requirements.

---

## ADR-009: Guardrailed Manual Extension for Agent Workflows
**Status**: Accepted
**Date**: 2026-05-01
**Decision**: Create comprehensive Guardrailed Manual Extension with 7 parts documenting failure modes, validated patterns, agent role bindings, workflow guardrails, skill wiring, detection commands, and summary/index.

**Rationale**:
- Institutional memory of what works and what doesn't prevents recursive mistakes
- Zero-ambiguity agent role bindings ensure clear responsibility
- Detection commands enable automated validation of compliance
- Fine-grained guardrails systematically remove drift and error factors

**Consequences**:
- All agent workflows must follow Guardrailed Manual specifications
- Proof artifacts are single source of truth for quality claims
- Failure modes documented to prevent repetition
- Validated patterns documented for reuse
- Detection commands run periodically to ensure compliance

---

## Deprecated Decisions

None currently.
