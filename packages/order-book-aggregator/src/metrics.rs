//! Prometheus metrics for the order book aggregator.

use prometheus::{Counter, Histogram, IntCounter, Registry};
use std::sync::OnceLock;

pub struct BookMetrics {
    pub updates_total: IntCounter,
    pub snapshots_total: IntCounter,
    pub update_latency_ns: Histogram,
    pub active_books: IntCounter,
}

impl BookMetrics {
    pub fn new() -> Self {
        Self {
            updates_total: IntCounter::new(
                "orderbook_updates_total",
                "Total number of order book updates"
            ).unwrap(),
            snapshots_total: IntCounter::new(
                "orderbook_snapshots_total",
                "Total number of order book snapshots"
            ).unwrap(),
            update_latency_ns: Histogram::with_opts(
                prometheus::HistogramOpts::new(
                    "orderbook_update_latency_ns",
                    "Order book update latency in nanoseconds"
                ).buckets(vec![
                    100.0, 500.0, 1000.0, 5000.0, 10_000.0, 50_000.0, 100_000.0, 500_000.0
                ])
            ).unwrap(),
            active_books: IntCounter::new(
                "orderbook_active_books",
                "Number of active order books"
            ).unwrap(),
        }
    }

    pub fn increment_updates(&self) {
        self.updates_total.inc();
    }

    pub fn increment_snapshots(&self) {
        self.snapshots_total.inc();
    }

    pub fn record_update_latency(&self, latency_ns: f64) {
        self.update_latency_ns.observe(latency_ns);
    }

    pub fn updates_per_second(&self) -> f64 {
        // Simple rate calculation - in production use Prometheus client
        self.updates_total.get() as f64 / 60.0 // Rough estimate
    }

    pub fn avg_latency_ns(&self) -> f64 {
        self.update_latency_ns.sample_sum() / self.update_latency_ns.sample_count()
    }

    pub fn register(&self, registry: &Registry) -> anyhow::Result<()> {
        registry.register(Box::new(self.updates_total.clone()))?;
        registry.register(Box::new(self.snapshots_total.clone()))?;
        registry.register(Box::new(self.update_latency_ns.clone()))?;
        registry.register(Box::new(self.active_books.clone()))?;
        Ok(())
    }
}

pub static GLOBAL_METRICS: OnceLock<BookMetrics> = OnceLock::new();

pub fn metrics() -> &'static BookMetrics {
    GLOBAL_METRICS.get_or_init(BookMetrics::new)
}
