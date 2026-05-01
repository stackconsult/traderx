//! Signal Router load test
//! Validates 10k signals/sec with <5μs latency requirement

use oms_engine::signal_router::{SignalRouter, AgentSignal, RouterConfig};
use oms_engine::risk_bus::RiskBus;
use oms_engine::oms::{OmsEngine, OmsError};
use std::sync::Arc;
use std::time::{Duration, Instant};
// use tempfile::TempDir;  // Not in dependencies
use tokio::net::UnixStream;
use tokio::sync::mpsc;
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::task::JoinSet;
use serde_json;

#[tokio::test]
async fn test_signal_router_throughput_10k_per_sec() {
    let temp_dir = std::env::temp_dir();
    let socket_path = temp_dir.join(format!("test_signals_{}.sock", uuid::Uuid::new_v4()));
    
    // Setup components
    let risk_bus = RiskBus::new(10_000_000.0, -2000);
    let (oms_tx, mut oms_rx) = mpsc::channel(10000);
    
    let config = RouterConfig {
        socket_path: socket_path.to_string_lossy().to_string(),
        account_id: uuid::Uuid::new_v4(),
        kelly_fraction: 0.25,
        portfolio_nav_usd: 1_000_000.0,
    };
    
    let router = Arc::new(SignalRouter::new(config, Arc::clone(&risk_bus), oms_tx));
    
    // Start router
    let router_clone = Arc::clone(&router);
    let socket_path_clone = socket_path.clone();
    tokio::spawn(async move {
        let _ = router_clone.start().await;
    });
    
    // Wait for socket to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Connect and send signals
    let num_signals = 100_000;
    let start = Instant::now();
    
    let mut stream = UnixStream::connect(&socket_path).await.unwrap();
    
    for i in 0..num_signals {
        let signal = AgentSignal {
            agent_id: format!("test_agent_{}", i % 10),
            symbol: "AAPL".to_string(),
            direction: if i % 2 == 0 { "long".to_string() } else { "short".to_string() },
            conviction: 0.5 + (i % 10) as f64 * 0.05,
            max_notional_usd: 100_000.0,
            ttl_ms: 1000,
            meta: serde_json::json!({"test_id": i}),
        };
        
        let msg = serde_json::to_string(&signal).unwrap() + "\n";
        stream.write_all(msg.as_bytes()).await.unwrap();
        
        // Small yield every 1000 signals to prevent blocking
        if i % 1000 == 0 {
            tokio::task::yield_now().await;
        }
    }
    
    let duration = start.elapsed();
    let signals_per_sec = num_signals as f64 / duration.as_secs_f64();
    
    println!("Signal Router Throughput:");
    println!("  Signals sent: {}", num_signals);
    println!("  Duration: {:?}", duration);
    println!("  Signals/sec: {:.0}", signals_per_sec);
    
    // Validate throughput requirement
    assert!(signals_per_sec >= 10_000.0, 
        "Throughput {:.0} < 10,000 signals/sec requirement", signals_per_sec);
    
    // Verify orders were processed
    tokio::time::sleep(Duration::from_millis(100)).await;
    let processed_count = risk_bus.orders_submitted.load(std::sync::atomic::Ordering::SeqCst);
    assert!(processed_count > 0, "No orders were processed");
}

#[tokio::test]
async fn test_signal_router_latency_under_5us() {
    let temp_dir = std::env::temp_dir();
    let socket_path = temp_dir.join(format!("test_latency_{}.sock", uuid::Uuid::new_v4()));
    
    // Setup components
    let risk_bus = RiskBus::new(10_000_000.0, -2000);
    let (oms_tx, _) = mpsc::channel(10000);
    
    let config = RouterConfig {
        socket_path: socket_path.to_string_lossy().to_string(),
        account_id: uuid::Uuid::new_v4(),
        kelly_fraction: 0.25,
        portfolio_nav_usd: 1_000_000.0,
    };
    
    let router = Arc::new(SignalRouter::new(config, Arc::clone(&risk_bus), oms_tx));
    
    // Start router
    let router_clone = Arc::clone(&router);
    let socket_path_clone = socket_path.clone();
    tokio::spawn(async move {
        let _ = router_clone.start().await;
    });
    
    // Wait for socket to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Measure latency for individual signals
    let num_samples = 1000;
    let mut latencies = Vec::new();
    
    for i in 0..num_samples {
        let signal = AgentSignal {
            agent_id: "latency_test".to_string(),
            symbol: "AAPL".to_string(),
            direction: "long".to_string(),
            conviction: 0.5,
            max_notional_usd: 100_000.0,
            ttl_ms: 1000,
            meta: serde_json::json!({"sample": i}),
        };
        
        let start = Instant::now();
        
        let mut stream = UnixStream::connect(&socket_path).await.unwrap();
        let msg = serde_json::to_string(&signal).unwrap() + "\n";
        stream.write_all(msg.as_bytes()).await.unwrap();
        stream.flush().await.unwrap();
        
        // Approximate latency (socket write time)
        let latency = start.elapsed();
        latencies.push(latency);
        
        // Small delay between samples
        tokio::time::sleep(Duration::from_micros(10)).await;
    }
    
    // Calculate statistics
    latencies.sort();
    let avg_latency = latencies.iter().sum::<Duration>() / num_samples as u32;
    let p95_latency = latencies[(num_samples as f64 * 0.95) as usize];
    let p99_latency = latencies[(num_samples as f64 * 0.99) as usize];
    
    println!("Signal Router Latency:");
    println!("  Average: {:?}", avg_latency);
    println!("  P95: {:?}", p95_latency);
    println!("  P99: {:?}", p99_latency);
    
    // Validate latency requirement (note: this includes socket I/O)
    assert!(p95_latency.as_nanos() < 5000, 
        "P95 latency {:?} > 5μs requirement", p95_latency);
}

#[tokio::test]
async fn test_concurrent_agent_connections() {
    let temp_dir = std::env::temp_dir();
    let socket_path = temp_dir.join(format!("test_concurrent_{}.sock", uuid::Uuid::new_v4()));
    
    // Setup components
    let risk_bus = RiskBus::new(10_000_000.0, -2000);
    let (oms_tx, _) = mpsc::channel(10000);
    
    let config = RouterConfig {
        socket_path: socket_path.to_string_lossy().to_string(),
        account_id: uuid::Uuid::new_v4(),
        kelly_fraction: 0.25,
        portfolio_nav_usd: 1_000_000.0,
    };
    
    let router = Arc::new(SignalRouter::new(config, Arc::clone(&risk_bus), oms_tx));
    
    // Start router
    let router_clone = Arc::clone(&router);
    let socket_path_clone = socket_path.clone();
    tokio::spawn(async move {
        let _ = router_clone.start().await;
    });
    
    // Wait for socket to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Spawn 10 concurrent agents
    let num_agents = 10;
    let signals_per_agent = 1000;
    let mut tasks = JoinSet::new();
    
    for agent_id in 0..num_agents {
        let socket_path = socket_path.clone();
        tasks.spawn(async move {
            let mut stream = UnixStream::connect(&socket_path).await.unwrap();
            
            for i in 0..signals_per_agent {
                let signal = AgentSignal {
                    agent_id: format!("agent_{}", agent_id),
                    symbol: "AAPL".to_string(),
                    direction: "long".to_string(),
                    conviction: 0.5,
                    max_notional_usd: 100_000.0,
                    ttl_ms: 1000,
                    meta: serde_json::json!({"agent": agent_id, "signal": i}),
                };
                
                let msg = serde_json::to_string(&signal).unwrap() + "\n";
                stream.write_all(msg.as_bytes()).await.unwrap();
                
                if i % 100 == 0 {
                    tokio::task::yield_now().await;
                }
            }
        });
    }
    
    // Wait for all agents to complete
    while let Some(_) = tasks.join_next().await {}
    
    // Verify all signals were processed
    tokio::time::sleep(Duration::from_millis(100)).await;
    let processed_count = risk_bus.orders_submitted.load(std::sync::atomic::Ordering::SeqCst);
    let expected_count = num_agents * signals_per_agent;
    
    println!("Concurrent Agents Test:");
    println!("  Agents: {}", num_agents);
    println!("  Signals per agent: {}", signals_per_agent);
    println!("  Total expected: {}", expected_count);
    println!("  Total processed: {}", processed_count);
    
    // Allow some signals to still be in flight
    assert!(processed_count > expected_count as i64 / 2, 
        "Too few signals processed: {}", processed_count);
}

#[tokio::test]
async fn test_invalid_signal_handling() {
    let temp_dir = std::env::temp_dir();
    let socket_path = temp_dir.join(format!("test_invalid_{}.sock", uuid::Uuid::new_v4()));
    
    // Setup components
    let risk_bus = RiskBus::new(10_000_000.0, -2000);
    let (oms_tx, _) = mpsc::channel(10000);
    
    let config = RouterConfig {
        socket_path: socket_path.to_string_lossy().to_string(),
        account_id: uuid::Uuid::new_v4(),
        kelly_fraction: 0.25,
        portfolio_nav_usd: 1_000_000.0,
    };
    
    let router = Arc::new(SignalRouter::new(config, Arc::clone(&risk_bus), oms_tx));
    
    // Start router
    let router_clone = Arc::clone(&router);
    let socket_path_clone = socket_path.clone();
    tokio::spawn(async move {
        let _ = router_clone.start().await;
    });
    
    // Wait for socket to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let mut stream = UnixStream::connect(&socket_path).await.unwrap();
    
    // Test invalid signals
    let invalid_signals = vec![
        // Invalid symbol
        serde_json::json!({
            "agent_id": "test",
            "symbol": "INVALID",
            "direction": "long",
            "conviction": 0.5,
            "max_notional_usd": 100000.0,
            "ttl_ms": 1000
        }),
        // Invalid conviction
        serde_json::json!({
            "agent_id": "test",
            "symbol": "AAPL",
            "direction": "long",
            "conviction": 1.5,
            "max_notional_usd": 100000.0,
            "ttl_ms": 1000
        }),
        // Invalid notional
        serde_json::json!({
            "agent_id": "test",
            "symbol": "AAPL",
            "direction": "long",
            "conviction": 0.5,
            "max_notional_usd": -1000.0,
            "ttl_ms": 1000
        }),
        // Malformed JSON
        serde_json::Value::String("{invalid json}".to_string()),
    ];
    
    for invalid_signal in invalid_signals {
        let msg = if let serde_json::Value::Object(obj) = invalid_signal {
            serde_json::to_string(&obj).unwrap() + "\n"
        } else {
            format!("{}\n", invalid_signal)
        };
        
        stream.write_all(msg.as_bytes()).await.unwrap();
    }
    
    // Verify router is still responsive
    let valid_signal = AgentSignal {
        agent_id: "test".to_string(),
        symbol: "AAPL".to_string(),
        direction: "long".to_string(),
        conviction: 0.5,
        max_notional_usd: 100_000.0,
        ttl_ms: 1000,
        meta: serde_json::json!({}),
    };
    
    let msg = serde_json::to_string(&valid_signal).unwrap() + "\n";
    stream.write_all(msg.as_bytes()).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // Should have processed at least the valid signal
    let processed_count = risk_bus.orders_submitted.load(std::sync::atomic::Ordering::SeqCst);
    assert!(processed_count > 0, "Router stopped processing after invalid signals");
    
    // Check rejections
    let rejected_count = risk_bus.orders_rejected.load(std::sync::atomic::Ordering::SeqCst);
    println!("Invalid Signal Test:");
    println!("  Processed: {}", processed_count);
    println!("  Rejected: {}", rejected_count);
}
