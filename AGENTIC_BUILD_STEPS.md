# TraderX Agentic Build Steps

> **Core Rule**: No timelines, no pseudo-architecture
> 
> **Only**: Concrete code changes and validation steps
> 
> **Execute**: Steps in dependency order

## SECURITY FIXES (BLOCKING)

### STEP 1: Fix Unix Socket Permissions
**File**: `packages/oms-engine/src/signal_router.rs`
**Action**: Add socket permission setting after bind
```rust
let socket = UnixListener::bind(socket_path)?;
// Set secure permissions (owner read/write only)
std::fs::set_permissions(socket_path, std::fs::Permissions::from_mode(0o600))?;
```
**Validates**: Socket file has 600 permissions (`ls -la /tmp/traderx_signals.sock`)

### STEP 2: Add Input Validation to Signal Router
**File**: `packages/oms-engine/src/signal_router.rs`
**Action**: Add validation function for AgentSignal
```rust
fn validate_signal(signal: &AgentSignal) -> Result<(), String> {
    if !SYMBOLS.contains(&signal.symbol.as_str()) {
        return Err("Invalid symbol".to_string());
    }
    if !(0.0..=1.0).contains(&signal.conviction) {
        return Err("Conviction must be 0-1".to_string());
    }
    if signal.max_notional_usd <= 0.0 || signal.max_notional_usd > 1_000_000.0 {
        return Err("Invalid notional amount".to_string());
    }
    Ok(())
}
```
**Validates**: Invalid signals rejected with error log

### STEP 3: Fix Atomic Ordering in Risk Bus
**File**: `packages/oms-engine/src/risk_bus.rs`
**Action**: Change all atomic operations to SeqCst
```rust
// Replace all Ordering::Relaxed with Ordering::SeqCst for critical ops
pub fn set_halt(&self, reason: &str) {
    self.global_halt.store(true, Ordering::SeqCst);
    warn!("TRADING HALTED: {}", reason);
}

pub fn check_risk(&self) -> RiskCheck {
    // Use SeqCst for all reads
    if self.kill_switch.load(Ordering::SeqCst) {
        return RiskCheck::KillSwitch;
    }
    // ... rest of checks
}
```
**Validates**: No data races under 100-thread contention test

### STEP 4: Add Redis Authentication
**File**: `packages/feature-store/src/online.py`
**Action**: Add password to Redis connection
```python
def __init__(self, redis_url: str, password: Optional[str] = None):
    self.client = redis.Redis.from_url(
        redis_url,
        password=password,
        ssl=True,  # Enable TLS
        socket_connect_timeout=5,
        socket_timeout=5
    )
```
**Validates**: Connection fails without password

## CRITICAL COMPONENT VALIDATION

### STEP 5: Create Rust Integration Test for Risk Bus
**File**: `packages/oms-engine/tests/risk_bus_atomicity.rs`
**Action**: Add atomicity test
```rust
#[tokio::test]
async fn test_risk_bus_atomicity() {
    let risk_bus = Arc::new(RiskBus::new());
    let mut handles = vec![];
    
    // 100 threads updating concurrently
    for i in 0..100 {
        let risk_bus = Arc::clone(&risk_bus);
        let handle = tokio::spawn(async move {
            for j in 0..1000 {
                risk_bus.update_nav(10_000_000.0 + (i * j) as f64);
                risk_bus.check_risk();
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify no corruption
    assert!(risk_bus.get_nav() > 0.0);
}
```
**Validates**: Test passes without assertion failures

### STEP 6: Create Rust Load Test for Signal Router
**File**: `packages/oms-engine/tests/signal_router_load.rs`
**Action**: Add load test
```rust
#[tokio::test]
async fn test_signal_router_throughput() {
    let (tx, rx) = UnixStream::connect("/tmp/traderx_signals.sock").await.unwrap();
    let start = Instant::now();
    
    for i in 0..100_000 {
        let signal = AgentSignal {
            agent_id: format!("test_agent_{}", i % 10),
            symbol: "BTC-USD",
            direction: "long",
            conviction: 0.5,
            max_notional_usd: 100_000.0,
            ttl_ms: 1000,
            meta: Default::default(),
        };
        
        let msg = serde_json::to_vec(&signal).unwrap();
        tx.write_all(&msg).await.unwrap();
    }
    
    let duration = start.elapsed();
    assert!(duration.as_secs() < 10); // 10k signals/sec
}
```
**Validates**: Test completes in <10 seconds

### STEP 7: Add Aeron Journal Recovery Test
**File**: `packages/oms-engine/tests/journal_recovery.rs`
**Action**: Add recovery test
```rust
#[tokio::test]
async fn test_journal_crash_recovery() {
    let journal_path = "/tmp/test_oms_journal.db";
    
    // Phase 1: Create orders
    let oms1 = OMSEngine::new(journal_path).await;
    let order_ids = vec![];
    for i in 0..1000 {
        let id = oms1.create_order("ACC001", "AAPL", "Buy", "Market", 100.0, None).await;
        order_ids.push(id);
        if i % 2 == 0 {
            oms1.fill_order(&id, 150.0, 100.0).await;
        }
    }
    
    // Simulate crash
    drop(oms1);
    
    // Phase 2: Recover
    let oms2 = OMSEngine::new(journal_path).await;
    let recovered = oms2.recover_from_journal().await;
    assert!(recovered);
    
    // Verify all orders recovered
    for id in &order_ids {
        assert!(oms2.get_order(id).await.is_some());
    }
}
```
**Validates**: All 1000 orders recovered

## HIGH PRIORITY COMPONENTS

### STEP 8: Add Portfolio Aggregation Performance Test
**File**: `packages/portfolio-aggregation/tests/performance.rs`
**Action**: Add performance test
```rust
#[tokio::test]
async fn test_portfolio_update_performance() {
    let aggregator = PortfolioAggregator::new(1000);
    let start = Instant::now();
    
    for i in 0..10_000 {
        let fill = FillEvent {
            strategy_id: format!("strategy_{}", i % 100),
            symbol: "BTC-USD",
            asset_class: AssetClass::Crypto,
            side: "buy",
            quantity: 1.0,
            fill_price_usd: 50_000.0,
            commission_usd: 10.0,
            timestamp_ns: chrono::Utc::now().timestamp_nanos(),
        };
        
        aggregator.process_fill(fill);
    }
    
    let duration = start.elapsed();
    assert!(duration.as_millis() < 1000); // <1ms average
}
```
**Validates**: Test completes in <1 second

### STEP 9: Add Model Serving Inference Test
**File**: `packages/model-serving/tests/inference_performance.rs`
**Action**: Add inference test
```rust
#[tokio::test]
async fn test_model_inference_latency() {
    let engine = InferenceEngine::new(/* config */).await;
    
    let request = InferenceRequest {
        model_name: "price_predictor".to_string(),
        model_version: Some("1".to_string()),
        symbol: "BTC-USD".to_string(),
        timestamp_ns: chrono::Utc::now().timestamp_nanos(),
        features: None,
    };
    
    let start = Instant::now();
    let response = engine.predict(request).await.unwrap();
    let latency = start.elapsed();
    
    assert!(latency.as_millis() < 1); // <1ms
    assert!(!response.predictions.is_empty());
}
```
**Validates**: Inference <1ms with valid predictions

## INTEGRATION VALIDATION

### STEP 10: Create End-to-End Test
**File**: `tests/integration.rs`
**Action**: Add integration test
```rust
#[tokio::test]
async fn test_end_to_end_workflow() {
    // 1. Start QuestDB
    let questdb = start_questdb().await;
    
    // 2. Ingest ticks
    let databento = DatabentoAdapter::new();
    databento.ingest_to_questdb("BTC-USD", Duration::from_secs(10)).await;
    
    // 3. Compute features
    let feature_store = FeatureStore::new().await;
    let features = feature_store.compute_features("BTC-USD").await;
    assert!(!features.is_empty());
    
    // 4. Model inference
    let model_server = ModelServer::new().await;
    let prediction = model_server.predict("BTC-USD", features).await;
    assert!(prediction.confidence > 0.5);
    
    // 5. Route signal
    let signal_router = SignalRouter::new().await;
    let signal = AgentSignal::from_prediction(prediction);
    let order_id = signal_router.route(signal).await;
    assert!(!order_id.is_empty());
    
    // 6. Execute order
    let oms = OMSEngine::new().await;
    let fill = oms.execute_order(&order_id).await;
    assert!(fill.quantity > 0.0);
    
    // 7. Update portfolio
    let portfolio = PortfolioAggregator::new().await;
    portfolio.process_fill(fill);
    let pnl = portfolio.get_strategy_pnl("default").await;
    assert!(pnl.realized_usd != 0.0);
}
```
**Validates**: Complete workflow without errors

## MONITORING SETUP

### STEP 11: Add Prometheus Metrics to Risk Bus
**File**: `packages/oms-engine/src/risk_bus.rs`
**Action**: Add metrics
```rust
use prometheus::{IntCounter, Histogram, IntGauge};

lazy_static! {
    static ref RISK_CHECKS_TOTAL: IntCounter = IntCounter::new(
        "risk_checks_total",
        "Total number of risk checks performed"
    ).unwrap();
    
    static ref NAV_USD: IntGauge = IntGauge::new(
        "portfolio_nav_usd",
        "Current portfolio NAV in USD"
    ).unwrap();
}

impl RiskBus {
    pub fn check_risk(&self) -> RiskCheck {
        RISK_CHECKS_TOTAL.inc();
        
        let nav = self.nav_fp.load(Ordering::SeqCst) as f64 * 1e-4;
        NAV_USD.set(nav as i64);
        
        // ... rest of check
    }
}
```
**Validates**: Metrics visible at http://localhost:9090/metrics

### STEP 12: Add Health Check Endpoints
**File**: `packages/model-serving/src/server.rs`
**Action**: Add health endpoint
```rust
async fn health_check() -> Result<Json<HealthStatus>, StatusCode> {
    let status = HealthStatus {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    Ok(Json(status))
}
```
**Validates**: GET /health returns 200 OK

## DEPLOYMENT CONFIGURATION

### STEP 13: Create Kubernetes Deployment for OMS
**File**: `deployments/oms-engine.yaml`
**Action**: Add deployment manifest
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: oms-engine
spec:
  replicas: 2
  selector:
    matchLabels:
      app: oms-engine
  template:
    metadata:
      labels:
        app: oms-engine
    spec:
      containers:
      - name: oms-engine
        image: traderx/oms-engine:latest
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "info"
        resources:
          requests:
            cpu: 100m
            memory: 512Mi
          limits:
            cpu: 1000m
            memory: 1Gi
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
```
**Validates**: `kubectl apply -f oms-engine.yaml` succeeds

### STEP 14: Add NetworkPolicy for Security
**File**: `deployments/network-policy.yaml`
**Action**: Add network policy
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: traderx-network-policy
spec:
  podSelector:
    matchLabels:
      app: traderx
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: traderx
    ports:
    - protocol: TCP
      port: 8080
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: redis
    ports:
    - protocol: TCP
      port: 6379
```
**Validates**: Network policy applied successfully

## VALIDATION AUTOMATION

### STEP 15: Create CI Pipeline
**File**: `.github/workflows/validation.yml`
**Action**: Add validation workflow
```yaml
name: Production Validation
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  security:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - name: Run security audit
      run: |
        cargo audit
        pip-audit -r requirements.txt
        
  performance:
    runs-on: ubuntu-latest
    needs: security
    steps:
    - uses: actions/checkout@v3
    - name: Run performance tests
      run: |
        cargo test --release --test performance
        python -m pytest tests/performance/
        
  integration:
    runs-on: ubuntu-latest
    needs: [security, performance]
    steps:
    - uses: actions/checkout@v3
    - name: Run integration tests
      run: |
        docker-compose up -d
        cargo test --test integration
        docker-compose down
```
**Validates**: All CI jobs pass on commit

## EXECUTION ORDER

Execute steps in this order:
1. Steps 1-4 (Security fixes) - BLOCKING
2. Steps 5-7 (Critical validation) - DEPENDS ON 1-4
3. Steps 8-9 (High priority) - DEPENDS ON 5-7
4. Step 10 (Integration) - DEPENDS ON 8-9
5. Steps 11-12 (Monitoring) - PARALLEL WITH 13-14
6. Steps 13-14 (Deployment) - DEPENDS ON 10
7. Step 15 (CI) - FINAL

Each step must complete successfully before proceeding to dependent steps.
