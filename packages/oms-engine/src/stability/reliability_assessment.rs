use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::interval;
use tracing::{info, warn, error, debug};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::observability::{AgentMetrics, StructuredLogger};
use crate::middleware::LlmProvider;
use crate::llm::LlmError;

/// Failure mode classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureMode {
    /// Model performance degradation
    ModelPerformance { model: String, metric: String, threshold: f64, actual: f64 },
    /// Provider outage or unavailability
    ProviderOutage { provider: LlmProvider, duration: Duration, error_count: u32 },
    /// Memory exhaustion
    MemoryExhaustion { component: String, usage_mb: f64, limit_mb: f64 },
    /// Network connectivity issues
    NetworkFailure { endpoint: String, error_type: String, retry_count: u32 },
    /// Circuit breaker activation
    CircuitBreakerOpen { provider: LlmProvider, failure_count: u32 },
    /// Context overflow or corruption
    ContextCorruption { context_id: Uuid, issue: String },
    /// Resource contention
    ResourceContention { resource: String, waiters: u32, avg_wait_ms: f64 },
    /// System overload
    SystemOverload { load_factor: f64, cpu_usage: f64, memory_usage: f64 },
    /// Configuration error
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

/// Main reliability assessment framework
pub struct ReliabilityAssessment {
    failure_modes: Vec<FailureMode>,
    sla_definitions: Vec<SlaDefinition>,
    sla_compliance: HashMap<String, SlaCompliance>,
    incidents: Vec<Incident>,
    metrics_history: Vec<ReliabilityMetrics>,
    assessment_config: AssessmentConfig,
    metrics: Option<Arc<AgentMetrics>>,
    logger: Option<Arc<StructuredLogger>>,
}

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
            failure_retention_period: Duration::from_secs(7 * 24 * 60 * 60), // 7 days
            incident_retention_period: Duration::from_secs(30 * 24 * 60 * 60), // 30 days
            metrics_retention_period: Duration::from_secs(90 * 24 * 60 * 60), // 90 days
            auto_escalation_threshold: Severity::High,
            compliance_window: Duration::from_hours(24),
        }
    }
}

impl ReliabilityAssessment {
    pub fn new(config: AssessmentConfig) -> Self {
        let mut assessment = Self {
            failure_modes: Vec::new(),
            sla_definitions: Self::default_sla_definitions(),
            sla_compliance: HashMap::new(),
            incidents: Vec::new(),
            metrics_history: Vec::new(),
            assessment_config: config,
            metrics: None,
            logger: None,
        };
        
        // Initialize SLA compliance tracking
        for sla in &assessment.sla_definitions {
            assessment.sla_compliance.insert(
                sla.name.clone(),
                SlaCompliance {
                    sla_name: sla.name.clone(),
                    current_value: 0.0,
                    target_value: 0.0,
                    compliance_percent: 100.0,
                    status: ComplianceStatus::Compliant,
                    last_updated: Utc::now(),
                    violations: Vec::new(),
                }
            );
        }
        
        assessment
    }
    
    pub fn with_metrics(mut self, metrics: Arc<AgentMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }
    
    pub fn with_logger(mut self, logger: Arc<StructuredLogger>) -> Self {
        self.logger = Some(logger);
        self
    }
    
    /// Record a failure mode
    pub async fn record_failure(&mut self, failure: FailureMode) {
        let severity = failure.severity();
        
        // Store failure
        self.failure_modes.push(failure.clone());
        
        // Check if this should create an incident
        if severity >= self.assessment_config.auto_escalation_threshold {
            self.create_incident(failure.clone()).await;
        }
        
        // Update SLA compliance
        self.update_sla_compliance_for_failure(&failure).await;
        
        // Log the failure
        if let Some(logger) = &self.logger {
            let metadata = HashMap::from([
                ("failure_type".to_string(), format!("{:?}", failure)),
                ("severity".to_string(), format!("{:?}", severity)),
                ("timestamp".to_string(), Utc::now().to_rfc3339()),
            ]);
            
            logger.log(
                Uuid::new_v4(),
                "reliability_assessment",
                crate::observability::LogLevel::Warn,
                "Failure mode recorded",
                metadata
            ).await;
        }
        
        // Record metrics
        if let Some(metrics) = &self.metrics {
            metrics.system_errors.inc();
        }
        
        // Cleanup old failures
        self.cleanup_old_failures();
    }
    
    /// Create an incident from a failure
    async fn create_incident(&mut self, failure: FailureMode) {
        let incident = Incident {
            incident_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            severity: failure.severity(),
            failure_mode: failure.clone(),
            description: format!("Incident created from failure: {:?}", failure),
            impact_assessment: self.assess_impact(&failure),
            resolution: None,
            affected_components: self.get_affected_components(&failure),
            duration: None,
        };
        
        self.incidents.push(incident.clone());
        
        // Log incident creation
        if let Some(logger) = &self.logger {
            let metadata = HashMap::from([
                ("incident_id".to_string(), incident.incident_id.to_string()),
                ("severity".to_string(), format!("{:?}", incident.severity)),
                ("failure_mode".to_string(), format!("{:?}", failure)),
            ]);
            
            logger.log(
                incident.incident_id,
                "reliability_assessment",
                crate::observability::LogLevel::Error,
                "Incident created",
                metadata
            ).await;
        }
        
        warn!("Incident {} created for failure: {:?}", incident.incident_id, failure);
    }
    
    /// Resolve an incident
    pub async fn resolve_incident(
        &mut self,
        incident_id: Uuid,
        resolution_type: ResolutionType,
        description: String,
        root_cause: Option<String>,
        preventive_measures: Vec<String>,
    ) {
        if let Some(incident) = self.incidents.iter_mut().find(|i| i.incident_id == incident_id) {
            let resolution = IncidentResolution {
                timestamp: Utc::now(),
                resolution_type: resolution_type.clone(),
                description,
                root_cause,
                preventive_measures,
            };
            
            incident.duration = Some(Utc::now().signed_duration_since(incident.timestamp).to_std().unwrap_or(Duration::ZERO));
            
            incident.resolution = Some(resolution);
            
            info!("Incident {} resolved", incident_id);
            
            // Log resolution
            if let Some(logger) = &self.logger {
                let metadata = HashMap::from([
                    ("incident_id".to_string(), incident_id.to_string()),
                    ("resolution_type".to_string(), format!("{:?}", resolution_type)),
                    ("duration_ms".to_string(), incident.duration.map(|d| d.as_millis()).unwrap_or(0).to_string()),
                ]);
                
                logger.log(
                    incident_id,
                    "reliability_assessment",
                    crate::observability::LogLevel::Info,
                    "Incident resolved",
                    metadata
                ).await;
            }
        }
    }
    
    /// Calculate current reliability metrics
    pub async fn calculate_metrics(&mut self) -> ReliabilityMetrics {
        let now = Utc::now();
        let window_start = now - self.assessment_config.compliance_window;
        
        // Filter incidents within the window
        let recent_incidents: Vec<_> = self.incidents
            .iter()
            .filter(|i| i.timestamp > window_start)
            .collect();
        
        let total_requests = self.metrics_history
            .iter()
            .map(|m| m.total_requests)
            .sum();
        
        let successful_requests = self.metrics_history
            .iter()
            .map(|m| m.successful_requests)
            .sum();
        
        let failed_requests = total_requests - successful_requests;
        
        let uptime_percentage = if total_requests > 0 {
            (successful_requests as f64 / total_requests as f64) * 100.0
        } else {
            100.0
        };
        
        let error_rate = if total_requests > 0 {
            (failed_requests as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };
        
        // Calculate MTBF and MTTR
        let (mtbf, mttr) = self.calculate_mtbf_mttr(&recent_incidents);
        
        let failure_rate_per_hour = if recent_incidents.len() > 0 {
            (recent_incidents.len() as f64 / self.assessment_config.compliance_window.as_secs_f64()) * 3600.0
        } else {
            0.0
        };
        
        let metrics = ReliabilityMetrics {
            timestamp: now,
            uptime_percentage,
            mean_time_between_failures: mtbf,
            mean_time_to_recovery: mttr,
            failure_rate_per_hour,
            availability_sla: self.calculate_availability_sla(),
            performance_sla: self.calculate_performance_sla(),
            error_rate,
            total_requests,
            successful_requests,
            failed_requests,
        };
        
        // Store metrics
        self.metrics_history.push(metrics.clone());
        
        // Cleanup old metrics
        self.cleanup_old_metrics();
        
        metrics
    }
    
    /// Get SLA compliance report
    pub async fn get_sla_compliance_report(&self) -> HashMap<String, &SlaCompliance> {
        self.sla_compliance
            .iter()
            .map(|(k, v)| (k.clone(), v))
            .collect()
    }
    
    /// Get active incidents
    pub fn get_active_incidents(&self) -> Vec<&Incident> {
        self.incidents
            .iter()
            .filter(|i| i.resolution.is_none())
            .collect()
    }
    
    /// Get failure mode analysis
    pub fn get_failure_analysis(&self) -> HashMap<String, u32> {
        let mut analysis = HashMap::new();
        
        for failure in &self.failure_modes {
            let key = format!("{:?}", failure);
            *analysis.entry(key).or_insert(0) += 1;
        }
        
        analysis
    }
    
    /// Start continuous assessment
    pub async fn start_continuous_assessment(&mut self) {
        info!("Starting continuous reliability assessment");
        
        let mut interval = interval(self.assessment_config.assessment_interval);
        
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    // Calculate metrics
                    let metrics = self.calculate_metrics().await;
                    
                    // Check SLA compliance
                    self.check_sla_compliance().await;
                    
                    // Log assessment
                    if let Some(logger) = &self.logger {
                        let metadata = HashMap::from([
                            ("uptime_percentage".to_string(), metrics.uptime_percentage.to_string()),
                            ("error_rate".to_string(), metrics.error_rate.to_string()),
                            ("active_incidents".to_string(), self.get_active_incidents().len().to_string()),
                            ("mttr_minutes".to_string(), metrics.mean_time_to_recovery.as_secs().to_string()),
                        ]);
                        
                        logger.log(
                            Uuid::new_v4(),
                            "reliability_assessment",
                            crate::observability::LogLevel::Info,
                            "Reliability assessment completed",
                            metadata
                        ).await;
                    }
                }
                
                // Handle graceful shutdown
                _ = tokio::signal::ctrl_c() => {
                    info!("Stopping reliability assessment");
                    break;
                }
            }
        }
    }
    
    // Private helper methods
    
    fn default_sla_definitions() -> Vec<SlaDefinition> {
        vec![
            SlaDefinition {
                name: "availability".to_string(),
                description: "System availability SLA".to_string(),
                target: SlaTarget::Availability { uptime_percent: 99.5 },
                penalty: SlaPenalty {
                    warning_threshold: 99.0,
                    critical_threshold: 98.0,
                    escalation_procedure: "Notify operations team".to_string(),
                },
                measurement_window: Duration::from_hours(24),
                alert_thresholds: vec![
                    SlaThreshold {
                        level: Severity::Medium,
                        threshold: 99.0,
                        action: "Monitor closely".to_string(),
                    },
                    SlaThreshold {
                        level: Severity::High,
                        threshold: 98.0,
                        action: "Alert on-call engineer".to_string(),
                    },
                ],
            },
            SlaDefinition {
                name: "latency".to_string(),
                description: "Response latency SLA".to_string(),
                target: SlaTarget::Latency { 
                    p50_ms: 100.0, 
                    p95_ms: 500.0, 
                    p99_ms: 1000.0 
                },
                penalty: SlaPenalty {
                    warning_threshold: 10.0,
                    critical_threshold: 25.0,
                    escalation_procedure: "Performance team notification".to_string(),
                },
                measurement_window: Duration::from_hours(1),
                alert_thresholds: vec![
                    SlaThreshold {
                        level: Severity::Medium,
                        threshold: 10.0,
                        action: "Review performance".to_string(),
                    },
                    SlaThreshold {
                        level: Severity::High,
                        threshold: 25.0,
                        action: "Scale resources".to_string(),
                    },
                ],
            },
            SlaDefinition {
                name: "error_rate".to_string(),
                description: "Error rate SLA".to_string(),
                target: SlaTarget::ErrorRate { max_error_rate: 1.0 },
                penalty: SlaPenalty {
                    warning_threshold: 2.0,
                    critical_threshold: 5.0,
                    escalation_procedure: "Engineering team alert".to_string(),
                },
                measurement_window: Duration::from_hours(1),
                alert_thresholds: vec![
                    SlaThreshold {
                        level: Severity::Medium,
                        threshold: 2.0,
                        action: "Monitor errors".to_string(),
                    },
                    SlaThreshold {
                        level: Severity::High,
                        threshold: 5.0,
                        action: "Investigate root cause".to_string(),
                    },
                ],
            },
        ]
    }
    
    fn assess_impact(&self, failure: &FailureMode) -> ImpactAssessment {
        // Simplified impact assessment
        match failure {
            FailureMode::ProviderOutage { provider, .. } => {
                ImpactAssessment {
                    users_affected: 1000,
                    requests_lost: 100,
                    revenue_impact: Some(1000.0),
                    customer_satisfaction_impact: Some(0.1),
                }
            },
            FailureMode::SystemOverload { .. } => {
                ImpactAssessment {
                    users_affected: 500,
                    requests_lost: 50,
                    revenue_impact: Some(500.0),
                    customer_satisfaction_impact: Some(0.05),
                }
            },
            _ => {
                ImpactAssessment {
                    users_affected: 100,
                    requests_lost: 10,
                    revenue_impact: Some(100.0),
                    customer_satisfaction_impact: Some(0.01),
                }
            },
        }
    }
    
    fn get_affected_components(&self, failure: &FailureMode) -> Vec<String> {
        match failure {
            FailureMode::ModelPerformance { model, .. } => vec![format!("model:{}", model)],
            FailureMode::ProviderOutage { provider, .. } => vec![format!("provider:{:?}", provider)],
            FailureMode::MemoryExhaustion { component, .. } => vec![component.clone()],
            FailureMode::NetworkFailure { endpoint, .. } => vec![format!("network:{}", endpoint)],
            FailureMode::CircuitBreakerOpen { provider, .. } => vec![format!("circuit_breaker:{:?}", provider)],
            FailureMode::ContextCorruption { .. } => vec!["context_manager".to_string()],
            FailureMode::ResourceContention { resource, .. } => vec![format!("resource:{}", resource)],
            FailureMode::SystemOverload { .. } => vec!["system".to_string()],
            FailureMode::ConfigurationError { component, .. } => vec![component.clone()],
        }
    }
    
    async fn update_sla_compliance_for_failure(&mut self, failure: &FailureMode) {
        // Update relevant SLA based on failure type
        match failure {
            FailureMode::ProviderOutage { .. } => {
                if let Some(compliance) = self.sla_compliance.get_mut("availability") {
                    compliance.status = ComplianceStatus::Warning;
                    compliance.compliance_percent = 95.0;
                    compliance.last_updated = Utc::now();
                }
            },
            FailureMode::ModelPerformance { .. } => {
                if let Some(compliance) = self.sla_compliance.get_mut("latency") {
                    compliance.status = ComplianceStatus::Warning;
                    compliance.compliance_percent = 90.0;
                    compliance.last_updated = Utc::now();
                }
            },
            _ => {
                if let Some(compliance) = self.sla_compliance.get_mut("error_rate") {
                    compliance.status = ComplianceStatus::Warning;
                    compliance.compliance_percent = 98.0;
                    compliance.last_updated = Utc::now();
                }
            },
        }
    }
    
    fn calculate_mtbf_mttr(&self, incidents: &[&Incident]) -> (Duration, Duration) {
        if incidents.is_empty() {
            return (Duration::from_secs(0), Duration::from_secs(0));
        }
        
        let resolved_incidents: Vec<_> = incidents
            .iter()
            .filter(|i| i.resolution.is_some())
            .collect();
        
        if resolved_incidents.is_empty() {
            return (Duration::from_secs(0), Duration::from_secs(0));
        }
        
        // Calculate MTTR (Mean Time To Recovery)
        let total_recovery_time: Duration = resolved_incidents
            .iter()
            .map(|i| i.duration.unwrap_or(Duration::ZERO))
            .sum();
        
        let mttr = total_recovery_time / resolved_incidents.len() as u32;
        
        // Calculate MTBF (Mean Time Between Failures)
        if resolved_incidents.len() < 2 {
            return (Duration::from_secs(0), mttr);
        }
        
        let mut failure_intervals = Vec::new();
        for window in resolved_incidents.windows(2) {
            if let (Some(first), Some(second)) = (window[0].duration, window[1].duration) {
                failure_intervals.push(first + second);
            }
        }
        
        if failure_intervals.is_empty() {
            return (Duration::from_secs(0), mttr);
        }
        
        let total_interval: Duration = failure_intervals.iter().sum();
        let mtbf = total_interval / failure_intervals.len() as u32;
        
        (mtbf, mttr)
    }
    
    fn calculate_availability_sla(&self) -> f64 {
        self.sla_compliance
            .get("availability")
            .map(|c| c.compliance_percent)
            .unwrap_or(100.0)
    }
    
    fn calculate_performance_sla(&self) -> f64 {
        self.sla_compliance
            .get("latency")
            .map(|c| c.compliance_percent)
            .unwrap_or(100.0)
    }
    
    async fn check_sla_compliance(&mut self) {
        for (sla_name, compliance) in &mut self.sla_compliance {
            // Check if compliance has degraded
            if compliance.compliance_percent < 95.0 {
                compliance.status = ComplianceStatus::Critical;
                
                // Log SLA violation
                if let Some(logger) = &self.logger {
                    let metadata = HashMap::from([
                        ("sla_name".to_string(), sla_name.clone()),
                        ("compliance_percent".to_string(), compliance.compliance_percent.to_string()),
                        ("status".to_string(), format!("{:?}", compliance.status)),
                    ]);
                    
                    logger.log(
                        Uuid::new_v4(),
                        "reliability_assessment",
                        crate::observability::LogLevel::Error,
                        "SLA violation detected",
                        metadata
                    ).await;
                }
            } else if compliance.compliance_percent < 98.0 {
                compliance.status = ComplianceStatus::Warning;
            } else {
                compliance.status = ComplianceStatus::Compliant;
            }
        }
    }
    
    fn cleanup_old_failures(&mut self) {
        let cutoff = Utc::now() - self.assessment_config.failure_retention_period;
        self.failure_modes.retain(|_| true); // Simplified - would need timestamp in FailureMode
    }
    
    fn cleanup_old_metrics(&mut self) {
        let cutoff = Utc::now() - self.assessment_config.metrics_retention_period;
        self.metrics_history.retain(|m| m.timestamp > cutoff);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_failure_mode_severity() {
        let failure = FailureMode::ProviderOutage {
            provider: LlmProvider::Ollama,
            duration: Duration::from_secs(600),
            error_count: 10,
        };
        
        assert_eq!(failure.severity(), Severity::Critical);
    }
    
    #[test]
    fn test_sla_definition_creation() {
        let config = AssessmentConfig::default();
        let assessment = ReliabilityAssessment::new(config);
        
        assert_eq!(assessment.sla_definitions.len(), 3);
        assert!(assessment.sla_definitions.iter().any(|s| s.name == "availability"));
    }
    
    #[test]
    fn test_incident_creation() {
        let config = AssessmentConfig::default();
        let mut assessment = ReliabilityAssessment::new(config);
        
        let failure = FailureMode::SystemOverload {
            load_factor: 0.95,
            cpu_usage: 0.90,
            memory_usage: 0.85,
        };
        
        // This should create an incident due to high severity
        // In a real test, we'd need to run this in an async context
        assert_eq!(failure.severity(), Severity::Critical);
    }
    
    #[test]
    fn test_mtbf_mttr_calculation() {
        let config = AssessmentConfig::default();
        let assessment = ReliabilityAssessment::new(config);
        
        // Test with no incidents
        let incidents: Vec<&Incident> = vec![];
        let (mtbf, mttr) = assessment.calculate_mtbf_mttr(&incidents);
        
        assert_eq!(mtbf, Duration::from_secs(0));
        assert_eq!(mttr, Duration::from_secs(0));
    }
}
