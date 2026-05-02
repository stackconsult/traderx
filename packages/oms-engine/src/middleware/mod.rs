use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use chrono::Utc;

pub mod message_bus;
pub mod llm_message_bus;
pub mod function_orchestrator;

pub use message_bus::MessageBus;
pub use llm_message_bus::{
    LlmMessageBus, LlmMessage, LlmMessageType, LlmProvider, LlmMessagePayload,
    MessagePriority, MessageContext, TradingContext, UserContext, SystemContext,
    LlmRoutingTable, LlmLoadBalancer, ContextPropagator, CircuitBreaker
};
pub use function_orchestrator::{
    FunctionOrchestrator, ExecutionStep, Workflow, StepStatus, WorkflowStatus,
    ValidationRule, ValidationType, ValidationResult, ExecutionContext, FunctionHandler
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentMessage {
    LlmRequest {
        request_id: Uuid,
        payload: String,
        sender: String,
        timestamp: chrono::DateTime<Utc>,
    },
    LlmResponse {
        request_id: Uuid,
        payload: String,
        confidence: f64,
        timestamp: chrono::DateTime<Utc>,
    },
    MlRequest {
        request_id: Uuid,
        payload: String,
        sender: String,
        timestamp: chrono::DateTime<Utc>,
    },
    MlResponse {
        request_id: Uuid,
        payload: String,
        confidence: f64,
        timestamp: chrono::DateTime<Utc>,
    },
    NeuralRequest {
        request_id: Uuid,
        payload: Vec<f32>,
        sender: String,
        timestamp: chrono::DateTime<Utc>,
    },
    NeuralResponse {
        request_id: Uuid,
        payload: Vec<f32>,
        confidence: f64,
        timestamp: chrono::DateTime<Utc>,
    },
    Signal {
        signal: crate::llm::AgentSignal,
        timestamp: chrono::DateTime<Utc>,
    },
    HealthCheck {
        request_id: Uuid,
        timestamp: chrono::DateTime<Utc>,
    },
    Shutdown,
}

#[derive(Debug, Error)]
pub enum MiddlewareError {
    #[error("Channel closed")]
    ChannelClosed,
    #[error("Invalid message type: {0}")]
    InvalidMessageType(String),
    #[error("Routing failed: {0}")]
    RoutingFailed(String),
    #[error("Timeout")]
    Timeout,
}

pub type MiddlewareResult<T> = Result<T, MiddlewareError>;

pub struct AgentOrchestrator {
    llm_tx: mpsc::Sender<AgentMessage>,
    ml_tx: mpsc::Sender<AgentMessage>,
    neural_tx: mpsc::Sender<AgentMessage>,
    message_count: Arc<RwLock<u64>>,
}

impl AgentOrchestrator {
    pub fn new(
        llm_tx: mpsc::Sender<AgentMessage>,
        ml_tx: mpsc::Sender<AgentMessage>,
        neural_tx: mpsc::Sender<AgentMessage>,
    ) -> Self {
        Self {
            llm_tx,
            ml_tx,
            neural_tx,
            message_count: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn route(&self, message: AgentMessage) -> MiddlewareResult<()> {
        let mut count = self.message_count.write().await;
        *count += 1;
        drop(count);
        
        match &message {
            AgentMessage::LlmRequest { .. } | AgentMessage::LlmResponse { .. } => {
                self.llm_tx.send(message).await.map_err(|_| MiddlewareError::ChannelClosed)?;
            }
            AgentMessage::MlRequest { .. } | AgentMessage::MlResponse { .. } => {
                self.ml_tx.send(message).await.map_err(|_| MiddlewareError::ChannelClosed)?;
            }
            AgentMessage::NeuralRequest { .. } | AgentMessage::NeuralResponse { .. } => {
                self.neural_tx.send(message).await.map_err(|_| MiddlewareError::ChannelClosed)?;
            }
            AgentMessage::Signal { signal, .. } => {
                let confidence = signal.conviction;
                if confidence > 0.8 {
                    self.neural_tx.send(message.clone()).await.map_err(|_| MiddlewareError::ChannelClosed)?;
                } else if confidence > 0.5 {
                    self.ml_tx.send(message.clone()).await.map_err(|_| MiddlewareError::ChannelClosed)?;
                } else {
                    self.llm_tx.send(message).await.map_err(|_| MiddlewareError::ChannelClosed)?;
                }
            }
            AgentMessage::HealthCheck { .. } => {
                self.llm_tx.send(message.clone()).await.ok();
                self.ml_tx.send(message.clone()).await.ok();
                self.neural_tx.send(message).await.ok();
            }
            AgentMessage::Shutdown => {
                self.llm_tx.send(message.clone()).await.ok();
                self.ml_tx.send(message.clone()).await.ok();
                self.neural_tx.send(message).await.ok();
            }
        }
        Ok(())
    }

    pub async fn get_message_count(&self) -> u64 {
        *self.message_count.read().await
    }
}
