//! Prometheus metrics for model serving.

use prometheus::{Counter, Histogram, IntCounter, IntGauge, Registry};
use std::sync::OnceLock;

pub struct ModelMetrics {
    pub inference_requests_total: IntCounter,
    pub inference_latency_ns: Histogram,
    pub model_loads_total: IntCounter,
    pub active_models: IntGauge,
    pub cache_hits_total: IntCounter,
    pub cache_misses_total: IntCounter,
    pub feature_fetch_latency_ns: Histogram,
    pub compression_ratio: Histogram,
}

impl ModelMetrics {
    pub fn new() -> Self {
        Self {
            inference_requests_total: IntCounter::new(
                "model_inference_requests_total",
                "Total number of inference requests"
            ).unwrap(),
            inference_latency_ns: Histogram::with_opts(
                prometheus::HistogramOpts::new(
                    "model_inference_latency_ns",
                    "Inference latency in nanoseconds"
                ).buckets(vec![
                    100_000.0, 500_000.0, 1_000_000.0, 5_000_000.0, 10_000_000.0
                ])
            ).unwrap(),
            model_loads_total: IntCounter::new(
                "model_loads_total",
                "Total number of model loads"
            ).unwrap(),
            active_models: IntGauge::new(
                "model_active_models",
                "Number of active loaded models"
            ).unwrap(),
            cache_hits_total: IntCounter::new(
                "model_cache_hits_total",
                "Total number of cache hits"
            ).unwrap(),
            cache_misses_total: IntCounter::new(
                "model_cache_misses_total",
                "Total number of cache misses"
            ).unwrap(),
            feature_fetch_latency_ns: Histogram::with_opts(
                prometheus::HistogramOpts::new(
                    "model_feature_fetch_latency_ns",
                    "Feature fetch latency in nanoseconds"
                ).buckets(vec![
                    10_000.0, 50_000.0, 100_000.0, 500_000.0, 1_000_000.0
                ])
            ).unwrap(),
            compression_ratio: Histogram::with_opts(
                prometheus::HistogramOpts::new(
                    "model_compression_ratio",
                    "TurboQuant compression ratio"
                ).buckets(vec![0.1, 0.2, 0.5, 0.8, 1.0])
            ).unwrap(),
        }
    }

    pub fn record_inference(&self, latency_ns: u64) {
        self.inference_requests_total.inc();
        self.inference_latency_ns.observe(latency_ns as f64);
    }

    pub fn record_model_load(&self) {
        self.model_loads_total.inc();
    }

    pub fn set_active_models(&self, count: i64) {
        self.active_models.set(count);
    }

    pub fn record_cache_hit(&self) {
        self.cache_hits_total.inc();
    }

    pub fn record_cache_miss(&self) {
        self.cache_misses_total.inc();
    }

    pub fn record_feature_fetch(&self, latency_ns: u64) {
        self.feature_fetch_latency_ns.observe(latency_ns as f64);
    }

    pub fn record_compression(&self, ratio: f64) {
        self.compression_ratio.observe(ratio);
    }
}

pub static GLOBAL_METRICS: OnceLock<ModelMetrics> = OnceLock::new();

pub fn metrics() -> &'static ModelMetrics {
    GLOBAL_METRICS.get_or_init(ModelMetrics::new)
}
