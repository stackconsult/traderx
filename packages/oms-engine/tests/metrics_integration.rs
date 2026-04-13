//! Metrics Integration Tests
//! Validates Prometheus metrics collection and exposure

use oms_engine::{RiskBus, SignalRouter, RouterConfig, AgentSignal};
use oms_engine::metrics::GLOBAL_RISK_METRICS;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

#[tokio::test]
async fn test_risk_bus_metrics_collection() {
    // Create Risk Bus instance
    let risk_bus = RiskBus::new(1_000_000.0, -2000);
    
    // Test order submission metrics
    risk_bus.record_order_submitted();
    risk_bus.record_order_submitted();
    
    // Test order rejection metrics
    risk_bus.record_order_rejected();
    
    // Test NAV updates
    risk_bus.update_nav(1_050_000.0);
    risk_bus.update_nav(950_000.0); // Triggers drawdown
    
    // Test halt status
    risk_bus.assert_kill_switch("test");
    
    // Verify metrics are exported
    let metrics = GLOBAL_RISK_METRICS.export().unwrap();
    
    assert!(metrics.contains("riskbus_orders_submitted_total 2"));
    assert!(metrics.contains("riskbus_orders_rejected_total 1"));
    assert!(metrics.contains("riskbus_current_nav 9500000000")); // NAV in basis points
    assert!(metrics.contains("riskbus_is_halted 1"));
    assert!(metrics.contains("riskbus_current_drawdown_bps"));
    assert!(metrics.contains("riskbus_risk_checks_total"));
}

#[tokio::test]
async fn test_signal_router_metrics_integration() {
    // Create Risk Bus and Signal Router
    let risk_bus = RiskBus::new(1_000_000.0, -2000);
    let (oms_tx, _) = mpsc::channel(100);
    
    let config = RouterConfig {
        socket_path: "/tmp/test_signals.sock".to_string(),
        account_id: Uuid::new_v4(),
        kelly_fraction: 0.25,
        portfolio_nav_usd: 100_000.0,
    };
    
    let router = SignalRouter::new(config, risk_bus.clone(), oms_tx);
    
    // Create test signal
    let signal = AgentSignal {
        agent_id: "test_agent".to_string(),
        symbol: "AAPL".to_string(),
        direction: "long".to_string(),
        conviction: 0.7,
        max_notional_usd: 10_000.0,
        ttl_ms: 1000,
        meta: serde_json::json!({}),
    };
    
    // Route signal (should increment metrics)
    let _outcome = router.route_signal(signal);
    
    // Verify metrics were updated
    let metrics = GLOBAL_RISK_METRICS.export().unwrap();
    assert!(metrics.contains("riskbus_orders_submitted_total"));
    assert!(metrics.contains("riskbus_risk_checks_total"));
    assert!(metrics.contains("riskbus_risk_check_duration_seconds"));
}

#[tokio::test]
async fn test_metrics_performance_overhead() {
    // Test that metrics collection adds minimal overhead
    let risk_bus = RiskBus::new(1_000_000.0, -2000);
    
    let iterations = 100_000;
    let start = std::time::Instant::now();
    
    for _ in 0..iterations {
        risk_bus.record_order_submitted();
    }
    
    let duration = start.elapsed();
    let avg_ns = duration.as_nanos() / iterations;
    
    // Should be less than 100ns per operation
    assert!(avg_ns < 100, "Metrics overhead too high: {}ns per operation", avg_ns);
    
    println!("Metrics overhead: {}ns per operation", avg_ns);
}

#[tokio::test]
async fn test_concurrent_metrics_updates() {
    // Test metrics under concurrent load
    let risk_bus = Arc::new(RiskBus::new(1_000_000.0, -2000));
    
    let num_threads = 10;
    let operations_per_thread = 1000;
    let mut handles = Vec::new();
    
    for _ in 0..num_threads {
        let risk_bus = Arc::clone(&risk_bus);
        let handle = tokio::task::spawn_blocking(move || {
            for _ in 0..operations_per_thread {
                risk_bus.record_order_submitted();
                risk_bus.update_nav(1_000_000.0 + (rand::random::<f64>() - 0.5) * 1000.0);
            }
        });
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify all operations were recorded
    let submitted = risk_bus.orders_submitted_count();
    assert_eq!(submitted, (num_threads * operations_per_thread) as i64);
    
    let metrics = GLOBAL_RISK_METRICS.export().unwrap();
    assert!(metrics.contains("riskbus_orders_submitted_total 10000"));
}

#[tokio::test]
async fn test_metrics_server_endpoint() {
    // Test metrics HTTP server
    use oms_engine::metrics_server::{MetricsServer, MetricsServerConfig};
    
    let config = MetricsServerConfig {
        bind_addr: "127.0.0.1:0".parse().unwrap(),
        rate_limit_per_sec: 100,
    };
    
    let server = MetricsServer::new(config);
    let router = server.build_router();
    
    // Test metrics endpoint
    let request = axum::extract::Request::builder()
        .uri("/metrics")
        .body(axum::body::Body::empty())
        .unwrap();
    
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let metrics = String::from_utf8(body.to_vec()).unwrap();
    assert!(metrics.contains("riskbus_"));
    
    // Test rate limiting
    let request = axum::extract::Request::builder()
        .uri("/metrics")
        .body(axum::body::Body::empty())
        .unwrap();
    
    let response = router.oneshot(request).await.unwrap();
    // First request should succeed
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    
    // Immediate second request might be rate limited depending on timing
    // This is expected behavior
}

#[tokio::test]
async fn test_symbol_specific_metrics() {
    let risk_bus = RiskBus::new(1_000_000.0, -2000);
    
    // Set symbol limits
    risk_bus.set_symbol_limit("AAPL", 100_000.0);
    risk_bus.set_symbol_limit("GOOGL", 200_000.0);
    
    // Get symbol metrics
    let aapl_metrics = GLOBAL_RISK_METRICS.get_symbol_metrics("AAPL");
    let googl_metrics = GLOBAL_RISK_METRICS.get_symbol_metrics("GOOGL");
    
    // Update symbol metrics
    aapl_metrics.exposure.set(50000);
    aapl_metrics.orders.inc();
    
    googl_metrics.exposure.set(100000);
    googl_metrics.orders.inc();
    googl_metrics.orders.inc();
    
    // Verify metrics are tracked
    let metrics = GLOBAL_RISK_METRICS.export().unwrap();
    assert!(metrics.contains("riskbus_symbol_exposure"));
    assert!(metrics.contains("riskbus_symbol_orders_total"));
}
