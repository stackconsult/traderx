use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use thiserror::Error;

pub mod client;
pub mod prompt;
pub mod context;
pub mod router;
pub mod ollama_client;
pub mod hybrid_router;

pub use client::LlmClient;
pub use prompt::PromptEngine;
pub use context::ContextManager;
pub use router::AgentRouter;
pub use ollama_client::{OllamaClient, OllamaModel, OllamaGenerateRequest, OllamaGenerateResponse};
pub use hybrid_router::{HybridProviderRouter, SelectionStrategy, ProviderPerformance, RoutingDecision, CloudLlmClient, ConvictionThresholds, LatencyThresholds};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSignal {
    pub agent_id: String,
    pub symbol: String,
    pub direction: String,
    pub conviction: f64,
    pub max_notional_usd: f64,
    pub ttl_ms: u64,
    pub meta: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub request_id: Uuid,
    pub agent_id: String,
    pub response_type: ResponseType,
    pub content: String,
    pub confidence: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseType {
    Analysis,
    Recommendation,
    RiskAssessment,
    MarketSummary,
    Error,
}

#[derive(Debug, Error, Clone)]
pub enum LlmError {
    #[error("API key missing for {provider}")]
    MissingApiKey { provider: String },
    #[error("Request failed: {0}")]
    RequestFailed(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Timeout: {0}")]
    Timeout(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Context too large")]
    ContextOverflow,
    #[error("Router error: {0}")]
    RouterError(String),
}

pub type LlmResult<T> = Result<T, LlmError>;

#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub request_id: Uuid,
    pub prompt: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub metadata: HashMap<String, String>,
}

impl Default for LlmRequest {
    fn default() -> Self {
        Self {
            request_id: Uuid::new_v4(),
            prompt: String::new(),
            model: "gpt-4".to_string(),
            max_tokens: 2048,
            temperature: 0.7,
            metadata: HashMap::new(),
        }
    }
}

pub struct LlmEngine {
    client: Arc<client::LlmClient>,
    prompt_engine: Arc<prompt::PromptEngine>,
    context_manager: Arc<context::ContextManager>,
    router: Arc<router::AgentRouter>,
    request_tx: mpsc::Sender<LlmRequest>,
}

impl LlmEngine {
    pub fn new(
        client: Arc<client::LlmClient>,
        prompt_engine: Arc<prompt::PromptEngine>,
        context_manager: Arc<context::ContextManager>,
        router: Arc<router::AgentRouter>,
    ) -> (Self, mpsc::Receiver<LlmRequest>) {
        let (request_tx, request_rx) = mpsc::channel(1024);
        (
            Self {
                client,
                prompt_engine,
                context_manager,
                router,
                request_tx,
            },
            request_rx,
        )
    }

    pub async fn process_signal(&self, signal: AgentSignal) -> LlmResult<AgentResponse> {
        let request = self.router.route_signal(&signal).await?;
        let prompt = self.prompt_engine.render(&request).await?;
        let enriched_prompt = self.context_manager.enrich(&prompt, &signal.agent_id).await?;
        
        let llm_request = LlmRequest {
            prompt: enriched_prompt,
            metadata: {
                let mut m = HashMap::new();
                m.insert("agent_id".to_string(), signal.agent_id.clone());
                m.insert("symbol".to_string(), signal.symbol.clone());
                m
            },
            ..Default::default()
        };
        
        let _ = self.request_tx.send(llm_request.clone()).await;
        
        let response = self.client.complete(llm_request).await?;
        
        self.context_manager
            .store_interaction(&signal.agent_id, &request.prompt, &response.content)
            .await?;
        
        Ok(response)
    }
}
