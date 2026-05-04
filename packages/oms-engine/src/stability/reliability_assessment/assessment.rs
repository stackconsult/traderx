use chrono::Utc;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::interval;
use tracing::{info, warn};
use uuid::Uuid;

use super::types::{
    AssessmentConfig, ComplianceStatus, FailureMode, ImpactAssessment, Incident,
    IncidentResolution, ReliabilityMetrics, ResolutionType, Severity, SlaCompliance, SlaDefinition,
    SlaPenalty, SlaTarget, SlaThreshold,
};
use crate::observability::{AgentMetrics, StructuredLogger};

/// Main reliability assessment framework
pub struct ReliabilityAssessment {
    failure_modes: Vec<FailureMode>,
    pub sla_definitions: Vec<SlaDefinition>,
    sla_compliance: HashMap<String, SlaCompliance>,
    incidents: Vec<Incident>,
    metrics_history: Vec<ReliabilityMetrics>,
    assessment_config: AssessmentConfig,
    metrics: Option<std::sync::Arc<AgentMetrics>>,
    logger: Option<std::sync::Arc<StructuredLogger>>,
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
                },
            );
        }

        assessment
    }

    pub fn with_metrics(mut self, metrics: std::sync::Arc<AgentMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    pub fn with_logger(mut self, logger: std::sync::Arc<StructuredLogger>) -> Self {
        self.logger = Some(logger);
        self
    }

    pub async fn record_failure(&mut self, failure: FailureMode) {
        let severity = failure.severity();

        self.failure_modes.push(failure.clone());

        if severity >= self.assessment_config.auto_escalation_threshold {
            self.create_incident(failure.clone()).await;
        }

        self.update_sla_compliance_for_failure(&failure).await;

        if let Some(logger) = &self.logger {
            let metadata = HashMap::from([
                ("failure_type".to_string(), format!("{:?}", failure)),
                ("severity".to_string(), format!("{:?}", severity)),
                ("timestamp".to_string(), Utc::now().to_rfc3339()),
            ]);

            logger
                .log(
                    Uuid::new_v4(),
                    "reliability_assessment",
                    crate::observability::LogLevel::Warn,
                    "Failure mode recorded",
                    metadata,
                )
                .await;
        }

        if let Some(metrics) = &self.metrics {
            metrics.system_errors.inc();
        }

        self.cleanup_old_failures();
    }

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

        if let Some(logger) = &self.logger {
            let metadata = HashMap::from([
                ("incident_id".to_string(), incident.incident_id.to_string()),
                ("severity".to_string(), format!("{:?}", incident.severity)),
                ("failure_mode".to_string(), format!("{:?}", failure)),
            ]);

            logger
                .log(
                    incident.incident_id,
                    "reliability_assessment",
                    crate::observability::LogLevel::Error,
                    "Incident created",
                    metadata,
                )
                .await;
        }

        warn!(
            "Incident {} created for failure: {:?}",
            incident.incident_id, failure
        );
    }

    pub async fn resolve_incident(
        &mut self,
        incident_id: Uuid,
        resolution_type: ResolutionType,
        description: String,
        root_cause: Option<String>,
        preventive_measures: Vec<String>,
    ) {
        if let Some(incident) = self
            .incidents
            .iter_mut()
            .find(|i| i.incident_id == incident_id)
        {
            let resolution = IncidentResolution {
                timestamp: Utc::now(),
                resolution_type: resolution_type.clone(),
                description,
                root_cause,
                preventive_measures,
            };

            incident.duration = Some(
                Utc::now()
                    .signed_duration_since(incident.timestamp)
                    .to_std()
                    .unwrap_or(Duration::ZERO),
            );
            incident.resolution = Some(resolution);

            info!("Incident {} resolved", incident_id);

            if let Some(logger) = &self.logger {
                let metadata = HashMap::from([
                    ("incident_id".to_string(), incident_id.to_string()),
                    (
                        "resolution_type".to_string(),
                        format!("{:?}", resolution_type),
                    ),
                    (
                        "duration_ms".to_string(),
                        incident
                            .duration
                            .map(|d| d.as_millis())
                            .unwrap_or(0)
                            .to_string(),
                    ),
                ]);

                logger
                    .log(
                        incident_id,
                        "reliability_assessment",
                        crate::observability::LogLevel::Info,
                        "Incident resolved",
                        metadata,
                    )
                    .await;
            }
        }
    }

    pub async fn calculate_metrics(&mut self) -> ReliabilityMetrics {
        let now = Utc::now();
        let window_start = now - self.assessment_config.compliance_window;

        let recent_incidents: Vec<_> = self
            .incidents
            .iter()
            .filter(|i| i.timestamp > window_start)
            .collect();

        let total_requests = self.metrics_history.iter().map(|m| m.total_requests).sum();

        let successful_requests = self
            .metrics_history
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

        let (mtbf, mttr) = self.calculate_mtbf_mttr(&recent_incidents);

        let failure_rate_per_hour = if recent_incidents.len() > 0 {
            (recent_incidents.len() as f64 / self.assessment_config.compliance_window.as_secs_f64())
                * 3600.0
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

        self.metrics_history.push(metrics.clone());
        self.cleanup_old_metrics();

        metrics
    }

    pub async fn get_sla_compliance_report(&self) -> HashMap<String, &SlaCompliance> {
        self.sla_compliance
            .iter()
            .map(|(k, v)| (k.clone(), v))
            .collect()
    }

    pub fn get_active_incidents(&self) -> Vec<&Incident> {
        self.incidents
            .iter()
            .filter(|i| i.resolution.is_none())
            .collect()
    }

    pub fn get_failure_analysis(&self) -> HashMap<String, u32> {
        let mut analysis = HashMap::new();

        for failure in &self.failure_modes {
            let key = format!("{:?}", failure);
            *analysis.entry(key).or_insert(0) += 1;
        }

        analysis
    }

    pub async fn start_continuous_assessment(&mut self) {
        info!("Starting continuous reliability assessment");

        let mut interval = interval(self.assessment_config.assessment_interval);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let metrics = self.calculate_metrics().await;
                    self.check_sla_compliance().await;

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

                _ = tokio::signal::ctrl_c() => {
                    info!("Stopping reliability assessment");
                    break;
                }
            }
        }
    }

    fn default_sla_definitions() -> Vec<SlaDefinition> {
        vec![
            SlaDefinition {
                name: "availability".to_string(),
                description: "System availability SLA".to_string(),
                target: SlaTarget::Availability {
                    uptime_percent: 99.5,
                },
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
                    p99_ms: 1000.0,
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
                target: SlaTarget::ErrorRate {
                    max_error_rate: 1.0,
                },
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
        match failure {
            FailureMode::ProviderOutage { provider: _, .. } => ImpactAssessment {
                users_affected: 1000,
                requests_lost: 100,
                revenue_impact: Some(1000.0),
                customer_satisfaction_impact: Some(0.1),
            },
            FailureMode::SystemOverload { .. } => ImpactAssessment {
                users_affected: 500,
                requests_lost: 50,
                revenue_impact: Some(500.0),
                customer_satisfaction_impact: Some(0.05),
            },
            _ => ImpactAssessment {
                users_affected: 100,
                requests_lost: 10,
                revenue_impact: Some(100.0),
                customer_satisfaction_impact: Some(0.01),
            },
        }
    }

    fn get_affected_components(&self, failure: &FailureMode) -> Vec<String> {
        match failure {
            FailureMode::ModelPerformance { model, .. } => vec![format!("model:{}", model)],
            FailureMode::ProviderOutage { provider, .. } => {
                vec![format!("provider:{:?}", provider)]
            }
            FailureMode::MemoryExhaustion { component, .. } => vec![component.clone()],
            FailureMode::NetworkFailure { endpoint, .. } => vec![format!("network:{}", endpoint)],
            FailureMode::CircuitBreakerOpen { provider, .. } => {
                vec![format!("circuit_breaker:{:?}", provider)]
            }
            FailureMode::ContextCorruption { .. } => vec!["context_manager".to_string()],
            FailureMode::ResourceContention { resource, .. } => {
                vec![format!("resource:{}", resource)]
            }
            FailureMode::SystemOverload { .. } => vec!["system".to_string()],
            FailureMode::ConfigurationError { component, .. } => vec![component.clone()],
        }
    }

    async fn update_sla_compliance_for_failure(&mut self, failure: &FailureMode) {
        match failure {
            FailureMode::ProviderOutage { .. } => {
                if let Some(compliance) = self.sla_compliance.get_mut("availability") {
                    compliance.status = ComplianceStatus::Warning;
                    compliance.compliance_percent = 95.0;
                    compliance.last_updated = Utc::now();
                }
            }
            FailureMode::ModelPerformance { .. } => {
                if let Some(compliance) = self.sla_compliance.get_mut("latency") {
                    compliance.status = ComplianceStatus::Warning;
                    compliance.compliance_percent = 90.0;
                    compliance.last_updated = Utc::now();
                }
            }
            _ => {
                if let Some(compliance) = self.sla_compliance.get_mut("error_rate") {
                    compliance.status = ComplianceStatus::Warning;
                    compliance.compliance_percent = 98.0;
                    compliance.last_updated = Utc::now();
                }
            }
        }
    }

    pub fn calculate_mtbf_mttr(&self, incidents: &[&Incident]) -> (Duration, Duration) {
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

        let total_recovery_time: Duration = resolved_incidents
            .iter()
            .map(|i| i.duration.unwrap_or(Duration::ZERO))
            .sum();

        let mttr = total_recovery_time / resolved_incidents.len() as u32;

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
            if compliance.compliance_percent < 95.0 {
                compliance.status = ComplianceStatus::Critical;

                if let Some(logger) = &self.logger {
                    let metadata = HashMap::from([
                        ("sla_name".to_string(), sla_name.clone()),
                        (
                            "compliance_percent".to_string(),
                            compliance.compliance_percent.to_string(),
                        ),
                        ("status".to_string(), format!("{:?}", compliance.status)),
                    ]);

                    logger
                        .log(
                            Uuid::new_v4(),
                            "reliability_assessment",
                            crate::observability::LogLevel::Error,
                            "SLA violation detected",
                            metadata,
                        )
                        .await;
                }
            } else if compliance.compliance_percent < 98.0 {
                compliance.status = ComplianceStatus::Warning;
            } else {
                compliance.status = ComplianceStatus::Compliant;
            }
        }
    }

    fn cleanup_old_failures(&mut self) {
        let _cutoff = Utc::now() - self.assessment_config.failure_retention_period;
        self.failure_modes.retain(|_| true);
    }

    fn cleanup_old_metrics(&mut self) {
        let cutoff = Utc::now() - self.assessment_config.metrics_retention_period;
        self.metrics_history.retain(|m| m.timestamp > cutoff);
    }
}
