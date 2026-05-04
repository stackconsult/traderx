//! Self-healing model system for automated health monitoring and recovery
//!
//! Provides health monitoring, anomaly detection, auto-tuning, and fallback management
//! for ML models in the trading system.

mod auto_tuner;
mod fallback_manager;
mod health_monitor;
mod types;

pub use auto_tuner::AutoTuner;
pub use fallback_manager::FallbackManager;
pub use health_monitor::HealthMonitor;
pub use types::{
    AnomalyType, HealthStatus, OptimizationStrategy, PerformanceMetrics, PerformanceThresholds,
};

use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use crate::observability::{AgentMetrics, StructuredLogger};

/// Main self-healing model system
pub struct SelfHealingModel {
    model_name: String,
    health_monitor: HealthMonitor,
    auto_tuner: AutoTuner,
    fallback_manager: FallbackManager,
    is_active: bool,
}

impl SelfHealingModel {
    pub fn new(
        model_name: String,
        fallback_models: Vec<String>,
        thresholds: PerformanceThresholds,
    ) -> Self {
        Self {
            model_name: model_name.clone(),
            health_monitor: HealthMonitor::new(thresholds),
            auto_tuner: AutoTuner::new(),
            fallback_manager: FallbackManager::new(model_name, fallback_models),
            is_active: true,
        }
    }

    pub fn with_metrics(mut self, metrics: Arc<AgentMetrics>) -> Self {
        self.health_monitor = self.health_monitor.with_metrics(metrics);
        self
    }

    pub fn with_logger(mut self, logger: Arc<StructuredLogger>) -> Self {
        self.health_monitor = self.health_monitor.with_logger(logger);
        self
    }

    pub async fn record_performance(&mut self, metrics: PerformanceMetrics) {
        self.health_monitor.record_metrics(metrics).await;
    }

    pub async fn monitor_and_heal(&mut self) -> bool {
        if !self.is_active {
            return false;
        }

        let health_status = self.health_monitor.check_health(&self.model_name);

        debug!(
            "Model {} health status: {:?}",
            self.model_name, health_status
        );

        let anomalies = self.health_monitor.detect_anomalies(&self.model_name);

        if !anomalies.is_empty() {
            warn!(
                "Detected {} anomalies for model {}",
                anomalies.len(),
                self.model_name
            );

            let mut healed = false;
            for anomaly in &anomalies {
                if let Some(strategy) = self.auto_tuner.select_strategy(anomaly) {
                    match self.auto_tuner.apply_strategy(strategy).await {
                        Ok(()) => {
                            info!(
                                "Successfully applied healing strategy for anomaly: {:?}",
                                anomaly
                            );
                            healed = true;
                        }
                        Err(e) => {
                            error!("Failed to apply healing strategy: {}", e);
                        }
                    }
                }
            }

            if !healed && matches!(health_status, HealthStatus::Critical { .. }) {
                warn!("Critical health status, attempting fallback");
                match self.fallback_manager.execute_fallback().await {
                    Ok(new_model) => {
                        info!("Successfully fell back to model: {}", new_model);
                        healed = true;
                    }
                    Err(e) => {
                        error!("Fallback failed: {}", e);
                    }
                }
            }

            healed
        } else {
            true
        }
    }

    pub async fn start_continuous_monitoring(&mut self, monitoring_interval: Duration) {
        info!(
            "Starting continuous monitoring for model: {}",
            self.model_name
        );

        let mut interval = interval(monitoring_interval);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if !self.monitor_and_heal().await {
                        error!("Self-healing failed for model: {}", self.model_name);
                    }
                }

                _ = tokio::signal::ctrl_c() => {
                    info!("Stopping monitoring for model: {}", self.model_name);
                    break;
                }
            }
        }
    }

    pub fn get_health_status(&self) -> HealthStatus {
        self.health_monitor.check_health(&self.model_name)
    }

    pub fn get_recent_metrics(&self, count: usize) -> Vec<PerformanceMetrics> {
        self.health_monitor
            .get_recent_metrics(&self.model_name, count)
    }

    pub fn current_model(&self) -> &str {
        self.fallback_manager.current_model()
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
        info!("Model {} deactivated", self.model_name);
    }

    pub fn reactivate(&mut self) {
        self.is_active = true;
        info!("Model {} reactivated", self.model_name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_metrics_error_rate() {
        let metrics = PerformanceMetrics {
            model_name: "test".to_string(),
            timestamp: Utc::now(),
            latency_ms: 100.0,
            memory_usage_mb: 1024.0,
            cpu_usage_percent: 50.0,
            accuracy_score: Some(0.9),
            request_count: 100,
            error_count: 5,
        };

        assert_eq!(metrics.error_rate(), 0.05);
    }

    #[tokio::test]
    async fn test_health_monitor() {
        let monitor = HealthMonitor::new(PerformanceThresholds::default());

        let healthy_metrics = PerformanceMetrics {
            model_name: "test".to_string(),
            timestamp: Utc::now(),
            latency_ms: 100.0,
            memory_usage_mb: 1024.0,
            cpu_usage_percent: 50.0,
            accuracy_score: Some(0.9),
            request_count: 100,
            error_count: 1,
        };

        monitor.record_metrics(healthy_metrics).await;

        let status = monitor.check_health("test");
        assert!(matches!(status, HealthStatus::Healthy));
    }

    #[test]
    fn test_auto_tuner_strategy_selection() {
        let tuner = AutoTuner::new();

        let anomaly = AnomalyType::PerformanceDegradation {
            metric: "latency".to_string(),
            current: 1000.0,
            threshold: 500.0,
        };

        let strategy = tuner.select_strategy(&anomaly);
        assert!(strategy.is_some());

        if let Some(OptimizationStrategy::ReduceBatchSize { new, .. }) = strategy {
            assert_eq!(new, 16);
        } else {
            panic!("Expected ReduceBatchSize strategy");
        }
    }

    #[test]
    fn test_fallback_manager() {
        let fallback = FallbackManager::new(
            "primary".to_string(),
            vec!["fallback1".to_string(), "fallback2".to_string()],
        );

        assert_eq!(fallback.current_model(), "primary");
        assert_eq!(fallback.get_next_fallback(), Some("fallback1".to_string()));
        assert_eq!(fallback.current_model(), "primary");
    }
}
