pub mod disruptor;
pub mod journal;
pub mod state_machine;
pub mod llm;
pub mod ml;
pub mod neural;
pub mod middleware;
pub mod observability;
pub mod signal_router;
pub mod risk_bus;
pub mod integration;
pub mod oms;
pub mod stability;
pub mod health;
pub mod observability_server;
pub mod orders;
pub mod adapters;
pub mod backtest;
pub mod agents;
pub mod portfolio;
pub mod metrics;
pub mod metrics_server;
pub mod engineering_orchestra;
pub mod cross_market;

pub use risk_bus::RiskBus;
pub use signal_router::{SignalRouter, AgentSignal, RouterConfig, RouteOutcome, RouteStatus};
pub use oms::{OmsEngine, OmsEvent, OmsError, Result};
pub use state_machine::{Order, OrderState, OrderEvent, Side, OrderType, StateMachineError};
pub use disruptor::{Disruptor, EventProcessor, Sequence, Barrier};
pub use journal::{EventJournal, JournalEntry, JournalConfig};
pub use orders::{AdvancedOrder, AdvancedOrderBuilder, AdvancedOrderType};
pub use orders::advanced::TimeInForce as AdvancedTimeInForce;
pub use adapters::{ExchangeAdapter, AdapterConfig, AdapterManager, BinanceAdapter, BybitAdapter};
pub use backtest::{BacktestEngine, BacktestConfig, BacktestResult, OrderBook, Tick, QueuePositionModel};
pub use agents::{Agent, AgentOrchestrator, AgentRole, Task, TaskResult, Workflow, SignalGeneratorAgent};
pub use integration::{
    create_trading_system, create_oms_engine, create_signal_router, create_risk_bus,
    TradingSystem, SystemConfig, SystemChannels
};
