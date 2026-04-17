//! TraderX OMS Engine - Main Integration Bootstrap
//! Wires all components for end-to-end trading functionality

use oms_engine::{
    OmsEngine, OmsEvent, Order, OrderState, OrderEvent, Side, OrderType,
    SignalRouter, RouterConfig, AgentSignal,
    RiskBus,
    observability_server::{ObservabilityServer, ObservabilityServerConfig},
};
use portfolio_aggregation::{PortfolioAggregator, AggregatorEvent, FillEvent, PriceUpdate};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{sleep, Instant};
use tracing::{info, warn, error};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting TraderX OMS Engine Integration Bootstrap");

    // 1. Create Risk Bus with initial capital
    let risk_bus = Arc::new(RiskBus::new(10_000_000.0, -2000));
    info!("Risk Bus initialized with $10M capital");

    // 2. Create Portfolio Aggregator
    let (portfolio_tx, portfolio_rx) = mpsc::channel(1000);
    let portfolio = PortfolioAggregator::new(portfolio_rx, "/tmp/portfolio.wal");
    let portfolio_clone = Arc::clone(&portfolio);

    // Start portfolio aggregation in background
    tokio::spawn(async move {
        portfolio_clone.run().await;
    });

    // 3. Create OMS Engine with callbacks
    let (oms_tx, mut oms_rx) = mpsc::channel(1000);

    // Risk checker callback
    let risk_checker = {
        let risk_bus = Arc::clone(&risk_bus);
        move |order: &Order| -> oms_engine::Result<()> {
            // Check symbol limit
            let notional = order.quantity * order.price.unwrap_or(Decimal::ONE);
            risk_bus.check_symbol(&order.symbol, notional.to_f64().unwrap_or(0.0))
                .map_err(|e| oms_engine::OmsError::RiskCheckFailed(e))?;
            Ok(())
        }
    };

    // Executor callback (simulated)
    let executor = {
        let oms_tx = oms_tx.clone();
        let portfolio_tx = portfolio_tx.clone();
        move |order: Order| -> oms_engine::Result<()> {
            info!("Executing order: {}", order.order_id);

            // Simulate execution after delay
            let oms_tx = oms_tx.clone();
            let portfolio_tx = portfolio_tx.clone();
            tokio::spawn(async move {
                sleep(Duration::from_millis(100)).await;

                // Simulate fill
                let fill_event = OmsEvent::OrderFilled {
                    order_id: order.order_id,
                    fill_quantity: order.quantity,
                    fill_price: order.price.unwrap_or(Decimal::from_str("100.0").unwrap()),
                    fill_time: chrono::Utc::now(),
                };

                // Send fill to OMS
                if let Err(e) = oms_tx.send(fill_event).await {
                    error!("Failed to send fill event: {}", e);
                }

                // Send fill to portfolio
                let portfolio_fill = FillEvent {
                    strategy_id: "test_strategy".to_string(),
                    symbol: order.symbol.clone(),
                    asset_class: portfolio_aggregation::AssetClass::Equity,
                    side: if order.side == Side::Buy { "buy" } else { "sell" }.to_string(),
                    quantity: order.quantity.to_f64().unwrap_or(0.0),
                    fill_price_usd: order.price.unwrap_or(Decimal::from_str("100.0").unwrap()).to_f64().unwrap_or(100.0),
                    commission_usd: 0.01,
                    timestamp_ns: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0),
                };

                if let Err(e) = portfolio_tx.send(AggregatorEvent::Fill(portfolio_fill)).await {
                    error!("Failed to send portfolio fill: {}", e);
                }
            });

            Ok(())
        }
    };

    // Position updater callback
    let position_updater = {
        let portfolio_tx = portfolio_tx.clone();
        move |order_id: Uuid, quantity: Decimal, price: Decimal| -> oms_engine::Result<()> {
            info!("Position update: {} {} @ {}", order_id, quantity, price);
            // Portfolio handles position updates via fills
            Ok(())
        }
    };

    let oms_engine = Arc::new(OmsEngine::new(
        1024,
        risk_checker,
        executor,
        position_updater,
    )?);

    // 4. Create Signal Router
    let router_config = RouterConfig {
        socket_path: "/tmp/traderx_signals.sock".to_string(),
        account_id: Uuid::new_v4(),
        kelly_fraction: 0.25,
        portfolio_nav_usd: 10_000_000.0,
    };

    let signal_router = SignalRouter::new(
        router_config,
        Arc::clone(&risk_bus),
        oms_tx,
    );

    // 5. Start Observability Server
    let obs_config = ObservabilityServerConfig {
        bind_addr: "127.0.0.1:9090".parse().unwrap(),
        metrics_rate_limit_per_sec: 10,
        health_rate_limit_per_sec: 100,
        enable_cors: false,
        cors_allowed_origins: Vec::new(),
    };

    let obs_server = ObservabilityServer::new(obs_config, Arc::clone(&risk_bus));
    tokio::spawn(async move {
        if let Err(e) = obs_server.serve().await {
            error!("Observability server error: {}", e);
        }
    });

    // 6. Start Signal Router
    let signal_router_handle = {
        let router = signal_router.clone();
        tokio::spawn(async move {
            if let Err(e) = router.start().await {
                error!("Signal router error: {}", e);
            }
        })
    };

    // 7. Process OMS events
    let oms_handle = {
        let oms = Arc::clone(&oms_engine);
        tokio::spawn(async move {
            while let Some(event) = oms_rx.recv().await {
                match event {
                    OmsEvent::OrderFilled { order_id, fill_quantity, fill_price, .. } => {
                        if let Err(e) = oms.process_fill(order_id, fill_quantity, fill_price).await {
                            error!("Fill processing error: {}", e);
                        }
                    }
                    OmsEvent::OrderCancelled { order_id, .. } => {
                        info!("Order cancelled: {}", order_id);
                        // Handle cancellation if needed
                    }
                    _ => {
                        debug!("Received OMS event: {:?}", std::mem::discriminant(&event));
                    }
                }
            }
        })
    };

    // 8. DEMONSTRATE TRADING FLOW
    info!("=== DEMONSTRATING TRADING FLOW ===");

    // Wait for services to start
    sleep(Duration::from_secs(1)).await;

    // Create test signal
    let test_signal = AgentSignal {
        agent_id: "test_agent".to_string(),
        symbol: "AAPL".to_string(),
        direction: "long".to_string(),
        conviction: 0.7,
        max_notional_usd: 10_000.0,
        ttl_ms: 5000,
        meta: serde_json::json!({"test": true}),
    };

    info!("Sending test signal: {} {}", test_signal.symbol, test_signal.direction);

    // Route signal through system
    let outcome = signal_router.route_signal(test_signal);
    info!("Signal routed: {:?}", outcome);

    // Wait for processing
    sleep(Duration::from_millis(500)).await;

    // Send price update to trigger P&L calculation
    let price_update = AggregatorEvent::Price(PriceUpdate {
        symbol: "AAPL".to_string(),
        price_usd: 101.0,
        timestamp_ns: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0),
    });

    if let Err(e) = portfolio_tx.send(price_update).await {
        error!("Failed to send price update: {}", e);
    }

    // Wait for final processing
    sleep(Duration::from_millis(500)).await;

    // 9. DISPLAY RESULTS
    info!("=== TRADING FLOW RESULTS ===");

    // Check order status
    if let Some(order) = outcome.order_id.and_then(|id| oms_engine.get_order(id)) {
        info!("Order Status: {:?}", order.state);
    }

    // Get portfolio P&L
    let (pnl_tx, pnl_rx) = mpsc::channel(1);
    let pnl_request = AggregatorEvent::GetStrategyPnl("test_strategy".to_string(), pnl_tx);

    if let Err(e) = portfolio_tx.send(pnl_request).await {
        error!("Failed to request P&L: {}", e);
    } else if let Some(pnl) = pnl_rx.recv().await {
        info!("Strategy P&L: ${:.2}", pnl.total_usd);
        info!("Unrealized: ${:.2}", pnl.unrealized_usd);
        info!("Realized: ${:.2}", pnl.realized_usd);
    }

    // Check risk status
    info!("Risk Bus Halted: {}", risk_bus.is_halted());
    info!("Current Drawdown: {} bps", risk_bus.dd_bps());

    // 10. Keep running
    info!("TraderX OMS Engine is running. Press Ctrl+C to stop.");

    // Wait for shutdown
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Shutting down...");
        }
        _ = signal_router_handle => {
            warn!("Signal router stopped unexpectedly");
        }
        _ = oms_handle => {
            warn!("OMS processor stopped unexpectedly");
        }
    }

    Ok(())
}
