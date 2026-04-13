# Production Readiness Research Expedition
## Remaining Build Steps Analysis & Implementation Plan

### BUILD STEP 11: Prometheus Metrics for Risk Bus

#### Research Findings:
1. **Critical Requirements**:
   - Metrics must not block hot path (<1μs overhead)
   - Use lock-free atomic counters for high-frequency operations
   - Export standard Prometheus metrics format at `/metrics`
   - Include business metrics: orders/sec, risk checks/sec, halt events

2. **Best Practices**:
   - Use `prometheus::IntCounter` with `AtomicU64` backend
   - Histogram for latency distributions (risk check time)
   - Gauge for current state (NAV, drawdown, position limits)
   - Labels for dimensionality (symbol, strategy, risk_type)

3. **Guardrails**:
   - Metrics collection must be async from critical path
   - No heap allocations in hot path
   - Use pre-allocated metric instances
   - Rate limit metric updates to avoid flooding

#### Implementation Plan:
```rust
// Add to Cargo.toml
prometheus = "0.13"
lazy_static = "1.4"

// Metrics structure
pub struct RiskBusMetrics {
    orders_submitted: IntCounter,
    orders_rejected: IntCounter,
    risk_checks_duration: Histogram,
    current_nav: Gauge,
    current_drawdown: Gauge,
    is_halted: IntGauge,
}
```

---

### BUILD STEP 12: Health Check Endpoints

#### Research Findings:
1. **Critical Requirements**:
   - Unauthenticated but rate-limited endpoints
   - Liveness: Is the service running?
   - Readiness: Is the service accepting traffic?
   - Detailed health: Component status breakdown

2. **Best Practices**:
   - `/health/live` - Simple 200 OK if process alive
   - `/health/ready` - Checks dependencies (Redis, DB, etc.)
   - `/health/detailed` - Full component status with metrics
   - Response format: JSON with status, timestamp, components

3. **Guardrails**:
   - Rate limiting to prevent abuse
   - No sensitive data in responses
   - Fast response (<10ms)
   - Proper HTTP status codes (503 for unhealthy)

#### Implementation Plan:
```rust
// Health check structure
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    timestamp: DateTime<Utc>,
    checks: HashMap<String, ComponentHealth>,
}

// Rate limiting middleware
async fn rate_limit_health(req: Request, next: Next) -> Result<Response, StatusCode>
```

---

### BUILD STEP 13: K8s Deployment for OMS

#### Research Findings:
1. **Critical Requirements**:
   - Non-root securityContext
   - Resource limits/requests
   - Probes using health endpoints
   - Graceful shutdown handling
   - Persistent volumes for journals/WALs

2. **Best Practices**:
   - Use `securityContext.runAsNonRoot: true`
   - `runAsUser: 1000` (non-privileged)
   - `readOnlyRootFilesystem: true`
   - Volume mounts for journal storage
   - Proper terminationGracePeriodSeconds

3. **Guardrails**:
   - No privileged containers
   - Resource quotas enforced
   - NetworkPolicy isolation
   - Secrets management (no env vars for secrets)
   - PodDisruptionBudget for availability

#### Implementation Plan:
```yaml
# Deployment manifest
apiVersion: apps/v1
kind: Deployment
metadata:
  name: oms-engine
spec:
  replicas: 3
  securityContext:
    runAsNonRoot: true
    runAsUser: 1000
    fsGroup: 1000
  template:
    spec:
      containers:
      - name: oms-engine
        securityContext:
          readOnlyRootFilesystem: true
          allowPrivilegeEscalation: false
        resources:
          requests:
            cpu: 100m
            memory: 256Mi
          limits:
            cpu: 500m
            memory: 512Mi
        livenessProbe:
          httpGet:
            path: /health/live
            port: 8080
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 8080
```

---

### BUILD STEP 14: NetworkPolicy for Security

#### Research Findings:
1. **Critical Requirements**:
   - Default-deny policy
   - Explicit allowlists for required traffic
   - Separate policies for each component
   - Namespace isolation

2. **Best Practices**:
   - Start with `default-deny` ingress/egress
   - Allow DNS (port 53 UDP/TCP)
   - Allow intra-namespace traffic for services
   - Allow external dependencies explicitly

3. **Guardrails**:
   - No wildcards in policies (`[]` instead of `[*]`)
   - Use namespace labels for isolation
   - Regular policy audits
   - Monitor denied traffic logs

#### Implementation Plan:
```yaml
# Default deny policies
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-all
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress

# Allow intra-namespace
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-same-namespace
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: traderx
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          name: traderx
```

---

### BUILD STEP 15: CI Validation Pipeline

#### Research Findings:
1. **Critical Requirements**:
   - Run all existing tests as gates
   - Security scanning (SAST/DAST)
   - Dependency vulnerability scanning
   - Container image scanning
   - Performance regression tests

2. **Best Practices**:
   - GitHub Actions workflow
   - Matrix builds for different components
   - Caching for dependencies
   - Parallel test execution
   - Fail fast on critical failures

3. **Guardrails**:
   - No secrets in logs
   - All tests must pass before merge
   - Security scan must pass
   - Performance benchmarks validated
   - Artifact signing for releases

#### Implementation Plan:
```yaml
# .github/workflows/validate.yml
name: Production Validation
on:
  pull_request:
    branches: [main]
  push:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        component: [oms-engine, portfolio-aggregation, model-serving]
    steps:
    - uses: actions/checkout@v4
    - uses: actions/cache@v3
      with:
        path: ~/.cargo/registry
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    - name: Run tests
      run: cargo test -p ${{ matrix.component }} --release
    - name: Run performance tests
      run: cargo test -p ${{ matrix.component }} --release performance

  security:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Run security audit
      run: cargo audit
    - name: Run SAST scan
      uses: github/super-linter@v4
```

---

## Execution Order & Dependencies

1. **BUILD STEP 11** (Prometheus Metrics) - No dependencies
2. **BUILD STEP 12** (Health Endpoints) - No dependencies  
3. **BUILD STEP 13** (K8s Deployment) - Needs health endpoints for probes
4. **BUILD STEP 14** (NetworkPolicy) - Needs K8s deployment manifest
5. **BUILD STEP 15** (CI Pipeline) - Final validation

## Critical Success Factors

1. **Performance**: Metrics must not impact trading latency
2. **Security**: Default-deny posture, no privileged access
3. **Observability**: Full visibility into system health
4. **Automation**: All validation must be automated
5. **Compliance**: Follow financial industry best practices

## Risk Mitigation

1. **Metrics Overhead**: Use lock-free atomics, async collection
2. **Health Check Abuse**: Rate limiting, no sensitive data
3. **K8s Misconfig**: Use PSP/OPA/Gatekeeper policies
4. **Network Gaps**: Regular policy audits, monitoring
5. **CI Bypass**: Require PR reviews, protected branches
