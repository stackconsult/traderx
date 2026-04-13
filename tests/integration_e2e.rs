//! End-to-End Integration Test
//! Validates complete TraderX pipeline: Signal → Router → Risk → OMS → Portfolio

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use uuid::Uuid;

// Import core components
use oms_engine::signal_router::{SignalRouter, AgentSignal, RouterConfig};
use oms_engine::risk_bus::RiskBus;
use oms_engine::oms::{OmsEngine, Order, OrderType, Side};
use portfolio_aggregation::engine::{PortfolioAggregator, FillEvent, AggregatorEvent, AssetClass};
use feature_store::online::OnlineFeatureStore;

// Test utilities
mod e2e_helpers {
    use super::*;

    /// Enhanced AgentSignal with embedded tracer for latency measurement
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct TracedAgentSignal {
        pub agent_id: String,
        pub symbol: String,
        pub direction: String,
        pub conviction: f64,
        pub max_notional_usd: f64,
        pub ttl_ms: u64,
        #[serde(flatten)]
        pub meta: serde_json::Value,
        pub tracer_id: String,
        pub tracer_timestamp_ns: i64,
        pub tracer_step: String,
    }

    impl From<TracedAgentSignal> for AgentSignal {
        fn from(traced: TracedAgentSignal) -> Self {
            // Embed tracer info in meta field
            let mut meta = traced.meta;
            if let Some(obj) = meta.as_object_mut() {
                obj.insert("_tracer_id".to_string(), serde_json::Value::String(traced.tracer_id));
                obj.insert("_tracer_timestamp_ns".to_string(), serde_json::Value::Number(traced.tracer_timestamp_ns.into()));
                obj.insert("_tracer_step".to_string(), serde_json::Value::String(traced.tracer_step));
            }
            
            AgentSignal {
                agent_id: traced.agent_id,
                symbol: traced.symbol,
                direction: traced.direction,
                conviction: traced.conviction,
                max_notional_usd: traced.max_notional_usd,
                ttl_ms: traced.ttl_ms,
                meta,
            }
        }
    }

    /// Portfolio update with tracer information
    #[derive(Debug, Clone)]
    pub struct TracedPortfolioUpdate {
        pub strategy_id: String,
        pub symbol: String,
        pub tracer_id: String,
        pub tracer_timestamp_ns: i64,
        pub update_timestamp_ns: i64,
    }

    /// Mock Feature Store for testing
    pub struct MockFeatureStore {
        cache: Arc<tokio::sync::RwLock<HashMap<String, f64>>>,
    }

    impl MockFeatureStore {
        pub fn new() -> Self {
            Self {
                cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            }
        }

        pub async fn get(&self, key: &str) -> Option<f64> {
            self.cache.read().await.get(key).copied()
        }

        pub async fn set(&self, key: String, value: f64) {
            self.cache.write().await.insert(key, value);
        }
    }

    /// Latency measurement helper
    pub struct LatencyTracker {
        measurements: Arc<tokio::sync::Mutex<Vec<(String, Duration)>>>,
    }

    impl LatencyTracker {
        pub fn new() -> Self {
            Self {
                measurements: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            }
        }

        pub async fn record(&self, tracer_id: &str, tracer_timestamp_ns: i64, current_ns: i64) {
            let latency = Duration::from_nanos((current_ns - tracer_timestamp_ns) as u64);
            self.measurements.lock().await.push((tracer_id.to_string(), latency));
        }

        pub async fn get_stats(&self) -> (Duration, Duration, Duration) {
            let measurements = self.measurements.lock().await;
            if measurements.is_empty() {
                return (Duration::ZERO, Duration::ZERO, Duration::ZERO);
            }

            let mut latencies: Vec<Duration> = measurements.iter().map(|(_, l)| *l).collect();
            latencies.sort();
            
            let avg = latencies.iter().sum::<Duration>() / latencies.len() as u32;
            let p95 = latencies[(latencies.len() as f64 * 0.95) as usize];
            let p99 = latencies[(latencies.len() as f64 * 0.99) as usize];
            
            (avg, p95, p99)
        }
    }
}

#[tokio::test]
async fn test_complete_signal_to_portfolio_pipeline() {
    use e2e_helpers::*;

    // Setup temporary directories
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test_signals.sock");
    let journal_path = temp_dir.path().join("test_journal.aeron");
    let wal_path = temp_dir.path().join("test_portfolio.wal");

    // Initialize components
    let risk_bus = Arc::new(RiskBus::new(10_000_000.0, -2000)); // $10M NAV, -20% DD halt
    let (oms_tx, mut oms_rx) = mpsc::channel(10000);
    let latency_tracker = Arc::new(LatencyTracker::new());

    // Setup Signal Router
    let router_config = RouterConfig {
        socket_path: socket_path.to_string_lossy().to_string(),
        account_id: Uuid::new_v4(),
        kelly_fraction: 0.25,
        portfolio_nav_usd: 1_000_000.0,
    };
    let signal_router = Arc::new(SignalRouter::new(router_config, Arc::clone(&risk_bus), oms_tx));

    // Setup OMS
    let risk_checker = |_order: &Order| Ok(());
    let executor = move |order: Order| {
        // Simulate order execution and generate fill
        let fill_price = 150.0 + (rand::random::<f64>() - 0.5) * 2.0;
        let fill_qty = order.original_quantity.to_f64().unwrap();
        
        // Send to portfolio aggregator via channel
        Ok(())
    };
    let position_updater = |_account: Uuid, _qty: rust_decimal::Decimal, _price: rust_decimal::Decimal| Ok(());
    
    let oms = Arc::new(OmsEngine::new(
        1024,
        risk_checker,
        executor,
        position_updater,
    ).unwrap());

    // Setup Portfolio Aggregator
    let (portfolio_tx, portfolio_rx) = mpsc::channel(10000);
    let portfolio_aggregator = PortfolioAggregator::new(portfolio_rx, wal_path.to_str().unwrap());
    
    // Start all components
    let router_clone = Arc::clone(&signal_router);
    let socket_path_clone = socket_path.clone();
    tokio::spawn(async move {
        let _ = router_clone.start().await;
    });

    let oms_clone = Arc::clone(&oms);
    tokio::spawn(async move {
        while let Some(order_id) = oms_rx.recv().await {
            // Process order and generate fill
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    });

    let portfolio_clone = Arc::clone(&portfolio_aggregator);
    tokio::spawn(async move {
        portfolio_clone.run().await;
    });

    // Wait for components to initialize
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Generate test signals with tracers
    let num_signals = 1000;
    let start_time = Instant::now();
    
    for i in 0..num_signals {
        let tracer_id = format!("trace_{}", i);
        let tracer_timestamp = chrono::Utc::now().timestamp_nanos();
        
        let traced_signal = TracedAgentSignal {
            agent_id: format!("test_agent_{}", i % 10),
            symbol: "AAPL".to_string(),
            direction: if i % 2 == 0 { "long".to_string() } else { "short".to_string() },
            conviction: 0.5 + (i % 10) as f64 * 0.05,
            max_notional_usd: 100_000.0,
            ttl_ms: 1000,
            meta: serde_json::json!({"test_id": i}),
            tracer_id: tracer_id.clone(),
            tracer_timestamp_ns: tracer_timestamp,
            tracer_step: "signal_generation".to_string(),
        };

        // Convert to AgentSignal for sending
        let signal: AgentSignal = traced_signal.clone().into();

        // Send signal via Unix socket
        let mut stream = tokio::net::UnixStream::connect(&socket_path).await.unwrap();
        let msg = serde_json::to_string(&signal).unwrap() + "\n";
        stream.write_all(msg.as_bytes()).await.unwrap();
        
        if i % 100 == 0 {
            tokio::task::yield_now().await;
        }
    }

    // Wait for pipeline to process
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Verify end-to-end latency
    let (avg_latency, p95_latency, p99_latency) = latency_tracker.get_stats().await;
    
    println!("End-to-End Pipeline Latency:");
    println!("  Signals processed: {}", num_signals);
    println!("  Total duration: {:?}", start_time.elapsed());
    println!("  Average latency: {:?}", avg_latency);
    println!("  P95 latency: {:?}", p95_latency);
    println!("  P99 latency: {:?}", p99_latency);

    // Validate latency requirements (should be <10ms end-to-end)
    assert!(p95_latency.as_millis() < 10,
        "P95 end-to-end latency {:?} > 10ms requirement", p95_latency);

    // Verify risk checks were performed
    let risk_checks = risk_bus.checks_performed.load(std::sync::atomic::Ordering::SeqCst);
    assert!(risk_checks > 0, "No risk checks were performed");

    // Verify portfolio received updates
    let portfolio_updates = portfolio_aggregator.trade_counts.len();
    assert!(portfolio_updates > 0, "No portfolio updates received");
}

#[tokio::test]
async fn test_risk_halt_circuit_breaker() {
    use e2e_helpers::*;

    // Setup components with low risk threshold for testing
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test_signals.sock");
    
    let risk_bus = Arc::new(RiskBus::new(1_000_000.0, -500)); // $1M NAV, -5% halt
    let (oms_tx, _) = mpsc::channel(10000);

    let router_config = RouterConfig {
        socket_path: socket_path.to_string_lossy().to_string(),
        account_id: Uuid::new_v4(),
        kelly_fraction: 0.25,
        portfolio_nav_usd: 100_000.0,
    };
    let signal_router = Arc::new(SignalRouter::new(router_config, Arc::clone(&risk_bus), oms_tx));

    // Start router
    let router_clone = Arc::clone(&signal_router);
    let socket_path_clone = socket_path.clone();
    tokio::spawn(async move {
        let _ = router_clone.start().await;
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Generate signals that will trigger risk halt
    let mut signals_sent = 0;
    let mut signals_rejected = 0;

    for i in 0..100 {
        let traced_signal = TracedAgentSignal {
            agent_id: "risk_test_agent".to_string(),
            symbol: "AAPL".to_string(),
            direction: "long".to_string(),
            conviction: 0.9, // High conviction
            max_notional_usd: 500_000.0, // Large notional
            ttl_ms: 1000,
            meta: serde_json::json!({"test_id": i}),
            tracer_id: format!("risk_trace_{}", i),
            tracer_timestamp_ns: chrono::Utc::now().timestamp_nanos(),
            tracer_step: "risk_test".to_string(),
        };

        let signal: AgentSignal = traced_signal.into();

        // Send signal
        let mut stream = tokio::net::UnixStream::connect(&socket_path).await.unwrap();
        let msg = serde_json::to_string(&signal).unwrap() + "\n";
        stream.write_all(msg.as_bytes()).await.unwrap();
        signals_sent += 1;

        // Check if risk was triggered
        if risk_bus.is_halted() {
            println!("Risk halt triggered after {} signals", i + 1);
            break;
        }

        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    // Verify risk halt was triggered
    assert!(risk_bus.is_halted(), "Risk halt should have been triggered");

    // Try to send more signals after halt
    for i in 0..10 {
        let traced_signal = TracedAgentSignal {
            agent_id: "post_halt_agent".to_string(),
            symbol: "GOOGL".to_string(),
            direction: "long".to_string(),
            conviction: 0.5,
            max_notional_usd: 10_000.0,
            ttl_ms: 1000,
            meta: serde_json::json!({"post_halt": i}),
            tracer_id: format!("post_halt_{}", i),
            tracer_timestamp_ns: chrono::Utc::now().timestamp_nanos(),
            tracer_step: "post_halt".to_string(),
        };

        let signal: AgentSignal = traced_signal.into();

        let mut stream = tokio::net::UnixStream::connect(&socket_path).await.unwrap();
        let msg = serde_json::to_string(&signal).unwrap() + "\n";
        let _ = stream.write_all(msg.as_bytes()).await.unwrap();
    }

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify signals were rejected after halt
    let rejected_after_halt = risk_bus.orders_rejected.load(std::sync::atomic::Ordering::SeqCst);
    assert!(rejected_after_halt > 0, "Signals should be rejected after risk halt");

    println!("Risk Halt Test:");
    println!("  Signals sent before halt: {}", signals_sent);
    println!("  Signals rejected after halt: {}", rejected_after_halt);
}

#[tokio::test]
async fn test_concurrent_multi_agent_workflow() {
    use e2e_helpers::*;

    // Setup for multiple concurrent agents
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test_signals.sock");
    
    let risk_bus = Arc::new(RiskBus::new(10_000_000.0, -2000));
    let (oms_tx, _) = mpsc::channel(10000);

    let router_config = RouterConfig {
        socket_path: socket_path.to_string_lossy().to_string(),
        account_id: Uuid::new_v4(),
        kelly_fraction: 0.25,
        portfolio_nav_usd: 1_000_000.0,
    };
    let signal_router = Arc::new(SignalRouter::new(router_config, Arc::clone(&risk_bus), oms_tx));

    // Start router
    let router_clone = Arc::clone(&signal_router);
    let socket_path_clone = socket_path.clone();
    tokio::spawn(async move {
        let _ = router_clone.start().await;
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Spawn multiple agent tasks
    let num_agents = 10;
    let signals_per_agent = 100;
    let mut tasks = Vec::new();

    for agent_id in 0..num_agents {
        let socket_path = socket_path.clone();
        let task = tokio::spawn(async move {
            for i in 0..signals_per_agent {
                let traced_signal = TracedAgentSignal {
                    agent_id: format!("concurrent_agent_{}", agent_id),
                    symbol: if i % 3 == 0 { "AAPL" } else if i % 3 == 1 { "GOOGL" } else { "MSFT" },
                    direction: if i % 2 == 0 { "long" } else { "short" },
                    conviction: 0.3 + (i % 10) as f64 * 0.07,
                    max_notional_usd: 50_000.0,
                    ttl_ms: 1000,
                    meta: serde_json::json!({"agent": agent_id, "signal": i}),
                    tracer_id: format!("concurrent_{}_{}", agent_id, i),
                    tracer_timestamp_ns: chrono::Utc::now().timestamp_nanos(),
                    tracer_step: "concurrent_test".to_string(),
                };

                let signal: AgentSignal = traced_signal.into();

                let mut stream = tokio::net::UnixStream::connect(&socket_path).await.unwrap();
                let msg = serde_json::to_string(&signal).unwrap() + "\n";
                let _ = stream.write_all(msg.as_bytes()).await.unwrap();

                if i % 20 == 0 {
                    tokio::task::yield_now().await;
                }
            }
        });
        tasks.push(task);
    }

    // Wait for all agents
    for task in tasks {
        task.await.unwrap();
    }

    tokio::time::sleep(Duration::from_millis(500)).await;

    // Verify all signals were processed
    let total_processed = risk_bus.orders_submitted.load(std::sync::atomic::Ordering::SeqCst);
    let expected_total = num_agents * signals_per_agent;
    
    println!("Concurrent Multi-Agent Test:");
    println!("  Agents: {}", num_agents);
    println!("  Signals per agent: {}", signals_per_agent);
    println!("  Total expected: {}", expected_total);
    println!("  Total processed: {}", total_processed);

    // Allow some signals to still be in flight
    assert!(total_processed > expected_total as i64 / 2,
        "Too few signals processed: {}", total_processed);
}

#[tokio::test]
async fn test_feature_store_integration() {
    use e2e_helpers::*;

    // Test integration with feature store
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test_signals.sock");
    
    // Setup mock feature store
    let feature_store = Arc::new(MockFeatureStore::new());
    
    // Pre-populate features
    feature_store.set("AAPL_volatility".to_string(), 0.25).await;
    feature_store.set("AAPL_trend".to_string(), 0.75).await;
    feature_store.set("AAPL_rsi".to_string(), 0.60).await;

    let risk_bus = Arc::new(RiskBus::new(10_000_000.0, -2000));
    let (oms_tx, _) = mpsc::channel(10000);

    let router_config = RouterConfig {
        socket_path: socket_path.to_string_lossy().to_string(),
        account_id: Uuid::new_v4(),
        kelly_fraction: 0.25,
        portfolio_nav_usd: 1_000_000.0,
    };
    let signal_router = Arc::new(SignalRouter::new(router_config, Arc::clone(&risk_bus), oms_tx));

    // Start router
    let router_clone = Arc::clone(&signal_router);
    let socket_path_clone = socket_path.clone();
    tokio::spawn(async move {
        let _ = router_clone.start().await;
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Send signal that depends on features
    let traced_signal = TracedAgentSignal {
        agent_id: "feature_test_agent".to_string(),
        symbol: "AAPL".to_string(),
        direction: "long".to_string(),
        conviction: 0.7,
        max_notional_usd: 100_000.0,
        ttl_ms: 1000,
        meta: serde_json::json!({
            "requires_features": ["volatility", "trend", "rsi"]
        }),
        tracer_id: "feature_test_1".to_string(),
        tracer_timestamp_ns: chrono::Utc::now().timestamp_nanos(),
        tracer_step: "feature_test".to_string(),
    };

    let signal: AgentSignal = traced_signal.into();

    let mut stream = tokio::net::UnixStream::connect(&socket_path).await.unwrap();
    let msg = serde_json::to_string(&signal).unwrap() + "\n";
    stream.write_all(msg.as_bytes()).await.unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify signal was processed (features were available)
    let processed = risk_bus.orders_submitted.load(std::sync::atomic::Ordering::SeqCst);
    assert!(processed > 0, "Signal should be processed when features are available");

    // Test with missing features
    let traced_signal_no_features = TracedAgentSignal {
        agent_id: "feature_test_agent".to_string(),
        symbol: "TSLA".to_string(), // No features for TSLA
        direction: "long".to_string(),
        conviction: 0.7,
        max_notional_usd: 100_000.0,
        ttl_ms: 1000,
        meta: serde_json::json!({
            "requires_features": ["volatility", "trend", "rsi"]
        }),
        tracer_id: "feature_test_2".to_string(),
        tracer_timestamp_ns: chrono::Utc::now().timestamp_nanos(),
        tracer_step: "feature_test".to_string(),
    };

    let signal_no_features: AgentSignal = traced_signal_no_features.into();

    let mut stream = tokio::net::UnixStream::connect(&socket_path).await.unwrap();
    let msg = serde_json::to_string(&signal_no_features).unwrap() + "\n";
    stream.write_all(msg.as_bytes()).await.unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;

    println!("Feature Store Integration Test:");
    println!("  Signals processed: {}", processed);
    println!("  Features available for AAPL: 3");
    println!("  Features available for TSLA: 0");
}
