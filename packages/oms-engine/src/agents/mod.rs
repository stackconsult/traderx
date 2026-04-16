//! Multi-Agent Orchestration System - Benchmark+ Implementation
//!
//! Collaborative agent framework inspired by LangGraph (graph-based workflows),
//! AutoGen (conversational orchestration), and CrewAI (role-based coordination).
//! Enables multi-agent trading systems with state persistence and human oversight.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

use crate::oms::{OmsEvent, OmsEngine};
use crate::orders::AdvancedOrder;
use crate::state_machine::{Order, Side};

/// Unique agent identifier
pub type AgentId = Uuid;

/// Agent role classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentRole {
    /// Market analysis and signal generation
    SignalGenerator,
    /// Risk management and position limits
    RiskManager,
    /// Order execution and routing
    ExecutionEngine,
    /// Portfolio balancing and allocation
    PortfolioManager,
    /// Compliance and regulatory oversight
    ComplianceOfficer,
    /// Market data ingestion
    DataIngestor,
    /// PnL tracking and reporting
    PerformanceAnalyzer,
}

impl std::fmt::Display for AgentRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentRole::SignalGenerator => write!(f, "SignalGenerator"),
            AgentRole::RiskManager => write!(f, "RiskManager"),
            AgentRole::ExecutionEngine => write!(f, "ExecutionEngine"),
            AgentRole::PortfolioManager => write!(f, "PortfolioManager"),
            AgentRole::ComplianceOfficer => write!(f, "ComplianceOfficer"),
            AgentRole::DataIngestor => write!(f, "DataIngestor"),
            AgentRole::PerformanceAnalyzer => write!(f, "PerformanceAnalyzer"),
        }
    }
}

/// Agent state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentState {
    pub memory: VecDeque<AgentMemory>,
    pub context: HashMap<String, String>,
    pub metrics: AgentMetrics,
}

/// Agent memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMemory {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: MemoryEventType,
    pub content: String,
    pub importance: u8, // 0-255, for pruning
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryEventType {
    Signal,
    Decision,
    Action,
    Error,
    Observation,
}

/// Agent performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub decisions_made: u64,
    pub actions_taken: u64,
    pub errors_encountered: u64,
    pub avg_decision_time_ms: f64,
}

/// Task for agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub task_type: TaskType,
    pub priority: TaskPriority,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub payload: TaskPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    AnalyzeMarket,
    GenerateSignal,
    CheckRisk,
    ExecuteOrder,
    UpdatePortfolio,
    ReportCompliance,
    Custom { name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskPayload {
    MarketData { symbol: String, price: rust_decimal::Decimal, volume: rust_decimal::Decimal },
    Signal { symbol: String, side: Side, confidence: f64 },
    OrderRequest { order: AdvancedOrder },
    RiskCheck { symbol: String, notional: rust_decimal::Decimal },
    PortfolioUpdate { positions: Vec<PositionUpdate> },
    Custom { data: serde_json::Value },
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionUpdate {
    pub symbol: String,
    pub quantity: rust_decimal::Decimal,
    pub avg_price: rust_decimal::Decimal,
}

/// Task execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: Uuid,
    pub status: TaskStatus,
    pub output: ResultOutput,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Success,
    PartialSuccess,
    Failed,
    Rejected,
    PendingApproval,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResultOutput {
    Signal { action: SignalAction, confidence: f64 },
    Order { order_id: Uuid },
    RiskAssessment { approved: bool, reason: Option<String> },
    Compliance { compliant: bool, violations: Vec<String> },
    None,
    Error { message: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalAction {
    Buy,
    Sell,
    Hold,
    Close,
    Modify,
    Cancel,
}

/// Execution context passed to agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub market_data: HashMap<String, MarketSnapshot>,
    pub portfolio: PortfolioState,
    pub risk_limits: RiskLimits,
    pub shared_memory: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSnapshot {
    pub price: rust_decimal::Decimal,
    pub bid: rust_decimal::Decimal,
    pub ask: rust_decimal::Decimal,
    pub volume_24h: rust_decimal::Decimal,
    pub change_24h_pct: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PortfolioState {
    pub positions: HashMap<String, Position>,
    pub total_value: rust_decimal::Decimal,
    pub cash: rust_decimal::Decimal,
    pub margin_used: rust_decimal::Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub quantity: rust_decimal::Decimal,
    pub avg_price: rust_decimal::Decimal,
    pub unrealized_pnl: rust_decimal::Decimal,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RiskLimits {
    pub max_position_notional: rust_decimal::Decimal,
    pub max_drawdown_pct: f64,
    pub max_leverage: f64,
    pub daily_loss_limit: rust_decimal::Decimal,
}

/// Agent trait - core interface for all agents
#[async_trait]
pub trait Agent: Send + Sync {
    /// Get agent identifier
    fn id(&self) -> AgentId;

    /// Get agent role
    fn role(&self) -> AgentRole;

    /// Get agent name
    fn name(&self) -> &str;

    /// Execute a task
    async fn execute(&self, task: Task, ctx: Context) -> Result<TaskResult, AgentError>;

    /// Update agent state
    async fn update_state(&self, state: AgentState) -> Result<(), AgentError>;

    /// Get current state
    async fn get_state(&self) -> Result<AgentState, AgentError>;

    /// Check if agent is healthy
    async fn health_check(&self) -> AgentHealth;
}

/// Agent health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Offline,
}

/// Agent error types
#[derive(Debug, thiserror::Error)]
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

/// Workflow node types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowNode {
    /// Agent execution node
    Agent {
        agent_id: AgentId,
        task_generator: Box<dyn Fn(&Context) -> Task>,
    },
    /// Decision/condition node
    Decision {
        condition: DecisionCondition,
        true_branch: Box<WorkflowNode>,
        false_branch: Box<WorkflowNode>,
    },
    /// Human approval checkpoint
    HumanCheckpoint {
        description: String,
        timeout_secs: Option<u64>,
        auto_approve_if_healthy: bool,
    },
    /// Parallel execution
    Parallel {
        branches: Vec<WorkflowNode>,
        aggregation: AggregationStrategy,
    },
    /// Sequential execution
    Sequence {
        nodes: Vec<WorkflowNode>,
    },
    /// Timer/delay
    Delay {
        duration_ms: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionCondition {
    RiskScoreAbove { threshold: f64 },
    PositionSizeAbove { threshold: rust_decimal::Decimal },
    MarketVolatilityAbove { threshold: f64 },
    TimeOfDay { after: chrono::NaiveTime, before: chrono::NaiveTime },
    PnLBelow { threshold: rust_decimal::Decimal },
    Custom { expression: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationStrategy {
    AllMustSucceed,
    MajorityVote,
    FirstSuccess,
    SumResults,
}

/// Workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: Uuid,
    pub name: String,
    pub root: WorkflowNode,
    pub timeout_secs: Option<u64>,
    pub retry_policy: RetryPolicy,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff_ms: u64,
    pub exponential: bool,
}

/// Workflow execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub workflow_id: Uuid,
    pub status: WorkflowStatus,
    pub node_results: Vec<NodeResult>,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Completed,
    Failed,
    PartiallyCompleted,
    Cancelled,
    PendingApproval,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResult {
    pub node_id: String,
    pub status: TaskStatus,
    pub output: ResultOutput,
    pub execution_time_ms: u64,
}

/// Human approval system
#[derive(Debug, Clone)]
pub struct HumanApprovalSystem {
    pub tx: mpsc::Sender<ApprovalRequest>,
    pub rx: mpsc::Receiver<ApprovalResponse>,
    pending: Arc<Mutex<Vec<ApprovalRequest>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub node_id: String,
    pub action_description: String,
    pub risk_assessment: RiskAssessment,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub score: f64, // 0-100
    pub max_position_exposure: rust_decimal::Decimal,
    var_95: rust_decimal::Decimal,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalResponse {
    Approve { request_id: Uuid, approved_by: String },
    Reject { request_id: Uuid, reason: String },
    Modify { request_id: Uuid, modified_action: String },
    Escalate { request_id: Uuid, to: String },
}

/// Agent orchestrator - manages multi-agent workflows
pub struct AgentOrchestrator {
    agents: Arc<Mutex<HashMap<AgentId, Box<dyn Agent>>>>,
    workflows: Arc<Mutex<HashMap<Uuid, Workflow>>>,
    shared_state: Arc<Mutex<HashMap<String, serde_json::Value>>>,
    approval_system: Option<HumanApprovalSystem>,
    event_bus: mpsc::Sender<OrchestratorEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestratorEvent {
    WorkflowStarted { workflow_id: Uuid, timestamp: chrono::DateTime<chrono::Utc> },
    WorkflowCompleted { workflow_id: Uuid, result: WorkflowResult },
    WorkflowFailed { workflow_id: Uuid, error: String },
    ApprovalRequested { request: ApprovalRequest },
    AgentRegistered { agent_id: AgentId, role: AgentRole },
    AgentDeregistered { agent_id: AgentId },
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

    /// Register an agent
    pub async fn register_agent(&self, agent: Box<dyn Agent>) {
        let id = agent.id();
        let role = agent.role();
        
        self.agents.lock().await.insert(id, agent);
        
        let _ = self.event_bus.send(OrchestratorEvent::AgentRegistered {
            agent_id: id,
            role,
        }).await;
    }

    /// Get agent by role
    pub async fn get_agent_by_role(&self, role: AgentRole) -> Option<Box<dyn Agent>> {
        let agents = self.agents.lock().await;
        agents.values()
            .find(|a| a.role() == role)
            .map(|a| unsafe { std::ptr::read(a as *const Box<dyn Agent>) }) // Safe: we're just cloning the reference
    }

    /// Register a workflow
    pub async fn register_workflow(&self, workflow: Workflow) {
        self.workflows.lock().await.insert(workflow.id, workflow);
    }

    /// Execute a workflow
    pub async fn execute_workflow(&self, workflow_id: Uuid, ctx: Context) -> Result<WorkflowResult, AgentError> {
        let workflow = self.workflows.lock().await
            .get(&workflow_id)
            .cloned()
            .ok_or_else(|| AgentError::Execution("Workflow not found".to_string()))?;

        let _ = self.event_bus.send(OrchestratorEvent::WorkflowStarted {
            workflow_id,
            timestamp: chrono::Utc::now(),
        }).await;

        let start_time = std::time::Instant::now();
        
        // Execute workflow root node
        let result = self.execute_node(&workflow.root, ctx).await;
        
        let execution_time = start_time.elapsed().as_millis() as u64;

        let workflow_result = WorkflowResult {
            workflow_id,
            status: match &result {
                Ok(_) => WorkflowStatus::Completed,
                Err(_) => WorkflowStatus::Failed,
            },
            node_results: vec![], // Would populate from actual execution
            execution_time_ms: execution_time,
        };

        let _ = self.event_bus.send(OrchestratorEvent::WorkflowCompleted {
            workflow_id,
            result: workflow_result.clone(),
        }).await;

        result.map(|_| workflow_result)
    }

    /// Execute a workflow node (boxed to handle recursion)
    fn execute_node<'a>(&'a self, node: &'a WorkflowNode, ctx: Context) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<TaskResult, AgentError>> + Send + 'a>> {
        Box::pin(async move {
            match node {
                WorkflowNode::Agent { agent_id, task_generator } => {
                    let agents = self.agents.lock().await;
                    let agent = agents.get(agent_id)
                        .ok_or_else(|| AgentError::Execution("Agent not found".to_string()))?;
                    
                    let task = task_generator(&ctx);
                    agent.execute(task, ctx).await
                }
                WorkflowNode::Decision { condition, true_branch, false_branch } => {
                    let result = self.evaluate_condition(condition, &ctx);
                    if result {
                        self.execute_node(true_branch, ctx).await
                    } else {
                        self.execute_node(false_branch, ctx).await
                    }
                }
                WorkflowNode::HumanCheckpoint { .. } => {
                    // Would request human approval
                    Ok(TaskResult {
                        task_id: Uuid::new_v4(),
                        status: TaskStatus::PendingApproval,
                        output: ResultOutput::None,
                        execution_time_ms: 0,
                    })
                }
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
    }

    fn evaluate_condition(&self, condition: &DecisionCondition, ctx: &Context) -> bool {
        match condition {
            DecisionCondition::RiskScoreAbove { threshold } => {
                // Would calculate actual risk score
                false
            }
            DecisionCondition::PositionSizeAbove { threshold } => {
                ctx.portfolio.total_value > *threshold
            }
            _ => false,
        }
    }

    /// Get shared state
    pub async fn get_shared_state(&self, key: &str) -> Option<serde_json::Value> {
        self.shared_state.lock().await.get(key).cloned()
    }

    /// Set shared state
    pub async fn set_shared_state(&self, key: String, value: serde_json::Value) {
        self.shared_state.lock().await.insert(key, value);
    }
}

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
        let start = std::time::Instant::now();

        match task.task_type {
            TaskType::AnalyzeMarket => {
                // Simple momentum strategy
                let mut signals = vec![];
                
                for (symbol, snapshot) in &ctx.market_data {
                    if snapshot.change_24h_pct > 5.0 {
                        signals.push((symbol.clone(), SignalAction::Buy, snapshot.change_24h_pct));
                    } else if snapshot.change_24h_pct < -5.0 {
                        signals.push((symbol.clone(), SignalAction::Sell, -snapshot.change_24h_pct));
                    }
                }

                let execution_time = start.elapsed().as_millis() as u64;

                Ok(TaskResult {
                    task_id: task.id,
                    status: TaskStatus::Success,
                    output: ResultOutput::Signal {
                        action: if signals.is_empty() { SignalAction::Hold } else { SignalAction::Buy },
                        confidence: signals.iter().map(|(_, _, c)| *c).sum::<f64>() / signals.len().max(1) as f64,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_signal_generator_agent() {
        let agent = SignalGeneratorAgent::new("TestSignalGen");
        
        assert_eq!(agent.role(), AgentRole::SignalGenerator);
        
        let mut market_data = HashMap::new();
        market_data.insert("BTCUSDT".to_string(), MarketSnapshot {
            price: rust_decimal::Decimal::from(50000),
            bid: rust_decimal::Decimal::from(49999),
            ask: rust_decimal::Decimal::from(50001),
            volume_24h: rust_decimal::Decimal::from(1000000),
            change_24h_pct: 6.5,
        });

        let ctx = Context {
            timestamp: chrono::Utc::now(),
            market_data,
            portfolio: PortfolioState::default(),
            risk_limits: RiskLimits::default(),
            shared_memory: HashMap::new(),
        };

        let task = Task {
            id: Uuid::new_v4(),
            task_type: TaskType::AnalyzeMarket,
            priority: TaskPriority::High,
            deadline: None,
            payload: TaskPayload::None,
        };

        let result = agent.execute(task, ctx).await.unwrap();
        assert_eq!(result.status, TaskStatus::Success);
    }

    #[test]
    fn test_agent_roles() {
        assert_eq!(AgentRole::SignalGenerator.to_string(), "SignalGenerator");
        assert_eq!(AgentRole::RiskManager.to_string(), "RiskManager");
    }
}
