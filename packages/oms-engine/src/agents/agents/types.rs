use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;
use chrono::{DateTime, NaiveTime, Utc};

use crate::orders::AdvancedOrder;
use crate::state_machine::Side;

/// Unique agent identifier
pub type AgentId = Uuid;

/// Agent role classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentRole {
    SignalGenerator,
    RiskManager,
    ExecutionEngine,
    PortfolioManager,
    ComplianceOfficer,
    DataIngestor,
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
    pub timestamp: DateTime<Utc>,
    pub event_type: MemoryEventType,
    pub content: String,
    pub importance: u8,
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
    pub deadline: Option<DateTime<Utc>>,
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
    pub timestamp: DateTime<Utc>,
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

/// Agent health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Offline,
}

/// Workflow node types
pub enum WorkflowNode {
    Agent {
        agent_id: AgentId,
        task_generator: Box<dyn Fn(&Context) -> Task + Send + Sync>,
    },
    Decision {
        condition: DecisionCondition,
        true_branch: Box<WorkflowNode>,
        false_branch: Box<WorkflowNode>,
    },
    HumanCheckpoint {
        description: String,
        timeout_secs: Option<u64>,
        auto_approve_if_healthy: bool,
    },
    Parallel {
        branches: Vec<WorkflowNode>,
        aggregation: AggregationStrategy,
    },
    Sequence {
        nodes: Vec<WorkflowNode>,
    },
    Delay {
        duration_ms: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionCondition {
    RiskScoreAbove { threshold: f64 },
    PositionSizeAbove { threshold: rust_decimal::Decimal },
    MarketVolatilityAbove { threshold: f64 },
    TimeOfDay { after: NaiveTime, before: NaiveTime },
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
#[derive(Debug)]
pub struct HumanApprovalSystem {
    pub tx: mpsc::Sender<ApprovalRequest>,
    pub rx: mpsc::Receiver<ApprovalResponse>,
    pub pending: Arc<Mutex<Vec<ApprovalRequest>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub node_id: String,
    pub action_description: String,
    pub risk_assessment: RiskAssessment,
    pub deadline: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub score: f64,
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

/// Orchestrator events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestratorEvent {
    WorkflowStarted { workflow_id: Uuid, timestamp: DateTime<Utc> },
    WorkflowCompleted { workflow_id: Uuid, result: WorkflowResult },
    WorkflowFailed { workflow_id: Uuid, error: String },
    ApprovalRequested { request: ApprovalRequest },
    AgentRegistered { agent_id: AgentId, role: AgentRole },
    AgentDeregistered { agent_id: AgentId },
}
