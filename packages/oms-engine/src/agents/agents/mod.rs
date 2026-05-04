//! Multi-Agent Orchestration System
//!
//! Collaborative agent framework for multi-agent trading systems with state persistence and human oversight.

use std::collections::HashMap;

mod agent;
mod orchestrator;
mod signal_generator;
mod types;

pub use agent::{Agent, AgentError};
pub use orchestrator::AgentOrchestrator;
pub use signal_generator::SignalGeneratorAgent;
pub use types::{
    AgentHealth, AgentId, AgentMemory, AgentMetrics, AgentRole, AgentState, AggregationStrategy,
    ApprovalRequest, ApprovalResponse, Context, DecisionCondition, HumanApprovalSystem,
    MarketSnapshot, MemoryEventType, NodeResult, OrchestratorEvent, PortfolioState, Position,
    PositionUpdate, ResultOutput, RetryPolicy, RiskAssessment, RiskLimits, SignalAction, Task,
    TaskPayload, TaskPriority, TaskResult, TaskStatus, TaskType, Workflow, WorkflowNode,
    WorkflowResult, WorkflowStatus,
};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_signal_generator_agent() {
        let agent = SignalGeneratorAgent::new("TestSignalGen");

        assert_eq!(agent.role(), AgentRole::SignalGenerator);

        let mut market_data = HashMap::new();
        market_data.insert(
            "BTCUSDT".to_string(),
            MarketSnapshot {
                price: rust_decimal::Decimal::from(50000),
                bid: rust_decimal::Decimal::from(49999),
                ask: rust_decimal::Decimal::from(50001),
                volume_24h: rust_decimal::Decimal::from(1000000),
                change_24h_pct: 6.5,
            },
        );

        let ctx = Context {
            timestamp: Utc::now(),
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
