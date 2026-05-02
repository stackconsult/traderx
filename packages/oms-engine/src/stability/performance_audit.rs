use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::observability::{AgentMetrics, StructuredLogger};

/// Benchmark test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub benchmark_id: Uuid,
    pub test_name: String,
    pub component: String,
    pub timestamp: DateTime<Utc>,
    pub duration_ms: f64,
    pub success: bool,
    pub throughput_ops_per_sec: Option<f64>,
    pub latency_p50_ms: Option<f64>,
    pub latency_p95_ms: Option<f64>,
    pub latency_p99_ms: Option<f64>,
    pub error_rate: Option<f64>,
    pub memory_usage_mb: Option<f64>,
    pub cpu_usage_percent: Option<f64>,
    pub details: HashMap<String, String>,
}

/// Compliance check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheck {
    pub check_id: Uuid,
    pub check_name: String,
    pub category: ComplianceCategory,
    pub passed: bool,
    pub score: f64,
    pub threshold: f64,
    pub timestamp: DateTime<Utc>,
    pub details: String,
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceCategory {
    Performance,
    Reliability,
    Security,
    CodeQuality,
    Documentation,
    Testing,
}

/// Audit report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub report_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub benchmark_results: Vec<BenchmarkResult>,
    pub compliance_checks: Vec<ComplianceCheck>,
    pub overall_score: f64,
    pub passed_checks: u32,
    pub failed_checks: u32,
    pub recommendations: Vec<String>,
    pub critical_issues: Vec<String>,
}

/// Performance benchmark suite
pub struct PerformanceAuditor {
    benchmarks: HashMap<String, Arc<dyn Benchmark>>,
    compliance_rules: HashMap<String, ComplianceRule>,
    audit_history: Arc<RwLock<Vec<AuditReport>>>,
    metrics: Option<Arc<AgentMetrics>>,
    logger: Option<Arc<StructuredLogger>>,
}

#[derive(Debug, Clone)]
pub struct ComplianceRule {
    pub name: String,
    pub category: ComplianceCategory,
    pub threshold: f64,
    pub check_fn: fn(&PerformanceAuditor) -> ComplianceCheck,
}

#[async_trait::async_trait]
pub trait Benchmark: Send + Sync {
    async fn run(&self) -> BenchmarkResult;
    fn benchmark_name(&self) -> &str;
    fn component_name(&self) -> &str;
}

impl PerformanceAuditor {
    pub fn new() -> Self {
        let mut auditor = Self {
            benchmarks: HashMap::new(),
            compliance_rules: HashMap::new(),
            audit_history: Arc::new(RwLock::new(Vec::new())),
            metrics: None,
            logger: None,
        };

        auditor.register_default_benchmarks();
        auditor.register_default_compliance_rules();

        auditor
    }

    pub fn with_metrics(mut self, metrics: Arc<AgentMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    pub fn with_logger(mut self, logger: Arc<StructuredLogger>) -> Self {
        self.logger = Some(logger);
        self
    }

    fn register_default_benchmarks(&mut self) {
        // Register default benchmarks
        // In production, these would be actual benchmark implementations
    }

    fn register_default_compliance_rules(&mut self) {
        // Register default compliance rules
        self.compliance_rules.insert(
            "latency_sla".to_string(),
            ComplianceRule {
                name: "Latency SLA Compliance".to_string(),
                category: ComplianceCategory::Performance,
                threshold: 100.0, // 100ms max latency
                check_fn: |auditor| auditor.check_latency_sla(),
            }
        );

        self.compliance_rules.insert(
            "error_rate".to_string(),
            ComplianceRule {
                name: "Error Rate Compliance".to_string(),
                category: ComplianceCategory::Reliability,
                threshold: 1.0, // 1% max error rate
                check_fn: |auditor| auditor.check_error_rate(),
            }
        );

        self.compliance_rules.insert(
            "memory_usage".to_string(),
            ComplianceRule {
                name: "Memory Usage Compliance".to_string(),
                category: ComplianceCategory::Performance,
                threshold: 500.0, // 500MB max
                check_fn: |auditor| auditor.check_memory_usage(),
            }
        );
    }

    /// Register a custom benchmark
    pub async fn register_benchmark(&mut self, benchmark: Arc<dyn Benchmark>) {
        self.benchmarks.insert(
            benchmark.benchmark_name().to_string(),
            benchmark
        );
    }

    /// Run all benchmarks
    pub async fn run_benchmarks(&self) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();
        let start_time = Instant::now();

        info!("Starting benchmark suite with {} benchmarks", self.benchmarks.len());

        for (name, benchmark) in &self.benchmarks {
            let benchmark_start = Instant::now();
            debug!("Running benchmark: {}", name);

            let result = benchmark.run().await;
            
            let duration = benchmark_start.elapsed();
            info!(
                "Benchmark {} completed in {:.2}ms - Success: {}",
                name,
                duration.as_millis(),
                result.success
            );

            results.push(result);
        }

        let total_duration = start_time.elapsed();
        info!("Benchmark suite completed in {:.2}ms", total_duration.as_millis());

        results
    }

    /// Run all compliance checks
    pub async fn run_compliance_checks(&self) -> Vec<ComplianceCheck> {
        let mut checks = Vec::new();

        info!("Starting compliance checks with {} rules", self.compliance_rules.len());

        for (name, rule) in &self.compliance_rules {
            debug!("Running compliance check: {}", name);
            let check = (rule.check_fn)(self);
            info!(
                "Compliance check {} - Passed: {}, Score: {:.2}",
                name,
                check.passed,
                check.score
            );
            checks.push(check);
        }

        checks
    }

    /// Generate comprehensive audit report
    pub async fn generate_audit_report(&self) -> AuditReport {
        let benchmark_results = self.run_benchmarks().await;
        let compliance_checks = self.run_compliance_checks().await;

        let passed_checks = compliance_checks.iter().filter(|c| c.passed).count() as u32;
        let failed_checks = compliance_checks.len() as u32 - passed_checks;

        let overall_score = if compliance_checks.is_empty() {
            0.0
        } else {
            compliance_checks.iter().map(|c| c.score).sum::<f64>() / compliance_checks.len() as f64
        };

        let critical_issues = compliance_checks
            .iter()
            .filter(|c| !c.passed && c.score < 50.0)
            .map(|c| format!("{}: {}", c.check_name, c.details))
            .collect();

        let recommendations = self.generate_recommendations(&benchmark_results, &compliance_checks);

        let report = AuditReport {
            report_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            benchmark_results,
            compliance_checks,
            overall_score,
            passed_checks,
            failed_checks,
            recommendations,
            critical_issues,
        };

        // Store report in history
        let mut history = self.audit_history.write().await;
        history.push(report.clone());

        // Log audit completion
        if let Some(logger) = &self.logger {
            let metadata = HashMap::from([
                ("report_id".to_string(), report.report_id.to_string()),
                ("overall_score".to_string(), report.overall_score.to_string()),
                ("passed_checks".to_string(), report.passed_checks.to_string()),
                ("failed_checks".to_string(), report.failed_checks.to_string()),
                ("critical_issues".to_string(), report.critical_issues.len().to_string()),
            ]);

            logger.log(
                report.report_id,
                "performance_auditor",
                crate::observability::LogLevel::Info,
                "Audit report generated",
                metadata
            ).await;
        }

        report
    }

    /// Get audit history
    pub async fn get_audit_history(&self) -> Vec<AuditReport> {
        self.audit_history.read().await.clone()
    }

    /// Generate recommendations based on audit results
    fn generate_recommendations(
        &self,
        benchmarks: &[BenchmarkResult],
        checks: &[ComplianceCheck],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Analyze benchmark results
        for benchmark in benchmarks {
            if !benchmark.success {
                recommendations.push(format!(
                    "Fix failing benchmark: {} - {}",
                    benchmark.component,
                    benchmark.test_name
                ));
            }

            if let Some(latency_p95) = benchmark.latency_p95_ms {
                if latency_p95 > 200.0 {
                    recommendations.push(format!(
                        "Optimize {} latency - P95 is {:.2}ms",
                        benchmark.component,
                        latency_p95
                    ));
                }
            }

            if let Some(memory) = benchmark.memory_usage_mb {
                if memory > 400.0 {
                    recommendations.push(format!(
                        "Reduce memory usage in {} - {:.2}MB",
                        benchmark.component,
                        memory
                    ));
                }
            }
        }

        // Analyze compliance checks
        for check in checks {
            if !check.passed {
                if let Some(remediation) = &check.remediation {
                    recommendations.push(remediation.clone());
                } else {
                    recommendations.push(format!(
                        "Address compliance issue: {} - {}",
                        check.check_name,
                        check.details
                    ));
                }
            }
        }

        // General recommendations
        if recommendations.is_empty() {
            recommendations.push("System is performing well. Continue monitoring.".to_string());
        }

        recommendations
    }

    // Compliance check implementations
    fn check_latency_sla(&self) -> ComplianceCheck {
        // Mock implementation - in production, this would check actual metrics
        let current_latency = 85.0; // Mock value
        let threshold = 100.0;
        let passed = current_latency <= threshold;
        let score = if passed { (threshold / current_latency) * 100.0 } else { (current_latency / threshold) * 100.0 };

        ComplianceCheck {
            check_id: Uuid::new_v4(),
            check_name: "Latency SLA Compliance".to_string(),
            category: ComplianceCategory::Performance,
            passed,
            score: (score as f64).min(100.0),
            threshold,
            timestamp: Utc::now(),
            details: format!("Current latency: {:.2}ms (threshold: {:.2}ms)", current_latency, threshold),
            remediation: if !passed {
                Some("Optimize code paths to reduce latency".to_string())
            } else {
                None
            },
        }
    }

    fn check_error_rate(&self) -> ComplianceCheck {
        // Mock implementation
        let current_error_rate = 0.5; // 0.5%
        let threshold = 1.0;
        let passed = current_error_rate <= threshold;
        let score = if passed { (threshold / current_error_rate) * 100.0 } else { (current_error_rate / threshold) * 100.0 };

        ComplianceCheck {
            check_id: Uuid::new_v4(),
            check_name: "Error Rate Compliance".to_string(),
            category: ComplianceCategory::Reliability,
            passed,
            score: (score as f64).min(100.0),
            threshold,
            timestamp: Utc::now(),
            details: format!("Current error rate: {:.2}% (threshold: {:.2}%)", current_error_rate, threshold),
            remediation: if !passed {
                Some("Investigate and fix error sources".to_string())
            } else {
                None
            },
        }
    }

    fn check_memory_usage(&self) -> ComplianceCheck {
        // Mock implementation
        let current_memory = 350.0; // 350MB
        let threshold = 500.0;
        let passed = current_memory <= threshold;
        let score = if passed { (threshold / current_memory) * 100.0 } else { (current_memory / threshold) * 100.0 };

        ComplianceCheck {
            check_id: Uuid::new_v4(),
            check_name: "Memory Usage Compliance".to_string(),
            category: ComplianceCategory::Performance,
            passed,
            score: (score as f64).min(100.0),
            threshold,
            timestamp: Utc::now(),
            details: format!("Current memory: {:.2}MB (threshold: {:.2}MB)", current_memory, threshold),
            remediation: if !passed {
                Some("Optimize memory allocation and usage".to_string())
            } else {
                None
            },
        }
    }
}

impl Default for PerformanceAuditor {
    fn default() -> Self {
        Self::new()
    }
}

// Mock benchmark implementations
struct LatencyBenchmark;

#[async_trait::async_trait]
impl Benchmark for LatencyBenchmark {
    async fn run(&self) -> BenchmarkResult {
        let start = Instant::now();
        
        // Simulate work
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        let duration = start.elapsed();
        
        BenchmarkResult {
            benchmark_id: Uuid::new_v4(),
            test_name: "Message Bus Latency".to_string(),
            component: "LlmMessageBus".to_string(),
            timestamp: Utc::now(),
            duration_ms: duration.as_millis() as f64,
            success: true,
            throughput_ops_per_sec: Some(1000.0),
            latency_p50_ms: Some(45.0),
            latency_p95_ms: Some(85.0),
            latency_p99_ms: Some(120.0),
            error_rate: Some(0.0),
            memory_usage_mb: Some(50.0),
            cpu_usage_percent: Some(15.0),
            details: HashMap::new(),
        }
    }

    fn benchmark_name(&self) -> &str {
        "latency_benchmark"
    }

    fn component_name(&self) -> &str {
        "middleware"
    }
}

struct ThroughputBenchmark;

#[async_trait::async_trait]
impl Benchmark for ThroughputBenchmark {
    async fn run(&self) -> BenchmarkResult {
        let start = Instant::now();
        
        // Simulate work
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let duration = start.elapsed();
        
        BenchmarkResult {
            benchmark_id: Uuid::new_v4(),
            test_name: "Request Throughput".to_string(),
            component: "OllamaClient".to_string(),
            timestamp: Utc::now(),
            duration_ms: duration.as_millis() as f64,
            success: true,
            throughput_ops_per_sec: Some(500.0),
            latency_p50_ms: None,
            latency_p95_ms: None,
            latency_p99_ms: None,
            error_rate: Some(0.1),
            memory_usage_mb: Some(75.0),
            cpu_usage_percent: Some(25.0),
            details: HashMap::new(),
        }
    }

    fn benchmark_name(&self) -> &str {
        "throughput_benchmark"
    }

    fn component_name(&self) -> &str {
        "llm"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_auditor_creation() {
        let auditor = PerformanceAuditor::new();
        assert_eq!(auditor.compliance_rules.len(), 3);
    }

    #[tokio::test]
    async fn test_audit_report_generation() {
        let auditor = PerformanceAuditor::new();
        let report = auditor.generate_audit_report().await;
        
        assert_eq!(report.benchmark_results.len(), 0); // No benchmarks registered
        assert_eq!(report.compliance_checks.len(), 3);
        assert!(report.overall_score >= 0.0 && report.overall_score <= 100.0);
    }

    #[test]
    fn test_compliance_check_result() {
        let check = ComplianceCheck {
            check_id: Uuid::new_v4(),
            check_name: "Test Check".to_string(),
            category: ComplianceCategory::Performance,
            passed: true,
            score: 95.0,
            threshold: 90.0,
            timestamp: Utc::now(),
            details: "Test details".to_string(),
            remediation: None,
        };
        
        assert!(check.passed);
        assert_eq!(check.score, 95.0);
    }
}
