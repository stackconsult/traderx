pub mod adapters;
pub mod agents;
pub mod api_server;
pub mod backtest;
pub mod cross_market;
pub mod disruptor;
pub mod engineering_orchestra;
pub mod fabric;
pub mod health;
pub mod infra;
pub mod integration;
pub mod journal;
pub mod llm;
pub mod mesh;
pub mod metrics;
pub mod metrics_server;
pub mod middleware;
pub mod ml;
pub mod neural;
pub mod observability;
pub mod observability_server;
pub mod oms;
pub mod orders;
pub mod portfolio;
pub mod portfolio_fabric;
pub mod risk_bus;
pub mod signal_router;
pub mod stability;
pub mod state_machine;
pub mod tools;

pub use adapters::{AdapterConfig, AdapterManager, BinanceAdapter, BybitAdapter, ExchangeAdapter};
pub use agents::{
    Agent, AgentOrchestrator, AgentRole, SignalGeneratorAgent, Task, TaskResult, Workflow,
};
pub use backtest::{
    BacktestConfig, BacktestEngine, BacktestResult, OrderBook, QueuePositionModel, Tick,
};
pub use disruptor::{Barrier, Disruptor, EventProcessor, Sequence};
pub use integration::{
    create_oms_engine, create_risk_bus, create_signal_router, create_trading_system,
    SystemChannels, SystemConfig, TradingSystem,
};
pub use journal::{EventJournal, JournalConfig, JournalEntry};
pub use oms::{OmsEngine, OmsError, OmsEvent, Result};
pub use orders::advanced::TimeInForce as AdvancedTimeInForce;
pub use orders::{AdvancedOrder, AdvancedOrderBuilder, AdvancedOrderType};
pub use risk_bus::RiskBus;
pub use signal_router::{AgentSignal, RouteOutcome, RouteStatus, RouterConfig, SignalRouter};
pub use state_machine::{Order, OrderEvent, OrderState, OrderType, Side, StateMachineError};
