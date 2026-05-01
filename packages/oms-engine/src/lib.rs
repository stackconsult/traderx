pub mod disruptor;
pub mod journal;
pub mod state_machine;
pub mod protocol;
pub mod oms;
pub mod aeron_journal;
pub mod risk_bus;
pub mod signal_router;
pub mod portfolio;
pub mod metrics;
pub mod metrics_server;
pub mod health;
pub mod observability_server;
pub mod orders;
pub mod adapters;
pub mod backtest;
pub mod agents;
pub mod integration;
pub mod engineering_orchestra;
pub mod llm;
pub mod ml;
pub mod neural;
pub mod observability;
pub mod middleware;

pub use risk_bus::RiskBus;
pub use signal_router::{SignalRouter, AgentSignal, RouterConfig};
pub use oms::{OmsEngine, OmsEvent, OmsError, Result};
pub use state_machine::{Order, OrderState, OrderEvent, Side, OrderType, StateMachineError};
pub use disruptor::{Disruptor, EventProcessor, Sequence, Barrier};
pub use journal::{EventJournal, JournalEntry, JournalConfig};
pub use protocol::{OrderProtocol, OrderFrame, SBEProtocol, ITCHProtocol, TimeInForce};
pub use aeron_journal::AeronJournal;
pub use orders::{AdvancedOrder, AdvancedOrderBuilder, AdvancedOrderType};
pub use orders::advanced::TimeInForce as AdvancedTimeInForce;
pub use adapters::{ExchangeAdapter, AdapterConfig, AdapterManager, BinanceAdapter, BybitAdapter};
pub use backtest::{BacktestEngine, BacktestConfig, BacktestResult, OrderBook, Tick, QueuePositionModel};
pub use agents::{Agent, AgentOrchestrator, AgentRole, Task, TaskResult, Workflow, SignalGeneratorAgent};
pub use integration::{
    create_trading_system, create_oms_engine, create_signal_router, create_risk_bus,
    TradingSystem, SystemConfig, SystemChannels
};
