use std::sync::Arc;
use std::collections::HashMap;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::debug;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::llm::LlmRequest;
use crate::observability::{AgentMetrics, StructuredLogger};

/// Model performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformanceMetrics {
    pub model_name: String,
    pub provider: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub avg_confidence: f64,
    pub success_rate: f64,
    pub last_used: DateTime<Utc>,
    pub last_success: DateTime<Utc>,
    pub last_failure: DateTime<Utc>,
}

impl ModelPerformanceMetrics {
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.successful_requests as f64 / self.total_requests as f64) * 100.0
        }
    }

    pub fn error_rate(&self) -> f64 {
        100.0 - self.success_rate()
    }

    pub fn health_score(&self) -> f64 {
        let success_weight = 0.4;
        let latency_weight = 0.3;
        let confidence_weight = 0.3;

        let success_score = self.success_rate() / 100.0;
        let latency_score = (1000.0 / (self.avg_latency_ms + 1.0)).min(1.0);
        let confidence_score = self.avg_confidence;

        success_score * success_weight
            + latency_score * latency_weight
            + confidence_score * confidence_weight
    }
}

/// Model selection criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionCriteria {
    pub min_confidence: f64,
    pub max_latency_ms: f64,
    pub min_success_rate: f64,
    pub cost_weight: f64,
    pub performance_weight: f64,
    pub reliability_weight: f64,
}

impl Default for SelectionCriteria {
    fn default() -> Self {
        Self {
            min_confidence: 0.5,
            max_latency_ms: 1000.0,
            min_success_rate: 90.0,
            cost_weight: 0.2,
            performance_weight: 0.5,
            reliability_weight: 0.3,
        }
    }
}

/// Model selection decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSelectionDecision {
    pub selected_model: String,
    pub selected_provider: String,
    pub reasoning: String,
    pub confidence: f64,
    pub estimated_latency_ms: f64,
    pub estimated_cost: f64,
    pub alternative_models: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

/// Conviction-based routing configuration
#[derive(Debug, Clone)]
pub struct ConvictionConfig {
    pub low_conviction_threshold: f64,      // < 0.3: use fastest, cheapest
    pub medium_conviction_threshold: f64,   // 0.3-0.7: use balanced
    pub high_conviction_threshold: f64,     // > 0.7: use best quality
    pub extreme_conviction_threshold: f64,  // > 0.9: use premium
}

impl Default for ConvictionConfig {
    fn default() -> Self {
        Self {
            low_conviction_threshold: 0.3,
            medium_conviction_threshold: 0.7,
            high_conviction_threshold: 0.9,
            extreme_conviction_threshold: 0.95,
        }
    }
}

/// Dynamic model selector
pub struct DynamicModelSelector {
    model_metrics: Arc<RwLock<HashMap<String, ModelPerformanceMetrics>>>,
    selection_criteria: SelectionCriteria,
    conviction_config: ConvictionConfig,
    available_models: Vec<String>,
    model_costs: HashMap<String, f64>, // cost per 1K tokens
    metrics: Option<Arc<AgentMetrics>>,
    logger: Option<Arc<StructuredLogger>>,
}

impl DynamicModelSelector {
    pub fn new(
        available_models: Vec<String>,
        model_costs: HashMap<String, f64>,
    ) -> Self {
        let mut selector = Self {
            model_metrics: Arc::new(RwLock::new(HashMap::new())),
            selection_criteria: SelectionCriteria::default(),
            conviction_config: ConvictionConfig::default(),
            available_models,
            model_costs,
            metrics: None,
            logger: None,
        };

        // Initialize metrics for all models
        selector.initialize_model_metrics();

        selector
    }

    pub fn with_selection_criteria(mut self, criteria: SelectionCriteria) -> Self {
        self.selection_criteria = criteria;
        self
    }

    pub fn with_conviction_config(mut self, config: ConvictionConfig) -> Self {
        self.conviction_config = config;
        self
    }

    pub fn with_metrics(mut self, metrics: Arc<AgentMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    pub fn with_logger(mut self, logger: Arc<StructuredLogger>) -> Self {
        self.logger = Some(logger);
        self
    }

    fn initialize_model_metrics(&mut self) {
        let mut metrics = HashMap::new();
        
        for model in &self.available_models {
            let provider = self.extract_provider(model);
            metrics.insert(
                model.clone(),
                ModelPerformanceMetrics {
                    model_name: model.clone(),
                    provider,
                    total_requests: 0,
                    successful_requests: 0,
                    failed_requests: 0,
                    avg_latency_ms: 0.0,
                    p50_latency_ms: 0.0,
                    p95_latency_ms: 0.0,
                    p99_latency_ms: 0.0,
                    avg_confidence: 0.0,
                    success_rate: 100.0,
                    last_used: Utc::now(),
                    last_success: Utc::now(),
                    last_failure: Utc::now(),
                }
            );
        }
        
        // In a real implementation, we'd write this to the Arc<RwLock>
        // For now, this is a placeholder
    }

    fn extract_provider(&self, model: &str) -> String {
        if model.contains("ollama") || model.contains("deepseek") || model.contains("gemma") {
            "ollama".to_string()
        } else if model.contains("gpt") {
            "openai".to_string()
        } else if model.contains("claude") {
            "anthropic".to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// Select model based on conviction and performance
    pub async fn select_model(
        &self,
        request: &LlmRequest,
        conviction: f64,
    ) -> ModelSelectionDecision {
        let _start_time = Instant::now();
        
        // Get current metrics
        let metrics = self.model_metrics.read().await;
        
        // Determine selection strategy based on conviction
        let (selected_model, reasoning) = match conviction {
            c if c < self.conviction_config.low_conviction_threshold => {
                self.select_for_low_conviction(&metrics, request).await
            },
            c if c < self.conviction_config.medium_conviction_threshold => {
                self.select_for_medium_conviction(&metrics, request).await
            },
            c if c < self.conviction_config.high_conviction_threshold => {
                self.select_for_high_conviction(&metrics, request).await
            },
            _ => {
                self.select_for_extreme_conviction(&metrics, request).await
            }
        };

        let selected_provider = self.extract_provider(&selected_model);
        let estimated_latency = metrics.get(&selected_model)
            .map(|m| m.avg_latency_ms)
            .unwrap_or(500.0);
        
        let estimated_cost = self.model_costs
            .get(&selected_model)
            .copied()
            .unwrap_or(0.0);

        let confidence = self.calculate_selection_confidence(&metrics, &selected_model).await;

        let alternative_models = self.get_alternative_models(&selected_model, conviction);

        let decision = ModelSelectionDecision {
            selected_model,
            selected_provider,
            reasoning,
            confidence,
            estimated_latency_ms: estimated_latency,
            estimated_cost,
            alternative_models,
            timestamp: Utc::now(),
        };

        // Log selection
        if let Some(logger) = &self.logger {
            let metadata = HashMap::from([
                ("selected_model".to_string(), decision.selected_model.clone()),
                ("selected_provider".to_string(), decision.selected_provider.clone()),
                ("reasoning".to_string(), decision.reasoning.clone()),
                ("confidence".to_string(), decision.confidence.to_string()),
                ("conviction".to_string(), conviction.to_string()),
                ("estimated_latency_ms".to_string(), decision.estimated_latency_ms.to_string()),
            ]);

            logger.log(
                request.request_id,
                "dynamic_model_selector",
                crate::observability::LogLevel::Info,
                "Model selection made",
                metadata
            ).await;
        }

        debug!(
            "Model selection: model={}, provider={}, confidence={:.2}, conviction={:.2}",
            decision.selected_model, decision.selected_provider, decision.confidence, conviction
        );

        decision
    }

    /// Record model usage and performance
    pub async fn record_usage(
        &self,
        model: &str,
        latency_ms: f64,
        success: bool,
        confidence: f64,
    ) {
        let mut metrics = self.model_metrics.write().await;
        
        let entry = metrics.entry(model.to_string()).or_insert_with(|| ModelPerformanceMetrics {
            model_name: model.to_string(),
            provider: self.extract_provider(model),
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_latency_ms: 0.0,
            p50_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            avg_confidence: 0.0,
            success_rate: 100.0,
            last_used: Utc::now(),
            last_success: Utc::now(),
            last_failure: Utc::now(),
        });

        // Update metrics with exponential moving average
        entry.total_requests += 1;
        
        if success {
            entry.successful_requests += 1;
            entry.last_success = Utc::now();
        } else {
            entry.failed_requests += 1;
            entry.last_failure = Utc::now();
        }

        // Update latency with EMA
        let alpha = 0.2; // smoothing factor
        entry.avg_latency_ms = if entry.avg_latency_ms == 0.0 {
            latency_ms
        } else {
            alpha * latency_ms + (1.0 - alpha) * entry.avg_latency_ms
        };

        // Update confidence with EMA
        entry.avg_confidence = if entry.avg_confidence == 0.0 {
            confidence
        } else {
            alpha * confidence + (1.0 - alpha) * entry.avg_confidence
        };

        entry.success_rate = entry.success_rate();
        entry.last_used = Utc::now();

        // Record to metrics
        if let Some(metrics) = &self.metrics {
            metrics.llm_request_duration.observe(latency_ms / 1000.0);
            if success {
                metrics.llm_requests_total.inc();
            } else {
                metrics.system_errors.inc();
            }
        }
    }

    /// Get model performance report
    pub async fn get_performance_report(&self) -> HashMap<String, ModelPerformanceMetrics> {
        self.model_metrics.read().await.clone()
    }

    /// Get best performing model
    pub async fn get_best_model(&self) -> Option<String> {
        let metrics = self.model_metrics.read().await;
        
        metrics.iter()
            .filter(|(_, m)| m.success_rate() >= self.selection_criteria.min_success_rate)
            .filter(|(_, m)| m.avg_latency_ms <= self.selection_criteria.max_latency_ms)
            .max_by(|a, b| a.1.health_score().partial_cmp(&b.1.health_score()).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, _)| name.clone())
    }

    // Selection strategy implementations
    async fn select_for_low_conviction(
        &self,
        metrics: &HashMap<String, ModelPerformanceMetrics>,
        _request: &LlmRequest,
    ) -> (String, String) {
        // For low conviction, select fastest and cheapest
        let best = metrics.iter()
            .filter(|(_, m)| m.avg_latency_ms < 500.0)
            .min_by(|a, b| {
                let cost_a = self.model_costs.get(a.0).copied().unwrap_or(1.0);
                let cost_b = self.model_costs.get(b.0).copied().unwrap_or(1.0);
                let latency_a = a.1.avg_latency_ms;
                let latency_b = b.1.avg_latency_ms;
                
                let score_a = cost_a * 0.5 + latency_a * 0.5;
                let score_b = cost_b * 0.5 + latency_b * 0.5;
                
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            });

        match best {
            Some((model, _)) => (
                model.clone(),
                format!("Low conviction: selected fastest/cheapest model {}", model)
            ),
            None => (
                self.available_models.first().unwrap_or(&"deepseek-coder:1.3b".to_string()).clone(),
                "Low conviction: using default model".to_string()
            )
        }
    }

    async fn select_for_medium_conviction(
        &self,
        metrics: &HashMap<String, ModelPerformanceMetrics>,
        _request: &LlmRequest,
    ) -> (String, String) {
        // For medium conviction, balance cost and performance
        let best = metrics.iter()
            .filter(|(_, m)| m.health_score() > 0.5)
            .max_by(|a, b| {
                let score_a = a.1.health_score();
                let score_b = b.1.health_score();
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            });

        match best {
            Some((model, _)) => (
                model.clone(),
                format!("Medium conviction: selected balanced model {}", model)
            ),
            None => (
                self.available_models.get(1).unwrap_or(&"deepseek-coder:1.3b".to_string()).clone(),
                "Medium conviction: using fallback model".to_string()
            )
        }
    }

    async fn select_for_high_conviction(
        &self,
        metrics: &HashMap<String, ModelPerformanceMetrics>,
        _request: &LlmRequest,
    ) -> (String, String) {
        // For high conviction, select best quality
        let best = metrics.iter()
            .filter(|(_, m)| m.success_rate() >= 95.0)
            .filter(|(_, m)| m.avg_confidence >= 0.8)
            .max_by(|a, b| {
                let score_a = a.1.avg_confidence * 0.6 + a.1.success_rate() / 100.0 * 0.4;
                let score_b = b.1.avg_confidence * 0.6 + b.1.success_rate() / 100.0 * 0.4;
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            });

        match best {
            Some((model, _)) => (
                model.clone(),
                format!("High conviction: selected high-quality model {}", model)
            ),
            None => (
                self.available_models.get(2).unwrap_or(&"deepseek-coder:1.3b".to_string()).clone(),
                "High conviction: using premium fallback model".to_string()
            )
        }
    }

    async fn select_for_extreme_conviction(
        &self,
        metrics: &HashMap<String, ModelPerformanceMetrics>,
        _request: &LlmRequest,
    ) -> (String, String) {
        // For extreme conviction, select premium model regardless of cost
        let best = metrics.iter()
            .filter(|(_, m)| m.success_rate() >= 98.0)
            .filter(|(_, m)| m.avg_confidence >= 0.9)
            .max_by(|a, b| {
                a.1.avg_confidence.partial_cmp(&b.1.avg_confidence).unwrap_or(std::cmp::Ordering::Equal)
            });

        match best {
            Some((model, _)) => (
                model.clone(),
                format!("Extreme conviction: selected premium model {}", model)
            ),
            None => (
                self.available_models.last().unwrap_or(&"deepseek-coder:1.3b".to_string()).clone(),
                "Extreme conviction: using best available model".to_string()
            )
        }
    }

    async fn calculate_selection_confidence(
        &self,
        metrics: &HashMap<String, ModelPerformanceMetrics>,
        model: &str,
    ) -> f64 {
        if let Some(metric) = metrics.get(model) {
            metric.health_score()
        } else {
            0.5
        }
    }

    fn get_alternative_models(&self, selected_model: &str, _conviction: f64) -> Vec<String> {
        self.available_models
            .iter()
            .filter(|m| *m != selected_model)
            .take(2)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_criteria_default() {
        let criteria = SelectionCriteria::default();
        assert_eq!(criteria.min_confidence, 0.5);
        assert_eq!(criteria.max_latency_ms, 1000.0);
    }

    #[test]
    fn test_conviction_config_default() {
        let config = ConvictionConfig::default();
        assert_eq!(config.low_conviction_threshold, 0.3);
        assert_eq!(config.medium_conviction_threshold, 0.7);
    }

    #[test]
    fn test_model_performance_health_score() {
        let metrics = ModelPerformanceMetrics {
            model_name: "test-model".to_string(),
            provider: "ollama".to_string(),
            total_requests: 100,
            successful_requests: 95,
            failed_requests: 5,
            avg_latency_ms: 100.0,
            p50_latency_ms: 90.0,
            p95_latency_ms: 150.0,
            p99_latency_ms: 200.0,
            avg_confidence: 0.85,
            success_rate: 95.0,
            last_used: Utc::now(),
            last_success: Utc::now(),
            last_failure: Utc::now(),
        };

        let score = metrics.health_score();
        assert!(score > 0.5);
        assert!(score <= 1.0);
    }

    #[tokio::test]
    async fn test_dynamic_model_selector_creation() {
        let available_models = vec![
            "deepseek-coder:1.3b".to_string(),
            "gemma-mini:2b".to_string(),
        ];
        let model_costs = HashMap::from([
            ("deepseek-coder:1.3b".to_string(), 0.0),
            ("gemma-mini:2b".to_string(), 0.0),
        ]);

        let selector = DynamicModelSelector::new(available_models, model_costs);
        assert_eq!(selector.available_models.len(), 2);
    }
}
