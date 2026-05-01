//! TraderX OMS Engine - Main Integration Bootstrap
//! Uses canonical integration module for architectural compliance

use oms_engine::{
    integration::{create_trading_system, SystemConfig},
    AgentSignal,
};
use tracing::{info, warn, error};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting TraderX OMS Engine Integration Bootstrap");

    // 1. Create system configuration
    let config = SystemConfig::default();
    info!("System configuration loaded");

    // 2. Create and start trading system using canonical factory
    let (system, system_handles) = create_trading_system(config).await?;
    info!("Trading system created and started");

    // 4. DEMONSTRATE TRADING FLOW
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
    let outcome = system.route_signal(test_signal).await;
    info!("Signal routed: {:?}\n", outcome);

    // Wait for processing
    sleep(Duration::from_millis(500)).await;

    // 5. DISPLAY RESULTS
    info!("=== TRADING FLOW RESULTS ===");

    // Check system state
    let state = system.get_state_summary().await;
    info!("Orders in system: {}", state.orders_count);
    info!("Risk Bus Halted: {}", state.risk_halted);
    info!("Current Drawdown: {} bps", state.drawdown_bps);

    // 6. Keep running
    info!("TraderX OMS Engine is running. Press Ctrl+C to stop.");

    // Wait for shutdown
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Ctrl+C received, shutting down.");
        }
        _ = system_handles.wait_for_shutdown() => {
            warn!("System tasks finished unexpectedly.");
        }
    }

    info!("TraderX OMS Engine stopped.");

    Ok(())
}
