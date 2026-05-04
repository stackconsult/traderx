use async_trait::async_trait;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::Mutex;
use uuid::Uuid;

use super::agent::{Agent, AgentError};
use super::types::{
    AgentHealth, AgentId, AgentRole, AgentState, Context, ResultOutput, SignalAction, Task,
    TaskResult, TaskStatus, TaskType,
};

/// Built-in signal generator agent
pub struct SignalGeneratorAgent {
    id: AgentId,
    name: String,
    state: Arc<Mutex<AgentState>>,
}

impl SignalGeneratorAgent {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            state: Arc::new(Mutex::new(AgentState::default())),
        }
    }
}

#[async_trait]
impl Agent for SignalGeneratorAgent {
    fn id(&self) -> AgentId {
        self.id
    }

    fn role(&self) -> AgentRole {
        AgentRole::SignalGenerator
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn execute(&self, task: Task, ctx: Context) -> Result<TaskResult, AgentError> {
        let start = Instant::now();

        match task.task_type {
            TaskType::AnalyzeMarket => {
                let mut signals = vec![];

                for (symbol, snapshot) in &ctx.market_data {
                    if snapshot.change_24h_pct > 5.0 {
                        signals.push((symbol.clone(), SignalAction::Buy, snapshot.change_24h_pct));
                    } else if snapshot.change_24h_pct < -5.0 {
                        signals.push((
                            symbol.clone(),
                            SignalAction::Sell,
                            -snapshot.change_24h_pct,
                        ));
                    }
                }

                let execution_time = start.elapsed().as_millis() as u64;

                Ok(TaskResult {
                    task_id: task.id,
                    status: TaskStatus::Success,
                    output: ResultOutput::Signal {
                        action: if signals.is_empty() {
                            SignalAction::Hold
                        } else {
                            SignalAction::Buy
                        },
                        confidence: signals.iter().map(|(_, _, c)| *c).sum::<f64>()
                            / signals.len().max(1) as f64,
                    },
                    execution_time_ms: execution_time,
                })
            }
            _ => Err(AgentError::InvalidTaskType),
        }
    }

    async fn update_state(&self, new_state: AgentState) -> Result<(), AgentError> {
        *self.state.lock().await = new_state;
        Ok(())
    }

    async fn get_state(&self) -> Result<AgentState, AgentError> {
        Ok(self.state.lock().await.clone())
    }

    async fn health_check(&self) -> AgentHealth {
        AgentHealth::Healthy
    }
}
