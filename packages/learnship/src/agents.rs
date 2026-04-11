//! Specialized agents for monitoring different aspects of the trading system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use tracing::{debug, error, info, warn};

/// Message types between agents and orchestrator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentMessage {
    /// Feature drift detected
    FeatureDrift {
        symbol: String,
        feature: String,
        psi_score: f64,
        timestamp: DateTime<Utc>,
    },
    /// Model performance degradation
    PerformanceDegradation {
        model_name: String,
        metric: String,
        current_value: f64,
        threshold: f64,
        timestamp: DateTime<Utc>,
    },
    /// Component health check failure
    HealthFailure {
        component: String,
        error: String,
        severity: HealthSeverity,
        timestamp: DateTime<Utc>,
    },
    /// Retraining recommendation
    RetrainRecommendation {
        model_name: String,
        reason: String,
        priority: Priority,
        timestamp: DateTime<Utc>,
    },
    /// Validation result
    ValidationResult {
        model_name: String,
        stage: ValidationStage,
        passed: bool,
        metrics: HashMap<String, f64>,
        timestamp: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    Drift,
    Performance,
    Health,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationStage {
    Backtest,
    Shadow,
    Paper,
}

/// Base trait for all agents.
pub trait Agent: Send + Sync {
    fn name(&self) -> &str;
    fn agent_type(&self) -> AgentType;
    async fn run(&mut self, tx: tokio::sync::mpsc::Sender<AgentMessage>) -> anyhow::Result<()>;
    async fn stop(&mut self) -> anyhow::Result<()>;
}

/// Feature drift monitoring agent.
pub struct DriftAgent {
    name: String,
    redis_url: String,
    symbols: Vec<String>,
    check_interval_secs: u64,
    psi_threshold: f64,
}

impl DriftAgent {
    pub fn new(
        name: String,
        redis_url: String,
        symbols: Vec<String>,
        check_interval_secs: u64,
        psi_threshold: f64,
    ) -> Self {
        Self {
            name,
            redis_url,
            symbols,
            check_interval_secs,
            psi_threshold,
        }
    }

    async fn check_drift(&self, symbol: &str) -> Vec<AgentMessage> {
        let mut messages = Vec::new();
        
        // Subscribe to drift alerts from feature store
        if let Ok(mut pubsub) = redis::Client::open(self.redis_url.as_str())
            .and_then(|c| c.get_async_connection())
            .and_then(|conn| async move {
                Ok(redis::aio::PubSub::new(conn).subscribe("feature_drift").await?)
            })
            .await
        {
            while let Ok(msg) = pubsub.recv().await {
                if let Ok(payload) = redis::from_redis_value::<Vec<u8>>(&msg) {
                    if let Ok(alert) = serde_json::from_slice::<serde_json::Value>(&payload) {
                        if alert["symbol"] == symbol {
                            let psi = alert["psi"].as_f64().unwrap_or(0.0);
                            if psi > self.psi_threshold {
                                messages.push(AgentMessage::FeatureDrift {
                                    symbol: symbol.to_string(),
                                    feature: alert["feature"].as_str().unwrap_or("").to_string(),
                                    psi_score: psi,
                                    timestamp: Utc::now(),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        messages
    }
}

impl Agent for DriftAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn agent_type(&self) -> AgentType {
        AgentType::Drift
    }

    async fn run(&mut self, tx: tokio::sync::mpsc::Sender<AgentMessage>) -> anyhow::Result<()> {
        info!("Starting drift agent: {}", self.name);
        
        let mut interval = tokio::time::interval(
            std::time::Duration::from_secs(self.check_interval_secs)
        );
        
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    for symbol in &self.symbols {
                        let messages = self.check_drift(symbol).await;
                        for msg in messages {
                            if let Err(e) = tx.send(msg).await {
                                error!("Failed to send drift message: {}", e);
                                return Ok(());
                            }
                        }
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    info!("Drift agent stopping");
                    break;
                }
            }
        }
        
        Ok(())
    }

    async fn stop(&mut self) -> anyhow::Result<()> {
        info!("Drift agent stopped");
        Ok(())
    }
}

/// Model performance monitoring agent.
pub struct PerformanceAgent {
    name: String,
    models: Vec<String>,
    check_interval_secs: u64,
    thresholds: HashMap<String, f64>, // metric -> threshold
}

impl PerformanceAgent {
    pub fn new(
        name: String,
        models: Vec<String>,
        check_interval_secs: u64,
        thresholds: HashMap<String, f64>,
    ) -> Self {
        Self {
            name,
            models,
            check_interval_secs,
            thresholds,
        }
    }

    async fn check_performance(&self, model_name: &str) -> Vec<AgentMessage> {
        let mut messages = Vec::new();
        
        // Fetch current metrics from MLflow or metrics store
        // This is a placeholder - in production, query actual metrics
        let current_metrics = self.fetch_model_metrics(model_name).await;
        
        for (metric, current_value) in current_metrics {
            if let Some(&threshold) = self.thresholds.get(&metric) {
                // Check if performance degraded (e.g., Sharpe ratio dropped)
                if metric == "sharpe_ratio" && current_value < threshold {
                    messages.push(AgentMessage::PerformanceDegradation {
                        model_name: model_name.to_string(),
                        metric,
                        current_value,
                        threshold,
                        timestamp: Utc::now(),
                    });
                }
                // Check if drawdown exceeded limit
                if metric == "max_drawdown" && current_value > threshold {
                    messages.push(AgentMessage::PerformanceDegradation {
                        model_name: model_name.to_string(),
                        metric,
                        current_value,
                        threshold,
                        timestamp: Utc::now(),
                    });
                }
            }
        }
        
        messages
    }

    async fn fetch_model_metrics(&self, model_name: &str) -> HashMap<String, f64> {
        // Placeholder implementation
        // In production, fetch from MLflow or Prometheus
        HashMap::from([
            ("sharpe_ratio".to_string(), 1.2),
            ("max_drawdown".to_string(), 0.15),
            ("win_rate".to_string(), 0.55),
        ])
    }
}

impl Agent for PerformanceAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn agent_type(&self) -> AgentType {
        AgentType::Performance
    }

    async fn run(&mut self, tx: tokio::sync::mpsc::Sender<AgentMessage>) -> anyhow::Result<()> {
        info!("Starting performance agent: {}", self.name);
        
        let mut interval = tokio::time::interval(
            std::time::Duration::from_secs(self.check_interval_secs)
        );
        
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    for model in &self.models {
                        let messages = self.check_performance(model).await;
                        for msg in messages {
                            if let Err(e) = tx.send(msg).await {
                                error!("Failed to send performance message: {}", e);
                                return Ok(());
                            }
                        }
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    info!("Performance agent stopping");
                    break;
                }
            }
        }
        
        Ok(())
    }

    async fn stop(&mut self) -> anyhow::Result<()> {
        info!("Performance agent stopped");
        Ok(())
    }
}

/// System health monitoring agent.
pub struct HealthAgent {
    name: String,
    components: Vec<String>,
    check_interval_secs: u64,
}

impl HealthAgent {
    pub fn new(
        name: String,
        components: Vec<String>,
        check_interval_secs: u64,
    ) -> Self {
        Self {
            name,
            components,
            check_interval_secs,
        }
    }

    async fn check_component_health(&self, component: &str) -> Option<AgentMessage> {
        // Perform health check based on component type
        let result = match component {
            c if c.starts_with("model-server") => self.check_model_server(c).await,
            c if c.starts_with("feature-store") => self.check_feature_store(c).await,
            c if c.starts_with("questdb") => self.check_questdb(c).await,
            c if c.starts_with("oms") => self.check_oms(c).await,
            _ => Ok(()),
        };
        
        if let Err(e) = result {
            Some(AgentMessage::HealthFailure {
                component: component.to_string(),
                error: e.to_string(),
                severity: self.determine_severity(&e),
                timestamp: Utc::now(),
            })
        } else {
            None
        }
    }

    async fn check_model_server(&self, endpoint: &str) -> anyhow::Result<()> {
        let url = format!("http://{}/health", endpoint);
        let response = reqwest::get(&url).await?;
        if response.status().is_success() {
            Ok(())
        } else {
            anyhow::bail!("Model server unhealthy: {}", response.status())
        }
    }

    async fn check_feature_store(&self, endpoint: &str) -> anyhow::Result<()> {
        let client = redis::Client::open(format!("redis://{}", endpoint))?;
        let mut conn = client.get_async_connection().await?;
        let _: String = redis::cmd("PING").query_async(&mut conn).await?;
        Ok(())
    }

    async fn check_questdb(&self, endpoint: &str) -> anyhow::Result<()> {
        let url = format!("http://{}/exec?query=SELECT%201", endpoint);
        let response = reqwest::get(&url).await?;
        if response.status().is_success() {
            Ok(())
        } else {
            anyhow::bail!("QuestDB unhealthy: {}", response.status())
        }
    }

    async fn check_oms(&self, endpoint: &str) -> anyhow::Result<()> {
        // Check OMS via gRPC health check or HTTP endpoint
        let url = format!("http://{}/health", endpoint);
        let response = reqwest::get(&url).await?;
        if response.status().is_success() {
            Ok(())
        } else {
            anyhow::bail!("OMS unhealthy: {}", response.status())
        }
    }

    fn determine_severity(&self, error: &anyhow::Error) -> HealthSeverity {
        let error_str = error.to_string().to_lowercase();
        if error_str.contains("critical") || error_str.contains("timeout") {
            HealthSeverity::Critical
        } else if error_str.contains("error") {
            HealthSeverity::High
        } else if error_str.contains("warning") {
            HealthSeverity::Medium
        } else {
            HealthSeverity::Low
        }
    }
}

impl Agent for HealthAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn agent_type(&self) -> AgentType {
        AgentType::Health
    }

    async fn run(&mut self, tx: tokio::sync::mpsc::Sender<AgentMessage>) -> anyhow::Result<()> {
        info!("Starting health agent: {}", self.name);
        
        let mut interval = tokio::time::interval(
            std::time::Duration::from_secs(self.check_interval_secs)
        );
        
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    for component in &self.components {
                        if let Some(msg) = self.check_component_health(component).await {
                            if let Err(e) = tx.send(msg).await {
                                error!("Failed to send health message: {}", e);
                                return Ok(());
                            }
                        }
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    info!("Health agent stopping");
                    break;
                }
            }
        }
        
        Ok(())
    }

    async fn stop(&mut self) -> anyhow::Result<()> {
        info!("Health agent stopped");
        Ok(())
    }
}
