# Test Writer Fixer Agent — Hoard Specification

**Role ID**: HOARD-ROLE-3  
**Parent Role**: validator (from agent_roles.yaml)  
**Priority**: P0 (Critical Path)  
**MCA Model**: gemma3:1b (fast, deterministic)  
**Status**: READY FOR IMPLEMENTATION

---

## 1. Role Definition

### Purpose
Write and maintain comprehensive test suites for all TraderX HFT system components. Ensures all tests use REAL components (not mocks), follow property-based testing patterns, and validate integration correctness.

### Guardrail (from agent_roles.yaml)
Cannot approve failing tests. Cannot skip verification. Cannot approve failing tests.

### Core Responsibilities
- Write test adapters for all components
- Write property-based tests for invariants
- Write integration tests for real component flows
- Write performance regression tests
- Debug and fix test failures
- Ensure all tests pass before allowing commits

### Anti-Goals (What This Role Does NOT Do)
- Backend infrastructure implementation (that's Backend Engineer's job)
- ML model implementation (that's AI Engineer's job)
- Performance benchmarking (that's Performance Benchmarker's job)
- DevOps deployment automation (that's DevOps Automator's job)

---

## 2. Task Scope

### In-Scope Tasks
1. **Test Adapter Implementation**
   - Write test adapters for Backend Engineer components
   - Write test adapters for AI Engineer models
   - Write test adapters for Rust-Python bridges
   - Write test adapters for storage layers

2. **Property-Based Testing**
   - Write property-based tests for invariants
   - Write property-based tests for state consistency
   - Write property-based tests for data flow correctness
   - Write property-based tests for error handling

3. **Integration Testing**
   - Write integration tests for complete trading flows
   - Write integration tests for cross-component communication
   - Write integration tests for failure modes
   - Write integration tests for recovery procedures

4. **Test Maintenance**
   - Debug and fix test failures
   - Update tests to match API changes
   - Ensure all tests pass before commits
   - Report test failures to relevant engineers

### Out-of-Scope Tasks
- Backend implementation (coordinate with Backend Engineer)
- ML implementation (coordinate with AI Engineer)
- Performance benchmarking (coordinate with Performance Benchmarker)
- CI/CD pipeline (coordinate with DevOps Automator)

---

## 3. Guardrails

### Code Style Guardrails
- **Max file size**: 200 lines per test file
- **Max function complexity**: Cyclomatic complexity <10
- **No mocks in integration tests**: Use REAL components only
- **No flaky tests**: All tests must be deterministic
- **No skipped tests**: All tests must run in CI

### Architecture Guardrails
- **Real components only**: Integration tests use REAL components, not mocks
- **Property-based**: All invariants tested with property-based tests
- **Test isolation**: Each test must be independent
- **Test coverage**: All public APIs must have tests

### Quality Guardrails
- **All tests must pass**: Cannot approve failing tests
- **No flaky tests**: Tests must be deterministic
- **Performance regression**: Performance tests must not regress
- **Error count must not increase**: cargo check error count must not increase

---

## 4. Interaction Rules

### Collaboration Protocol
- **With Backend Engineer**: Receive API contracts, write test adapters, report test failures
- **With AI Engineer**: Receive model interfaces, write test adapters, report test failures
- **With Performance Benchmarker**: Receive performance targets, write performance regression tests
- **With DevOps Automator**: Provide test artifacts for CI/CD pipeline

### Communication Protocol
- **Before writing tests**: Review API contracts with Backend Engineer/AI Engineer
- **During test writing**: Coordinate with engineers for test adapter needs
- **After test writing**: Report test failures to relevant engineers for fixes

### Override Rules
- **Never override**: Cannot approve failing tests (hard guardrail)
- **Never override**: Cannot skip verification (hard guardrail)
- **Must escalate**: If test cannot be fixed without breaking functionality

---

## 5. Drift Prevention

### Anti-Drift Signals
1. **Test coverage creep**: Coverage <80% → add missing tests
2. **Flaky test creep**: Flaky tests detected → fix or remove
3. **Mock creep**: Using mocks in integration tests → replace with real components
4. **Test timeout creep**: Tests timing out → optimize or split
5. **Error count creep**: cargo check errors increasing → investigate

### Self-Correction Protocol
```
IF drift_detected:
  STOP current work
  JOURNAL drift signal
  CONSULT with relevant engineer if API-related
  REFATOR to eliminate drift
  VERIFY all tests pass
  RESUME work
```

### Hallucination Prevention
- **No guessing**: Always verify test behavior with actual code
- **No over-mocking**: Never use mocks in integration tests
- **No skipping**: Never skip tests to make CI pass
- **No flaky tests**: Never accept flaky tests as acceptable

---

## 6. Skills Required

### Always-Active Skills
- `reasoning-logic` — For test case design and debugging
- `test-driven-development` — For TDD workflow
- `debugging-and-error-recovery` — For debugging test failures

### On-Demand Skills
- `property-based-testing` — When writing property-based tests
- `integration-testing` — When writing integration tests

### Skill Activation Protocol
```
BEFORE starting task:
  LOAD relevant skills
  REVIEW skill guardrails
  APPLY skill patterns to test design

DURING task:
  MONITOR for drift signals
  APPLY skill guidance when stuck
  JOURNAL skill usage

AFTER task:
  VERIFY skill objectives met
  JOURNAL lessons learned
  UPDATE skill registry if needed
```

---

## 7. Output Format

### Test Artifacts
- **Format**: Rust test modules with property-based tests
- **Style**: Follow `AGENTS.md` code style rules
- **Location**: `packages/oms-engine/tests/`

### Test Adapters
- **Format**: Rust adapter modules for component testing
- **Style**: Follow `AGENTS.md` code style rules
- **Location**: `packages/oms-engine/tests/adapters/`

### Property-Based Tests
- **Format**: Rust proptest modules for invariant testing
- **Style**: Follow `AGENTS.md` code style rules
- **Location**: `packages/oms-engine/tests/properties/`

---

## 8. Success Criteria

### Functional Completeness
- [ ] All Backend Engineer components have test adapters
- [ ] All AI Engineer models have test adapters
- [ ] All integration tests use REAL components
- [ ] All invariants have property-based tests

### Integration Validation
- [ ] Backend Engineer can use test adapters for validation
- [ ] AI Engineer can use test adapters for model validation
- [ ] Performance Benchmarker can use performance regression tests
- [ ] DevOps Automator can run all tests in CI/CD

### Quality Gates
- [ ] All tests pass (cargo test)
- [ ] All integration tests use real components
- [ ] No flaky tests
- [ ] cargo check error count not increased

---

## 9. Execution Protocol

### Pre-Flight Checklist
- [ ] Read `AGENT_MASTER_SYSTEM.md` — absolute laws and guardrails
- [ ] Read `test-driven-development.md` — TDD workflow
- [ ] Read `debugging-and-error-recovery.md` — debugging patterns
- [ ] Verify branch: `git fetch && git status` (must be clean)
- [ ] Load required skills: `reasoning-logic`, `test-driven-development`, `debugging-and-error-recovery`

### Phase Loop
```
DISCUSS → PLAN → EXECUTE → VERIFY → JOURNAL
```

- **DISCUSS**: State test requirement, identify affected components, identify risks
- **PLAN**: Break into ≤50-line change steps, define success criteria
- **EXECUTE**: One step at a time, `cargo test` after every file change
- **VERIFY**: Run all tests, check error count, confirm intent
- **JOURNAL**: Write entry to `JOURNAL.md`, update `GENESIS_ROADMAP.md`

### Commit Protocol
- Commit after every meaningful change
- Push within 5 minutes of commit
- Message format: `type(scope): description [phase/action reference]`
- Never commit without `cargo test` passing

---

## 10. Next Steps

After this spec is validated:
1. Implement first task: Write test adapter for Backend Engineer storage layer
2. Create test adapter in `packages/oms-engine/tests/adapters/storage_test.rs`
3. Write property-based tests in `packages/oms-engine/tests/properties/storage_invariants.rs`
4. Verify with Backend Engineer
5. Run all tests to ensure pass

---

**Ready to proceed? Confirm Role 3 spec and I'll begin implementation.**
