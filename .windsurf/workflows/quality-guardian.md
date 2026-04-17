---
description: Continuous quality enforcement - validates code quality, tests, security, performance, and documentation at every commit
---

# quality-guardian

**MANDATORY EXECUTION**: Before EVERY commit and EVERY PR.

**Purpose**: Ensure A-grade production code with comprehensive validation.

---

## Quality Gates

### Gate 1: Code Quality (A-Grade Standard)

**Zero Tolerance For**:
- [ ] TODO comments in production code
- [ ] Commented-out code
- [ ] Magic numbers (use named constants)
- [ ] Silent failures (all errors handled)
- [ ] Unhandled exceptions
- [ ] Blocking I/O in hot paths
- [ ] Hardcoded secrets
- [ ] Direct DB queries in business logic

**Validation Commands**:
```bash
# Python
ruff check src/ tests/
mypy src/ --strict
black src/ --check

# Rust
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check

# JavaScript/TypeScript
eslint . --ext .ts,.tsx
prettier --check .
```

**Proof Artifact**: `proofs/quality-[component]-[timestamp].json`

```json
{
  "timestamp": "2026-04-15T11:30:00Z",
  "component": "trading-engine",
  "grade": "A",
  "lint_errors": 0,
  "type_errors": 0,
  "format_issues": 0,
  "violations": [],
  "validator": "quality-guardian"
}
```

---

### Gate 2: Test Coverage (90% Unit, 80% Integration)

**Requirements**:
- [ ] Unit tests for every public function
- [ ] Integration tests for every module
- [ ] Property-based tests for complex logic
- [ ] Edge case coverage documented

**Validation Commands**:
```bash
# Python
pytest --cov=src --cov-report=xml --cov-report=html
coverage report --fail-under=90

# Rust
cargo test --all-features
cargo tarpaulin --out Xml --out Html

# JavaScript
jest --coverage --coverageThreshold='{"global":{"branches":90}}'
```

**Proof Artifact**: `proofs/coverage-[component]-[timestamp].json`

```json
{
  "timestamp": "2026-04-15T11:30:00Z",
  "component": "trading-engine",
  "unit_coverage": 94.2,
  "integration_coverage": 85.7,
  "overall_grade": "A",
  "uncovered_lines": [
    {"file": "src/core/engine.py", "lines": [142, 143, 144]}
  ],
  "validator": "quality-guardian"
}
```

---

### Gate 3: Security (Zero Critical/High)

**Requirements**:
- [ ] No secrets in code
- [ ] Input validation on all entry points
- [ ] SQL injection prevention (parameterized queries)
- [ ] XSS prevention in any UI code
- [ ] Dependency vulnerabilities scanned

**Validation Commands**:
```bash
# Python
bandit -r src/ -f json -o proofs/security-scan.json
pip-audit --format=json --output=proofs/dependency-audit.json

# Rust
cargo audit --json

# General
semgrep --config=auto --json --output=proofs/semgrep.json
```

**Proof Artifact**: `proofs/security-[component]-[timestamp].json`

```json
{
  "timestamp": "2026-04-15T11:30:00Z",
  "component": "trading-engine",
  "critical": 0,
  "high": 0,
  "medium": 2,
  "low": 5,
  "grade": "A",
  "scan_tools": ["bandit", "pip-audit", "semgrep"],
  "validator": "quality-guardian"
}
```

---

### Gate 4: Performance (Within 10% of Baseline)

**Requirements**:
- [ ] Benchmarks run for critical paths
- [ ] No regression > 10% from baseline
- [ ] Memory usage profiled
- [ ] Latency measured for trading operations

**Validation Commands**:
```bash
# Python
pytest tests/benchmarks/ --benchmark-only --benchmark-json=proofs/benchmark.json
python -m memory_profiler src/main.py

# Rust
cargo bench
```

**Proof Artifact**: `proofs/perf-[component]-[timestamp].json`

```json
{
  "timestamp": "2026-04-15T11:30:00Z",
  "component": "trading-engine",
  "latency_ms": {
    "order_submit": 12.4,
    "market_data_process": 2.1,
    "risk_check": 5.8
  },
  "memory_mb": 145.2,
  "baseline_comparison": "+3.2%",
  "grade": "A",
  "validator": "quality-guardian"
}
```

---

### Gate 5: Documentation (Complete & Current)

**Requirements**:
- [ ] Every public function documented
- [ ] README updated for any API changes
- [ ] Architecture decisions in DECISIONS.md
- [ ] CHANGELOG updated
- [ ] Usage examples provided

**Validation Commands**:
```bash
# Check documentation coverage
pdoc --http : src/  # Verify all public APIs documented

# Check for stale docs
python scripts/check_docs.py
```

**Proof Artifact**: `proofs/docs-[component]-[timestamp].json`

```json
{
  "timestamp": "2026-04-15T11:30:00Z",
  "component": "trading-engine",
  "public_functions": 42,
  "documented_functions": 42,
  "coverage": 100,
  "readme_updated": true,
  "changelog_updated": true,
  "grade": "A",
  "validator": "quality-guardian"
}
```

---

## Self-Learning & Self-Healing Integration

### Self-Learning
```python
class QualityLearner:
    """Captures quality patterns and learns from each validation."""
    
    def capture_pattern(self, validation_result):
        # Store successful patterns
        # Identify recurring issues
        # Build knowledge base
        pass
    
    def suggest_improvements(self, code_snippet):
        # Based on past patterns, suggest improvements
        # Cross-reference with similar past issues
        pass
```

### Self-Healing
```python
class QualityHealer:
    """Auto-fixes common quality issues."""
    
    def auto_fix(self, issue_type, file_path):
        if issue_type == "formatting":
            return self.run_formatter(file_path)
        elif issue_type == "imports":
            return self.sort_imports(file_path)
        # etc.
```

---

## Grading System

### A-Grade (Excellent)
- All gates pass
- Zero violations
- Coverage >= 90%
- No security issues
- Performance neutral or improved
- Documentation 100%

### B-Grade (Good)
- All gates pass
- Minor violations (< 5)
- Coverage >= 85%
- No critical/high security issues
- Performance < 5% regression
- Documentation >= 95%

### C-Grade (Acceptable)
- All gates pass
- Some violations (< 10)
- Coverage >= 80%
- Only low security issues
- Performance < 10% regression
- Documentation >= 90%

### D-Grade (Needs Work)
- Some gates fail
- Multiple violations
- Coverage < 80%
- Medium security issues
- Performance regression > 10%
- Documentation incomplete

### F-Grade (Failed)
- Multiple gates fail
- Many violations
- Coverage < 70%
- High/critical security issues
- Severe performance regression
- Documentation missing

---

## Execution Flow

```
┌─────────────────┐
│  Start Commit   │
└────────┬────────┘
         ▼
┌─────────────────┐
│ Quality Gate 1  │──❌──┐
│  Code Quality   │      │
└────────┬────────┘      │
         ✅               │
         ▼                │
┌─────────────────┐      │
│ Quality Gate 2  │──❌──┤
│  Test Coverage  │      │
└────────┬────────┘      │
         ✅               │
         ▼                │
┌─────────────────┐      │
│ Quality Gate 3  │──❌──┤
│    Security     │      │
└────────┬────────┘      │
         ✅               │
         ▼                │
┌─────────────────┐      │
│ Quality Gate 4  │──❌──┤
│   Performance   │      │
└────────┬────────┘      │
         ✅               │
         ▼                │
┌─────────────────┐      │
│ Quality Gate 5  │──❌──┤
│  Documentation  │      │
└────────┬────────┘      │
         ✅               │
         ▼                │
┌─────────────────┐      │
│ Generate Proofs  │      │
└────────┬────────┘      │
         ▼                │
┌─────────────────┐      │
│   Grade Work    │      │
└────────┬────────┘      │
         ✅               │
         ▼                ▼
┌─────────────────┐  ┌──────────┐
│  ALLOW COMMIT   │  │ FIX AND  │
│                 │  │ RE-RUN   │
└─────────────────┘  └──────────┘
```

---

## Integration Points

### Before Commit
```bash
# Run quality guardian
/workflow quality-guardian

# Only if grade >= B
/commit
```

### Before PR
```bash
# Full quality check
/workflow quality-guardian --full

# Multi-persona review
/review

# UAT validation
/verify-work
```

### Continuous Monitoring
```bash
# Add to CI pipeline
- name: Quality Guardian
  run: python scripts/quality_guardian.py
```

---

## Success Criteria

Quality Guardian ensures:
- ✅ All code meets A/B grade standard
- ✅ Comprehensive test coverage
- ✅ Zero critical security issues
- ✅ Performance maintained or improved
- ✅ Documentation always current
- ✅ Self-learning patterns captured
- ✅ Self-healing fixes applied
- ✅ Production-ready artifacts generated
