//! Reliability assessment framework for system health monitoring
//!
//! Provides failure mode classification, SLA compliance tracking, incident management,
//! and reliability metrics calculation.

mod assessment;
mod types;

pub use assessment::ReliabilityAssessment;
pub use types::{
    AssessmentConfig, ComplianceStatus, FailureMode, ImpactAssessment, Incident,
    IncidentResolution, ReliabilityMetrics, ResolutionType, Severity, SlaCompliance, SlaDefinition,
    SlaPenalty, SlaTarget, SlaThreshold, SlaViolation,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::LlmProvider;
    use chrono::Utc;
    use std::time::Duration;

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
        assert!(assessment
            .sla_definitions
            .iter()
            .any(|s| s.name == "availability"));
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

        assert_eq!(failure.severity(), Severity::Critical);
    }

    #[test]
    fn test_mtbf_mttr_calculation() {
        let config = AssessmentConfig::default();
        let assessment = ReliabilityAssessment::new(config);

        let incidents: Vec<&Incident> = vec![];
        let (mtbf, mttr) = assessment.calculate_mtbf_mttr(&incidents);

        assert_eq!(mtbf, Duration::from_secs(0));
        assert_eq!(mttr, Duration::from_secs(0));
    }
}
