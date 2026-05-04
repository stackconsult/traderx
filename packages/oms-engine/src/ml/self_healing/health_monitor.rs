use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;
use chrono::Utc;

use crate::observability::{AgentMetrics, StructuredLogger};
use super::types::{HealthStatus, PerformanceMetrics, PerformanceThresholds, AnomalyType};

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
    
    pub async fn record_metrics(&mut self, metrics: PerformanceMetrics) {
        if self.metrics_history.len() >= self.max_history_size {
            self.metrics_history.remove(0);
        }
        self.metrics_history.push(metrics.clone());
        
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
                Uuid::new_v4(),
                "self_healing",
                crate::observability::LogLevel::Info,
                "Model metrics recorded",
                metadata_map
            ).await;
        }
        
        if let Some(agent_metrics) = &self.metrics {
            agent_metrics.llm_request_duration.observe(metrics.latency_ms / 1000.0);
            agent_metrics.llm_requests_total.inc();
            if metrics.error_count > 0 {
                agent_metrics.system_errors.inc();
            }
        }
    }
    
    pub fn check_health(&self, model_name: &str) -> HealthStatus {
        let recent_metrics: Vec<_> = self.metrics_history
            .iter()
            .filter(|m| m.model_name == model_name)
            .rev()
            .take(10)
            .collect();
        
        if recent_metrics.is_empty() {
            return HealthStatus::Offline;
        }
        
        let latest = &recent_metrics[0];
        
        if !latest.is_healthy(&self.thresholds) {
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
    
    pub fn detect_anomalies(&self, model_name: &str) -> Vec<AnomalyType> {
        let mut anomalies = Vec::new();
        
        let model_metrics: Vec<_> = self.metrics_history
            .iter()
            .filter(|m| m.model_name == model_name)
            .collect();
        
        if model_metrics.len() < 5 {
            return anomalies;
        }
        
        let latest = &model_metrics[model_metrics.len() - 1];
        
        if latest.latency_ms > self.thresholds.max_latency_ms {
            anomalies.push(AnomalyType::PerformanceDegradation {
                metric: "latency".to_string(),
                current: latest.latency_ms,
                threshold: self.thresholds.max_latency_ms,
            });
        }
        
        if model_metrics.len() >= 10 {
            let recent_memory: Vec<f64> = model_metrics.iter()
                .rev()
                .take(10)
                .map(|m| m.memory_usage_mb)
                .collect();
            
            let memory_trend = self.calculate_trend(&recent_memory);
            if memory_trend > 10.0 {
                anomalies.push(AnomalyType::MemoryLeak {
                    memory_usage_mb: latest.memory_usage_mb,
                    trend: memory_trend,
                });
            }
        }
        
        if let (Some(current_accuracy), Some(baseline_accuracy)) = (latest.accuracy_score, self.get_baseline_accuracy(model_name)) {
            if current_accuracy < baseline_accuracy - 0.1 {
                anomalies.push(AnomalyType::AccuracyDrift {
                    current_accuracy,
                    baseline: baseline_accuracy,
                });
            }
        }
        
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
        
        let n = values.len() as f64;
        let sum_x: f64 = (0..values.len()).map(|i| i as f64).sum();
        let sum_y: f64 = values.iter().sum();
        let sum_xy: f64 = values.iter().enumerate().map(|(i, y)| i as f64 * y).sum();
        let sum_x2: f64 = (0..values.len()).map(|i| (i as f64).powi(2)).sum();
        
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));
        slope
    }
    
    fn get_baseline_accuracy(&self, model_name: &str) -> Option<f64> {
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
