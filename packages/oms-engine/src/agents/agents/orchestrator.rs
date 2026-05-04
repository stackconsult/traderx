use chrono::Utc;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

use super::agent::{Agent, AgentError};
use super::types::{
    AgentId, AgentRole, Context, DecisionCondition, OrchestratorEvent, ResultOutput, Task,
    TaskResult, TaskStatus, Workflow, WorkflowNode, WorkflowResult, WorkflowStatus,
};

/// Agent orchestrator - manages multi-agent workflows
pub struct AgentOrchestrator {
    agents: Arc<Mutex<HashMap<AgentId, Box<dyn Agent>>>>,
    workflows: Arc<Mutex<HashMap<Uuid, Workflow>>>,
    shared_state: Arc<Mutex<HashMap<String, serde_json::Value>>>,
    approval_system: Option<super::types::HumanApprovalSystem>,
    event_bus: mpsc::Sender<OrchestratorEvent>,
}

impl AgentOrchestrator {
    pub fn new(event_bus: mpsc::Sender<OrchestratorEvent>) -> Self {
        Self {
            agents: Arc::new(Mutex::new(HashMap::new())),
            workflows: Arc::new(Mutex::new(HashMap::new())),
            shared_state: Arc::new(Mutex::new(HashMap::new())),
            approval_system: None,
            event_bus,
        }
    }

    pub async fn register_agent(&self, agent: Box<dyn Agent>) {
        let id = agent.id();
        let role = agent.role();

        self.agents.lock().await.insert(id, agent);

        let _ = self
            .event_bus
            .send(OrchestratorEvent::AgentRegistered { agent_id: id, role })
            .await;
    }

    pub async fn get_agent_by_role(&self, role: AgentRole) -> Option<Box<dyn Agent>> {
        let agents = self.agents.lock().await;
        agents
            .values()
            .find(|a| a.role() == role)
            .map(|a| unsafe { std::ptr::read(a as *const Box<dyn Agent>) })
    }

    pub async fn register_workflow(&self, workflow: Workflow) {
        self.workflows.lock().await.insert(workflow.id, workflow);
    }

    pub async fn execute_workflow(
        &self,
        workflow_id: Uuid,
        ctx: Context,
    ) -> Result<WorkflowResult, AgentError> {
        let workflows = self.workflows.lock().await;
        let workflow_ref = workflows
            .get(&workflow_id)
            .ok_or_else(|| AgentError::Execution("Workflow not found".to_string()))?;
        let _workflow_name = workflow_ref.name.clone();
        let _workflow_timeout = workflow_ref.timeout_secs;

        let _ = self
            .event_bus
            .send(OrchestratorEvent::WorkflowStarted {
                workflow_id,
                timestamp: Utc::now(),
            })
            .await;

        let start_time = Instant::now();

        let result = self.execute_node(&workflow_ref.root, ctx).await;

        let execution_time = start_time.elapsed().as_millis() as u64;

        let workflow_result = WorkflowResult {
            workflow_id,
            status: match &result {
                Ok(_) => WorkflowStatus::Completed,
                Err(_) => WorkflowStatus::Failed,
            },
            node_results: vec![],
            execution_time_ms: execution_time,
        };

        let _ = self
            .event_bus
            .send(OrchestratorEvent::WorkflowCompleted {
                workflow_id,
                result: workflow_result.clone(),
            })
            .await;

        result.map(|_| workflow_result)
    }

    fn execute_node<'a>(
        &'a self,
        node: &'a WorkflowNode,
        ctx: Context,
    ) -> Pin<Box<dyn Future<Output = Result<TaskResult, AgentError>> + Send + 'a>> {
        Box::pin(async move {
            match node {
                WorkflowNode::Agent {
                    agent_id,
                    task_generator,
                } => {
                    let agents = self.agents.lock().await;
                    let agent = agents
                        .get(agent_id)
                        .ok_or_else(|| AgentError::Execution("Agent not found".to_string()))?;

                    let task = task_generator(&ctx);
                    agent.execute(task, ctx).await
                }
                WorkflowNode::Decision {
                    condition,
                    true_branch,
                    false_branch,
                } => {
                    let result = self.evaluate_condition(condition, &ctx);
                    if result {
                        self.execute_node(true_branch, ctx).await
                    } else {
                        self.execute_node(false_branch, ctx).await
                    }
                }
                WorkflowNode::HumanCheckpoint { .. } => Ok(TaskResult {
                    task_id: Uuid::new_v4(),
                    status: TaskStatus::PendingApproval,
                    output: ResultOutput::None,
                    execution_time_ms: 0,
                }),
                WorkflowNode::Sequence { nodes } => {
                    for node in nodes {
                        let _ = self.execute_node(node, ctx.clone()).await?;
                    }
                    Ok(TaskResult {
                        task_id: Uuid::new_v4(),
                        status: TaskStatus::Success,
                        output: ResultOutput::None,
                        execution_time_ms: 0,
                    })
                }
                _ => Ok(TaskResult {
                    task_id: Uuid::new_v4(),
                    status: TaskStatus::Success,
                    output: ResultOutput::None,
                    execution_time_ms: 0,
                }),
            }
        })
    }

    fn evaluate_condition(&self, condition: &DecisionCondition, ctx: &Context) -> bool {
        match condition {
            DecisionCondition::RiskScoreAbove { threshold: _ } => false,
            DecisionCondition::PositionSizeAbove { threshold } => {
                ctx.portfolio.total_value > *threshold
            }
            _ => false,
        }
    }

    pub async fn get_shared_state(&self, key: &str) -> Option<serde_json::Value> {
        self.shared_state.lock().await.get(key).cloned()
    }

    pub async fn set_shared_state(&self, key: String, value: serde_json::Value) {
        self.shared_state.lock().await.insert(key, value);
    }
}
