// Benchmark Harness
// Phase 0: Foundation - Performance benchmarking infrastructure

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::{Duration, Instant};

/// Benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub warmup_iterations: usize,
    pub measurement_iterations: usize,
    pub sample_size: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            warmup_iterations: 100,
            measurement_iterations: 1000,
            sample_size: 100,
        }
    }
}

/// Benchmark result
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub mean_ns: f64,
    pub stddev_ns: f64,
    pub min_ns: f64,
    pub max_ns: f64,
    pub iterations: usize,
}

impl BenchmarkResult {
    pub fn new(name: String, samples: Vec<Duration>) -> Self {
        let ns_samples: Vec<f64> = samples.iter().map(|d| d.as_nanos() as f64).collect();
        let mean = ns_samples.iter().sum::<f64>() / ns_samples.len() as f64;
        let variance = ns_samples.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / ns_samples.len() as f64;
        let stddev = variance.sqrt();
        let min = ns_samples.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = ns_samples.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        
        Self {
            name,
            mean_ns: mean,
            stddev_ns: stddev,
            min_ns: min,
            max_ns: max,
            iterations: samples.len(),
        }
    }
    
    /// Check if benchmark meets target latency
    pub fn meets_latency_target(&self, target_ns: f64) -> bool {
        self.mean_ns <= target_ns
    }
}

/// Benchmark harness
pub struct BenchmarkHarness {
    config: BenchmarkConfig,
}

impl BenchmarkHarness {
    /// Create a new benchmark harness
    pub fn new(config: BenchmarkConfig) -> Self {
        Self { config }
    }
    
    /// Run a benchmark
    pub fn run_benchmark<F>(&self, name: &str, mut f: F) -> BenchmarkResult
    where
        F: FnMut() -> Duration,
    {
        // Warmup
        for _ in 0..self.config.warmup_iterations {
            let _ = f();
        }
        
        // Measurement
        let mut samples = Vec::with_capacity(self.config.sample_size);
        for _ in 0..self.config.measurement_iterations {
            samples.push(f());
        }
        
        BenchmarkResult::new(name.to_string(), samples)
    }
    
    /// Run multiple benchmarks and compare
    pub fn run_comparison<F>(&self, benchmarks: Vec<(&str, F)>) -> Vec<BenchmarkResult>
    where
        F: FnMut() -> Duration,
    {
        benchmarks
            .into_iter()
            .map(|(name, f)| self.run_benchmark(name, f))
            .collect()
    }
}

impl Default for BenchmarkHarness {
    fn default() -> Self {
        Self::new(BenchmarkConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_benchmark_harness() {
        let harness = BenchmarkHarness::default();
        
        let result = harness.run_benchmark("test_operation", || {
            std::thread::sleep(Duration::from_nanos(100));
            Duration::from_nanos(100)
        });
        
        assert_eq!(result.name, "test_operation");
        assert!(result.iterations > 0);
        assert!(result.mean_ns >= 100.0);
    }
    
    #[test]
    fn test_latency_target() {
        let result = BenchmarkResult {
            name: "test".to_string(),
            mean_ns: 50.0,
            stddev_ns: 10.0,
            min_ns: 40.0,
            max_ns: 60.0,
            iterations: 100,
        };
        
        assert!(result.meets_latency_target(100.0));
        assert!(!result.meets_latency_target(25.0));
    }
}
