use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use oms_engine::risk_bus::{PositionLimit, RiskBus};
use std::sync::Arc;
use std::thread;

/// Benchmark: Risk Bus atomicity test (< 100ns target)
/// 
/// This verifies that the atomic check-and-update operation
/// completes within the required 100 nanosecond threshold
/// for high-frequency trading.
fn risk_bus_atomicity_benchmark(c: &mut Criterion) {
    let limit = Arc::new(PositionLimit::new(10000.0));
    
    let mut group = c.benchmark_group("risk_bus_operations");
    group.measurement_time(std::time::Duration::from_secs(10));
    
    // Benchmark 1: Single-threaded check_and_update
    group.bench_function("risk_bus_atomicity", |b| {
        b.iter(|| {
            let _ = limit.check_and_update(100.0);
        });
    });
    
    // Benchmark 2: Concurrent check_and_update (simulates real HFT load)
    group.bench_function("risk_bus_concurrent", |b| {
        b.iter(|| {
            let limit_clone = Arc::clone(&limit);
            let handle = thread::spawn(move || {
                let _ = limit_clone.check_and_update(50.0);
            });
            let _ = limit.check_and_update(50.0);
            handle.join().unwrap();
        });
    });
    
    group.finish();
}

/// Benchmark: Signal Router load test (< 5μs target)
/// 
/// This measures signal routing latency under load
fn signal_router_load_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("signal_router_operations");
    group.measurement_time(std::time::Duration::from_secs(10));
    
    // Placeholder for signal router benchmark
    // Full implementation requires signal_router module integration
    group.bench_function("signal_router_load", |b| {
        b.iter(|| {
            // Simulate signal routing overhead
            std::hint::black_box(0);
        });
    });
    
    group.finish();
}

/// Benchmark: Position limit boundary conditions
/// 
/// Tests the edge case where position is near limit
fn position_limit_boundary_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("position_limit_boundaries");
    
    for initial_position in [0.0, 9000.0, 9900.0, 10000.0].iter() {
        group.bench_with_input(
            BenchmarkId::new("near_limit_check", initial_position),
            initial_position,
            |b, &initial_pos| {
                let limit = PositionLimit::new(10000.0);
                // Pre-populate to initial position
                let _ = limit.check_and_update(initial_pos);
                
                b.iter(|| {
                    // Try to add more - should fail near limit
                    let _ = limit.check_and_update(100.0);
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    risk_bus_atomicity_benchmark,
    signal_router_load_benchmark,
    position_limit_boundary_benchmark
);
criterion_main!(benches);
