use async_trait::async_trait;
use thiserror::Error;

use super::types::{AgentId, AgentRole, AgentState, Task, Context, TaskResult, AgentHealth};

/// Agent error types
#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Task execution failed: {0}")]
    Execution(String),
    #[error("Invalid task type for this agent")]
    InvalidTaskType,
    #[error("Context missing required data: {0}")]
    MissingContext(String),
    #[error("State operation failed: {0}")]
    StateError(String),
    #[error("Communication error: {0}")]
    Communication(String),
}

/// Agent trait - core interface for all agents
#[async_trait]
pub trait Agent: Send + Sync {
    fn id(&self) -> AgentId;
    fn role(&self) -> AgentRole;
    fn name(&self) -> &str;
    async fn execute(&self, task: Task, ctx: Context) -> Result<TaskResult, AgentError>;
    async fn update_state(&self, state: AgentState) -> Result<(), AgentError>;
    async fn get_state(&self) -> Result<AgentState, AgentError>;
    async fn health_check(&self) -> AgentHealth;
}
