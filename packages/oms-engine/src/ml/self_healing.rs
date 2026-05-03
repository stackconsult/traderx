use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use tokio::time::interval;
use tracing::{info, warn, error, debug};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::observability::{AgentMetrics, StructuredLogger};

/// Health status of a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded { reason: String, severity: f64 },
    Critical { reason: String, error_count: u32 },
    Offline,
}

/// Performance metrics for model monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub model_name: String,
    pub timestamp: DateTime<Utc>,
    pub latency_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub accuracy_score: Option<f64>,
    pub request_count: u64,
    pub error_count: u64,
}

impl PerformanceMetrics {
    pub fn error_rate(&self) -> f64 {
        if self.request_count == 0 {
            0.0
        } else {
            self.error_count as f64 / self.request_count as f64
        }
    }
    
    pub fn is_healthy(&self, thresholds: &PerformanceThresholds) -> bool {
        self.latency_ms <= thresholds.max_latency_ms
            && self.memory_usage_mb <= thresholds.max_memory_mb
            && self.cpu_usage_percent <= thresholds.max_cpu_percent
            && self.error_rate() <= thresholds.max_error_rate
            && self.accuracy_score.map_or(true, |acc| acc >= thresholds.min_accuracy)
    }
}

/// Performance thresholds for health checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    pub max_latency_ms: f64,
    pub max_memory_mb: f64,
    pub max_cpu_percent: f64,
    pub max_error_rate: f64,
    pub min_accuracy: f64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_latency_ms: 500.0,      // 500ms target
            max_memory_mb: 8192.0,      // 8GB max
            max_cpu_percent: 80.0,      // 80% CPU max
            max_error_rate: 0.05,       // 5% error rate max
            min_accuracy: 0.8,          // 80% accuracy min
        }
    }
}

/// Anomaly types detected in model behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    PerformanceDegradation { metric: String, current: f64, threshold: f64 },
    MemoryLeak { memory_usage_mb: f64, trend: f64 },
    AccuracyDrift { current_accuracy: f64, baseline: f64 },
    HighErrorRate { current_rate: f64, threshold: f64 },
    ResourceExhaustion { resource: String, usage: f64 },
}

/// Health monitoring system for models
pub struct HealthMonitor {
    thresholds: PerformanceThresholds,
    metrics_history: Vec<PerformanceMetrics>,
    max_history_size: usize,
    metrics: Option<Arc<AgentMetrics>>,
    logger: Option<Arc<StructuredLogger>>,
}

impl HealthMonitor {
    pub fn new(thresholds: PerformanceThresholds) -> Self {
        Self {
            thresholds,
            metrics_history: Vec::new(),
            max_history_size: 1000,
            metrics: None,
            logger: None,
        }
    }
    
    pub fn with_metrics(mut self, metrics: Arc<AgentMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }
    
    pub fn with_logger(mut self, logger: Arc<StructuredLogger>) -> Self {
        self.logger = Some(logger);
        self
    }
    
    /// Record new performance metrics
    pub async fn record_metrics(&mut self, metrics: PerformanceMetrics) {
        // Maintain history size
        if self.metrics_history.len() >= self.max_history_size {
            self.metrics_history.remove(0);
        }
        self.metrics_history.push(metrics.clone());
        
        // Log metrics
        if let Some(logger) = &self.logger {
            let metadata = serde_json::json!({
                "model": metrics.model_name,
                "latency_ms": metrics.latency_ms,
                "memory_mb": metrics.memory_usage_mb,
                "cpu_percent": metrics.cpu_usage_percent,
                "accuracy": metrics.accuracy_score,
                "error_rate": metrics.error_rate(),
            });
            
            let metadata_map: HashMap<String, String> = metadata.as_object()
                .unwrap_or(&serde_json::Map::new())
                .iter()
                .map(|(k, v)| (k.clone(), v.to_string()))
                .collect();
            
            logger.log(
                uuid::Uuid::new_v4(),
                "self_healing",
                crate::observability::LogLevel::Info,
                "Model metrics recorded",
                metadata_map
            ).await;
        }
        
        // Update Prometheus metrics
        if let Some(agent_metrics) = &self.metrics {
            agent_metrics.llm_request_duration.observe(metrics.latency_ms / 1000.0);
            agent_metrics.llm_requests_total.inc();
            if metrics.error_count > 0 {
                agent_metrics.system_errors.inc();
            }
        }
    }
    
    /// Check model health based on recent metrics
    pub fn check_health(&self, model_name: &str) -> HealthStatus {
        let recent_metrics: Vec<_> = self.metrics_history
            .iter()
            .filter(|m| m.model_name == model_name)
            .rev()
            .take(10) // Last 10 measurements
            .collect();
        
        if recent_metrics.is_empty() {
            return HealthStatus::Offline;
        }
        
        let latest = &recent_metrics[0];
        
        if !latest.is_healthy(&self.thresholds) {
            // Determine specific issues
            if latest.latency_ms > self.thresholds.max_latency_ms {
                return HealthStatus::Degraded {
                    reason: format!("High latency: {:.2}ms > {:.2}ms", latest.latency_ms, self.thresholds.max_latency_ms),
                    severity: (latest.latency_ms / self.thresholds.max_latency_ms - 1.0),
                };
            }
            
            if latest.memory_usage_mb > self.thresholds.max_memory_mb {
                return HealthStatus::Degraded {
                    reason: format!("High memory: {:.2}MB > {:.2}MB", latest.memory_usage_mb, self.thresholds.max_memory_mb),
                    severity: (latest.memory_usage_mb / self.thresholds.max_memory_mb - 1.0),
                };
            }
            
            if latest.error_rate() > self.thresholds.max_error_rate {
                return HealthStatus::Critical {
                    reason: format!("High error rate: {:.2}% > {:.2}%", latest.error_rate() * 100.0, self.thresholds.max_error_rate * 100.0),
                    error_count: latest.error_count as u32,
                };
            }
            
            if let Some(accuracy) = latest.accuracy_score {
                if accuracy < self.thresholds.min_accuracy {
                    return HealthStatus::Degraded {
                        reason: format!("Low accuracy: {:.2}% < {:.2}%", accuracy * 100.0, self.thresholds.min_accuracy * 100.0),
                        severity: (self.thresholds.min_accuracy - accuracy),
                    };
                }
            }
        }
        
        HealthStatus::Healthy
    }
    
    /// Detect anomalies in model behavior
    pub fn detect_anomalies(&self, model_name: &str) -> Vec<AnomalyType> {
        let mut anomalies = Vec::new();
        
        let model_metrics: Vec<_> = self.metrics_history
            .iter()
            .filter(|m| m.model_name == model_name)
            .collect();
        
        if model_metrics.len() < 5 {
            return anomalies; // Not enough data
        }
        
        let latest = &model_metrics[model_metrics.len() - 1];
        
        // Check performance degradation
        if latest.latency_ms > self.thresholds.max_latency_ms {
            anomalies.push(AnomalyType::PerformanceDegradation {
                metric: "latency".to_string(),
                current: latest.latency_ms,
                threshold: self.thresholds.max_latency_ms,
            });
        }
        
        // Check memory leak (trend analysis)
        if model_metrics.len() >= 10 {
            let recent_memory: Vec<f64> = model_metrics.iter()
                .rev()
                .take(10)
                .map(|m| m.memory_usage_mb)
                .collect();
            
            let memory_trend = self.calculate_trend(&recent_memory);
            if memory_trend > 10.0 { // Growing by more than 10MB per measurement
                anomalies.push(AnomalyType::MemoryLeak {
                    memory_usage_mb: latest.memory_usage_mb,
                    trend: memory_trend,
                });
            }
        }
        
        // Check accuracy drift
        if let (Some(current_accuracy), Some(baseline_accuracy)) = (latest.accuracy_score, self.get_baseline_accuracy(model_name)) {
            if current_accuracy < baseline_accuracy - 0.1 { // 10% drop
                anomalies.push(AnomalyType::AccuracyDrift {
                    current_accuracy,
                    baseline: baseline_accuracy,
                });
            }
        }
        
        // Check high error rate
        if latest.error_rate() > self.thresholds.max_error_rate {
            anomalies.push(AnomalyType::HighErrorRate {
                current_rate: latest.error_rate(),
                threshold: self.thresholds.max_error_rate,
            });
        }
        
        anomalies
    }
    
    fn calculate_trend(&self, values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }
        
        // Simple linear regression slope
        let n = values.len() as f64;
        let sum_x: f64 = (0..values.len()).map(|i| i as f64).sum();
        let sum_y: f64 = values.iter().sum();
        let sum_xy: f64 = values.iter().enumerate().map(|(i, y)| i as f64 * y).sum();
        let sum_x2: f64 = (0..values.len()).map(|i| (i as f64).powi(2)).sum();
        
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));
        slope
    }
    
    fn get_baseline_accuracy(&self, model_name: &str) -> Option<f64> {
        // Get average accuracy from first 10 measurements as baseline
        let baseline_metrics: Vec<_> = self.metrics_history
            .iter()
            .filter(|m| m.model_name == model_name)
            .take(10)
            .filter_map(|m| m.accuracy_score)
            .collect();
        
        if baseline_metrics.is_empty() {
            None
        } else {
            Some(baseline_metrics.iter().sum::<f64>() / baseline_metrics.len() as f64)
        }
    }
    
    /// Get recent metrics for a model
    pub fn get_recent_metrics(&self, model_name: &str, count: usize) -> Vec<PerformanceMetrics> {
        self.metrics_history
            .iter()
            .filter(|m| m.model_name == model_name)
            .rev()
            .take(count)
            .cloned()
            .collect()
    }
}

/// Auto-tuning strategies for model optimization
#[derive(Debug, Clone)]
pub enum OptimizationStrategy {
    ReduceBatchSize { current: usize, new: usize },
    IncreaseTimeout { current: Duration, new: Duration },
    AdjustMemoryLimit { current: f64, new: f64 },
    SwitchModel { from: String, to: String },
    RestartModel,
    RetrainModel,
}

/// Auto-tuning engine for model optimization
pub struct AutoTuner {
    optimization_strategies: Vec<OptimizationStrategy>,
    tuning_history: Vec<(DateTime<Utc>, OptimizationStrategy, bool)>, // timestamp, strategy, success
    max_history_size: usize,
}

impl AutoTuner {
    pub fn new() -> Self {
        Self {
            optimization_strategies: Vec::new(),
            tuning_history: Vec::new(),
            max_history_size: 100,
        }
    }
    
    /// Select appropriate optimization strategy based on anomaly
    pub fn select_strategy(&self, anomaly: &AnomalyType) -> Option<OptimizationStrategy> {
        match anomaly {
            AnomalyType::PerformanceDegradation { metric, .. } => {
                match metric.as_str() {
                    "latency" => Some(OptimizationStrategy::ReduceBatchSize { current: 32, new: 16 }),
                    "memory" => Some(OptimizationStrategy::AdjustMemoryLimit { current: 8192.0, new: 4096.0 }),
                    _ => None,
                }
            },
            AnomalyType::MemoryLeak { .. } => Some(OptimizationStrategy::RestartModel),
            AnomalyType::AccuracyDrift { .. } => Some(OptimizationStrategy::RetrainModel),
            AnomalyType::HighErrorRate { .. } => Some(OptimizationStrategy::IncreaseTimeout { 
                current: Duration::from_secs(30), 
                new: Duration::from_secs(60) 
            }),
            AnomalyType::ResourceExhaustion { resource, .. } => {
                match resource.as_str() {
                    "memory" => Some(OptimizationStrategy::SwitchModel { 
                        from: "deepseek-coder:6.7b".to_string(), 
                        to: "deepseek-coder:1.3b".to_string() 
                    }),
                    _ => None,
                }
            },
        }
    }
    
    /// Apply optimization strategy
    pub async fn apply_strategy(&mut self, strategy: OptimizationStrategy) -> Result<(), String> {
        let timestamp = Utc::now();
        
        let success = match &strategy {
            OptimizationStrategy::ReduceBatchSize { new, .. } => {
                info!("Applying batch size reduction to {}", new);
                // In a real implementation, this would adjust the model's batch size
                true
            },
            OptimizationStrategy::IncreaseTimeout { new, .. } => {
                info!("Applying timeout increase to {:?}", new);
                // In a real implementation, this would adjust request timeouts
                true
            },
            OptimizationStrategy::AdjustMemoryLimit { new, .. } => {
                info!("Applying memory limit adjustment to {}MB", new);
                // In a real implementation, this would adjust memory limits
                true
            },
            OptimizationStrategy::SwitchModel { from, to } => {
                info!("Switching model from {} to {}", from, to);
                // In a real implementation, this would handle model switching
                true
            },
            OptimizationStrategy::RestartModel => {
                info!("Restarting model");
                // In a real implementation, this would restart the model
                true
            },
            OptimizationStrategy::RetrainModel => {
                info!("Initiating model retraining");
                // In a real implementation, this would trigger retraining
                true
            },
        };
        
        // Record in history
        self.tuning_history.push((timestamp, strategy, success));
        
        // Maintain history size
        if self.tuning_history.len() > self.max_history_size {
            self.tuning_history.remove(0);
        }
        
        if success {
            Ok(())
        } else {
            Err("Failed to apply optimization strategy".to_string())
        }
    }
    
    /// Get tuning effectiveness statistics
    pub fn get_tuning_stats(&self) -> (usize, f64) {
        let total_attempts = self.tuning_history.len();
        let successful_attempts = self.tuning_history.iter().filter(|(_, _, success)| *success).count();
        let success_rate = if total_attempts > 0 {
            successful_attempts as f64 / total_attempts as f64
        } else {
            0.0
        };
        
        (total_attempts, success_rate)
    }
}

/// Fallback manager for model failover
pub struct FallbackManager {
    fallback_chain: Vec<String>,
    current_model: String,
    fallback_history: Vec<(DateTime<Utc>, String, String, bool)>, // timestamp, from, to, success
}

impl FallbackManager {
    pub fn new(primary_model: String, fallback_models: Vec<String>) -> Self {
        let mut fallback_chain = vec![primary_model.clone()];
        fallback_chain.extend(fallback_models);
        
        Self {
            fallback_chain,
            current_model: primary_model,
            fallback_history: Vec::new(),
        }
    }
    
    /// Get next available model in fallback chain
    pub fn get_next_fallback(&self) -> Option<String> {
        let current_index = self.fallback_chain.iter()
            .position(|m| m == &self.current_model)?;
        
        self.fallback_chain.get(current_index + 1).cloned()
    }
    
    /// Execute fallback to next model
    pub async fn execute_fallback(&mut self) -> Result<String, String> {
        let from_model = self.current_model.clone();
        
        if let Some(to_model) = self.get_next_fallback() {
            info!("Executing fallback from {} to {}", from_model, to_model);
            
            // In a real implementation, this would handle the actual model switching
            let success = true; // Placeholder
            
            self.fallback_history.push((Utc::now(), from_model.clone(), to_model.clone(), success));
            self.current_model = to_model.clone();
            
            if success {
                Ok(to_model)
            } else {
                Err("Fallback failed".to_string())
            }
        } else {
            Err("No more fallback models available".to_string())
        }
    }
    
    /// Reset to primary model
    pub fn reset_to_primary(&mut self) {
        if let Some(primary) = self.fallback_chain.first() {
            self.current_model = primary.clone();
            info!("Reset to primary model: {}", primary);
        }
    }
    
    /// Get current model
    pub fn current_model(&self) -> &str {
        &self.current_model
    }
}

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
    
    /// Record performance metrics
    pub fn record_performance(&mut self, metrics: PerformanceMetrics) {
        self.health_monitor.record_metrics(metrics);
    }
    
    /// Detect anomalies and trigger healing
    pub async fn monitor_and_heal(&mut self) -> bool {
        if !self.is_active {
            return false;
        }
        
        // 1. Health Assessment
        let health_status = self.health_monitor.check_health(&self.model_name);
        
        debug!("Model {} health status: {:?}", self.model_name, health_status);
        
        // 2. Anomaly Detection
        let anomalies = self.health_monitor.detect_anomalies(&self.model_name);
        
        if !anomalies.is_empty() {
            warn!("Detected {} anomalies for model {}", anomalies.len(), self.model_name);
            
            // 3. Apply healing strategies for each anomaly
            let mut healed = false;
            for anomaly in &anomalies {
                if let Some(strategy) = self.auto_tuner.select_strategy(anomaly) {
                    match self.auto_tuner.apply_strategy(strategy).await {
                        Ok(()) => {
                            info!("Successfully applied healing strategy for anomaly: {:?}", anomaly);
                            healed = true;
                        },
                        Err(e) => {
                            error!("Failed to apply healing strategy: {}", e);
                        }
                    }
                }
            }
            
            // 4. If still unhealthy, try fallback
            if !healed && matches!(health_status, HealthStatus::Critical { .. }) {
                warn!("Critical health status, attempting fallback");
                match self.fallback_manager.execute_fallback().await {
                    Ok(new_model) => {
                        info!("Successfully fell back to model: {}", new_model);
                        healed = true;
                    },
                    Err(e) => {
                        error!("Fallback failed: {}", e);
                    }
                }
            }
            
            healed
        } else {
            true // No anomalies detected
        }
    }
    
    /// Start continuous monitoring loop
    pub async fn start_continuous_monitoring(&mut self, monitoring_interval: Duration) {
        info!("Starting continuous monitoring for model: {}", self.model_name);
        
        let mut interval = interval(monitoring_interval);
        
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if !self.monitor_and_heal().await {
                        error!("Self-healing failed for model: {}", self.model_name);
                    }
                }
                
                // Handle graceful shutdown
                _ = tokio::signal::ctrl_c() => {
                    info!("Stopping monitoring for model: {}", self.model_name);
                    break;
                }
            }
        }
    }
    
    /// Get current health status
    pub fn get_health_status(&self) -> HealthStatus {
        self.health_monitor.check_health(&self.model_name)
    }
    
    /// Get recent performance metrics
    pub fn get_recent_metrics(&self, count: usize) -> Vec<PerformanceMetrics> {
        self.health_monitor.get_recent_metrics(&self.model_name, count)
    }
    
    /// Get current model (may be different due to fallbacks)
    pub fn current_model(&self) -> &str {
        self.fallback_manager.current_model()
    }
    
    /// Check if model is active
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    
    /// Deactivate the model
    pub fn deactivate(&mut self) {
        self.is_active = false;
        info!("Model {} deactivated", self.model_name);
    }
    
    /// Reactivate the model
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
    
    #[test]
    fn test_health_monitor() {
        let mut monitor = HealthMonitor::new(PerformanceThresholds::default());
        
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
        
        monitor.record_metrics(healthy_metrics);
        
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
        let mut fallback = FallbackManager::new(
            "primary".to_string(),
            vec!["fallback1".to_string(), "fallback2".to_string()],
        );
        
        assert_eq!(fallback.current_model(), "primary");
        
        assert_eq!(fallback.get_next_fallback(), Some("fallback1".to_string()));
        
        // This would normally be async, but for testing we'll check the logic
        assert_eq!(fallback.current_model(), "primary");
    }
}
