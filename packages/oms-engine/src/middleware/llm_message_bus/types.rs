use std::collections::HashMap;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ============================================================================
// MESSAGE TYPES
// ============================================================================

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
    Request(serde_json::Value),
    Response(crate::llm::AgentResponse),
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

// ============================================================================
// CONTEXT STRUCTURES
// ============================================================================

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

// ============================================================================
// PROVIDER TYPES
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LlmProvider {
    Ollama,
    OpenAI,
    Anthropic,
    Hybrid,
}
