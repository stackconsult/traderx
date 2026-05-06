// Monitoring Baseline
// Phase 0: Foundation - Monitoring infrastructure

use prometheus::{Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramVec, Registry};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Monitoring configuration
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    pub enable_prometheus: bool,
    pub prometheus_port: u16,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_prometheus: true,
            prometheus_port: 9090,
        }
    }
}

/// Monitoring metrics
pub struct MonitoringMetrics {
    // Counters
    pub ticks_ingested: Counter,
    pub ticks_processed: Counter,
    pub ticks_dropped: Counter,
    pub batches_processed: Counter,
    pub model_predictions: Counter,

    // Gauges
    pub active_connections: Gauge,
    pub memory_usage_bytes: Gauge,
    pub cache_size: Gauge,
    pub queue_length: Gauge,

    // Histograms
    pub tick_latency: Histogram,
    pub batch_processing_time: Histogram,
    pub model_inference_time: Histogram,
}

impl MonitoringMetrics {
    /// Create new monitoring metrics
    pub fn new() -> Self {
        Self {
            ticks_ingested: Counter::new("ticks_ingested_total", "Total ticks ingested").unwrap(),
            ticks_processed: Counter::new("ticks_processed_total", "Total ticks processed")
                .unwrap(),
            ticks_dropped: Counter::new("ticks_dropped_total", "Total ticks dropped").unwrap(),
            batches_processed: Counter::new("batches_processed_total", "Total batches processed")
                .unwrap(),
            model_predictions: Counter::new("model_predictions_total", "Total model predictions")
                .unwrap(),

            active_connections: Gauge::new("active_connections", "Active connections").unwrap(),
            memory_usage_bytes: Gauge::new("memory_usage_bytes", "Memory usage in bytes").unwrap(),
            cache_size: Gauge::new("cache_size", "Cache size").unwrap(),
            queue_length: Gauge::new("queue_length", "Queue length").unwrap(),

            tick_latency: Histogram::with_opts(
                prometheus::HistogramOpts::new("tick_latency", "Tick latency in seconds")
                    .buckets(vec![0.001, 0.01, 0.1, 1.0, 10.0]),
            )
            .unwrap(),
            batch_processing_time: Histogram::with_opts(
                prometheus::HistogramOpts::new(
                    "batch_processing_time",
                    "Batch processing time in seconds",
                )
                .buckets(vec![0.01, 0.1, 1.0, 10.0, 100.0]),
            )
            .unwrap(),
            model_inference_time: Histogram::with_opts(
                prometheus::HistogramOpts::new(
                    "model_inference_time",
                    "Model inference time in seconds",
                )
                .buckets(vec![0.001, 0.01, 0.1, 1.0, 10.0]),
            )
            .unwrap(),
        }
    }

    /// Register metrics with Prometheus registry
    pub fn register(&self, registry: &Registry) -> Result<(), Box<dyn std::error::Error>> {
        registry.register(Box::new(self.ticks_ingested.clone()))?;
        registry.register(Box::new(self.ticks_processed.clone()))?;
        registry.register(Box::new(self.ticks_dropped.clone()))?;
        registry.register(Box::new(self.batches_processed.clone()))?;
        registry.register(Box::new(self.model_predictions.clone()))?;
        registry.register(Box::new(self.active_connections.clone()))?;
        registry.register(Box::new(self.memory_usage_bytes.clone()))?;
        registry.register(Box::new(self.cache_size.clone()))?;
        registry.register(Box::new(self.queue_length.clone()))?;
        registry.register(Box::new(self.tick_latency.clone()))?;
        registry.register(Box::new(self.batch_processing_time.clone()))?;
        registry.register(Box::new(self.model_inference_time.clone()))?;
        Ok(())
    }
}

/// Monitoring baseline
pub struct MonitoringBaseline {
    config: MonitoringConfig,
    metrics: MonitoringMetrics,
    registry: Registry,
    start_time: std::time::Instant,
}

impl MonitoringBaseline {
    /// Create a new monitoring baseline
    pub fn new(config: MonitoringConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let metrics = MonitoringMetrics::new();
        let registry = Registry::new();
        metrics.register(&registry)?;

        Ok(Self {
            config,
            metrics,
            registry,
            start_time: std::time::Instant::now(),
        })
    }

    /// Get metrics
    pub fn get_metrics(&self) -> &MonitoringMetrics {
        &self.metrics
    }

    /// Get Prometheus registry
    pub fn get_registry(&self) -> &Registry {
        &self.registry
    }

    /// Get uptime in seconds
    pub fn uptime_secs(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    /// Record tick ingestion
    pub fn record_tick_ingested(&self) {
        self.metrics.ticks_ingested.inc();
    }

    /// Record tick processing
    pub fn record_tick_processed(&self) {
        self.metrics.ticks_processed.inc();
    }

    /// Record tick drop
    pub fn record_tick_dropped(&self) {
        self.metrics.ticks_dropped.inc();
    }

    /// Record batch processing
    pub fn record_batch_processed(&self) {
        self.metrics.batches_processed.inc();
    }

    /// Record model prediction
    pub fn record_model_prediction(&self) {
        self.metrics.model_predictions.inc();
    }

    /// Update active connections
    pub fn update_active_connections(&self, count: i64) {
        self.metrics.active_connections.set(count as f64);
    }

    /// Update memory usage
    pub fn update_memory_usage(&self, bytes: i64) {
        self.metrics.memory_usage_bytes.set(bytes as f64);
    }

    /// Update cache size
    pub fn update_cache_size(&self, size: i64) {
        self.metrics.cache_size.set(size as f64);
    }

    /// Update queue length
    pub fn update_queue_length(&self, length: i64) {
        self.metrics.queue_length.set(length as f64);
    }

    /// Observe tick latency
    pub fn observe_tick_latency(&self, seconds: f64) {
        self.metrics.tick_latency.observe(seconds);
    }

    /// Observe batch processing time
    pub fn observe_batch_processing_time(&self, seconds: f64) {
        self.metrics.batch_processing_time.observe(seconds);
    }

    /// Observe model inference time
    pub fn observe_model_inference_time(&self, seconds: f64) {
        self.metrics.model_inference_time.observe(seconds);
    }
}

impl Default for MonitoringBaseline {
    fn default() -> Self {
        Self::new(MonitoringConfig::default()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitoring_baseline() {
        let monitoring = MonitoringBaseline::default();

        monitoring.record_tick_ingested();
        monitoring.record_tick_processed();
        monitoring.record_tick_dropped();
        monitoring.record_batch_processed();
        monitoring.record_model_prediction();

        assert!(monitoring.uptime_secs() >= 0.0);
    }
}
