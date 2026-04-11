//! Learnship orchestrator - meta-agent that coordinates specialized agents
//! and makes retraining/deployment decisions.

use crate::agents::{Agent, AgentMessage, AgentType, DriftAgent, PerformanceAgent, HealthAgent};
use crate::validation::ValidationPipeline;
use crate::experiment::ExperimentTracker;
use crate::healing::HealingEngine;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};

/// Orchestrator configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub redis_url: String,
    pub mlflow_uri: String,
    pub symbols: Vec<String>,
    pub models: Vec<String>,
    pub components: Vec<String>,
    pub drift_check_interval_secs: u64,
    pub performance_check_interval_secs: u64,
    pub health_check_interval_secs: u64,
    pub drift_psi_threshold: f64,
    pub performance_thresholds: HashMap<String, f64>,
    pub retrain_cooldown_mins: u64,
    pub max_concurrent_retrains: usize,
}

/// Decision made by the orchestrator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestratorDecision {
    TriggerRetraining {
        model_name: String,
        reason: String,
        priority: Priority,
    },
    DeployModel {
        model_name: String,
        version: String,
        validation_results: HashMap<String, f64>,
    },
    RollbackModel {
        model_name: String,
        reason: String,
    },
    HealComponent {
        component: String,
        action: HealingAction,
    },
    Ignore {
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Meta-orchestrator for the Learnship system.
pub struct LearnshipOrchestrator {
    config: OrchestratorConfig,
    agents: HashMap<String, Box<dyn Agent>>,
    message_rx: mpsc::Receiver<AgentMessage>,
    decision_tx: mpsc::Sender<OrchestratorDecision>,
    
    // State tracking
    drift_history: Arc<RwLock<HashMap<String, Vec<f64>>>>,
    performance_history: Arc<RwLock<HashMap<String, Vec<f64>>>>,
    retrain_cooldowns: Arc<RwLock<HashMap<String, chrono::DateTime<chrono::Utc>>>>,
    active_retrains: Arc<RwLock<HashSet<String>>>,
    
    // Subsystems
    validation_pipeline: Arc<ValidationPipeline>,
    experiment_tracker: Arc<ExperimentTracker>,
    healing_engine: Arc<HealingEngine>,
}

impl LearnshipOrchestrator {
    pub fn new(
        config: OrchestratorConfig,
        decision_tx: mpsc::Sender<OrchestratorDecision>,
    ) -> anyhow::Result<Arc<Self>> {
        let (message_tx, message_rx) = mpsc::channel(1000);
        
        // Initialize agents
        let mut agents: HashMap<String, Box<dyn Agent>> = HashMap::new();
        
        // Drift agent
        agents.insert(
            "drift_agent".to_string(),
            Box::new(DriftAgent::new(
                "drift_agent".to_string(),
                config.redis_url.clone(),
                config.symbols.clone(),
                config.drift_check_interval_secs,
                config.drift_psi_threshold,
            )),
        );
        
        // Performance agent
        agents.insert(
            "performance_agent".to_string(),
            Box::new(PerformanceAgent::new(
                "performance_agent".to_string(),
                config.models.clone(),
                config.performance_check_interval_secs,
                config.performance_thresholds.clone(),
            )),
        );
        
        // Health agent
        agents.insert(
            "health_agent".to_string(),
            Box::new(HealthAgent::new(
                "health_agent".to_string(),
                config.components.clone(),
                config.health_check_interval_secs,
            )),
        );
        
        // Initialize subsystems
        let validation_pipeline = Arc::new(ValidationPipeline::new(
            config.mlflow_uri.clone(),
            config.redis_url.clone(),
        )?);
        
        let experiment_tracker = Arc::new(ExperimentTracker::new(
            config.mlflow_uri.clone(),
        )?);
        
        let healing_engine = Arc::new(HealingEngine::new());
        
        Ok(Arc::new(Self {
            config,
            agents,
            message_rx,
            decision_tx,
            drift_history: Arc::new(RwLock::new(HashMap::new())),
            performance_history: Arc::new(RwLock::new(HashMap::new())),
            retrain_cooldowns: Arc::new(RwLock::new(HashMap::new())),
            active_retrains: Arc::new(RwLock::new(HashSet::new())),
            validation_pipeline,
            experiment_tracker,
            healing_engine,
        }))
    }

    /// Start the orchestrator and all agents.
    pub async fn start(self: Arc<Self>) -> anyhow::Result<()> {
        info!("Starting Learnship orchestrator...");
        
        // Start all agents
        let mut agent_tasks = Vec::new();
        for (name, mut agent) in self.agents.into_iter() {
            let (tx, rx) = mpsc::channel(100);
            let orchestrator = Arc::clone(&self);
            
            // Agent task
            let task = tokio::spawn(async move {
                if let Err(e) = agent.run(tx).await {
                    error!("Agent {} failed: {}", name, e);
                }
            });
            agent_tasks.push(task);
            
            // Message forwarder task
            let orchestrator_clone = Arc::clone(&self);
            let forward_task = tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    if let Err(e) = orchestrator_clone.process_message(msg).await {
                        error!("Failed to process message: {}", e);
                    }
                }
            });
            agent_tasks.push(forward_task);
        }
        
        // Start orchestrator message processing
        let orchestrator = Arc::clone(&self);
        let process_task = tokio::spawn(async move {
            orchestrator.run_message_loop().await;
        });
        
        // Start periodic analysis
        let orchestrator = Arc::clone(&self);
        let analysis_task = tokio::spawn(async move {
            orchestrator.run_periodic_analysis().await;
        });
        
        // Wait for shutdown
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("Shutting down orchestrator...");
            }
            _ = async { tokio::try_join!(process_task, analysis_task) } => {
                error!("Orchestrator tasks exited unexpectedly");
            }
        }
        
        // Cleanup agents
        for task in agent_tasks {
            task.abort();
        }
        
        Ok(())
    }

    /// Process incoming messages from agents.
    async fn process_message(&self, message: AgentMessage) -> anyhow::Result<()> {
        match message {
            AgentMessage::FeatureDrift { symbol, feature, psi_score, timestamp } => {
                self.handle_feature_drift(symbol, feature, psi_score, timestamp).await?;
            }
            AgentMessage::PerformanceDegradation { model_name, metric, current_value, threshold, timestamp } => {
                self.handle_performance_degradation(model_name, metric, current_value, threshold, timestamp).await?;
            }
            AgentMessage::HealthFailure { component, error, severity, timestamp } => {
                self.handle_health_failure(component, error, severity, timestamp).await?;
            }
            AgentMessage::RetrainRecommendation { model_name, reason, priority, timestamp } => {
                self.handle_retrain_recommendation(model_name, reason, priority, timestamp).await?;
            }
            AgentMessage::ValidationResult { model_name, stage, passed, metrics, timestamp } => {
                self.handle_validation_result(model_name, stage, passed, metrics, timestamp).await?;
            }
        }
        Ok(())
    }

    async fn handle_feature_drift(
        &self,
        symbol: String,
        feature: String,
        psi_score: f64,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> anyhow::Result<()> {
        warn!("Feature drift detected: {}:{} PSI={}", symbol, feature, psi_score);
        
        // Track drift history
        let mut history = self.drift_history.write().await;
        let key = format!("{}:{}", symbol, feature);
        history.entry(key).or_insert_with(Vec::new).push(psi_score);
        
        // Check if drift is persistent (multiple detections)
        if let Some(scores) = history.get(&key) {
            if scores.len() >= 3 && scores.iter().rev().take(3).all(|&s| s > self.config.drift_psi_threshold) {
                // Persistent drift - recommend retraining for models using this feature
                for model in &self.config.models {
                    if self.model_uses_feature(model, &feature) {
                        let decision = OrchestratorDecision::TriggerRetraining {
                            model_name: model.clone(),
                            reason: format!("Persistent feature drift in {}:{}", symbol, feature),
                            priority: Priority::High,
                        };
                        if let Err(e) = self.decision_tx.send(decision).await {
                            error!("Failed to send retraining decision: {}", e);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    async fn handle_performance_degradation(
        &self,
        model_name: String,
        metric: String,
        current_value: f64,
        threshold: f64,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> anyhow::Result<()> {
        warn!("Performance degradation: {} {}={} < {}", model_name, metric, current_value, threshold);
        
        // Track performance history
        let mut history = self.performance_history.write().await;
        history.entry(model_name.clone())
            .or_insert_with(Vec::new)
            .push(current_value);
        
        // Check if in cooldown
        let cooldowns = self.retrain_cooldowns.read().await;
        if let Some(&last_retrain) = cooldowns.get(&model_name) {
            let cooldown_end = last_retrain + chrono::Duration::minutes(self.config.retrain_cooldown_mins as i64);
            if chrono::Utc::now() < cooldown_end {
                debug!("Model {} in cooldown, skipping retrain", model_name);
                return Ok(());
            }
        }
        
        // Check if we have capacity for retraining
        let active_retrains = self.active_retrains.read().await;
        if active_retrains.len() >= self.config.max_concurrent_retrains {
            warn!("Max concurrent retrains reached, queuing {}", model_name);
            return Ok(());
        }
        drop(active_retrains);
        
        // Trigger retraining
        let decision = OrchestratorDecision::TriggerRetraining {
            model_name: model_name.clone(),
            reason: format!("Performance degradation: {} {} < {}", metric, current_value, threshold),
            priority: Priority::Medium,
        };
        
        if let Err(e) = self.decision_tx.send(decision).await {
            error!("Failed to send retraining decision: {}", e);
        } else {
            // Add to active retrains
            let mut active = self.active_retrains.write().await;
            active.insert(model_name.clone());
            
            // Set cooldown
            let mut cooldowns = self.retrain_cooldowns.write().await;
            cooldowns.insert(model_name, chrono::Utc::now());
        }
        
        Ok(())
    }

    async fn handle_health_failure(
        &self,
        component: String,
        error: String,
        severity: crate::agents::HealthSeverity,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> anyhow::Result<()> {
        error!("Health failure: {} - {}", component, error);
        
        // Use healing engine to determine action
        let action = self.healing_engine.determine_action(&component, &error, &severity).await;
        
        let decision = OrchestratorDecision::HealComponent {
            component: component.clone(),
            action,
        };
        
        if let Err(e) = self.decision_tx.send(decision).await {
            error!("Failed to send healing decision: {}", e);
        }
        
        Ok(())
    }

    async fn handle_retrain_recommendation(
        &self,
        model_name: String,
        reason: String,
        priority: crate::agents::Priority,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> anyhow::Result<()> {
        info!("Retrain recommendation: {} - {}", model_name, reason);
        
        // Create experiment for retraining
        let experiment_id = self.experiment_tracker
            .create_experiment(&model_name, &reason).await?;
        
        // Start validation pipeline
        let validation_result = self.validation_pipeline
            .validate_model(&model_name, &experiment_id).await?;
        
        if validation_result.passed {
            let decision = OrchestratorDecision::DeployModel {
                model_name,
                version: validation_result.new_version,
                validation_results: validation_result.metrics,
            };
            
            if let Err(e) = self.decision_tx.send(decision).await {
                error!("Failed to send deployment decision: {}", e);
            }
        } else {
            warn!("Model {} failed validation: {:?}", model_name, validation_result.metrics);
        }
        
        Ok(())
    }

    async fn handle_validation_result(
        &self,
        model_name: String,
        stage: crate::agents::ValidationStage,
        passed: bool,
        metrics: HashMap<String, f64>,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> anyhow::Result<()> {
        debug!("Validation result: {} {:?} - passed: {}", model_name, stage, passed);
        
        // Update experiment tracking
        self.experiment_tracker
            .log_validation_result(&model_name, stage, passed, &metrics).await?;
        
        // If final stage passed, remove from active retrains
        if matches!(stage, crate::agents::ValidationStage::Paper) && passed {
            let mut active = self.active_retrains.write().await;
            active.remove(&model_name);
        }
        
        Ok(())
    }

    async fn run_message_loop(&self) {
        while let Some(message) = self.message_rx.recv().await {
            if let Err(e) = self.process_message(message).await {
                error!("Error processing message: {}", e);
            }
        }
    }

    async fn run_periodic_analysis(&self) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 minutes
        
        loop {
            interval.tick().await;
            
            // Analyze drift trends
            self.analyze_drift_trends().await;
            
            // Analyze performance trends
            self.analyze_performance_trends().await;
            
            // Clean up old history
            self.cleanup_history().await;
        }
    }

    async fn analyze_drift_trends(&self) {
        let history = self.drift_history.read().await;
        for (key, scores) in history.iter() {
            if scores.len() >= 10 {
                let recent_avg = scores.iter().rev().take(5).sum::<f64>() / 5.0;
                let older_avg = scores.iter().rev().skip(5).take(5).sum::<f64>() / 5.0;
                
                if recent_avg > older_avg * 1.5 {
                    warn!("Drift accelerating for {}", key);
                }
            }
        }
    }

    async fn analyze_performance_trends(&self) {
        let history = self.performance_history.read().await;
        for (model, scores) in history.iter() {
            if scores.len() >= 10 {
                let recent_avg = scores.iter().rev().take(5).sum::<f64>() / 5.0;
                let older_avg = scores.iter().rev().skip(5).take(5).sum::<f64>() / 5.0;
                
                if recent_avg < older_avg * 0.8 {
                    warn!("Performance declining for {}", model);
                }
            }
        }
    }

    async fn cleanup_history(&self) {
        let max_history = 100;
        
        {
            let mut history = self.drift_history.write().await;
            for scores in history.values_mut() {
                if scores.len() > max_history {
                    scores.drain(0..scores.len() - max_history);
                }
            }
        }
        
        {
            let mut history = self.performance_history.write().await;
            for scores in history.values_mut() {
                if scores.len() > max_history {
                    scores.drain(0..scores.len() - max_history);
                }
            }
        }
    }

    fn model_uses_feature(&self, model: &str, feature: &str) -> bool {
        // In production, this would check model metadata or feature importance
        // For now, assume all models use all features
        true
    }
}

use crate::healing::HealingAction;
