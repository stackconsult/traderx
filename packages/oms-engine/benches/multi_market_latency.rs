// Multi-market Latency Benchmark
// Phase C: Comparative Testing - Task C4: Performance regression suite

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

/// Benchmark multi-market grid operations
fn benchmark_multi_market_grid(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_market_grid");
    
    group.measurement_time(Duration::from_secs(10));
    
    // Benchmark grid initialization
    group.bench_function("initialize", |b| {
        b.iter(|| {
            let _ = crate::mesh::MultiMarketGridAllocator::new();
        });
    });
    
    // Benchmark grid access
    let mut allocator = crate::mesh::MultiMarketGridAllocator::new();
    allocator.initialize(1234567890);
    
    group.bench_function("get_grid", |b| {
        b.iter(|| {
            let _ = allocator.get_grid(&crate::mesh::MarketClass::Equities);
        });
    });
}

/// Benchmark pattern matching latency
fn benchmark_pattern_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_matching");
    
    group.measurement_time(Duration::from_secs(10));
    
    let mut matcher = crate::fabric::PatternMatcher::new();
    matcher.compile_pattern(1, "0,0,eq,42;5,30,gt,10".to_string()).unwrap();
    
    let mut grid = [0u8; 600];
    grid[0] = 42;
    grid[5 * 60 + 30] = 20;
    
    group.bench_function("match_pattern", |b| {
        b.iter(|| {
            let _ = matcher.match_pattern(1, black_box(&grid));
        });
    });
}

/// Benchmark funnel index lookup
fn benchmark_funnel_index(c: &mut Criterion) {
    let mut group = c.benchmark_group("funnel_index");
    
    group.measurement_time(Duration::from_secs(10));
    
    let mut funnel = crate::fabric::FlashFunnel::new();
    funnel.add_pattern(1, "equities".to_string());
    funnel.add_pattern(2, "fx".to_string());
    
    let index = crate::fabric::FunnelIndex::new(funnel);
    
    group.bench_function("lookup", |b| {
        b.iter(|| {
            let _ = index.lookup(black_box(1));
        });
    });
}

/// Benchmark advanced model inference
fn benchmark_advanced_model(c: &mut Criterion) {
    let mut group = c.benchmark_group("advanced_model");
    
    group.measurement_time(Duration::from_secs(10));
    
    let config = crate::fabric::AdvancedModelConfig::default();
    let shm_config = crate::mesh::ShmBridgeConfig {
        shm_size: 4096,
        shm_path: "/tmp/benchmark_shm".to_string(),
    };
    
    let shm_bridge = crate::mesh::PyTorchShmBridge::new(shm_config).unwrap();
    let pipeline = crate::fabric::AdvancedModelPipeline::new(config, shm_bridge);
    
    let grid = [42u8; 600];
    
    group.bench_function("infer", |b| {
        b.iter(|| {
            let _ = pipeline.infer(black_box(&grid));
        });
    });
}

criterion_group!(
    benches,
    benchmark_multi_market_grid,
    benchmark_pattern_matching,
    benchmark_funnel_index,
    benchmark_advanced_model
);

criterion_main!(benches);
