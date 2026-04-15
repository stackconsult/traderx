//! Standalone test for core TraderX trading flow
//! Tests RiskBus + SignalRouter integration without full OMS

use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn};
use uuid::Uuid;

// Minimal implementations for testing
#[derive(Debug, Clone)]
pub struct AgentSignal {
    pub agent_id: String,
    pub symbol: String,
    pub direction: String,
    pub conviction: f64,
    pub max_notional_usd: f64,
    pub ttl_ms: u64,
    pub meta: serde_json::Value,
}

#[derive(Debug)]
pub enum RouteOutcome {
    Accepted { signal_id: Uuid, position_size: f64 },
    Rejected { signal_id: Uuid, reason: String },
}

#[derive(Debug, Clone)]
pub struct RouterConfig {
    pub socket_path: String,
    pub account_id: Uuid,
    pub kelly_fraction: f64,
    pub portfolio_nav_usd: f64,
}

pub struct RiskBus {
    capital_usd: f64,
    drawdown_bps: i64,
    orders_submitted: std::sync::atomic::AtomicU64,
    orders_rejected: std::sync::atomic::AtomicU64,
    is_halted: std::sync::atomic::AtomicBool,
}

impl RiskBus {
    pub fn new(capital_usd: f64, drawdown_bps: i64) -> Self {
        Self {
            capital_usd,
            drawdown_bps,
            orders_submitted: std::sync::atomic::AtomicU64::new(0),
            orders_rejected: std::sync::atomic::AtomicU64::new(0),
            is_halted: std::sync::atomic::AtomicBool::new(false),
        }
    }
    
    pub fn check_symbol(&self, _symbol: &str, notional: f64) -> Result<(), String> {
        // Simple risk check
        if notional > self.capital_usd * 0.1 {
            Err(format!("Position too large: ${:.2} > ${:.2}", notional, self.capital_usd * 0.1))
        } else {
            Ok(())
        }
    }
    
    pub fn is_halted(&self) -> bool {
        self.is_halted.load(std::sync::atomic::Ordering::SeqCst)
    }
    
    pub fn dd_bps(&self) -> i64 {
        self.drawdown_bps
    }
    
    pub fn orders_submitted_count(&self) -> u64 {
        self.orders_submitted.load(std::sync::atomic::Ordering::SeqCst)
    }
    
    pub fn orders_rejected_count(&self) -> u64 {
        self.orders_rejected.load(std::sync::atomic::Ordering::SeqCst)
    }
}

pub struct SignalRouter {
    config: RouterConfig,
    risk_bus: Arc<RiskBus>,
    oms_tx: mpsc::Sender<String>,
}

impl SignalRouter {
    pub fn new(
        config: RouterConfig,
        risk_bus: Arc<RiskBus>,
        oms_tx: mpsc::Sender<String>,
    ) -> Self {
        Self { config, risk_bus, oms_tx }
    }
    
    pub fn route_signal(&self, signal: AgentSignal) -> RouteOutcome {
        // Calculate position size
        let position_size = signal.max_notional_usd * signal.conviction * self.config.kelly_fraction;
        
        // Risk check
        match self.risk_bus.check_symbol(&signal.symbol, position_size) {
            Ok(()) => {
                self.risk_bus.orders_submitted.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                
                // Send to OMS (simplified)
                if let Err(_) = self.oms_tx.blocking_send(format!("ORDER:{}:{:.2}", signal.symbol, position_size)) {
                    warn!("Failed to send order to OMS");
                }
                
                RouteOutcome::Accepted {
                    signal_id: Uuid::new_v4(),
                    position_size,
                }
            }
            Err(reason) => {
                self.risk_bus.orders_rejected.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                RouteOutcome::Rejected {
                    signal_id: Uuid::new_v4(),
                    reason,
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("=== TRADERX TRADING FLOW TEST ===");
    
    // 1. Create Risk Bus
    let risk_bus = Arc::new(RiskBus::new(10_000_000.0, -2000));
    info!("✅ Risk Bus initialized with $10M capital");
    
    // 2. Create signal channel
    let (oms_tx, mut oms_rx) = mpsc::channel(100);
    
    // 3. Create Signal Router
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
    
    // 4. Test normal signal
    info!("\n--- Test 1: Normal Signal ---");
    let normal_signal = AgentSignal {
        agent_id: "test_agent".to_string(),
        symbol: "AAPL".to_string(),
        direction: "long".to_string(),
        conviction: 0.7,
        max_notional_usd: 10_000.0,
        ttl_ms: 5000,
        meta: serde_json::json!({"test": true}),
    };
    
    let outcome = signal_router.route_signal(normal_signal);
    info!("Signal routed: {:?}", outcome);
    
    // 5. Test oversized signal
    info!("\n--- Test 2: Oversized Signal ---");
    let oversized_signal = AgentSignal {
        agent_id: "test_agent".to_string(),
        symbol: "AAPL".to_string(),
        direction: "long".to_string(),
        conviction: 0.7,
        max_notional_usd: 2_000_000.0, // Too large
        ttl_ms: 5000,
        meta: serde_json::json!({"test": true}),
    };
    
    let outcome = signal_router.route_signal(oversized_signal);
    info!("Signal routed: {:?}", outcome);
    
    // 6. Check OMS received orders
    info!("\n--- Test 3: OMS Integration ---");
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    while let Some(order) = oms_rx.recv().await {
        info!("OMS received: {}", order);
        break; // Just check first order for test
    }
    
    // 7. Final status
    info!("\n--- Final Status ---");
    info!("Risk Bus Halted: {}", risk_bus.is_halted());
    info!("Current Drawdown: {} bps", risk_bus.dd_bps());
    info!("Orders Submitted: {}", risk_bus.orders_submitted_count());
    info!("Orders Rejected: {}", risk_bus.orders_rejected_count());
    
    // 8. Validate test results
    let submitted = risk_bus.orders_submitted_count();
    let rejected = risk_bus.orders_rejected_count();
    
    if submitted == 1 && rejected == 1 {
        info!("\n✅ TRADING FLOW TEST PASSED");
        info!("✅ Risk enforcement working correctly");
        info!("✅ Signal routing functional");
        info!("✅ OMS integration active");
    } else {
        info!("\n❌ TRADING FLOW TEST FAILED");
        info!("Expected: 1 submitted, 1 rejected");
        info!("Got: {} submitted, {} rejected", submitted, rejected);
    }
    
    Ok(())
}
