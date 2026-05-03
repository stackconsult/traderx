use std::sync::Arc;
use prometheus::{
    Counter, Gauge, Histogram, Registry,
    Opts, HistogramOpts,
};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

pub mod metrics;

#[derive(Clone)]
pub struct AgentMetrics {
    pub registry: Registry,
    pub llm_requests_total: Counter,
    pub llm_request_duration: Histogram,
    pub llm_tokens_used: Counter,
    pub ml_predictions_total: Counter,
    pub ml_inference_duration: Histogram,
    pub ml_cache_hits: Counter,
    pub neural_inferences_total: Counter,
    pub neural_inference_duration: Histogram,
    pub agent_routing_decisions: Counter,
    pub active_agents: Gauge,
    pub system_errors: Counter,
}

impl AgentMetrics {
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();
        
        let llm_requests_total = Counter::with_opts(
            Opts::new("llm_requests_total", "Total LLM API requests")
        )?;
        registry.register(Box::new(llm_requests_total.clone()))?;
        
        let llm_request_duration = Histogram::with_opts(
            HistogramOpts::new("llm_request_duration_seconds", "LLM request latency")
                .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0])
        )?;
        registry.register(Box::new(llm_request_duration.clone()))?;
        
        let llm_tokens_used = Counter::with_opts(
            Opts::new("llm_tokens_used_total", "Total tokens consumed")
        )?;
        registry.register(Box::new(llm_tokens_used.clone()))?;
        
        let ml_predictions_total = Counter::with_opts(
            Opts::new("ml_predictions_total", "Total ML predictions")
        )?;
        registry.register(Box::new(ml_predictions_total.clone()))?;
        
        let ml_inference_duration = Histogram::with_opts(
            HistogramOpts::new("ml_inference_duration_seconds", "ML inference latency")
                .buckets(vec![0.0001, 0.001, 0.005, 0.01, 0.05, 0.1, 0.5])
        )?;
        registry.register(Box::new(ml_inference_duration.clone()))?;
        
        let ml_cache_hits = Counter::with_opts(
            Opts::new("ml_cache_hits_total", "ML cache hits")
        )?;
        registry.register(Box::new(ml_cache_hits.clone()))?;
        
        let neural_inferences_total = Counter::with_opts(
            Opts::new("neural_inferences_total", "Total neural inferences")
        )?;
        registry.register(Box::new(neural_inferences_total.clone()))?;
        
        let neural_inference_duration = Histogram::with_opts(
            HistogramOpts::new("neural_inference_duration_seconds", "Neural inference latency")
                .buckets(vec![0.00001, 0.0001, 0.001, 0.005, 0.01, 0.05])
        )?;
        registry.register(Box::new(neural_inference_duration.clone()))?;
        
        let agent_routing_decisions = Counter::with_opts(
            Opts::new("agent_routing_decisions_total", "Agent routing decisions")
        )?;
        registry.register(Box::new(agent_routing_decisions.clone()))?;
        
        let active_agents = Gauge::with_opts(
            Opts::new("active_agents", "Currently active agents")
        )?;
        registry.register(Box::new(active_agents.clone()))?;
        
        let system_errors = Counter::with_opts(
            Opts::new("system_errors_total", "Total system errors")
        )?;
        registry.register(Box::new(system_errors.clone()))?;
        
        Ok(Self {
            registry,
            llm_requests_total,
            llm_request_duration,
            llm_tokens_used,
            ml_predictions_total,
            ml_inference_duration,
            ml_cache_hits,
            neural_inferences_total,
            neural_inference_duration,
            agent_routing_decisions,
            active_agents,
            system_errors,
        })
    }
}

pub struct StructuredLogger {
    correlation_map: Arc<RwLock<HashMap<Uuid, Vec<LogEntry>>>>,
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub level: LogLevel,
    pub component: String,
    pub message: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl StructuredLogger {
    pub fn new() -> Self {
        Self {
            correlation_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn log(&self, correlation_id: Uuid, component: &str, level: LogLevel, message: &str, metadata: HashMap<String, String>) {
        let entry = LogEntry {
            timestamp: Utc::now(),
            correlation_id,
            level,
            component: component.to_string(),
            message: message.to_string(),
            metadata,
        };
        
        let mut map = self.correlation_map.write().await;
        map.entry(correlation_id).or_insert_with(Vec::new).push(entry.clone());
        
        match &entry.level {
            LogLevel::Debug => debug!(
                correlation_id = %correlation_id,
                component = %component,
                ?entry.metadata,
                "{}", message
            ),
            LogLevel::Info => info!(
                correlation_id = %correlation_id,
                component = %component,
                ?entry.metadata,
                "{}", message
            ),
            LogLevel::Warn => warn!(
                correlation_id = %correlation_id,
                component = %component,
                ?entry.metadata,
                "{}", message
            ),
            LogLevel::Error => error!(
                correlation_id = %correlation_id,
                component = %component,
                ?entry.metadata,
                "{}", message
            ),
        }
    }

    pub async fn get_trace(&self, correlation_id: Uuid) -> Vec<LogEntry> {
        let map = self.correlation_map.read().await;
        map.get(&correlation_id).cloned().unwrap_or_default()
    }
}

#[derive(Clone, Debug)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Clone)]
pub struct HealthCheck {
    pub component: String,
    pub status: HealthStatus,
    pub last_check: DateTime<Utc>,
    pub latency_ms: u64,
    pub message: String,
}

pub struct HealthMonitor {
    checks: Arc<RwLock<HashMap<String, HealthCheck>>>,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            checks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn update(&self, component: &str, status: HealthStatus, latency_ms: u64, message: &str) {
        let mut checks = self.checks.write().await;
        checks.insert(component.to_string(), HealthCheck {
            component: component.to_string(),
            status,
            last_check: Utc::now(),
            latency_ms,
            message: message.to_string(),
        });
    }

    pub async fn get_status(&self, component: &str) -> Option<HealthCheck> {
        let checks = self.checks.read().await;
        checks.get(component).cloned()
    }

    pub async fn get_all_statuses(&self) -> Vec<HealthCheck> {
        let checks = self.checks.read().await;
        checks.values().cloned().collect()
    }

    pub async fn is_healthy(&self) -> bool {
        let checks = self.checks.read().await;
        checks.values().all(|c| matches!(c.status, HealthStatus::Healthy))
    }
}
