//! Integration Factory - Canonical Component Creation
//! 
//! This module provides factory functions for creating all TraderX components
//! with the correct wiring and API usage. All binaries MUST use these functions
//! to ensure consistency and prevent architectural drift.

use std::sync::Arc;
use uuid::Uuid;
use rust_decimal::prelude::ToPrimitive;
use crate::{
    OmsEngine, SignalRouter, RiskBus, Order, OmsError, OmsEvent,
    observability_server::{ObservabilityServer, ObservabilityServerConfig},
};
use super::{
    types::{TradingSystem, SystemComponents, SystemChannels, OmsChannels},
    config::{SystemConfig, OmsConfig, RouterConfig, RiskConfig, ObservabilityConfig},
};

/// Create a complete trading system with all components wired together
/// 
/// This is the **canonical** way to create a TraderX system. All binaries
/// should use this function instead of constructing components directly.
/// 
/// # Arguments
/// - `config` - System configuration
/// 
/// # Returns
/// - `TradingSystem` with all components properly initialized
/// 
/// # Example
/// ```rust
/// let config = SystemConfig::default();
/// let system = create_trading_system(config).await?;
/// let outcome = system.route_signal(signal).await?;
/// ```
pub async fn create_trading_system(config: SystemConfig) -> Result<(TradingSystem, TradingSystemHandles), OmsError> {
    // Create channels first
    let channels = SystemChannels::new(config.oms.channel_capacity, config.oms.channel_capacity);
    
    // Create components in dependency order
    let risk_bus = create_risk_bus(&config.risk);
    let oms_engine = create_oms_engine(&config.oms, &risk_bus, channels.oms.oms_tx.clone()).await?;
    let signal_router = create_signal_router(&config.router, &risk_bus, channels.signal_router.order_tx.clone()).await?;
    
    let components = SystemComponents {
        oms_engine,
        signal_router,
        risk_bus,
    };
    
    // Start the system first, taking ownership of the receiver
    let (oms_tx, _oms_rx) = tokio::sync::mpsc::channel(config.oms.channel_capacity);
    
    // Create a new system with fresh channels (since we need to move the receiver)
    let (_dummy_tx, dummy_rx) = tokio::sync::mpsc::channel(1);
    let system = TradingSystem {
        config: config.clone(),
        components,
        channels: SystemChannels {
            signal_router: channels.signal_router,
            oms: OmsChannels {
                oms_tx,
                oms_rx: dummy_rx, // Dummy receiver since we'll use the real one
            },
        },
    };
    
    // Start the system with the real receiver
    let handles = start_trading_system_internal(&system.components, channels.oms.oms_rx).await?;
    
    Ok((system, handles))
}

/// Create OMS Engine with canonical configuration
/// 
/// # Arguments
/// - `config` - OMS configuration
/// - `risk_bus` - Risk checking component
/// - `oms_tx` - Event channel for OMS events
/// 
/// # Returns
/// - `Arc<OmsEngine>` ready for use
pub async fn create_oms_engine(
    config: &OmsConfig,
    risk_bus: &Arc<RiskBus>,
    oms_tx: tokio::sync::mpsc::Sender<OmsEvent>,
) -> Result<Arc<OmsEngine>, OmsError> {
    // Risk checker callback
    let risk_bus_clone = Arc::clone(risk_bus);
    let risk_checker = move |order: &Order| -> Result<(), OmsError> {
        // Check symbol limit
        let notional = order.original_quantity * order.price.unwrap_or_else(|| rust_decimal::Decimal::ONE);
        let notional_f64 = notional.to_f64().unwrap_or(0.0);
        
        risk_bus_clone.check_symbol(&order.symbol, notional_f64)
            .map_err(|e| OmsError::RiskCheckFailed(e.to_string()))?;
        
        Ok(())
    };
    
    // Executor callback
    let oms_tx_clone = oms_tx.clone();
    let executor = move |order: Order| -> Result<(), OmsError> {
        tracing::info!("Executing order: {}", order.order_id);
        
        // Simulate execution (in production, this would send to exchange)
        let oms_tx_clone = oms_tx_clone.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            // Simulate fill
            let fill_event = OmsEvent::OrderFilled {
                order_id: order.order_id,
                fill_quantity: order.original_quantity,
                fill_price: order.price.unwrap_or_else(|| rust_decimal::Decimal::from(100)),
                total_filled: order.original_quantity,
                order_state: crate::OrderState::Filled,
            };
            
            if let Err(e) = oms_tx_clone.send(fill_event).await {
                tracing::error!("Failed to send fill event: {}", e);
            }
        });
        
        Ok(())
    };
    
    // Position updater callback
    let position_updater = move |account_id: Uuid, quantity: rust_decimal::Decimal, price: rust_decimal::Decimal| -> Result<(), OmsError> {
        tracing::info!("Position update: {} {} @ {}", account_id, quantity, price);
        Ok(())
    };
    
    let oms_engine = OmsEngine::new(
        config.ring_buffer_size,
        risk_checker,
        executor,
        position_updater,
    )?;
    
    Ok(Arc::new(oms_engine))
}

/// Create Signal Router with canonical configuration
/// 
/// # Arguments
/// - `config` - Router configuration
/// - `risk_bus` - Risk checking component
/// - `order_tx` - Order submission channel
/// 
/// # Returns
/// - `Arc<SignalRouter>` ready for use
pub async fn create_signal_router(
    config: &RouterConfig,
    risk_bus: &Arc<RiskBus>,
    order_tx: tokio::sync::mpsc::Sender<Order>,
) -> Result<Arc<SignalRouter>, OmsError> {
    let router_config = crate::RouterConfig {
        socket_path: config.socket_path.clone(),
        account_id: config.account_id,
        kelly_fraction: config.kelly_fraction,
        portfolio_nav_usd: config.portfolio_nav_usd,
    };
    
    let signal_router = SignalRouter::new(router_config, Arc::clone(risk_bus), order_tx);
    
    Ok(Arc::new(signal_router))
}

/// Create Risk Bus with canonical configuration
/// 
/// # Arguments
/// - `config` - Risk configuration
/// 
/// # Returns
/// - `Arc<RiskBus>` ready for use
pub fn create_risk_bus(config: &RiskConfig) -> Arc<RiskBus> {
    let risk_bus = RiskBus::new(config.initial_capital_usd, config.max_drawdown_bps);
    
    // Configure symbol limits
    for (symbol, limit) in &config.symbol_limits {
        risk_bus.set_symbol_limit(symbol, *limit);
    }
    
    risk_bus
}

/// Create Observability Server with canonical configuration
/// 
/// # Arguments
/// - `config` - Observability configuration
/// - `risk_bus` - Risk bus for health checks
/// 
/// # Returns
/// - `ObservabilityServer` ready to start
pub fn create_observability_server(
    config: &ObservabilityConfig,
    risk_bus: &Arc<RiskBus>,
) -> Result<ObservabilityServer, OmsError> {
    let obs_config = ObservabilityServerConfig {
        bind_addr: config.metrics_bind_addr.parse()
            .map_err(|e| OmsError::ProtocolError(format!("Invalid bind address: {}", e)))?,
        metrics_rate_limit_per_sec: config.metrics_rate_limit,
        health_rate_limit_per_sec: config.health_rate_limit,
        enable_cors: config.enable_cors,
        cors_allowed_origins: Vec::new(),
    };
    
    Ok(ObservabilityServer::new(obs_config, Arc::clone(risk_bus)))
}

/// Internal function to start system with owned receiver
async fn start_trading_system_internal(
    components: &SystemComponents,
    oms_rx: tokio::sync::mpsc::Receiver<OmsEvent>,
) -> Result<TradingSystemHandles, OmsError> {
    // Start signal router
    let signal_router_handle = {
        let router = Arc::clone(&components.signal_router);
        tokio::spawn(async move {
            if let Err(e) = router.start().await {
                tracing::error!("Signal router error: {}", e);
            }
        })
    };
    
    // Start OMS event processing
    let oms_handle = {
        let oms = Arc::clone(&components.oms_engine);
        tokio::spawn(async move {
            let mut oms_rx = oms_rx;
            while let Some(event) = oms_rx.recv().await {
                match event {
                    OmsEvent::OrderFilled { order_id, fill_quantity, fill_price, .. } => {
                        if let Err(e) = oms.process_fill(order_id, fill_quantity, fill_price).await {
                            tracing::error!("Fill processing error: {}", e);
                        }
                    }
                    OmsEvent::OrderCancelled { order_id, .. } => {
                        tracing::info!("Order cancelled: {}", order_id);
                    }
                    _ => {
                        tracing::debug!("Received OMS event: {:?}", std::mem::discriminant(&event));
                    }
                }
            }
        })
    };
    
    // Start observability server
    let observability_handle = {
        let obs_config = crate::integration::config::ObservabilityConfig::default();
        let obs_server = create_observability_server(&obs_config, &components.risk_bus)
            .map_err(|e| {
                tracing::error!("Failed to create observability server: {}", e);
                e
            })?;
        tokio::spawn(async move {
            if let Err(e) = obs_server.serve().await {
                tracing::error!("Observability server error: {}", e);
            }
        })
    };
    
    Ok(TradingSystemHandles {
        signal_router: signal_router_handle,
        oms: oms_handle,
        observability: observability_handle,
    })
}

/// Handles for running trading system tasks
pub struct TradingSystemHandles {
    pub signal_router: tokio::task::JoinHandle<()>,
    pub oms: tokio::task::JoinHandle<()>,
    pub observability: tokio::task::JoinHandle<()>,
}

impl TradingSystemHandles {
    /// Wait for all tasks to complete (for graceful shutdown)
    pub async fn wait_for_shutdown(self) {
        let _ = tokio::try_join!(
            self.signal_router,
            self.oms,
            self.observability,
        );
    }
    
    /// Abort all tasks (for emergency shutdown)
    pub fn abort_all(self) {
        self.signal_router.abort();
        self.oms.abort();
        self.observability.abort();
    }
}
