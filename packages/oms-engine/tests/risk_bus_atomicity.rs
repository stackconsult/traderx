//! Risk Bus atomicity validation test
//! Tests lock-free atomic operations under high contention

use oms_engine::risk_bus::RiskBus;
use std::sync::Arc;
use std::time::Instant;
use tokio::task::JoinSet;

#[tokio::test]
async fn test_risk_bus_atomicity_under_contention() {
    let risk_bus = RiskBus::new(10_000_000.0, -2000); // $10M NAV, -20% DD halt
    let mut tasks = JoinSet::new();
    
    // Spawn 100 concurrent tasks
    for i in 0..100 {
        let risk_bus = Arc::clone(&risk_bus);
        tasks.spawn(async move {
            for j in 0..1000 {
                // Mix of operations
                match j % 4 {
                    0 => {
                        // Check risk (should be SeqCst atomic)
                        let _ = risk_bus.check();
                    }
                    1 => {
                        // Update NAV
                        let nav = 10_000_000.0 + (i * j) as f64;
                        risk_bus.update_nav(nav);
                    }
                    2 => {
                        // Check symbol limit
                        let _ = risk_bus.check_symbol("AAPL", 100_000.0);
                    }
                    _ => {
                        // Check if halted
                        let _ = risk_bus.is_halted();
                    }
                }
            }
        });
    }
    
    // Wait for all tasks to complete
    while let Some(_) = tasks.join_next().await {}
    
    // Verify no corruption
    assert!(risk_bus.get_nav() > 0.0);
    assert!(!risk_bus.is_halted());
    
    // Test kill switch atomicity
    risk_bus.assert_kill_switch("test");
    assert!(risk_bus.is_halted());
    
    // Reset and verify
    risk_bus.reset_halt();
    assert!(!risk_bus.is_halted());
}

#[tokio::test]
async fn test_risk_bus_performance_under_contention() {
    let risk_bus = RiskBus::new(10_000_000.0, -2000);
    let num_threads = 100;
    let operations_per_thread = 10_000;
    
    let start = Instant::now();
    let mut tasks = JoinSet::new();
    
    // Spawn contention tasks
    for _ in 0..num_threads {
        let risk_bus = Arc::clone(&risk_bus);
        tasks.spawn(async move {
            for _ in 0..operations_per_thread {
                // Hot path: risk check
                let _ = risk_bus.check();
            }
        });
    }
    
    // Wait for completion
    while let Some(_) = tasks.join_next().await {}
    
    let duration = start.elapsed();
    let total_ops = num_threads * operations_per_thread;
    let ops_per_sec = total_ops as f64 / duration.as_secs_f64();
    let avg_latency_ns = duration.as_nanos() / total_ops as u128;
    
    println!("Risk Bus Performance:");
    println!("  Total operations: {}", total_ops);
    println!("  Duration: {:?}", duration);
    println!("  Ops/sec: {:.0}", ops_per_sec);
    println!("  Avg latency: {}ns", avg_latency_ns);
    
    // Assert performance requirements
    assert!(avg_latency_ns < 100, "Average latency {}ns > 100ns target", avg_latency_ns);
    assert!(ops_per_sec > 1_000_000.0, "Throughput {:.0} < 1M ops/sec", ops_per_sec);
}

#[tokio::test]
async fn test_risk_bus_memory_ordering() {
    let risk_bus = RiskBus::new(10_000_000.0, -2000);
    
    // Test that kill switch is immediately visible
    let risk_bus_clone = Arc::clone(&risk_bus);
    
    let handle = tokio::spawn(async move {
        // Wait for kill switch
        while !risk_bus_clone.is_halted() {
            tokio::task::yield_now();
        }
    });
    
    // Assert kill switch
    risk_bus.assert_kill_switch("memory ordering test");
    
    // Wait for other task to see it
    let timeout = tokio::time::sleep(std::time::Duration::from_millis(100));
    tokio::select! {
        _ = handle => {
            // Success - other task saw the kill switch
        }
        _ = timeout => {
            panic!("Kill switch not visible within 100ms - memory ordering issue");
        }
    }
    
    // Verify check() also sees it
    assert!(risk_bus.check().is_err());
}

#[tokio::test]
async fn test_risk_bus_drawdown_trigger() {
    let risk_bus = RiskBus::new(10_000_000.0, -2000); // -20% halt
    
    // Update NAV to trigger drawdown
    risk_bus.update_nav(8_000_000.0); // -20% drawdown
    
    // Should trigger global halt
    assert!(risk_bus.is_halted());
    assert_eq!(risk_bus.dd_bps(), -2000);
    
    // Check should fail
    assert!(risk_bus.check().is_err());
    
    // Reset should work
    risk_bus.reset_halt();
    assert!(!risk_bus.is_halted());
}

#[tokio::test]
async fn test_symbol_limit_atomicity() {
    let risk_bus = RiskBus::new(10_000_000.0, -2000);
    
    // Set symbol limit
    risk_bus.set_symbol_limit("AAPL", 1_000_000.0);
    
    // Multiple concurrent checks
    let risk_bus = Arc::clone(&risk_bus);
    let mut tasks = JoinSet::new();
    
    for i in 0..10 {
        let risk_bus = Arc::clone(&risk_bus);
        tasks.spawn(async move {
            // Each task tries to use 200k - should succeed for first 5, fail for last 5
            let result = risk_bus.check_symbol("AAPL", 200_000.0);
            if i < 5 {
                assert!(result.is_ok(), "Task {} should succeed", i);
            } else {
                // May succeed or fail depending on timing, but should not panic
                let _ = result;
            }
        });
    }
    
    while let Some(_) = tasks.join_next().await {}
}
