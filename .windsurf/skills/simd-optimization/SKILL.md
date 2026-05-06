# SIMD Optimization Skill

---

name: simd-optimization
description: Optimizes hot-path code using SIMD instructions for performance. Use when profiling shows CPU bottlenecks in tight loops or when latency targets (<200μs) are not being met.

---

## When to Activate

Use when:
- Profiling shows CPU bottlenecks in tight loops
- Latency targets (<200μs) are not being met
- Performance Benchmarker identifies hot paths needing optimization
- Processing large arrays of data (vectors, matrices)
- Implementing mathematical operations (dot products, convolutions)

## Core Principles

### Profile Before Optimize
NEVER optimize without profiling data. Use criterion, perf, or flamegraph to identify actual bottlenecks.

### SIMD Basics
SIMD (Single Instruction, Multiple Data) processes multiple data points in parallel:
- **AVX2**: 256-bit registers (8 floats, 4 doubles)
- **AVX-512**: 512-bit registers (16 floats, 8 doubles)
- **NEON**: ARM SIMD (for Apple Silicon)

### Rust SIMD Support
Rust has excellent SIMD support via:
- `std::simd` module (nightly)
- `packed_simd` crate (stable)
- `wide` crate (portable SIMD)
- Auto-vectorization by compiler

## Implementation Checklist

- [ ] Profile to identify hot path
- [ ] Verify bottleneck is CPU-bound
- [ ] Check if SIMD is applicable
- [ ] Use portable SIMD crates (packed_simd, wide)
- [ ] Benchmark before and after optimization
- [ ] Verify correctness with property-based tests
- [ ] Document SIMD assumptions

## Common SIMD Patterns

### Vector Addition
```rust
// Scalar version (slow)
fn add_scalar(a: &[f32], b: &[f32], out: &mut [f32]) {
    for i in 0..a.len() {
        out[i] = a[i] + b[i];
    }
}

// SIMD version (fast)
use packed_simd::f32x8;

fn add_simd(a: &[f32], b: &[f32], out: &mut [f32]) {
    assert!(a.len() % 8 == 0);
    
    for i in (0..a.len()).step_by(8) {
        let a_vec = f32x8::from_slice(&a[i..i+8]);
        let b_vec = f32x8::from_slice(&b[i..i+8]);
        let result = a_vec + b_vec;
        result.write_to_slice(&mut out[i..i+8]);
    }
}
```

### Dot Product
```rust
// Scalar version
fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

// SIMD version
fn dot_product_simd(a: &[f32], b: &[f32]) -> f32 {
    assert!(a.len() % 8 == 0);
    
    let mut sum = f32x8::splat(0.0);
    
    for i in (0..a.len()).step_by(8) {
        let a_vec = f32x8::from_slice(&a[i..i+8]);
        let b_vec = f32x8::from_slice(&b[i..i+8]);
        sum = sum + a_vec * b_vec;
    }
    
    sum.reduce_sum()
}
```

### TraderX-Specific: BAM Grid Processing
```rust
use packed_simd::u8x16;

// Process BAM grid in chunks of 16 cells
fn process_bam_grid_simd(grid: &[u8], weights: &[f32]) -> f32 {
    assert!(grid.len() % 16 == 0);
    assert!(weights.len() % 16 == 0);
    
    let mut score = 0.0;
    
    for i in (0..grid.len()).step_by(16) {
        let grid_vec = u8x16::from_slice(&grid[i..i+16]);
        let weight_vec = f32x8::from_slice(&weights[i..i+8]);
        
        // Convert u8 to f32 and multiply
        let grid_f32 = f32x8::from(grid_vec);
        let product = grid_f32 * weight_vec;
        score += product.reduce_sum();
    }
    
    score
}
```

## Common Pitfalls

- ❌ Optimizing without profiling → ALWAYS profile first
- ❌ Optimizing cold paths → only optimize hot paths
- ❌ Using non-portable intrinsics → use portable SIMD crates
- ❌ Not benchmarking → always benchmark before/after
- ❌ Breaking correctness → verify with tests

## Verification

After SIMD optimization:
- [ ] Profiling showed CPU bottleneck
- [ ] SIMD implementation correct (verified by tests)
- [ ] Benchmark shows improvement (>2x speedup expected)
- [ ] Code is portable (works on x86_64 and ARM)
- [ ] No correctness regressions
- [ ] Documentation explains SIMD assumptions
