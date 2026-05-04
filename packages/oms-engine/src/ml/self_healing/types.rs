use std::time::Duration;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

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
            max_latency_ms: 500.0,
            max_memory_mb: 8192.0,
            max_cpu_percent: 80.0,
            max_error_rate: 0.05,
            min_accuracy: 0.8,
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
