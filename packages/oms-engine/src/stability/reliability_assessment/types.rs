use std::time::Duration;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::middleware::LlmProvider;

/// Failure mode classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureMode {
    ModelPerformance { model: String, metric: String, threshold: f64, actual: f64 },
    ProviderOutage { provider: LlmProvider, duration: Duration, error_count: u32 },
    MemoryExhaustion { component: String, usage_mb: f64, limit_mb: f64 },
    NetworkFailure { endpoint: String, error_type: String, retry_count: u32 },
    CircuitBreakerOpen { provider: LlmProvider, failure_count: u32 },
    ContextCorruption { context_id: Uuid, issue: String },
    ResourceContention { resource: String, waiters: u32, avg_wait_ms: f64 },
    SystemOverload { load_factor: f64, cpu_usage: f64, memory_usage: f64 },
    ConfigurationError { component: String, setting: String, issue: String },
}

/// Severity classification for failures
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

impl FailureMode {
    pub fn severity(&self) -> Severity {
        match self {
            FailureMode::ModelPerformance { .. } => Severity::Medium,
            FailureMode::ProviderOutage { duration, .. } => {
                if *duration > Duration::from_secs(300) {
                    Severity::Critical
                } else if *duration > Duration::from_secs(60) {
                    Severity::High
                } else {
                    Severity::Medium
                }
            },
            FailureMode::MemoryExhaustion { .. } => Severity::High,
            FailureMode::NetworkFailure { retry_count, .. } => {
                if *retry_count > 5 {
                    Severity::High
                } else {
                    Severity::Medium
                }
            },
            FailureMode::CircuitBreakerOpen { .. } => Severity::High,
            FailureMode::ContextCorruption { .. } => Severity::Medium,
            FailureMode::ResourceContention { waiters, .. } => {
                if *waiters > 10 {
                    Severity::High
                } else {
                    Severity::Medium
                }
            },
            FailureMode::SystemOverload { load_factor, .. } => {
                if *load_factor > 0.9 {
                    Severity::Critical
                } else if *load_factor > 0.7 {
                    Severity::High
                } else {
                    Severity::Medium
                }
            },
            FailureMode::ConfigurationError { .. } => Severity::Low,
        }
    }
}

/// Service Level Agreement (SLA) definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaDefinition {
    pub name: String,
    pub description: String,
    pub target: SlaTarget,
    pub penalty: SlaPenalty,
    pub measurement_window: Duration,
    pub alert_thresholds: Vec<SlaThreshold>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlaTarget {
    Latency { p50_ms: f64, p95_ms: f64, p99_ms: f64 },
    Availability { uptime_percent: f64 },
    ErrorRate { max_error_rate: f64 },
    Throughput { min_requests_per_sec: u64 },
    MemoryUsage { max_usage_mb: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaPenalty {
    pub warning_threshold: f64,
    pub critical_threshold: f64,
    pub escalation_procedure: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaThreshold {
    pub level: Severity,
    pub threshold: f64,
    pub action: String,
}

/// SLA compliance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaCompliance {
    pub sla_name: String,
    pub current_value: f64,
    pub target_value: f64,
    pub compliance_percent: f64,
    pub status: ComplianceStatus,
    pub last_updated: DateTime<Utc>,
    pub violations: Vec<SlaViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceStatus {
    Compliant,
    Warning,
    Critical,
    Violated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaViolation {
    pub timestamp: DateTime<Utc>,
    pub actual_value: f64,
    pub target_value: f64,
    pub severity: Severity,
    pub duration: Duration,
}

/// Reliability metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityMetrics {
    pub timestamp: DateTime<Utc>,
    pub uptime_percentage: f64,
    pub mean_time_between_failures: Duration,
    pub mean_time_to_recovery: Duration,
    pub failure_rate_per_hour: f64,
    pub availability_sla: f64,
    pub performance_sla: f64,
    pub error_rate: f64,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
}

/// Incident tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub incident_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub severity: Severity,
    pub failure_mode: FailureMode,
    pub description: String,
    pub impact_assessment: ImpactAssessment,
    pub resolution: Option<IncidentResolution>,
    pub affected_components: Vec<String>,
    pub duration: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessment {
    pub users_affected: u32,
    pub requests_lost: u64,
    pub revenue_impact: Option<f64>,
    pub customer_satisfaction_impact: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentResolution {
    pub timestamp: DateTime<Utc>,
    pub resolution_type: ResolutionType,
    pub description: String,
    pub root_cause: Option<String>,
    pub preventive_measures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionType {
    Automatic,
    Manual,
    Escalated,
}

/// Assessment configuration
#[derive(Debug, Clone)]
pub struct AssessmentConfig {
    pub assessment_interval: Duration,
    pub failure_retention_period: Duration,
    pub incident_retention_period: Duration,
    pub metrics_retention_period: Duration,
    pub auto_escalation_threshold: Severity,
    pub compliance_window: Duration,
}

impl Default for AssessmentConfig {
    fn default() -> Self {
        Self {
            assessment_interval: Duration::from_secs(60),
            failure_retention_period: Duration::from_secs(7 * 24 * 60 * 60),
            incident_retention_period: Duration::from_secs(30 * 24 * 60 * 60),
            metrics_retention_period: Duration::from_secs(90 * 24 * 60 * 60),
            auto_escalation_threshold: Severity::High,
            compliance_window: Duration::from_hours(24),
        }
    }
}
