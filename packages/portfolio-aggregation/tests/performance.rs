//! Portfolio Aggregation Performance Tests
//! Validates 10k updates/sec with <1ms latency requirement

use portfolio_aggregation::engine::{PortfolioAggregator, FillEvent, AggregatorEvent, AssetClass};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;

#[tokio::test]
async fn test_portfolio_update_performance_10k_per_sec() {
    let (event_tx, event_rx) = mpsc::channel(10000);
    let wal_path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
    let aggregator = PortfolioAggregator::new(event_rx, wal_path.to_str().unwrap());
    
    // Start aggregator in background
    let agg_clone = Arc::clone(&aggregator);
    tokio::spawn(async move {
        agg_clone.run().await;
    });
    
    // Test parameters
    let num_updates = 10_000;
    let num_strategies = 100;
    let symbols = vec!["AAPL", "GOOGL", "MSFT", "TSLA", "AMZN", "BTC-USD", "ETH-USD"];
    
    // Generate test fills
    let mut rng = SmallRng::seed_from_u64(42);
    let fills: Vec<FillEvent> = (0..num_updates)
        .map(|i| {
            FillEvent {
                strategy_id: format!("strategy_{}", rng.gen_range(0..num_strategies)),
                symbol: symbols[rng.gen_range(0..symbols.len())].to_string(),
                asset_class: if rng.gen_bool(0.5) { AssetClass::Equity } else { AssetClass::Crypto },
                side: if rng.gen_bool(0.5) { "buy".to_string() } else { "sell".to_string() },
                quantity: rng.gen_range(1.0..1000.0),
                fill_price_usd: rng.gen_range(50.0..5000.0),
                commission_usd: rng.gen_range(0.1..10.0),
                timestamp_ns: chrono::Utc::now().timestamp_nanos(),
            }
        })
        .collect();
    
    // Measure throughput
    let start = Instant::now();
    
    for fill in fills {
        let event = AggregatorEvent::Fill(fill);
        if let Err(_) = event_tx.send(event).await {
            panic!("Failed to send event to aggregator");
        }
    }
    
    // Wait for processing
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let duration = start.elapsed();
    let updates_per_sec = num_updates as f64 / duration.as_secs_f64();
    
    println!("Portfolio Aggregation Throughput:");
    println!("  Updates sent: {}", num_updates);
    println!("  Duration: {:?}", duration);
    println!("  Updates/sec: {:.0}", updates_per_sec);
    
    // Validate throughput requirement
    assert!(updates_per_sec >= 10_000.0,
        "Throughput {:.0} < 10,000 updates/sec requirement", updates_per_sec);
    
    // Verify state consistency
    let (tx, rx) = mpsc::channel(1);
    let query_event = AggregatorEvent::GetStrategyPnl("strategy_0".to_string(), tx);
    event_tx.send(query_event).await.unwrap();
    
    if let Some(pnl) = rx.recv().await {
        assert!(pnl.num_trades > 0, "No trades recorded for strategy_0");
        println!("Strategy 0 P&L: ${:.2} from {} trades", pnl.total_usd, pnl.num_trades);
    }
}

#[tokio::test]
async fn test_portfolio_latency_under_1ms() {
    let (event_tx, event_rx) = mpsc::channel(10000);
    let wal_path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
    let aggregator = PortfolioAggregator::new(event_rx, wal_path.to_str().unwrap());
    
    // Start aggregator
    let agg_clone = Arc::clone(&aggregator);
    tokio::spawn(async move {
        agg_clone.run().await;
    });
    
    // Measure individual fill processing latency
    let num_samples = 1000;
    let mut latencies = Vec::new();
    
    for i in 0..num_samples {
        let fill = FillEvent {
            strategy_id: "latency_test".to_string(),
            symbol: "AAPL".to_string(),
            asset_class: AssetClass::Equity,
            side: "buy".to_string(),
            quantity: 100.0,
            fill_price_usd: 150.0,
            commission_usd: 1.0,
            timestamp_ns: chrono::Utc::now().timestamp_nanos(),
        };
        
        let start = Instant::now();
        
        // Send fill
        let event = AggregatorEvent::Fill(fill);
        event_tx.send(event).await.unwrap();
        
        // Query P&L to ensure processing
        let (tx, rx) = mpsc::channel(1);
        let query = AggregatorEvent::GetStrategyPnl("latency_test".to_string(), tx);
        event_tx.send(query).await.unwrap();
        
        // Wait for response
        let _ = rx.recv().await;
        
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
    
    println!("Portfolio Aggregation Latency:");
    println!("  Average: {:?}", avg_latency);
    println!("  P95: {:?}", p95_latency);
    println!("  P99: {:?}", p99_latency);
    
    // Validate latency requirement
    assert!(p95_latency.as_millis() < 1,
        "P95 latency {:?} > 1ms requirement", p95_latency);
}

#[tokio::test]
async fn test_concurrent_strategy_updates() {
    let (event_tx, event_rx) = mpsc::channel(10000);
    let wal_path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
    let aggregator = PortfolioAggregator::new(event_rx, wal_path.to_str().unwrap());
    
    // Start aggregator
    let agg_clone = Arc::clone(&aggregator);
    tokio::spawn(async move {
        agg_clone.run().await;
    });
    
    // Spawn concurrent update tasks
    let num_tasks = 10;
    let updates_per_task = 1000;
    let mut tasks = Vec::new();
    
    for task_id in 0..num_tasks {
        let event_tx = event_tx.clone();
        let task = tokio::spawn(async move {
            let mut rng = SmallRng::seed_from_u64(task_id);
            
            for i in 0..updates_per_task {
                let fill = FillEvent {
                    strategy_id: format!("strategy_{}", task_id),
                    symbol: if i % 2 == 0 { "AAPL" } else { "GOOGL" },
                    asset_class: AssetClass::Equity,
                    side: if rng.gen_bool(0.5) { "buy" } else { "sell" },
                    quantity: rng.gen_range(10.0..100.0),
                    fill_price_usd: 150.0 + rng.gen_range(-10.0..10.0),
                    commission_usd: 1.0,
                    timestamp_ns: chrono::Utc::now().timestamp_nanos(),
                };
                
                let event = AggregatorEvent::Fill(fill);
                if event_tx.send(event).await.is_err() {
                    break;
                }
                
                if i % 100 == 0 {
                    tokio::task::yield_now().await;
                }
            }
        });
        tasks.push(task);
    }
    
    // Wait for all tasks
    for task in tasks {
        task.await.unwrap();
    }
    
    // Wait for processing
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Verify all strategies have trades
    for task_id in 0..num_tasks {
        let (tx, rx) = mpsc::channel(1);
        let query = AggregatorEvent::GetStrategyPnl(format!("strategy_{}", task_id), tx);
        event_tx.send(query).await.unwrap();
        
        if let Some(pnl) = rx.recv().await {
            assert!(pnl.num_trades > 0, 
                "Strategy {} has no trades", task_id);
        }
    }
    
    println!("Concurrent updates test passed for {} strategies", num_tasks);
}

#[tokio::test]
async fn test_portfolio_wal_recovery_performance() {
    let wal_path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
    
    // Phase 1: Generate fills and write to WAL
    {
        let (event_tx, event_rx) = mpsc::channel(10000);
        let aggregator = PortfolioAggregator::new(event_rx, wal_path.to_str().unwrap());
        
        let agg_clone = Arc::clone(&aggregator);
        tokio::spawn(async move {
            agg_clone.run().await;
        });
        
        // Generate 5000 fills
        for i in 0..5000 {
            let fill = FillEvent {
                strategy_id: format!("strategy_{}", i % 10),
                symbol: "AAPL".to_string(),
                asset_class: AssetClass::Equity,
                side: "buy".to_string(),
                quantity: 100.0,
                fill_price_usd: 150.0,
                commission_usd: 1.0,
                timestamp_ns: chrono::Utc::now().timestamp_nanos(),
            };
            
            let event = AggregatorEvent::Fill(fill);
            event_tx.send(event).await.unwrap();
        }
        
        // Wait for WAL writes
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // Phase 2: Test recovery performance
    let recovery_start = Instant::now();
    
    let (event_tx, event_rx) = mpsc::channel(10000);
    let aggregator = PortfolioAggregator::new(event_rx, wal_path.to_str().unwrap());
    
    let agg_clone = Arc::clone(&aggregator);
    tokio::spawn(async move {
        agg_clone.run().await;
    });
    
    // Wait for recovery
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    let recovery_time = recovery_start.elapsed();
    
    println!("WAL Recovery Performance:");
    println!("  Recovery time: {:?}", recovery_time);
    
    // Verify recovered state
    let (tx, rx) = mpsc::channel(1);
    let query = AggregatorEvent::GetStrategyPnl("strategy_0".to_string(), tx);
    event_tx.send(query).await.unwrap();
    
    if let Some(pnl) = rx.recv().await {
        assert!(pnl.num_trades > 0, "No trades recovered for strategy_0");
        println!("Recovered {} trades for strategy_0", pnl.num_trades);
    }
    
    // Recovery should be fast (<1 second for 5000 events)
    assert!(recovery_time.as_secs() < 1, 
        "Recovery time {:?} > 1 second", recovery_time);
}

#[tokio::test]
async fn test_memory_usage_scaling() {
    let (event_tx, event_rx) = mpsc::channel(10000);
    let wal_path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
    let aggregator = PortfolioAggregator::new(event_rx, wal_path.to_str().unwrap());
    
    // Start aggregator
    let agg_clone = Arc::clone(&aggregator);
    tokio::spawn(async move {
        agg_clone.run().await;
    });
    
    // Measure memory before
    let memory_before = get_memory_usage();
    
    // Generate many unique positions
    let num_unique_positions = 10_000;
    for i in 0..num_unique_positions {
        let fill = FillEvent {
            strategy_id: format!("strategy_{}", i),
            symbol: format!("SYMBOL_{}", i),
            asset_class: AssetClass::Equity,
            side: "buy",
            quantity: 100.0,
            fill_price_usd: 150.0,
            commission_usd: 1.0,
            timestamp_ns: chrono::Utc::now().timestamp_nanos(),
        };
        
        let event = AggregatorEvent::Fill(fill);
        event_tx.send(event).await.unwrap();
    }
    
    // Wait for processing
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Measure memory after
    let memory_after = get_memory_usage();
    let memory_per_position = (memory_after - memory_before) as f64 / num_unique_positions as f64;
    
    println!("Memory Usage Scaling:");
    println!("  Positions: {}", num_unique_positions);
    println!("  Memory before: {} KB", memory_before / 1024);
    println!("  Memory after: {} KB", memory_after / 1024);
    println!("  Memory per position: {:.2} bytes", memory_per_position);
    
    // Each position should use minimal memory (<1KB)
    assert!(memory_per_position < 1024.0,
        "Memory per position {:.2} > 1KB", memory_per_position);
}

#[cfg(unix)]
fn get_memory_usage() -> usize {
    use std::fs;
    let status = fs::read_to_string("/proc/self/status").unwrap();
    for line in status.lines() {
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            return parts[1].parse::<usize>().unwrap() * 1024; // Convert KB to bytes
        }
    }
    0
}

#[cfg(not(unix))]
fn get_memory_usage() -> usize {
    // Placeholder for non-Unix systems
    0
}
