# Skill: Security Audit Gate

## Role
Security hardening and compliance validation lead. Ensures zero unwrap, zero unsafe, zero TODO, and deterministic audit trails.

## Mandatory Gates
- No `unwrap()` in production code paths
- No `unsafe` blocks in any code
- No `TODO` or `FIXME` comments in committed code
- All public functions must have tests
- All error paths must be explicit (`Result` types)
- All decisions must have deterministic hash chains
- All audit logs must be cryptographically chained

## Validation Protocol
1. `cargo test --all-targets` must pass 100%
2. `cargo clippy -- -D warnings` must pass
3. `grep -r "unwrap()" src/` must return empty (test code exempt)
4. `grep -r "unsafe" src/` must return empty
5. `grep -ri "todo\|fixme" src/` must return empty
6. `cargo audit` must report zero vulnerabilities

## Compliance Requirements
- Deterministic hash chain for every trade decision
- Immutable append-only audit trail
- Circuit breaker with <1ms response time
- Human override capability for all guard gates
- Full provenance tracking from signal to execution

## References
- `packages/oms-engine/src/stability/reliability_assessment.rs`
- `packages/oms-engine/src/aeron_journal.rs`
