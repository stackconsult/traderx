# Developer Onboarding

**Version**: 1.0
**Last Updated**: 2026-05-01

---

## Development Environment Setup

### Prerequisites
- Python 3.11+
- Rust 1.70+ (for oms-engine)
- PostgreSQL 15
- Redis 7
- Docker Desktop
- Node.js 18+ (for UI development)
- GitHub CLI

### Repository Setup

#### Clone Repository
```bash
git clone https://github.com/your-org/traderx.git
cd traderx
```

#### Branch Strategy
- `main`: Production branch
- `feature/github-mcp-setup`: MCP integration branch
- `develop`: Development branch
- `feature/*`: Feature branches

#### Install Python Dependencies
```bash
cd app
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

#### Install Rust Dependencies
```bash
cd packages/oms-engine
cargo build
```

#### Setup Database
```bash
# Start PostgreSQL
brew services start postgresql@15

# Create database
createdb traderx_dev

# Run migrations
python scripts/migrate.py
```

#### Setup Redis
```bash
# Start Redis
brew services start redis
```

#### Environment Variables
```bash
# Copy environment template
cp .env.example .env

# Edit .env with your values
# DATABASE_URL, REDIS_URL, GITHUB_TOKEN, etc.
```

---

## Code Structure

### Project Layout
```
traderx/
├── app/                    # Python application
│   ├── agents/            # Agent implementations
│   ├── feeds/             # Data feeds and adapters
│   ├── messaging/         # Inter-agent messaging
│   ├── schemas/           # Pydantic schemas
│   ├── services/          # Business logic services
│   └── routers/           # FastAPI routers
├── packages/              # Rust packages
│   ├── oms-engine/        # Order Management System (Rust)
│   └── learnship/         # Learning system
├── traderx/               # BAM/Fabric system
│   ├── bam/               # BAM implementation
│   └── fabric/            # Fabric routing
├── docs/                  # Documentation
│   ├── architecture/      # Architecture docs
│   ├── operations/        # Operations docs
│   ├── onboarding/        # Onboarding docs
│   └── user/              # User docs
├── tests/                 # Test suite
│   ├── integration/       # Integration tests
│   ├── performance/       # Performance tests
│   └── unit/             # Unit tests
├── scripts/               # Utility scripts
├── .windsurf/             # Windsurf configuration
│   ├── skills/            # Agent skills
│   ├── workflows/         # Agent workflows
│   ├── rules/             # Agent rules
│   └── plans/             # Deployment plans
└── proofs/                # Proof artifacts
```

### Key Directories

#### `app/agents/`
Contains agent implementations:
- `trader_signal.py`: Signal generation agent
- `trader_portfolio.py`: Portfolio management agent
- `trader_risk.py`: Risk management agent
- `trader_correlation.py`: Correlation analysis agent
- `trader_anomaly.py`: Anomaly detection agent
- `trader_audit.py`: Audit logging agent
- `trader_router.py`: Message routing agent

#### `app/feeds/`
Contains data feed implementations:
- `adapters/`: Broker adapters (IBKR, Questrade, Polygon, etc.)
- `pipeline.py`: Ingestion pipeline
- `dead_letter.py`: Dead letter queue

#### `packages/oms-engine/`
Rust-based Order Management System:
- `src/lib.rs`: Main library
- `src/state_machine/`: Order state machine
- `src/oms/`: Order management logic
- `src/bin/main.rs`: Main executable

#### `traderx/`
BAM/Fabric system:
- `bam/`: BAM implementation
- `fabric/`: Fabric routing

---

## Testing Guidelines

### Test-Driven Development (TDD)
- Write tests before implementation
- Use RED→GREEN→REFACTOR cycle
- Test with REAL components (no mocks in integration tests)

### Test Types

#### Unit Tests
- Test individual functions and classes
- Fast execution (< 1s)
- Use mocks for external dependencies

#### Integration Tests
- Test component interactions
- Use REAL components (database, risk engine, etc.)
- Testcontainers for external services
- Slower execution (seconds to minutes)

#### Performance Tests
- Validate latency requirements
- Validate throughput requirements
- Use realistic data volumes

### Running Tests

#### Python Tests
```bash
# Run all tests
pytest

# Run specific test
pytest tests/test_signal_router.py

# Run with coverage
pytest --cov=app --cov-report=html

# Run integration tests
pytest tests/integration/
```

#### Rust Tests
```bash
# Run all tests
cargo test --package oms-engine

# Run specific test
cargo test --package oms-engine test_name

# Run with coverage
cargo tarpaulin --package oms-engine
```

### Test Requirements
- All tests must pass before commit
- Coverage must be > 80%
- Performance tests must meet SLOs
- Integration tests must use real components

---

## Contribution Process

### Workflow

#### 1. Create Feature Branch
```bash
git checkout -b feature/your-feature-name
```

#### 2. Make Changes
- Follow code style guidelines
- Write tests for new code
- Update documentation
- Run quality checks

#### 3. Quality Checks
```bash
# Run preflight checklist
/windsurf/workflows/preflight-checklist.md

# Run quality guardian
/windsurf/workflows/quality-guardian.md

# Run production guard
/windsurf/workflows/production-guard.md
```

#### 4. Commit Changes
```bash
git add .
git commit -m "feat(scope): description"
```

#### 5. Push and Create PR
```bash
git push origin feature/your-feature-name
gh pr create
```

#### 6. Code Review
- Address review feedback
- Ensure all checks pass
- Get approval from maintainers

#### 7. Merge
- Merge to develop or main
- Delete feature branch

### Commit Message Format
```
type(scope): description

# Types
feat: New feature
fix: Bug fix
docs: Documentation
style: Code style
refactor: Code refactoring
test: Test changes
chore: Maintenance

# Examples
feat(agents): Add correlation analysis agent
fix(oms): Correct order ID import
docs(architecture): Update architecture overview
```

### Code Style Guidelines

#### Python
- Follow PEP 8
- Use type hints
- Maximum line length: 100 characters
- Use docstrings for functions and classes
- No `// TODO` in committed code

#### Rust
- Follow Rust style guidelines
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- No `unwrap()` in production paths
- No `// TODO` in committed code

### Code Review Checklist
- [ ] Code follows style guidelines
- [ ] Tests added and passing
- [ ] Documentation updated
- [ ] No `// TODO` in code
- [ ] No `unwrap()` in production paths
- [ ] Error handling proper
- [ ] Performance acceptable
- [ ] Security reviewed

---

## Development Workflows

### Session Start (Mandatory)
Before any work session:
```bash
/windsurf/workflows/session-start.md
```

### Preflight Checklist (Mandatory)
After session start:
```bash
/windsurf/workflows/preflight-checklist.md
```

### Quality Guardian (Every Commit)
Before every commit:
```bash
/windsurf/workflows/quality-guardian.md
```

### Production Guard (Before Merge)
Before merge to main:
```bash
/windsurf/workflows/production-guard.md
```

---

## Agent Development

### Agent Structure
```python
from abc import ABC, abstractmethod
from app.schemas.envelope import AgentEnvelope

class BaseAgent(ABC):
    @abstractmethod
    async def handle_message(self, envelope: AgentEnvelope) -> AgentEnvelope:
        pass
    
    @abstractmethod
    async def initialize(self):
        pass
    
    @abstractmethod
    async def shutdown(self):
        pass
```

### Agent Roles
- **SignalMiner**: Signal generation
- **PortfolioArchitect**: Portfolio management
- **RiskGuardian**: Risk enforcement
- **CorrelationWeaver**: Correlation analysis
- **LiquidityScout**: Broker adapters
- **FabricRouter**: Message routing
- **AuditLedger**: Audit logging
- **JournalWriter**: Observability

### Agent Handoff Protocol
1. Create HandoffPackage
2. Update OWNERSHIP.md
3. Commit changes
4. Notify next agent
5. Accept handoff

---

## BAM/Fabric Development

### BAM Signal Format
- Format: `DDDD.SSSSSSSS.TTTT` (24-bit)
- Domain bits: 4 bits
- Sequence bits: 8 bits
- Type bits: 4 bits

### Dual Key Format
- Format: `signal_hex::bam_raw`

### Fabric Nodes
- Each node has BAM signal
- Each node has parent (except root)
- Each node has domain classification
- Each node has validation rules

### BAM Registry
```python
TRADERX_BAM = {
    "EQUITY": "00000",
    "FX": "00001",
    "CRYPTO": "00010",
    # ...
}

def encode_signal(domain, seq, type_):
    return f"{domain_bits}.{seq_bits}.{type_bits}"
```

---

## Risk Management

### Risk Checks
All trades undergo automated risk checks:
- Position limit validation
- Capital adequacy check
- Regulatory compliance check
- Concentration limit check

### Risk Limits
- Position limits per symbol
- Daily loss limits
- Maximum drawdown thresholds
- Concentration limits

### Risk Bus
```python
# Risk check function
def check_symbol(symbol, notional):
    if notional > position_limits[symbol]:
        return RiskResult.Reject("Position limit exceeded")
    if notional > daily_loss_limit:
        return RiskResult.Reject("Daily loss limit exceeded")
    return RiskResult.Pass()
```

---

## Debugging

### Debug Mode
```bash
# Enable debug logging
export LOG_LEVEL=debug

# Run with debug mode
python -m app.main --debug
```

### Common Debugging Tools
- Python debugger: `pdb`
- Rust debugger: `lldb`
- Logging: Structured JSON logging
- Metrics: Prometheus metrics

### Debugging Tips
- Check logs in `/var/log/traderx/`
- Use correlation IDs to trace requests
- Check error codes in troubleshooting guide
- Use performance profiling for slow code

---

## Performance Guidelines

### Latency Targets
- Risk check: <100ns
- Signal to order: <1ms
- Order submission: <10ms
- Audit logging: <5ms

### Throughput Targets
- Signal processing: 10k signals/sec
- Order submission: 1k orders/sec
- Audit logging: 5k events/sec

### Performance Testing
```bash
# Run performance tests
pytest tests/performance/

# Run Rust benchmarks
cargo bench --package oms-engine

# Profile application
python -m cProfile -o profile.stats app/main.py
```

---

## Security Guidelines

### Security Best Practices
- No hardcoded secrets
- Use environment variables for sensitive data
- Use RBAC for access control
- Enable audit logging
- Regular security scans

### Security Tools
- `cargo audit`: Rust security audit
- `trufflehog`: Secret scanning
- `snyk`: Dependency scanning
- `bandit`: Python security scanning

### Security Checklist
- [ ] No secrets in code
- [ ] Environment variables used
- [ ] RBAC configured
- [ ] Audit logging enabled
- [ ] Security scans passing

---

## Getting Help

### Documentation
- User guide: `docs/user/user_guide.md`
- Architecture: `docs/architecture/overview.md`
- Operations: `docs/operations/runbook.md`
- Troubleshooting: `docs/operations/troubleshooting.md`

### Internal Resources
- AGENTS.md: Project-specific guidelines
- AGENT_MASTER_SYSTEM.md: Full governance
- JOURNAL.md: Session history

### External Resources
- Python docs: https://docs.python.org/
- Rust docs: https://doc.rust-lang.org/
- FastAPI docs: https://fastapi.tiangolo.com/
- Pydantic docs: https://docs.pydantic.dev/

### Support Channels
- Slack: #traderx-dev
- Email: dev@traderx.example.com
- GitHub Issues: https://github.com/your-org/traderx/issues

---

## Common Tasks

### Add New Agent
1. Create agent file in `app/agents/`
2. Implement BaseAgent interface
3. Add agent to OWNERSHIP.md
4. Write tests
5. Update documentation
6. Run quality checks

### Add New Broker Adapter
1. Create adapter file in `app/feeds/adapters/`
2. Implement LiquidityPort interface
3. Add circuit breaker
4. Write integration tests
5. Update documentation
6. Run quality checks

### Add New BAM Signal
1. Update BAM registry in `traderx/bam/`
2. Add fabric node
3. Update documentation
4. Write tests
5. Run quality checks

### Database Migration
1. Create migration file
2. Test migration on staging
3. Run migration on production
4. Verify migration
5. Update documentation

---

## Next Steps

After completing this onboarding:
1. Set up development environment
2. Review code structure
3. Run existing tests
4. Pick up a small task
5. Get code review
6. Start contributing

Welcome to TraderX!
