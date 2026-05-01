# TraderX Architecture

**Version**: 1.0
**Last Updated**: 2026-05-01

---

## System Components

### Core Components

#### 1. Signal Generation Layer
- **SignalMiner Agent**: Generates trading signals from market data
- **DeltaLag Integration**: Processes signals with configurable lag parameters
- **Signal Router**: Routes signals to appropriate execution agents

#### 2. BAM/Fabric System
- **BAM (Binary Asset Mapping)**: Maps financial domains to binary codes
- **Fabric Router**: Routes messages based on BAM signals
- **TLTT (Three-Level Tiered Truth)**: Three-tier validation system
- **Registry**: Central registry for BAM codes and fabric nodes

#### 3. Agent Orchestration
- **Orchestrator**: Coordinates agent execution and handoffs
- **FabricRouter**: Routes messages between agents
- **Agent Handoff Protocol**: Formal protocol for agent transitions

#### 4. Risk Management
- **RiskBus**: Real-time risk checking engine
- **RiskGuardian**: Security and circuit breaker enforcement
- **PortfolioArchitect**: Portfolio optimization and risk management

#### 5. Execution Layer
- **LiquidityScout**: Manages broker adapter connections
- **Hexagonal Adapters**: Unified interface for multiple broker APIs
- **Order Management System (OMS)**: Order lifecycle management

#### 6. Compliance & Audit
- **RiskComplianceAgent**: Regulatory compliance enforcement
- **AuditLedger**: Immutable audit trail using ZK-Merkle chains
- **Compliance Engine**: UMIR, SEC, FINRA rule enforcement

#### 7. Observability
- **JournalWriter**: Structured logging and metrics collection
- **Prometheus Metrics**: Performance and operational metrics
- **Alerting System**: Real-time alerts on system events

---

## Data Flow

### Signal to Order Flow

```
Market Data → SignalMiner → DeltaLag → Signal Router → RiskBus → OMS → LiquidityScout → Broker
```

### Agent Communication Flow

```
Agent A → AgentEnvelope → FabricRouter → BAM Lookup → Agent B
```

### Audit Trail Flow

```
Trade Event → AuditLedger → ZK-Merkle Chain → Immutable Storage
```

---

## BAM/Fabric System

### BAM (Binary Asset Mapping)

The BAM system maps financial domains to binary codes for efficient routing and validation.

**Format**: `DDDD.SSSSSSSS.TTTT` (24-bit)
- `DDDD`: Domain bits (4 bits)
- `SSSSSSSS`: Sequence bits (8 bits)
- `TTTT`: Type bits (4 bits)

**Dual Key Format**: `signal_hex::bam_raw`

### Fabric Nodes

Fabric nodes are the routing points in the system. Each node:
- Has a BAM signal identifier
- Has a parent node (except root)
- Has a domain classification
- Has validation rules

### TLTT (Three-Level Tiered Truth)

Three-tier validation system:
1. **Vertical Gate**: Entry validation
2. **Diagonal Gate**: Cross-validation
3. **Horizontal Gate**: Exit validation

---

## Agent Orchestration

### Agent Roles

1. **Orchestrator**: Meta-coordinator, session management
2. **SchemaSynthesizer**: Database schema design
3. **BAMFabricEngineer**: BAM/Fabric integration
4. **APIContractAgent**: FastAPI layer design
5. **RiskComplianceAgent**: Regulatory compliance
6. **TestEvaluationAgent**: Test strategy and execution
7. **SignalMiner**: Signal research and generation
8. **PortfolioArchitect**: Portfolio optimization
9. **CorrelationWeaver**: Correlation analysis
10. **LiquidityScout**: Liquidity execution
11. **FabricRouter**: Agent routing and coordination
12. **AuditLedger**: Audit trail and compliance
13. **JournalWriter**: Observability and logging
14. **RiskGuardian**: Security and circuit breakers

### Agent Handoff Protocol

1. Agent A completes work
2. Agent A creates HandoffPackage
3. Agent A updates OWNERSHIP.md
4. Agent A commits changes
5. Orchestrator notifies Agent B
6. Agent B accepts handoff
7. Agent B begins work

---

## Technology Stack

### Backend
- **Rust**: High-performance trading engine (oms-engine)
- **Python 3.11+**: Strategy execution, agents, APIs
- **FastAPI**: REST API layer
- **PostgreSQL 15**: Primary database with RLS
- **Redis 7**: Caching and message queue
- **Qdrant**: Vector database for ML features

### Frontend
- **React 18**: User interface
- **TypeScript**: Type-safe frontend code
- **Lucide Icons**: Icon library
- **TailwindCSS**: Styling
- **shadcn/ui**: Component library

### Infrastructure
- **Kubernetes**: Container orchestration
- **Docker**: Containerization
- **GitHub Actions**: CI/CD
- **Prometheus**: Metrics collection
- **Grafana**: Metrics visualization

---

## Security Architecture

### Multi-Tenancy
- Row-Level Security (RLS) in PostgreSQL
- Tenant isolation at database level
- Tenant-specific BAM signal routing

### Audit Trail
- Immutable ZK-Merkle chains
- RFC 3161 timestamp anchoring
- Tamper-resistant audit logs

### Circuit Breakers
- Per-adapter circuit breakers
- Automatic failover
- Health monitoring

---

## Performance Characteristics

### Latency Targets
- Risk check: <100ns
- Signal to order: <1ms
- Order submission: <10ms
- Audit logging: <5ms

### Throughput Targets
- Signal processing: 10k signals/sec
- Order submission: 1k orders/sec
- Audit logging: 5k events/sec

---

## Deployment Architecture

### Three-Tier Architecture

1. **Retail Tier**: Basic features, single-tenant
2. **Fund Tier**: Advanced features, multi-tenant
3. **Institutional Tier**: Full features, dedicated infrastructure

### Deployment Model
- Horizontal scaling via Kubernetes
- Auto-scaling based on load
- Blue-green deployments
- Canary releases for critical updates

---

## Monitoring and Observability

### Metrics
- Business metrics (trades, P&L, positions)
- Technical metrics (latency, throughput, error rates)
- Security metrics (failed logins, risk breaches)

### Logging
- Structured JSON logging
- Correlation IDs for request tracing
- Agent-specific log streams

### Alerting
- Real-time alerts on critical events
- Alert routing based on severity
- Integration with incident management systems

---

## Compliance and Regulation

### Regulatory Framework
- **UMIR 10.11**: Canadian market regulations
- **SEC 15c3-5**: US pre-trade controls
- **FINRA Rule 3110**: Supervision requirements

### Compliance Features
- Pre-trade risk checks
- Order documentation
- Audit trail retention
- Regulatory reporting

---

## Development Workflow

### Guardrailed Development
- Session-start workflow (mandatory)
- Preflight checklist (mandatory)
- Quality guardian (every commit)
- Production guard (before merge)
- Spec-driven development
- Test-driven development

### Quality Gates
- Code quality validation
- Type checking
- Security scanning
- Performance testing
- Documentation completeness

---

## Future Enhancements

### Planned Features
- Machine learning-based signal generation
- Advanced portfolio optimization
- Cross-asset trading
- International market support
- Advanced analytics and reporting

### Scalability Improvements
- Distributed execution across multiple regions
- Edge computing for lower latency
- Advanced caching strategies
- Database sharding for scale
