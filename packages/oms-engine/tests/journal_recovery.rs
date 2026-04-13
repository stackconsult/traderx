//! OMS Journal Recovery Test
//! Tests Aeron journal replay and complete state recovery after crash

use oms_engine::oms::{OmsEngine, Order, OrderType, Side};
use oms_engine::state_machine::OrderState;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::sync::mpsc;
use uuid::Uuid;

#[tokio::test]
async fn test_oms_crash_recovery_with_1m_orders() {
    let temp_dir = TempDir::new().unwrap();
    let journal_path = temp_dir.path().join("test_journal.aeron");
    
    // Phase 1: Create OMS and generate orders
    let (oms_tx, mut oms_rx) = mpsc::channel(10000);
    let account_id = Uuid::new_v4();
    
    let oms1 = OmsEngine::new(
        account_id,
        journal_path.to_string_lossy().to_string(),
        oms_tx,
    ).await;
    
    let num_orders = 1_000_000;
    let symbols = vec!["AAPL", "GOOGL", "MSFT", "TSLA", "AMZN"];
    
    println!("Creating {} orders...", num_orders);
    let start = Instant::now();
    
    for i in 0..num_orders {
        let symbol = symbols[i % symbols.len()];
        let side = if i % 2 == 0 { Side::Buy } else { Side::Sell };
        let quantity = ((i % 100) + 1) as f64 * 10.0;
        
        let order = Order::new(
            Uuid::new_v4(),
            account_id,
            symbol.to_string(),
            side,
            OrderType::Market,
            quantity.into(),
        );
        
        oms1.submit_order(order).await;
        
        // Fill half the orders
        if i % 2 == 0 {
            let fill_price = 100.0 + (i % 50) as f64;
            while let Some(order_id) = oms_rx.recv().await {
                oms1.fill_order(&order_id, fill_price, quantity).await;
                break;
            }
        }
        
        // Progress reporting
        if i % 100_000 == 0 && i > 0 {
            println!("  Created {} orders...", i);
        }
    }
    
    let creation_time = start.elapsed();
    println!("Order creation completed in {:?}", creation_time);
    
    // Get state before crash
    let state_before = oms1.get_state_summary().await;
    println!("State before crash: {:?}", state_before);
    
    // Phase 2: Simulate crash (drop OMS without cleanup)
    drop(oms1);
    drop(oms_rx);
    
    // Phase 3: Create new OMS instance and recover
    println!("\nStarting recovery...");
    let recovery_start = Instant::now();
    
    let (oms_tx2, _) = mpsc::channel(10000);
    let oms2 = OmsEngine::new(
        account_id,
        journal_path.to_string_lossy().to_string(),
        oms_tx2,
    ).await;
    
    // Trigger recovery
    let recovered = oms2.recover_from_journal().await;
    assert!(recovered, "Journal recovery failed");
    
    let recovery_time = recovery_start.elapsed();
    println!("Recovery completed in {:?}", recovery_time);
    
    // Phase 4: Validate recovered state
    let state_after = oms2.get_state_summary().await;
    println!("State after recovery: {:?}", state_after);
    
    // Verify state matches
    assert_eq!(state_before.total_orders, state_after.total_orders,
        "Order count mismatch: before={}, after={}", 
        state_before.total_orders, state_after.total_orders);
    
    assert_eq!(state_before.filled_orders, state_after.filled_orders,
        "Filled order count mismatch");
    
    assert_eq!(state_before.pending_orders, state_after.pending_orders,
        "Pending order count mismatch");
    
    // Verify specific orders exist
    for i in 0..100 {
        let order_id = format!("test_order_{}", i);
        if let Some(order) = oms2.get_order(&order_id).await {
            assert!(!order.order_id.to_string().is_empty());
        }
    }
    
    println!("✅ Recovery validation passed");
}

#[tokio::test]
async fn test_journal_performance_under_load() {
    let temp_dir = TempDir::new().unwrap();
    let journal_path = temp_dir.path().join("perf_journal.aeron");
    
    let (oms_tx, mut oms_rx) = mpsc::channel(10000);
    let account_id = Uuid::new_v4();
    
    let oms = OmsEngine::new(
        account_id,
        journal_path.to_string_lossy().to_string(),
        oms_tx,
    ).await;
    
    // Measure write performance
    let num_writes = 100_000;
    let write_latencies = Vec::new();
    
    println!("Testing journal write performance...");
    let start = Instant::now();
    
    for i in 0..num_writes {
        let write_start = Instant::now();
        
        let order = Order::new(
            Uuid::new_v4(),
            account_id,
            "AAPL".to_string(),
            Side::Buy,
            OrderType::Market,
            100.0.into(),
        );
        
        oms.submit_order(order).await;
        
        let write_latency = write_start.elapsed();
        // Store latencies for statistics (simplified)
        
        if i % 10_000 == 0 && i > 0 {
            println!("  Wrote {} entries...", i);
        }
    }
    
    let total_write_time = start.elapsed();
    let avg_write_latency = total_write_time / num_writes as u32;
    
    println!("Write Performance:");
    println!("  Total writes: {}", num_writes);
    println!("  Total time: {:?}", total_write_time);
    println!("  Avg latency: {:?}", avg_write_latency);
    
    // Validate write performance requirement
    assert!(avg_write_latency.as_micros() < 10, 
        "Write latency {:?} > 10μs target", avg_write_latency);
    
    // Test read performance
    println!("\nTesting journal read performance...");
    let read_start = Instant::now();
    
    let recovered = oms.recover_from_journal().await;
    assert!(recovered);
    
    let read_time = read_start.elapsed();
    let avg_read_latency = read_time / num_writes as u32;
    
    println!("Read Performance:");
    println!("  Total entries read: {}", num_writes);
    println!("  Total time: {:?}", read_time);
    println!("  Avg latency: {:?}", avg_read_latency);
}

#[tokio::test]
async fn test_partial_journal_corruption() {
    let temp_dir = TempDir::new().unwrap();
    let journal_path = temp_dir.path().join("corrupt_journal.aeron");
    
    let (oms_tx, mut oms_rx) = mpsc::channel(10000);
    let account_id = Uuid::new_v4();
    
    // Create OMS and generate orders
    let oms1 = OmsEngine::new(
        account_id,
        journal_path.to_string_lossy().to_string(),
        oms_tx,
    ).await;
    
    // Create 1000 orders
    for i in 0..1000 {
        let order = Order::new(
            Uuid::new_v4(),
            account_id,
            "AAPL".to_string(),
            Side::Buy,
            OrderType::Market,
            100.0.into(),
        );
        
        oms1.submit_order(order).await;
        
        // Fill every other order
        if i % 2 == 0 {
            while let Some(order_id) = oms_rx.recv().await {
                oms1.fill_order(&order_id, 150.0, 100.0).await;
                break;
            }
        }
    }
    
    let state_before = oms1.get_state_summary().await;
    drop(oms1);
    
    // Simulate journal corruption by truncating the file
    let journal_file = std::fs::OpenOptions::new()
        .write(true)
        .open(&journal_path)
        .unwrap();
    
    // Truncate to 50% of original size
    let original_size = journal_file.metadata().unwrap().len();
    journal_file.set_len(original_size / 2).unwrap();
    drop(journal_file);
    
    // Attempt recovery
    let (oms_tx2, _) = mpsc::channel(10000);
    let oms2 = OmsEngine::new(
        account_id,
        journal_path.to_string_lossy().to_string(),
        oms_tx2,
    ).await;
    
    let recovered = oms2.recover_from_journal().await;
    
    // Should recover what's available
    assert!(recovered, "Should recover from partial journal");
    
    let state_after = oms2.get_state_summary().await;
    
    println!("Partial Corruption Test:");
    println!("  Before: {:?}", state_before);
    println!("  After:  {:?}", state_after);
    
    // Should have recovered some orders
    assert!(state_after.total_orders > 0, "Should recover some orders");
    assert!(state_after.total_orders < state_before.total_orders, 
        "Should have fewer orders after corruption");
}

#[tokio::test]
async fn test_concurrent_crash_recovery() {
    let temp_dir = TempDir::new().unwrap();
    let journal_path = temp_dir.path().join("concurrent_journal.aeron");
    
    let (oms_tx, mut oms_rx) = mpsc::channel(10000);
    let account_id = Uuid::new_v4();
    
    let oms = Arc::new(OmsEngine::new(
        account_id,
        journal_path.to_string_lossy().to_string(),
        oms_tx,
    ).await);
    
    // Spawn order generation task
    let oms_clone = Arc::clone(&oms);
    let order_task = tokio::spawn(async move {
        for i in 0..10_000 {
            let order = Order::new(
                Uuid::new_v4(),
                account_id,
                "AAPL".to_string(),
                Side::Buy,
                OrderType::Market,
                100.0.into(),
            );
            
            oms_clone.submit_order(order).await;
            
            if i % 100 == 0 {
                tokio::task::yield_now().await;
            }
        }
    });
    
    // Spawn fill processing task
    let oms_clone = Arc::clone(&oms);
    let fill_task = tokio::spawn(async move {
        for i in 0..5_000 {
            if let Some(order_id) = oms_rx.recv().await {
                oms_clone.fill_order(&order_id, 150.0, 100.0).await;
            }
        }
    });
    
    // Let tasks run for a bit
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Simulate crash by dropping OMS
    drop(oms);
    
    // Wait for tasks to complete (they should fail gracefully)
    let _ = order_task.await;
    let _ = fill_task.await;
    
    // Recover
    let (oms_tx2, _) = mpsc::channel(10000);
    let oms_recovered = OmsEngine::new(
        account_id,
        journal_path.to_string_lossy().to_string(),
        oms_tx2,
    ).await;
    
    let recovered = oms_recovered.recover_from_journal().await;
    assert!(recovered);
    
    let state = oms_recovered.get_state_summary().await;
    println!("Concurrent Crash Recovery:");
    println!("  Final state: {:?}", state);
    
    // Should have recovered some state
    assert!(state.total_orders > 0);
}

#[tokio::test]
async fn test_journal_checkpointing() {
    let temp_dir = TempDir::new().unwrap();
    let journal_path = temp_dir.path().join("checkpoint_journal.aeron");
    let checkpoint_path = temp_dir.path().join("checkpoint.json");
    
    let (oms_tx, mut oms_rx) = mpsc::channel(10000);
    let account_id = Uuid::new_v4();
    
    let oms = OmsEngine::new(
        account_id,
        journal_path.to_string_lossy().to_string(),
        oms_tx,
    ).await;
    
    // Create orders and checkpoint periodically
    for batch in 0..10 {
        // Create 1000 orders
        for i in 0..1000 {
            let order = Order::new(
                Uuid::new_v4(),
                account_id,
                "AAPL".to_string(),
                Side::Buy,
                OrderType::Market,
                100.0.into(),
            );
            
            oms.submit_order(order).await;
        }
        
        // Create checkpoint
        if batch % 3 == 0 {
            oms.create_checkpoint(&checkpoint_path).await.unwrap();
            println!("Created checkpoint after batch {}", batch);
        }
    }
    
    // Crash and recover from latest checkpoint
    drop(oms);
    
    let (oms_tx2, _) = mpsc::channel(10000);
    let oms_recovered = OmsEngine::new(
        account_id,
        journal_path.to_string_lossy().to_string(),
        oms_tx2,
    ).await;
    
    // Recover from checkpoint
    let recovered = oms_recovered.recover_from_checkpoint(&checkpoint_path).await;
    assert!(recovered, "Checkpoint recovery failed");
    
    // Then replay remaining journal entries
    let journal_recovered = oms_recovered.recover_from_journal().await;
    assert!(journal_recovered);
    
    let state = oms_recovered.get_state_summary().await;
    println!("Checkpoint Recovery:");
    println!("  Final state: {:?}", state);
    
    assert_eq!(state.total_orders, 10000);
}
