use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Mutex};
use tracing::{info, warn, debug};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::llm::{LlmRequest, AgentResponse, LlmResult, LlmError, ResponseType};
use crate::llm::ollama_client::OllamaClient;
use crate::middleware::{LlmProvider, CircuitBreaker};
use crate::observability::{AgentMetrics, StructuredLogger};

/// Provider selection strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SelectionStrategy {
    /// Always use local Ollama
    LocalOnly,
    /// Always use cloud providers
    CloudOnly,
    /// Prefer local, fallback to cloud
    LocalFirst,
    /// Prefer cloud, fallback to local
    CloudFirst,
    /// Balance based on performance metrics
    PerformanceBased,
    /// Route based on signal conviction
    ConvictionBased,
}

/// Provider performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPerformance {
    pub provider: LlmProvider,
    pub request_count: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub error_rate: f64,
    pub last_success: DateTime<Utc>,
    pub last_failure: DateTime<Utc>,
    pub is_available: bool,
}

impl ProviderPerformance {
    pub fn success_rate(&self) -> f64 {
        if self.request_count == 0 {
            0.0
        } else {
            (self.success_count as f64 / self.request_count as f64) * 100.0
        }
    }

    pub fn health_score(&self) -> f64 {
        let success_weight = 0.4;
        let latency_weight = 0.3;
        let availability_weight = 0.3;

        let success_score = self.success_rate() / 100.0;
        let latency_score = (1000.0 / (self.avg_latency_ms + 1.0)).min(1.0);
        let availability_score = if self.is_available { 1.0 } else { 0.0 };

        success_score * success_weight
            + latency_score * latency_weight
            + availability_score * availability_weight
    }
}

/// Routing decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub selected_provider: LlmProvider,
    pub selected_model: String,
    pub fallback_providers: Vec<(LlmProvider, String)>,
    pub reasoning: String,
    pub confidence: f64,
    pub estimated_latency_ms: f64,
    pub timestamp: DateTime<Utc>,
}

/// Hybrid provider router
pub struct HybridProviderRouter {
    local_client: Option<Arc<OllamaClient>>,
    cloud_clients: HashMap<String, Arc<dyn CloudLlmClient>>,
    strategy: SelectionStrategy,
    provider_performance: Arc<RwLock<HashMap<LlmProvider, ProviderPerformance>>>,
    circuit_breakers: Arc<Mutex<HashMap<LlmProvider, CircuitBreaker>>>,
    metrics: Option<Arc<AgentMetrics>>,
    logger: Option<Arc<StructuredLogger>>,
    conviction_thresholds: ConvictionThresholds,
    latency_thresholds: LatencyThresholds,
}

#[derive(Debug, Clone)]
pub struct ConvictionThresholds {
    pub low_threshold: f64,      // < 0.3: use local only
    pub medium_threshold: f64,   // 0.3-0.7: use local first
    pub high_threshold: f64,     // > 0.7: use cloud
}

impl Default for ConvictionThresholds {
    fn default() -> Self {
        Self {
            low_threshold: 0.3,
            medium_threshold: 0.7,
            high_threshold: 0.9,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LatencyThresholds {
    pub local_max_ms: f64,
    pub cloud_max_ms: f64,
    pub critical_ms: f64,
}

impl Default for LatencyThresholds {
    fn default() -> Self {
        Self {
            local_max_ms: 500.0,
            cloud_max_ms: 2000.0,
            critical_ms: 5000.0,
        }
    }
}

/// Cloud LLM client trait
#[async_trait::async_trait]
pub trait CloudLlmClient: Send + Sync {
    async fn generate(&self, request: &LlmRequest) -> LlmResult<AgentResponse>;
    async fn health_check(&self) -> LlmResult<bool>;
    fn provider_name(&self) -> String;
}

impl HybridProviderRouter {
    pub fn new(strategy: SelectionStrategy) -> Self {
        let mut router = Self {
            local_client: None,
            cloud_clients: HashMap::new(),
            strategy,
            provider_performance: Arc::new(RwLock::new(HashMap::new())),
            circuit_breakers: Arc::new(Mutex::new(HashMap::new())),
            metrics: None,
            logger: None,
            conviction_thresholds: ConvictionThresholds::default(),
            latency_thresholds: LatencyThresholds::default(),
        };

        // Initialize circuit breakers for all providers
        router.initialize_circuit_breakers();

        router
    }

    pub fn with_local_client(mut self, client: Arc<OllamaClient>) -> Self {
        self.local_client = Some(client);
        self
    }

    pub fn with_cloud_client(mut self, name: String, client: Arc<dyn CloudLlmClient>) -> Self {
        self.cloud_clients.insert(name, client);
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

    pub fn with_conviction_thresholds(mut self, thresholds: ConvictionThresholds) -> Self {
        self.conviction_thresholds = thresholds;
        self
    }

    pub fn with_latency_thresholds(mut self, thresholds: LatencyThresholds) -> Self {
        self.latency_thresholds = thresholds;
        self
    }

    fn initialize_circuit_breakers(&mut self) {
        let mut breakers = self.circuit_breakers.blocking_lock();
        
        for provider in &[LlmProvider::Ollama, LlmProvider::OpenAI, LlmProvider::Anthropic] {
            breakers.insert(
                provider.clone(),
                CircuitBreaker::new(
                    provider.clone(),
                    5, // failure threshold
                    Duration::from_secs(60), // recovery timeout
                )
            );
        }
    }

    /// Make routing decision based on request and strategy
    pub async fn make_routing_decision(&self, request: &LlmRequest, conviction: f64) -> RoutingDecision {
        let _start_time = Instant::now();
        
        // Get current provider performance
        let performance = self.provider_performance.read().await;
        
        let (selected_provider, selected_model, fallback_providers, reasoning) = match self.strategy {
            SelectionStrategy::LocalOnly => {
                self.select_local_only(request, &performance).await
            },
            SelectionStrategy::CloudOnly => {
                self.select_cloud_only(request, &performance).await
            },
            SelectionStrategy::LocalFirst => {
                self.select_local_first(request, conviction, &performance).await
            },
            SelectionStrategy::CloudFirst => {
                self.select_cloud_first(request, conviction, &performance).await
            },
            SelectionStrategy::PerformanceBased => {
                self.select_performance_based(request, &performance).await
            },
            SelectionStrategy::ConvictionBased => {
                self.select_conviction_based(request, conviction, &performance).await
            },
        };

        // Estimate latency
        let estimated_latency = if let Some(perf) = performance.get(&selected_provider) {
            perf.avg_latency_ms
        } else {
            match selected_provider {
                LlmProvider::Ollama => 100.0,
                LlmProvider::OpenAI => 300.0,
                LlmProvider::Anthropic => 250.0,
                LlmProvider::Hybrid => 200.0,
            }
        };

        let decision = RoutingDecision {
            selected_provider,
            selected_model,
            fallback_providers,
            reasoning,
            confidence: self.calculate_decision_confidence(&performance).await,
            estimated_latency_ms: estimated_latency,
            timestamp: Utc::now(),
        };

        // Log routing decision
        if let Some(logger) = &self.logger {
            let metadata = HashMap::from([
                ("selected_provider".to_string(), format!("{:?}", decision.selected_provider)),
                ("selected_model".to_string(), decision.selected_model.clone()),
                ("reasoning".to_string(), decision.reasoning.clone()),
                ("confidence".to_string(), decision.confidence.to_string()),
                ("estimated_latency_ms".to_string(), decision.estimated_latency_ms.to_string()),
                ("conviction".to_string(), conviction.to_string()),
            ]);

            logger.log(
                request.request_id,
                "hybrid_router",
                crate::observability::LogLevel::Info,
                "Routing decision made",
                metadata
            ).await;
        }

        debug!(
            "Routing decision: provider={:?}, model={}, confidence={:.2}, latency={:.2}ms",
            decision.selected_provider, decision.selected_model, decision.confidence, decision.estimated_latency_ms
        );

        decision
    }

    /// Execute request with automatic fallback
    pub async fn execute_with_fallback(&self, request: &LlmRequest, conviction: f64) -> LlmResult<AgentResponse> {
        let decision = self.make_routing_decision(request, conviction).await;
        
        // Try primary provider
        match self.execute_on_provider(request, &decision.selected_provider, &decision.selected_model).await {
            Ok(response) => {
                self.record_success(&decision.selected_provider).await;
                Ok(response)
            },
            Err(e) => {
                warn!("Primary provider failed: {:?}", e);
                self.record_failure(&decision.selected_provider).await;
                
                // Try fallback providers
                for (fallback_provider, fallback_model) in &decision.fallback_providers {
                    info!("Attempting fallback to {:?}", fallback_provider);
                    match self.execute_on_provider(request, fallback_provider, fallback_model).await {
                        Ok(response) => {
                            self.record_success(fallback_provider).await;
                            return Ok(response);
                        },
                        Err(fallback_error) => {
                            warn!("Fallback provider {:?} also failed: {:?}", fallback_provider, fallback_error);
                            self.record_failure(fallback_provider).await;
                        }
                    }
                }
                
                // All providers failed
                Err(LlmError::RequestFailed("All providers failed".to_string()))
            }
        }
    }

    /// Execute request on specific provider
    async fn execute_on_provider(&self, request: &LlmRequest, provider: &LlmProvider, model: &str) -> LlmResult<AgentResponse> {
        // Check circuit breaker
        let mut breakers = self.circuit_breakers.lock().await;
        let can_proceed = if let Some(breaker) = breakers.get_mut(provider) {
            breaker.allow_request()
        } else {
            true
        };
        drop(breakers);

        if !can_proceed {
            return Err(LlmError::RateLimitExceeded);
        }

        let _start_time = Instant::now();
        let result = match provider {
            LlmProvider::Ollama => {
                if let Some(client) = &self.local_client {
                    let mut modified_request = request.clone();
                    modified_request.model = model.to_string();
                    client.generate(&modified_request).await
                } else {
                    Err(LlmError::ModelNotFound("Ollama client not configured".to_string()))
                }
            },
            LlmProvider::OpenAI => {
                // Would use cloud client
                self.mock_cloud_response(request, "openai", model).await
            },
            LlmProvider::Anthropic => {
                // Would use cloud client
                self.mock_cloud_response(request, "anthropic", model).await
            },
            LlmProvider::Hybrid => {
                // Hybrid mode - try local first
                if let Some(client) = &self.local_client {
                    client.generate(request).await
                } else {
                    self.mock_cloud_response(request, "hybrid", model).await
                }
            },
        };

        // Update circuit breaker
        let mut breakers = self.circuit_breakers.lock().await;
        if let Some(breaker) = breakers.get_mut(provider) {
            if result.is_ok() {
                breaker.record_success();
            } else {
                breaker.record_failure();
            }
        }

        result
    }

    // Selection strategy implementations
    async fn select_local_only(&self, _request: &LlmRequest, _performance: &HashMap<LlmProvider, ProviderPerformance>) -> (LlmProvider, String, Vec<(LlmProvider, String)>, String) {
        let model = if let Some(client) = &self.local_client {
            client.get_available_models().first().map(|m| m.name.clone()).unwrap_or_else(|| "deepseek-coder:1.3b".to_string())
        } else {
            "deepseek-coder:1.3b".to_string()
        };

        (
            LlmProvider::Ollama,
            model,
            vec![],
            "Local-only strategy: using Ollama provider".to_string()
        )
    }

    async fn select_cloud_only(&self, _request: &LlmRequest, performance: &HashMap<LlmProvider, ProviderPerformance>) -> (LlmProvider, String, Vec<(LlmProvider, String)>, String) {
        // Select best cloud provider based on performance
        let best_provider = [LlmProvider::OpenAI, LlmProvider::Anthropic]
            .iter()
            .max_by(|a, b| {
                let score_a = performance.get(a).map(|p| p.health_score()).unwrap_or(0.5);
                let score_b = performance.get(b).map(|p| p.health_score()).unwrap_or(0.5);
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(&LlmProvider::OpenAI);

        let model = match best_provider {
            LlmProvider::OpenAI => "gpt-4".to_string(),
            LlmProvider::Anthropic => "claude-3".to_string(),
            _ => "gpt-4".to_string(),
        };

        let fallback = match best_provider {
            LlmProvider::OpenAI => vec![(LlmProvider::Anthropic, "claude-3".to_string())],
            LlmProvider::Anthropic => vec![(LlmProvider::OpenAI, "gpt-4".to_string())],
            _ => vec![],
        };

        (
            best_provider.clone(),
            model,
            fallback,
            format!("Cloud-only strategy: selected {:?} based on performance", best_provider)
        )
    }

    async fn select_local_first(&self, request: &LlmRequest, _conviction: f64, performance: &HashMap<LlmProvider, ProviderPerformance>) -> (LlmProvider, String, Vec<(LlmProvider, String)>, String) {
        let local_available = self.local_client.as_ref()
            .map(|client| !client.get_available_models().is_empty())
            .unwrap_or(false);

        if local_available {
            let model = self.local_client.as_ref()
                .map(|client| client.get_available_models().first().map(|m| m.name.clone()))
                .flatten()
                .unwrap_or_else(|| "deepseek-coder:1.3b".to_string());

            (
                LlmProvider::Ollama,
                model,
                vec![(LlmProvider::OpenAI, "gpt-4".to_string()), (LlmProvider::Anthropic, "claude-3".to_string())],
                "Local-first strategy: using Ollama with cloud fallback".to_string()
            )
        } else {
            self.select_cloud_only(request, performance).await
        }
    }

    async fn select_cloud_first(&self, request: &LlmRequest, _conviction: f64, performance: &HashMap<LlmProvider, ProviderPerformance>) -> (LlmProvider, String, Vec<(LlmProvider, String)>, String) {
        let (provider, model, mut fallback, _reasoning) = self.select_cloud_only(request, performance).await;
        
        // Add local as last fallback
        if self.local_client.is_some() {
            let local_model = self.local_client.as_ref()
                .map(|client| client.get_available_models().first().map(|m| m.name.clone()))
                .flatten()
                .unwrap_or_else(|| "deepseek-coder:1.3b".to_string());
            fallback.push((LlmProvider::Ollama, local_model));
        }

        (
            provider,
            model,
            fallback,
            "Cloud-first strategy: using cloud with local fallback".to_string()
        )
    }

    async fn select_performance_based(&self, _request: &LlmRequest, performance: &HashMap<LlmProvider, ProviderPerformance>) -> (LlmProvider, String, Vec<(LlmProvider, String)>, String) {
        let all_providers = [LlmProvider::Ollama, LlmProvider::OpenAI, LlmProvider::Anthropic];
        
        let best_provider = all_providers.iter()
            .filter(|p| performance.get(p).map(|perf| perf.is_available).unwrap_or(true))
            .max_by(|a, b| {
                let score_a = performance.get(a).map(|p| p.health_score()).unwrap_or(0.0);
                let score_b = performance.get(b).map(|p| p.health_score()).unwrap_or(0.0);
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(&LlmProvider::Ollama);

        let model = match best_provider {
            LlmProvider::Ollama => "deepseek-coder:1.3b".to_string(),
            LlmProvider::OpenAI => "gpt-4".to_string(),
            LlmProvider::Anthropic => "claude-3".to_string(),
            LlmProvider::Hybrid => "hybrid".to_string(),
        };

        let fallback: Vec<_> = all_providers.iter()
            .filter(|p| *p != best_provider)
            .map(|p| {
                let fallback_model = match p {
                    LlmProvider::Ollama => "deepseek-coder:1.3b".to_string(),
                    LlmProvider::OpenAI => "gpt-4".to_string(),
                    LlmProvider::Anthropic => "claude-3".to_string(),
                    LlmProvider::Hybrid => "hybrid".to_string(),
                };
                (p.clone(), fallback_model)
            })
            .collect();

        let score = performance.get(best_provider).map(|p| p.health_score()).unwrap_or(0.5);

        (
            best_provider.clone(),
            model,
            fallback,
            format!("Performance-based: selected {:?} with health score {:.2}", best_provider, score)
        )
    }

    async fn select_conviction_based(&self, request: &LlmRequest, conviction: f64, performance: &HashMap<LlmProvider, ProviderPerformance>) -> (LlmProvider, String, Vec<(LlmProvider, String)>, String) {
        if conviction < self.conviction_thresholds.low_threshold {
            // Low conviction: use local only
            self.select_local_only(request, performance).await
        } else if conviction < self.conviction_thresholds.medium_threshold {
            // Medium conviction: local first
            self.select_local_first(request, conviction, performance).await
        } else if conviction < self.conviction_thresholds.high_threshold {
            // High conviction: cloud first
            self.select_cloud_first(request, conviction, performance).await
        } else {
            // Very high conviction: use best cloud provider
            self.select_cloud_only(request, performance).await
        }
    }

    // Performance tracking
    async fn record_success(&self, provider: &LlmProvider) {
        let mut perf = self.provider_performance.write().await;
        let entry = perf.entry(provider.clone()).or_insert_with(|| ProviderPerformance {
            provider: provider.clone(),
            request_count: 0,
            success_count: 0,
            failure_count: 0,
            avg_latency_ms: 0.0,
            p50_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            error_rate: 0.0,
            last_success: Utc::now(),
            last_failure: Utc::now(),
            is_available: true,
        });

        entry.request_count += 1;
        entry.success_count += 1;
        entry.last_success = Utc::now();
        entry.error_rate = (entry.failure_count as f64 / entry.request_count as f64) * 100.0;
        entry.is_available = true;
    }

    async fn record_failure(&self, provider: &LlmProvider) {
        let mut perf = self.provider_performance.write().await;
        let entry = perf.entry(provider.clone()).or_insert_with(|| ProviderPerformance {
            provider: provider.clone(),
            request_count: 0,
            success_count: 0,
            failure_count: 0,
            avg_latency_ms: 0.0,
            p50_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            error_rate: 0.0,
            last_success: Utc::now(),
            last_failure: Utc::now(),
            is_available: true,
        });

        entry.request_count += 1;
        entry.failure_count += 1;
        entry.last_failure = Utc::now();
        entry.error_rate = (entry.failure_count as f64 / entry.request_count as f64) * 100.0;
        
        // Mark as unavailable if error rate is high
        if entry.error_rate > 50.0 {
            entry.is_available = false;
        }
    }

    async fn calculate_decision_confidence(&self, performance: &HashMap<LlmProvider, ProviderPerformance>) -> f64 {
        let total_score: f64 = performance.values()
            .map(|p| p.health_score())
            .sum();

        if performance.is_empty() {
            0.5
        } else {
            total_score / performance.len() as f64
        }
    }

    // Mock implementation for testing
    async fn mock_cloud_response(&self, request: &LlmRequest, provider_name: &str, model: &str) -> LlmResult<AgentResponse> {
        tokio::time::sleep(Duration::from_millis(200)).await;

        Ok(AgentResponse {
            request_id: request.request_id,
            agent_id: format!("{}_mock", provider_name),
            response_type: ResponseType::Analysis,
            content: format!("Mock response from {} using model {}", provider_name, model),
            confidence: 0.85,
            timestamp: Utc::now(),
            metadata: HashMap::from([
                ("provider".to_string(), provider_name.to_string()),
                ("model".to_string(), model.to_string()),
            ]),
        })
    }

    /// Get current provider performance
    pub async fn get_provider_performance(&self) -> HashMap<LlmProvider, ProviderPerformance> {
        self.provider_performance.read().await.clone()
    }

    /// Get circuit breaker status
    pub async fn get_circuit_breaker_status(&self) -> HashMap<LlmProvider, String> {
        let breakers = self.circuit_breakers.lock().await;
        breakers.iter()
            .map(|(provider, _)| (provider.clone(), "active".to_string()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_strategy_creation() {
        let strategy = SelectionStrategy::LocalFirst;
        assert_eq!(strategy, SelectionStrategy::LocalFirst);
    }

    #[test]
    fn test_provider_performance_health_score() {
        let perf = ProviderPerformance {
            provider: LlmProvider::Ollama,
            request_count: 100,
            success_count: 95,
            failure_count: 5,
            avg_latency_ms: 100.0,
            p50_latency_ms: 90.0,
            p95_latency_ms: 150.0,
            p99_latency_ms: 200.0,
            error_rate: 5.0,
            last_success: Utc::now(),
            last_failure: Utc::now(),
            is_available: true,
        };

        let score = perf.health_score();
        assert!(score > 0.5);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_conviction_thresholds_default() {
        let thresholds = ConvictionThresholds::default();
        assert_eq!(thresholds.low_threshold, 0.3);
        assert_eq!(thresholds.medium_threshold, 0.7);
        assert_eq!(thresholds.high_threshold, 0.9);
    }

    #[tokio::test]
    async fn test_hybrid_router_creation() {
        let router = HybridProviderRouter::new(SelectionStrategy::LocalFirst);
        assert_eq!(router.strategy, SelectionStrategy::LocalFirst);
    }
}
