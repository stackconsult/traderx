//! Multi-Agent Orchestration System
//!
//! Collaborative agent framework for multi-agent trading systems with state persistence and human oversight.

pub mod agents;

pub use agents::{
    AgentId, AgentRole, AgentState, AgentMemory, MemoryEventType, AgentMetrics,
    Task, TaskType, TaskPriority, TaskPayload, PositionUpdate, TaskResult, TaskStatus,
    ResultOutput, SignalAction, Context, MarketSnapshot, PortfolioState, Position,
    RiskLimits, AgentHealth, WorkflowNode, DecisionCondition, AggregationStrategy,
    Workflow, RetryPolicy, WorkflowResult, WorkflowStatus, NodeResult,
    HumanApprovalSystem, ApprovalRequest, RiskAssessment, ApprovalResponse,
    OrchestratorEvent, Agent, AgentError, AgentOrchestrator, SignalGeneratorAgent
};
