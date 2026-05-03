use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Mutex};
use tokio::time::interval;
use tracing::{info, warn};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::llm::{AgentResponse, LlmResult, LlmError};
use crate::observability::{AgentMetrics, StructuredLogger};

/// LLM-aware message with context propagation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessage {
    pub message_id: Uuid,
    pub correlation_id: Uuid,
    pub message_type: LlmMessageType,
    pub payload: LlmMessagePayload,
    pub context: MessageContext,
    pub priority: MessagePriority,
    pub timestamp: DateTime<Utc>,
    pub ttl: Duration,
    pub retry_count: u32,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmMessageType {
    /// Request for LLM processing
    LlmRequest { model: String, provider: LlmProvider },
    /// Response from LLM processing
    LlmResponse { model: String, provider: LlmProvider },
    /// Health check message
    HealthCheck,
    /// Model status update
    ModelStatus { model: String, status: String },
    /// Context propagation message
    ContextUpdate,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmMessagePayload {
    Request(serde_json::Value), // Serialized LlmRequest
    Response(AgentResponse),
    Health { status: String, details: HashMap<String, String> },
    ModelInfo { name: String, available: bool },
    Context { key: String, value: serde_json::Value },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContext {
    pub trading_context: Option<TradingContext>,
    pub user_context: Option<UserContext>,
    pub system_context: Option<SystemContext>,
    pub propagation_chain: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingContext {
    pub symbol: String,
    pub direction: String,
    pub conviction: f64,
    pub max_notional: f64,
    pub risk_level: String,
    pub strategy_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: String,
    pub session_id: String,
    pub preferences: HashMap<String, String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemContext {
    pub node_id: String,
    pub cluster_id: String,
    pub load_factor: f64,
    pub memory_usage: f64,
    pub active_models: Vec<String>,
}

/// LLM routing table for provider selection
#[derive(Debug, Clone)]
pub struct LlmRoutingTable {
    routes: HashMap<String, Vec<LlmRoute>>,
    default_provider: LlmProvider,
    load_balancer: LlmLoadBalancer,
}

#[derive(Debug, Clone)]
pub struct LlmRoute {
    pub provider: LlmProvider,
    pub model: String,
    pub weight: f64,
    pub conditions: Vec<RoutingCondition>,
    pub health_status: RouteHealth,
}

#[derive(Debug, Clone)]
pub enum RoutingCondition {
    ConvictionRange { min: f64, max: f64 },
    SymbolPattern(String),
    TimeWindow { start: u32, end: u32 }, // Hour of day
    ModelSize { min: u64, max: u64 },
    LatencyRequirement { max_ms: u64 },
}

#[derive(Debug, Clone)]
pub enum RouteHealth {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
}

/// Load balancer for LLM providers
#[derive(Debug, Clone)]
pub struct LlmLoadBalancer {
    provider_stats: HashMap<LlmProvider, ProviderStats>,
    balancing_strategy: BalancingStrategy,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LlmProvider {
    Ollama,
    OpenAI,
    Anthropic,
    Hybrid,
}

#[derive(Debug, Clone)]
pub struct ProviderStats {
    pub request_count: u64,
    pub success_count: u64,
    pub avg_latency_ms: f64,
    pub error_rate: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum BalancingStrategy {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    LatencyBased,
}

/// Context propagator for maintaining context across message flow
pub struct ContextPropagator {
    context_store: Arc<RwLock<HashMap<Uuid, MessageContext>>>,
    propagation_rules: Vec<PropagationRule>,
    cleanup_interval: Duration,
    context_ttl: Duration,
}

#[derive(Debug, Clone)]
pub struct PropagationRule {
    pub source_key: String,
    pub target_keys: Vec<String>,
    pub condition: PropagationCondition,
}

#[derive(Debug, Clone)]
pub enum PropagationCondition {
    Always,
    IfPresent(String),
    IfValue(String),
    IfProvider(LlmProvider),
}

/// Circuit breaker for LLM providers
pub struct CircuitBreaker {
    provider: LlmProvider,
    state: CircuitState,
    failure_count: u32,
    failure_threshold: u32,
    recovery_timeout: Duration,
    last_failure_time: Option<Instant>,
    request_count: u64,
    success_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CircuitState {
    Closed,      // Normal operation
    Open,        // Failing, reject requests
    HalfOpen,    // Testing recovery
}

/// Main LLM-aware message bus
pub struct LlmMessageBus {
    routing_table: LlmRoutingTable,
    context_propagator: ContextPropagator,
    circuit_breakers: HashMap<LlmProvider, CircuitBreaker>,
    message_queue: Arc<Mutex<Vec<LlmMessage>>>,
    metrics: Option<Arc<AgentMetrics>>,
    logger: Option<Arc<StructuredLogger>>,
    max_queue_size: usize,
    processing_interval: Duration,
}

impl LlmMessageBus {
    pub fn new() -> Self {
        let routing_table = LlmRoutingTable::new();
        let context_propagator = ContextPropagator::new();
        
        Self {
            routing_table,
            context_propagator,
            circuit_breakers: HashMap::new(),
            message_queue: Arc::new(Mutex::new(Vec::new())),
            metrics: None,
            logger: None,
            max_queue_size: 10000,
            processing_interval: Duration::from_millis(100),
        }
    }
    
    pub fn with_metrics(mut self, metrics: Arc<AgentMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }
    
    pub fn with_logger(mut self, logger: Arc<StructuredLogger>) -> Self {
        self.logger = Some(logger);
        self
    }
    
    /// Send a message to the LLM message bus
    pub async fn send_message(&mut self, message: LlmMessage) -> Result<(), LlmError> {
        // Check queue size
        {
            let mut queue = self.message_queue.lock().await;
            if queue.len() >= self.max_queue_size {
                return Err(LlmError::ContextOverflow);
            }
            queue.push(message);
        }
        
        // Record metrics
        if let Some(metrics) = &self.metrics {
            metrics.llm_requests_total.inc();
        }
        
        Ok(())
    }
    
    /// Process queued messages
    pub async fn process_messages(&mut self) -> Vec<LlmResult<LlmMessage>> {
        let mut results = Vec::new();
        
        // Get messages to process
        let messages: Vec<LlmMessage> = {
            let mut queue = self.message_queue.lock().await;
            let batch_size = queue.len().min(100); // Process in batches
            queue.drain(0..batch_size).collect()
        };
        
        for message in messages {
            let result = self.process_single_message(message).await;
            results.push(result);
        }
        
        results
    }
    
    /// Process a single message
    async fn process_single_message(&mut self, message: LlmMessage) -> LlmResult<LlmMessage> {
        let start_time = Instant::now();
        
        // Check TTL
        if Utc::now().signed_duration_since(message.timestamp).to_std().unwrap_or(Duration::MAX) > message.ttl {
            return Err(LlmError::Timeout("Message expired".to_string()));
        }
        
        // Propagate context
        let enriched_message = self.context_propagator
            .propagate_context(message)
            .await?;
        
        // Route message based on type and provider
        let processed_message = match enriched_message.message_type.clone() {
            LlmMessageType::LlmRequest { model, provider } => {
                self.process_llm_request(enriched_message, &model, &provider).await?
            },
            LlmMessageType::LlmResponse { model, provider } => {
                self.process_llm_response(enriched_message, &model, &provider).await?
            },
            LlmMessageType::HealthCheck => {
                self.process_health_check(enriched_message).await?
            },
            LlmMessageType::ModelStatus { model, status } => {
                self.process_model_status(enriched_message, &model, &status).await?
            },
            LlmMessageType::ContextUpdate => {
                self.process_context_update(enriched_message).await?
            },
        };
        
        // Record processing metrics
        let duration = start_time.elapsed();
        if let Some(metrics) = &self.metrics {
            metrics.llm_request_duration.observe(duration.as_secs_f64());
        }
        
        // Log processing
        if let Some(logger) = &self.logger {
            let metadata = HashMap::from([
                ("message_id".to_string(), processed_message.message_id.to_string()),
                ("message_type".to_string(), format!("{:?}", processed_message.message_type)),
                ("processing_time_ms".to_string(), duration.as_millis().to_string()),
            ]);
            
            logger.log(
                processed_message.correlation_id,
                "llm_message_bus",
                crate::observability::LogLevel::Info,
                "Message processed",
                metadata
            ).await;
        }
        
        Ok(processed_message)
    }
    
    /// Process LLM request messages
    async fn process_llm_request(
        &mut self,
        mut message: LlmMessage,
        model: &str,
        provider: &LlmProvider,
    ) -> LlmResult<LlmMessage> {
        // Check circuit breaker
        if let Some(circuit_breaker) = self.circuit_breakers.get_mut(provider) {
            if !circuit_breaker.allow_request() {
                return Err(LlmError::RateLimitExceeded);
            }
        }
        
        // Select optimal route
        let _route = self.routing_table.select_route(model, provider)?;
        
        // Process request based on provider
        let result = match provider {
            LlmProvider::Ollama => {
                // In a real implementation, this would call the Ollama client
                self.mock_ollama_request(&message).await
            },
            LlmProvider::OpenAI => {
                self.mock_openai_request(&message).await
            },
            LlmProvider::Anthropic => {
                self.mock_anthropic_request(&message).await
            },
            LlmProvider::Hybrid => {
                self.mock_hybrid_request(&message).await
            },
        };
        
        // Update circuit breaker
        if let Some(circuit_breaker) = self.circuit_breakers.get_mut(provider) {
            if result.is_ok() {
                circuit_breaker.record_success();
            } else {
                circuit_breaker.record_failure();
            }
        }
        
        // Update message with response
        match result {
            Ok(response) => {
                message.message_type = LlmMessageType::LlmResponse {
                    model: model.to_string(),
                    provider: provider.clone(),
                };
                message.payload = LlmMessagePayload::Response(response);
                Ok(message)
            },
            Err(e) => {
                // Increment retry count
                message.retry_count += 1;
                if message.retry_count < message.max_retries {
                    // Re-queue for retry
                    self.send_message(message.clone()).await?;
                }
                Err(e)
            }
        }
    }
    
    /// Process LLM response messages
    async fn process_llm_response(
        &mut self,
        message: LlmMessage,
        _model: &str,
        _provider: &LlmProvider,
    ) -> LlmResult<LlmMessage> {
        // Update context with response data
        self.context_propagator
            .update_context_from_response(&message)
            .await?;
        
        Ok(message)
    }
    
    /// Process health check messages
    async fn process_health_check(&mut self, message: LlmMessage) -> LlmResult<LlmMessage> {
        let health_status = self.check_all_providers_health().await;
        
        let mut response_message = message;
        response_message.message_type = LlmMessageType::HealthCheck;
        response_message.payload = LlmMessagePayload::Health {
            status: "healthy".to_string(),
            details: health_status,
        };
        
        Ok(response_message)
    }
    
    /// Process model status messages
    async fn process_model_status(
        &mut self,
        message: LlmMessage,
        model: &str,
        status: &str,
    ) -> LlmResult<LlmMessage> {
        // Update routing table with model status
        self.routing_table.update_model_status(model, status);
        
        let mut response_message = message;
        response_message.message_type = LlmMessageType::ModelStatus {
            model: model.to_string(),
            status: status.to_string(),
        };
        response_message.payload = LlmMessagePayload::ModelInfo {
            name: model.to_string(),
            available: status == "ready",
        };
        
        Ok(response_message)
    }
    
    /// Process context update messages
    async fn process_context_update(&mut self, message: LlmMessage) -> LlmResult<LlmMessage> {
        // Extract context from payload
        if let LlmMessagePayload::Context { key, value } = &message.payload {
            self.context_propagator
                .update_context(message.correlation_id, key.clone(), value.clone())
                .await?;
        }
        
        Ok(message)
    }
    
    /// Check health of all providers
    async fn check_all_providers_health(&self) -> HashMap<String, String> {
        let mut health_status = HashMap::new();
        
        for (provider, circuit_breaker) in &self.circuit_breakers {
            let status = match circuit_breaker.state {
                CircuitState::Closed => "healthy",
                CircuitState::Open => "unhealthy",
                CircuitState::HalfOpen => "degraded",
            };
            health_status.insert(format!("{:?}", provider), status.to_string());
        }
        
        health_status
    }
    
    // Mock implementations for testing
    async fn mock_ollama_request(&self, _message: &LlmMessage) -> LlmResult<AgentResponse> {
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        Ok(AgentResponse {
            request_id: Uuid::new_v4(),
            agent_id: "ollama_mock".to_string(),
            response_type: crate::llm::ResponseType::Analysis,
            content: "Mock Ollama response".to_string(),
            confidence: 0.8,
            timestamp: Utc::now(),
            metadata: HashMap::from([
                ("provider".to_string(), "ollama".to_string()),
                ("model".to_string(), "deepseek-coder:1.3b".to_string()),
            ]),
        })
    }
    
    async fn mock_openai_request(&self, _message: &LlmMessage) -> LlmResult<AgentResponse> {
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        Ok(AgentResponse {
            request_id: Uuid::new_v4(),
            agent_id: "openai_mock".to_string(),
            response_type: crate::llm::ResponseType::Analysis,
            content: "Mock OpenAI response".to_string(),
            confidence: 0.9,
            timestamp: Utc::now(),
            metadata: HashMap::from([
                ("provider".to_string(), "openai".to_string()),
                ("model".to_string(), "gpt-4".to_string()),
            ]),
        })
    }
    
    async fn mock_anthropic_request(&self, _message: &LlmMessage) -> LlmResult<AgentResponse> {
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        Ok(AgentResponse {
            request_id: Uuid::new_v4(),
            agent_id: "anthropic_mock".to_string(),
            response_type: crate::llm::ResponseType::Analysis,
            content: "Mock Anthropic response".to_string(),
            confidence: 0.88,
            timestamp: Utc::now(),
            metadata: HashMap::from([
                ("provider".to_string(), "anthropic".to_string()),
                ("model".to_string(), "claude-3".to_string()),
            ]),
        })
    }
    
    async fn mock_hybrid_request(&self, message: &LlmMessage) -> LlmResult<AgentResponse> {
        // Use context to determine best provider
        if let Some(trading_ctx) = &message.context.trading_context {
            if trading_ctx.conviction > 0.8 {
                self.mock_openai_request(message).await
            } else {
                self.mock_ollama_request(message).await
            }
        } else {
            self.mock_ollama_request(message).await
        }
    }
    
    /// Start continuous message processing
    pub async fn start_processing(&mut self) {
        info!("Starting LLM message bus processing");
        
        let mut interval = interval(self.processing_interval);
        
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let results = self.process_messages().await;
                    
                    // Log processing results
                    let success_count = results.iter().filter(|r| r.is_ok()).count();
                    let error_count = results.len() - success_count;
                    
                    if error_count > 0 {
                        warn!("Message processing: {} success, {} errors", success_count, error_count);
                    }
                }
                
                // Handle graceful shutdown
                _ = tokio::signal::ctrl_c() => {
                    info!("Stopping LLM message bus processing");
                    break;
                }
            }
        }
    }
}

impl LlmRoutingTable {
    pub fn new() -> Self {
        let mut routes = HashMap::new();
        
        // Default routing rules
        routes.insert("default".to_string(), vec![
            LlmRoute {
                provider: LlmProvider::Ollama,
                model: "deepseek-coder:1.3b".to_string(),
                weight: 0.6,
                conditions: vec![
                    RoutingCondition::ConvictionRange { min: 0.5, max: 1.0 },
                    RoutingCondition::LatencyRequirement { max_ms: 500 },
                ],
                health_status: RouteHealth::Healthy,
            },
            LlmRoute {
                provider: LlmProvider::OpenAI,
                model: "gpt-4".to_string(),
                weight: 0.3,
                conditions: vec![
                    RoutingCondition::ConvictionRange { min: 0.8, max: 1.0 },
                ],
                health_status: RouteHealth::Healthy,
            },
            LlmRoute {
                provider: LlmProvider::Anthropic,
                model: "claude-3".to_string(),
                weight: 0.1,
                conditions: vec![
                    RoutingCondition::ConvictionRange { min: 0.9, max: 1.0 },
                ],
                health_status: RouteHealth::Healthy,
            },
        ]);
        
        Self {
            routes,
            default_provider: LlmProvider::Ollama,
            load_balancer: LlmLoadBalancer::new(),
        }
    }
    
    pub fn select_route(&self, model: &str, provider: &LlmProvider) -> LlmResult<&LlmRoute> {
        let key = format!("{}:{:?}", model, provider);
        
        if let Some(route_list) = self.routes.get(&key) {
            // Select best route based on load balancing
            Ok(self.load_balancer.select_route(route_list))
        } else {
            // Fallback to default
            let default_routes = self.routes.get("default")
                .ok_or_else(|| LlmError::RouterError("No default routes available".to_string()))?;
            
            Ok(self.load_balancer.select_route(default_routes))
        }
    }
    
    pub fn update_model_status(&mut self, model: &str, status: &str) {
        for routes in self.routes.values_mut() {
            for route in routes {
                if route.model == model {
                    route.health_status = match status {
                        "ready" => RouteHealth::Healthy,
                        "loading" => RouteHealth::Degraded { reason: "Loading".to_string() },
                        _ => RouteHealth::Unhealthy { reason: status.to_string() },
                    };
                }
            }
        }
    }
}

impl LlmLoadBalancer {
    pub fn new() -> Self {
        Self {
            provider_stats: HashMap::new(),
            balancing_strategy: BalancingStrategy::WeightedRoundRobin,
        }
    }
    
    pub fn select_route<'a>(&self, routes: &'a [LlmRoute]) -> &'a LlmRoute {
        // Simple weighted selection for now
        // In production, this would implement sophisticated load balancing
        if routes.is_empty() {
            panic!("No routes available for selection");
        }
        routes.iter()
            .max_by(|a, b| a.weight.partial_cmp(&b.weight).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(&routes[0])
    }
}

impl ContextPropagator {
    pub fn new() -> Self {
        Self {
            context_store: Arc::new(RwLock::new(HashMap::new())),
            propagation_rules: vec![
                PropagationRule {
                    source_key: "symbol".to_string(),
                    target_keys: vec!["trading_symbol".to_string(), "market_symbol".to_string()],
                    condition: PropagationCondition::Always,
                },
                PropagationRule {
                    source_key: "user_id".to_string(),
                    target_keys: vec!["requester_id".to_string(), "session_user".to_string()],
                    condition: PropagationCondition::Always,
                },
            ],
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            context_ttl: Duration::from_secs(3600), // 1 hour
        }
    }
    
    pub async fn propagate_context(&self, message: LlmMessage) -> LlmResult<LlmMessage> {
        let mut enriched_message = message;
        
        // Apply propagation rules
        for rule in &self.propagation_rules {
            if self.should_apply_rule(&rule, &enriched_message) {
                self.apply_propagation_rule(&mut enriched_message, &rule).await?;
            }
        }
        
        Ok(enriched_message)
    }
    
    pub async fn update_context(&self, correlation_id: Uuid, key: String, value: serde_json::Value) -> LlmResult<()> {
        let mut store = self.context_store.write().await;
        let context = store.entry(correlation_id).or_insert_with(|| MessageContext {
            trading_context: None,
            user_context: None,
            system_context: None,
            propagation_chain: Vec::new(),
        });
        
        // Update context based on key
        match key.as_str() {
            "symbol" | "direction" | "conviction" | "max_notional" => {
                // Update trading context
                // This is simplified - in production would be more sophisticated
            },
            "user_id" | "session_id" => {
                // Update user context
            },
            _ => {
                // Store in propagation chain
                context.propagation_chain.push(format!("{}:{}", key, value));
            }
        }
        
        Ok(())
    }
    
    pub async fn update_context_from_response(&self, message: &LlmMessage) -> LlmResult<()> {
        // Extract relevant information from response and update context
        if let LlmMessagePayload::Response(response) = &message.payload {
            // Update context with response metadata
            for (key, value) in &response.metadata {
                self.update_context(message.correlation_id, key.clone(), serde_json::Value::String(value.clone())).await?;
            }
        }
        
        Ok(())
    }
    
    fn should_apply_rule(&self, rule: &PropagationRule, message: &LlmMessage) -> bool {
        match &rule.condition {
            PropagationCondition::Always => true,
            PropagationCondition::IfPresent(_key) => {
                // Check if key exists in context
                // Simplified implementation
                false
            },
            PropagationCondition::IfValue(_value) => {
                // Check if context has specific value
                false
            },
            PropagationCondition::IfProvider(provider) => {
                // Check message provider
                matches!(&message.message_type, LlmMessageType::LlmRequest { provider: p, .. } if p == provider)
            },
        }
    }
    
    async fn apply_propagation_rule(&self, _message: &mut LlmMessage, _rule: &PropagationRule) -> LlmResult<()> {
        // Apply propagation logic
        // Simplified implementation
        Ok(())
    }
}

impl CircuitBreaker {
    pub fn new(provider: LlmProvider, failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            provider,
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold,
            recovery_timeout,
            last_failure_time: None,
            request_count: 0,
            success_count: 0,
        }
    }
    
    pub fn allow_request(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() > self.recovery_timeout {
                        self.state = CircuitState::HalfOpen;
                        info!("Circuit breaker for {:?} transitioning to half-open", self.provider);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            CircuitState::HalfOpen => true,
        }
    }
    
    pub fn record_success(&mut self) {
        self.request_count += 1;
        self.success_count += 1;
        
        match self.state {
            CircuitState::HalfOpen => {
                // Reset to closed on first success in half-open
                self.state = CircuitState::Closed;
                self.failure_count = 0;
                info!("Circuit breaker for {:?} reset to closed", self.provider);
            },
            _ => {}
        }
    }
    
    pub fn record_failure(&mut self) {
        self.request_count += 1;
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());
        
        match self.state {
            CircuitState::Closed => {
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitState::Open;
                    warn!("Circuit breaker for {:?} opened after {} failures", self.provider, self.failure_count);
                }
            },
            CircuitState::HalfOpen => {
                self.state = CircuitState::Open;
                warn!("Circuit breaker for {:?} re-opened", self.provider);
            },
            CircuitState::Open => {}
        }
    }
    
    pub fn error_rate(&self) -> f64 {
        if self.request_count == 0 {
            0.0
        } else {
            (self.request_count - self.success_count) as f64 / self.request_count as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_llm_message_creation() {
        let message = LlmMessage {
            message_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            message_type: LlmMessageType::LlmRequest {
                model: "test".to_string(),
                provider: LlmProvider::Ollama,
            },
            payload: LlmMessagePayload::Context {
                key: "test".to_string(),
                value: serde_json::Value::String("value".to_string()),
            },
            context: MessageContext {
                trading_context: None,
                user_context: None,
                system_context: None,
                propagation_chain: Vec::new(),
            },
            priority: MessagePriority::Normal,
            timestamp: Utc::now(),
            ttl: Duration::from_secs(30),
            retry_count: 0,
            max_retries: 3,
        };
        
        assert_eq!(message.retry_count, 0);
        assert_eq!(message.max_retries, 3);
    }
    
    #[tokio::test]
    async fn test_circuit_breaker() {
        let mut breaker = CircuitBreaker::new(LlmProvider::Ollama, 3, Duration::from_secs(60));
        
        // Initially closed, should allow requests
        assert!(breaker.allow_request());
        
        // Record failures
        for _ in 0..3 {
            breaker.record_failure();
        }
        
        // Should be open now
        assert!(!breaker.allow_request());
        assert_eq!(breaker.state, CircuitState::Open);
    }
    
    #[tokio::test]
    async fn test_context_propagator() {
        let propagator = ContextPropagator::new();
        
        let correlation_id = Uuid::new_v4();
        propagator.update_context(
            correlation_id,
            "symbol".to_string(),
            serde_json::Value::String("AAPL".to_string())
        ).await.unwrap();
        
        // Verify context was stored
        let store = propagator.context_store.read().await;
        assert!(store.contains_key(&correlation_id));
    }
    
    #[tokio::test]
    async fn test_routing_table() {
        let routing_table = LlmRoutingTable::new();
        
        let route = routing_table.select_route("test", &LlmProvider::Ollama).unwrap();
        assert_eq!(route.provider, LlmProvider::Ollama);
    }
}
