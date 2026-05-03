//! Integration Types - Canonical Channel and System Types
//! 
//! This module defines the exact channel types and system interfaces that must
//! be used throughout TraderX to prevent API drift and ensure consistency.

use std::sync::Arc;
use tokio::sync::mpsc;
use crate::{OmsEngine, SignalRouter, RiskBus, Order, AgentSignal};
use crate::signal_router::RouteOutcome;
use super::config::SystemConfig;

/// Canonical channel types for signal routing
#[derive(Debug)]
pub struct SignalRouterChannels {
    /// Order submission channel (SignalRouter → OMS)
    pub order_tx: mpsc::Sender<Order>,
    /// Order receiver (used by OMS)
    pub order_rx: mpsc::Receiver<Order>,
}

impl SignalRouterChannels {
    pub fn new(capacity: usize) -> Self {
        let (order_tx, order_rx) = mpsc::channel(capacity);
        Self { order_tx, order_rx }
    }
}

/// Canonical channel types for OMS events
#[derive(Debug)]
pub struct OmsChannels {
    /// OMS event channel for system communication
    pub oms_tx: mpsc::Sender<OmsEvent>,
    /// OMS event receiver
    pub oms_rx: mpsc::Receiver<OmsEvent>,
}

impl OmsChannels {
    pub fn new(capacity: usize) -> Self {
        let (oms_tx, oms_rx) = mpsc::channel(capacity);
        Self { oms_tx, oms_rx }
    }
}

/// All system channels in one place
pub struct SystemChannels {
    pub signal_router: SignalRouterChannels,
    pub oms: OmsChannels,
}

impl SystemChannels {
    pub fn new(order_capacity: usize, oms_capacity: usize) -> Self {
        Self {
            signal_router: SignalRouterChannels::new(order_capacity),
            oms: OmsChannels::new(oms_capacity),
        }
    }
}

/// Complete trading system with all components wired together
pub struct TradingSystem {
    pub config: SystemConfig,
    pub components: SystemComponents,
    pub channels: SystemChannels,
}

/// All system components properly initialized
pub struct SystemComponents {
    pub oms_engine: Arc<OmsEngine>,
    pub signal_router: Arc<SignalRouter>,
    pub risk_bus: Arc<RiskBus>,
}

impl TradingSystem {
    /// Route a signal through the complete system
    pub async fn route_signal(&self, signal: AgentSignal) -> RouteOutcome {
        self.components.signal_router.route(signal).await
    }
    
    /// Get system health status
    pub fn is_healthy(&self) -> bool {
        !self.components.risk_bus.is_halted()
    }
    
    /// Get current system state
    pub async fn get_state_summary(&self) -> SystemState {
        SystemState {
            orders_count: self.components.oms_engine.get_orders_by_account(self.config.oms.account_id).len(),
            risk_halted: self.components.risk_bus.is_halted(),
            drawdown_bps: self.components.risk_bus.dd_bps(),
        }
    }
}

/// System state summary
#[derive(Debug, Clone)]
pub struct SystemState {
    pub orders_count: usize,
    pub risk_halted: bool,
    pub drawdown_bps: i32,
}

// Re-export OmsEvent for use in integration
pub use crate::OmsEvent;
