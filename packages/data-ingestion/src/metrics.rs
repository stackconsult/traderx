use prometheus::{register_counter, register_gauge, Counter, Gauge};
use std::sync::OnceLock;

pub struct IngestionMetrics {
    pub ticks_written: Counter,
    pub bars_written:  Counter,
    pub flush_errors:  Counter,
    pub channel_depth: Gauge,
    pub write_latency_ns: Gauge,
}

static METRICS: OnceLock<IngestionMetrics> = OnceLock::new();

pub fn metrics() -> &'static IngestionMetrics {
    METRICS.get_or_init(|| IngestionMetrics {
        ticks_written: register_counter!(
            "traderx_ingestion_ticks_total",
            "Total ticks written to QuestDB"
        ).unwrap(),
        bars_written: register_counter!(
            "traderx_ingestion_bars_total",
            "Total OHLCV bars written to QuestDB"
        ).unwrap(),
        flush_errors: register_counter!(
            "traderx_ingestion_flush_errors_total",
            "Number of ILP flush errors"
        ).unwrap(),
        channel_depth: register_gauge!(
            "traderx_ingestion_channel_depth",
            "Current depth of the ingestion channel"
        ).unwrap(),
        write_latency_ns: register_gauge!(
            "traderx_ingestion_write_latency_ns",
            "Last ILP flush latency in nanoseconds"
        ).unwrap(),
    })
}
