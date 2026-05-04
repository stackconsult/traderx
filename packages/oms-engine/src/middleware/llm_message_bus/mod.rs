//! LLM-aware message bus with context propagation and routing
//!
//! Provides intelligent message routing, context propagation, and circuit breaking
//! for LLM provider interactions in the trading system.

mod types;
mod routing;
mod circuit_breaker;
mod context_propagator;

pub use types::{
    LlmMessage, LlmMessageType, LlmMessagePayload, MessagePriority,
    MessageContext, TradingContext, UserContext, SystemContext,
    LlmProvider
};
pub use routing::{
    LlmRoutingTable, LlmRoute, RoutingCondition, RouteHealth,
    LlmLoadBalancer, ProviderStats, BalancingStrategy
};
pub use circuit_breaker::{CircuitBreaker, CircuitState};
pub use context_propagator::{ContextPropagator, PropagationRule, PropagationCondition};

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Mutex};
use tokio::time::interval;
use tracing::{info, warn};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::llm::{AgentResponse, LlmResult, LlmError};
use crate::observability::{AgentMetrics, StructuredLogger};

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
    
    pub async fn send_message(&mut self, message: LlmMessage) -> Result<(), LlmError> {
        {
            let mut queue = self.message_queue.lock().await;
            if queue.len() >= self.max_queue_size {
                return Err(LlmError::ContextOverflow);
            }
            queue.push(message);
        }
        
        if let Some(metrics) = &self.metrics {
            metrics.llm_requests_total.inc();
        }
        
        Ok(())
    }
    
    pub async fn process_messages(&mut self) -> Vec<LlmResult<LlmMessage>> {
        let mut results = Vec::new();
        
        let messages: Vec<LlmMessage> = {
            let mut queue = self.message_queue.lock().await;
            let batch_size = queue.len().min(100);
            queue.drain(0..batch_size).collect()
        };
        
        for message in messages {
            let result = self.process_single_message(message).await;
            results.push(result);
        }
        
        results
    }
    
    async fn process_single_message(&mut self, message: LlmMessage) -> LlmResult<LlmMessage> {
        let start_time = Instant::now();
        
        if Utc::now().signed_duration_since(message.timestamp).to_std().unwrap_or(Duration::MAX) > message.ttl {
            return Err(LlmError::Timeout("Message expired".to_string()));
        }
        
        let enriched_message = self.context_propagator
            .propagate_context(message)
            .await?;
        
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
        
        let duration = start_time.elapsed();
        if let Some(metrics) = &self.metrics {
            metrics.llm_request_duration.observe(duration.as_secs_f64());
        }
        
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
    
    async fn process_llm_request(
        &mut self,
        mut message: LlmMessage,
        model: &str,
        provider: &LlmProvider,
    ) -> LlmResult<LlmMessage> {
        if let Some(circuit_breaker) = self.circuit_breakers.get_mut(provider) {
            if !circuit_breaker.allow_request() {
                return Err(LlmError::RateLimitExceeded);
            }
        }
        
        let _route = self.routing_table.select_route(model, provider)?;
        
        let result = match provider {
            LlmProvider::Ollama => {
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
        
        if let Some(circuit_breaker) = self.circuit_breakers.get_mut(provider) {
            if result.is_ok() {
                circuit_breaker.record_success();
            } else {
                circuit_breaker.record_failure();
            }
        }
        
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
                message.retry_count += 1;
                if message.retry_count < message.max_retries {
                    self.send_message(message.clone()).await?;
                }
                Err(e)
            }
        }
    }
    
    async fn process_llm_response(
        &mut self,
        message: LlmMessage,
        _model: &str,
        _provider: &LlmProvider,
    ) -> LlmResult<LlmMessage> {
        self.context_propagator
            .update_context_from_response(&message)
            .await?;
        
        Ok(message)
    }
    
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
    
    async fn process_model_status(
        &mut self,
        message: LlmMessage,
        model: &str,
        status: &str,
    ) -> LlmResult<LlmMessage> {
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
    
    async fn process_context_update(&mut self, message: LlmMessage) -> LlmResult<LlmMessage> {
        if let LlmMessagePayload::Context { key, value } = &message.payload {
            self.context_propagator
                .update_context(message.correlation_id, key.clone(), value.clone())
                .await?;
        }
        
        Ok(message)
    }
    
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
    
    pub async fn start_processing(&mut self) {
        info!("Starting LLM message bus processing");
        
        let mut interval = interval(self.processing_interval);
        
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let results = self.process_messages().await;
                    
                    let success_count = results.iter().filter(|r| r.is_ok()).count();
                    let error_count = results.len() - success_count;
                    
                    if error_count > 0 {
                        warn!("Message processing: {} success, {} errors", success_count, error_count);
                    }
                }
                
                _ = tokio::signal::ctrl_c() => {
                    info!("Stopping LLM message bus processing");
                    break;
                }
            }
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
        
        assert!(breaker.allow_request());
        
        for _ in 0..3 {
            breaker.record_failure();
        }
        
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
