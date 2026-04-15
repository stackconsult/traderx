//! Prometheus Metrics for Risk Bus
//! Lock-free metrics collection with <1μs overhead

use prometheus::{
    Counter, Gauge, Histogram, IntCounter, IntGauge, IntGaugeVec, Registry,
    core::{AtomicU64, GenericCounter},
    proto::MetricFamily,
    TextEncoder, Encoder,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, trace};
use axum::http::StatusCode;

/// Lock-free metrics collector for Risk Bus
pub struct RiskBusMetrics {
    // Order metrics
    pub orders_submitted: GenericCounter<AtomicU64>,
    pub orders_rejected: GenericCounter<AtomicU64>,
    pub orders_per_second: GenericCounter<AtomicU64>,
    
    // Risk check metrics
    pub risk_checks_total: GenericCounter<AtomicU64>,
    pub risk_checks_duration: Histogram,
    pub risk_check_failures: GenericCounter<AtomicU64>,
    
    // State metrics
    pub current_nav: IntGauge,
    pub current_drawdown: IntGauge,
    pub is_halted: IntGauge,
    pub position_utilization: Gauge,
    
    // Symbol-specific metrics
    pub symbol_exposure: IntGaugeVec,
    pub symbol_orders: IntCounterVec,
    
    // Registry
    registry: Registry,
}

/// Per-symbol metrics
#[derive(Debug, Clone)]
pub struct SymbolMetrics {
    pub exposure: prometheus::IntGauge,
    pub orders: prometheus::IntCounter,
}

impl RiskBusMetrics {
    /// Create new metrics instance
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();
        
        // Order metrics
        let orders_submitted = IntCounter::new(
            "riskbus_orders_submitted_total",
            "Total number of orders submitted to risk bus"
        )?;
        registry.register(Box::new(orders_submitted.clone()))?;
        
        let orders_rejected = IntCounter::new(
            "riskbus_orders_rejected_total",
            "Total number of orders rejected by risk checks"
        )?;
        registry.register(Box::new(orders_rejected.clone()))?;
        
        let orders_per_second = IntCounter::new(
            "riskbus_orders_per_second",
            "Orders processed per second"
        )?;
        registry.register(Box::new(orders_per_second.clone()))?;
        
        // Risk check metrics
        let risk_checks_total = IntCounter::new(
            "riskbus_risk_checks_total",
            "Total number of risk checks performed"
        )?;
        registry.register(Box::new(risk_checks_total.clone()))?;
        
        let risk_check_failures = IntCounter::new(
            "riskbus_risk_check_failures_total",
            "Total number of risk check failures"
        )?;
        registry.register(Box::new(risk_check_failures.clone()))?;
        
        let risk_checks_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "riskbus_risk_check_duration_seconds",
                "Time spent performing risk checks"
            ).buckets(vec![0.000001, 0.000005, 0.00001, 0.00005, 0.0001, 0.0005, 0.001])
        )?;
        registry.register(Box::new(risk_checks_duration.clone()))?;
        
        // State metrics
        let current_nav = IntGauge::new(
            "riskbus_current_nav",
            "Current portfolio NAV in basis points (×10000)"
        )?;
        registry.register(Box::new(current_nav.clone()))?;
        
        let current_drawdown = IntGauge::new(
            "riskbus_current_drawdown_bps",
            "Current drawdown in basis points"
        )?;
        registry.register(Box::new(current_drawdown.clone()))?;
        
        let is_halted = IntGauge::new(
            "riskbus_is_halted",
            "Risk bus halt status (1=halted, 0=normal)"
        )?;
        registry.register(Box::new(is_halted.clone()))?;
        
        let position_utilization = Gauge::with_opts(
            prometheus::GaugeOpts::new(
                "riskbus_position_utilization_ratio",
                "Ratio of position limits utilized"
            )
        )?;
        registry.register(Box::new(position_utilization.clone()))?;
        
        // Symbol-specific metrics
        let symbol_exposure = IntGaugeVec::new(
            prometheus::Opts::new(
                "riskbus_symbol_exposure",
                "Current exposure for symbol"
            ),
            &["symbol"]
        )?;
        registry.register(Box::new(symbol_exposure.clone()))?;
        
        let symbol_orders = IntCounterVec::new(
            prometheus::Opts::new(
                "riskbus_symbol_orders_total",
                "Total orders for symbol"
            ),
            &["symbol"]
        )?;
        registry.register(Box::new(symbol_orders.clone()))?;
        
        Ok(Self {
            orders_submitted,
            orders_rejected,
            orders_per_second,
            risk_checks_total,
            risk_checks_duration,
            risk_check_failures,
            current_nav,
            current_drawdown,
            is_halted,
            position_utilization,
            symbol_exposure,
            symbol_orders,
            registry,
        })
    }
    
    /// Record order submission - lock-free operation
    #[inline]
    pub fn record_order_submitted(&self) {
        self.orders_submitted.inc();
        self.orders_per_second.inc();
        trace!("Order submitted metric updated");
    }
    
    /// Record order rejection - lock-free operation
    #[inline]
    pub fn record_order_rejected(&self) {
        self.orders_rejected.inc();
        trace!("Order rejected metric updated");
    }
    
    /// Start timing a risk check - returns timer
    #[inline]
    pub fn start_risk_check_timer(&self) -> RiskCheckTimer {
        self.risk_checks_total.inc();
        RiskCheckTimer::new(self.risk_checks_duration.clone())
    }
    
    /// Record risk check failure - lock-free operation
    #[inline]
    pub fn record_risk_check_failure(&self) {
        self.risk_check_failures.inc();
        trace!("Risk check failure metric updated");
    }
    
    /// Update NAV - lock-free operation
    #[inline]
    pub fn update_nav(&self, nav_bps: i64) {
        self.current_nav.set(nav_bps);
        trace!("NAV metric updated: {}", nav_bps);
    }
    
    /// Update drawdown - lock-free operation
    #[inline]
    pub fn update_drawdown(&self, drawdown_bps: i64) {
        self.current_drawdown.set(drawdown_bps);
        trace!("Drawdown metric updated: {}", drawdown_bps);
    }
    
    /// Update halt status - lock-free operation
    #[inline]
    pub fn update_halt_status(&self, halted: bool) {
        self.is_halted.set(if halted { 1 } else { 0 });
        trace!("Halt status metric updated: {}", halted);
    }
    
    /// Update position utilization - lock-free operation
    #[inline]
    pub fn update_position_utilization(&self, ratio: f64) {
        self.position_utilization.set(ratio);
        trace!("Position utilization updated: {:.4}", ratio);
    }
    
    /// Get symbol metrics for a specific symbol
    pub fn get_symbol_metrics(&self, symbol: &str) -> SymbolMetrics {
        SymbolMetrics {
            exposure: self.symbol_exposure.with_label_values(&[symbol]),
            orders: self.symbol_orders.with_label_values(&[symbol]),
        }
    }
    
    /// Export metrics in Prometheus format
    pub fn export(&self) -> Result<String, prometheus::Error> {
        let metric_families = self.registry.gather();
        let encoder = TextEncoder::new();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer).unwrap())
    }
}

/// Timer for measuring risk check duration
pub struct RiskCheckTimer {
    start: Instant,
    histogram: Histogram,
}

impl RiskCheckTimer {
    fn new(histogram: Histogram) -> Self {
        Self {
            start: Instant::now(),
            histogram,
        }
    }
    
    /// Finish timing and record duration
    #[inline]
    pub fn finish(self) {
        let duration = self.start.elapsed();
        self.histogram.observe(duration.as_secs_f64());
        trace!("Risk check duration: {:?}", duration);
    }
}


/// Global metrics instance
lazy_static::lazy_static! {
    pub static ref GLOBAL_RISK_METRICS: Arc<RiskBusMetrics> = {
        Arc::new(RiskBusMetrics::new().expect("Failed to create metrics"))
    };
}

/// Metrics HTTP handler for Axum
pub async fn metrics_handler() -> Result<String, StatusCode> {
    match GLOBAL_RISK_METRICS.export() {
        Ok(metrics) => Ok(metrics),
        Err(e) => {
            error!("Failed to export metrics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Rate limiter for metrics endpoint
pub struct MetricsRateLimiter {
    last_request: std::sync::Mutex<std::time::Instant>,
    requests_per_second: u32,
}

impl MetricsRateLimiter {
    pub fn new(requests_per_second: u32) -> Self {
        Self {
            last_request: std::sync::Mutex::new(std::time::Instant::now()),
            requests_per_second,
        }
    }
    
    pub fn check_rate_limit(&self) -> bool {
        let mut last = self.last_request.lock().unwrap();
        let now = std::time::Instant::now();
        
        if now.duration_since(*last).as_secs_f64() >= 1.0 / self.requests_per_second as f64 {
            *last = now;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_metrics_creation() {
        let metrics = RiskBusMetrics::new().unwrap();
        
        // Test basic operations
        metrics.record_order_submitted();
        metrics.record_order_rejected();
        metrics.update_nav(1000000);
        metrics.update_drawdown(100);
        metrics.update_halt_status(true);
        
        let exported = metrics.export().unwrap();
        assert!(exported.contains("riskbus_orders_submitted_total 1"));
        assert!(exported.contains("riskbus_orders_rejected_total 1"));
        assert!(exported.contains("riskbus_current_nav 1000000"));
        assert!(exported.contains("riskbus_is_halted 1"));
    }
    
    #[test]
    fn test_risk_check_timer() {
        let metrics = RiskBusMetrics::new().unwrap();
        let timer = metrics.start_risk_check_timer();
        std::thread::sleep(Duration::from_millis(1));
        timer.finish();
        
        let exported = metrics.export().unwrap();
        assert!(exported.contains("riskbus_risk_check_duration_seconds"));
    }
    
    #[test]
    fn test_rate_limiter() {
        let limiter = MetricsRateLimiter::new(10);
        
        // First request should pass
        assert!(limiter.check_rate_limit());
        
        // Immediate second request should fail
        assert!(!limiter.check_rate_limit());
        
        // Wait and try again
        std::thread::sleep(Duration::from_millis(100));
        assert!(limiter.check_rate_limit());
    }
}
