//! Signal Router — the critical missing bridge between AI agents and the OMS.
//!
//! Receives `AgentSignal` from Python agents via Unix socket (JSON lines),
//! applies Kelly sizing, passes through the risk bus, then emits `Order`
//! into the OMS LMAX disruptor ring buffer.
//!
//! Latency budget: <5μs from signal receipt to OMS submission.

use crate::risk_bus::RiskBus;
use crate::state_machine::{Order, OrderType, Side};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use tokio::io::{AsyncBufReadExt, BufReader};
#[cfg(unix)]
use tokio::net::UnixListener;
#[cfg(not(unix))]
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Signal emitted by any Python AI agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSignal {
    /// Unique agent identifier (e.g. "risk_consensus_agent").
    pub agent_id: String,

    /// Target symbol (e.g. "AAPL", "BTC-USD").
    pub symbol: String,

    /// "long", "short", or "flat".
    pub direction: String,

    /// Agent conviction 0.0–1.0 (used for Kelly sizing).
    pub conviction: f64,

    /// Maximum notional in USD this signal may deploy.
    pub max_notional_usd: f64,

    /// Signal TTL — if not executed within this many ms, discard.
    pub ttl_ms: u64,

    /// Arbitrary metadata (strategy params, feature values, etc.).
    #[serde(default)]
    pub meta: serde_json::Value,
}

/// Outcome of routing a signal.
#[derive(Debug, Serialize)]
pub struct RouteOutcome {
    pub signal_id: Uuid,
    pub order_id: Option<Uuid>,
    pub status: RouteStatus,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub enum RouteStatus {
    Submitted,
    RiskHalt,
    SymbolLimit,
    Expired,
    Error,
}

/// Configuration for the signal router.
pub struct RouterConfig {
    /// Path to Unix domain socket (agents connect here).
    pub socket_path: String,
    /// Account UUID to stamp on every generated order.
    pub account_id: Uuid,
    /// Fraction of Kelly to use (e.g. 0.25 = quarter-Kelly).
    pub kelly_fraction: f64,
    /// Current portfolio NAV (updated live by portfolio engine).
    pub portfolio_nav_usd: f64,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            socket_path: "/tmp/traderx_signals.sock".into(),
            account_id: Uuid::nil(),
            kelly_fraction: 0.25,
            portfolio_nav_usd: 1_000_000.0,
        }
    }
}

/// Signal router — owns the Unix socket listener and drives OMS submissions.
pub struct SignalRouter {
    cfg: RouterConfig,
    risk_bus: Arc<RiskBus>,
    oms_tx: mpsc::Sender<Order>,
}

impl SignalRouter {
    pub fn new(cfg: RouterConfig, risk_bus: Arc<RiskBus>, oms_tx: mpsc::Sender<Order>) -> Self {
        Self { cfg, risk_bus, oms_tx }
    }

    /// Validate incoming signal parameters
    fn validate_signal(signal: &AgentSignal) -> Result<(), String> {
        // Validate symbol against allowed list
        const ALLOWED_SYMBOLS: &[&str] = &[
            "AAPL", "GOOGL", "MSFT", "TSLA", "AMZN", "META", "NVDA", 
            "BTC-USD", "ETH-USD", "SPY", "QQQ", "GLD"
        ];
        
        if !ALLOWED_SYMBOLS.contains(&signal.symbol.as_str()) {
            return Err(format!("Invalid symbol: {}", signal.symbol));
        }
        
        // Validate conviction range
        if !(0.0..=1.0).contains(&signal.conviction) {
            return Err("Conviction must be between 0.0 and 1.0".to_string());
        }
        
        // Validate notional amount
        if signal.max_notional_usd <= 0.0 || signal.max_notional_usd > 1_000_000.0 {
            return Err("Max notional must be > 0 and <= $1,000,000".to_string());
        }
        
        // Validate direction
        if !["long", "short", "flat"].contains(&signal.direction.as_str()) {
            return Err("Direction must be 'long', 'short', or 'flat'".to_string());
        }
        
        // Validate TTL
        if signal.ttl_ms == 0 || signal.ttl_ms > 60_000 {
            return Err("TTL must be > 0ms and <= 60,000ms".to_string());
        }
        
        Ok(())
    }

    /// Spawn the Unix socket listener. Returns immediately.
    #[cfg(unix)]
    pub async fn start(self: Arc<Self>) -> anyhow::Result<()> {
        let _ = std::fs::remove_file(&self.cfg.socket_path);
        let listener = UnixListener::bind(&self.cfg.socket_path)?;
        // Set secure permissions (owner read/write only)
        std::fs::set_permissions(&self.cfg.socket_path, std::fs::Permissions::from_mode(0o600))?;
        info!("SignalRouter listening on {}", self.cfg.socket_path);

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let router = Arc::clone(&self);
                        tokio::spawn(async move {
                            router.handle_connection(stream).await;
                        });
                    }
                    Err(e) => {
                        error!("SignalRouter accept error: {}", e);
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                }
            }
        });

        Ok(())
    }

    /// Stub for non-Unix platforms
    #[cfg(not(unix))]
    pub async fn start(self: Arc<Self>) -> anyhow::Result<()> {
        warn!("Unix socket listener not available on non-Unix platforms");
        Ok(())
    }

    #[cfg(unix)]
    async fn handle_connection(&self, stream: tokio::net::UnixStream) {
        let reader = BufReader::new(stream);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            match serde_json::from_str::<AgentSignal>(&line) {
                Ok(signal) => {
                    // Validate signal before routing
                    match Self::validate_signal(&signal) {
                        Ok(()) => {
                            let outcome = self.route(signal).await;
                            debug!("Route outcome: {:?}", outcome.status);
                        }
                        Err(validation_error) => {
                            warn!("Signal validation failed: {} — agent: {}", 
                                validation_error, signal.agent_id);
                            // Could send error response back to agent if needed
                        }
                    }
                }
                Err(e) => {
                    warn!("Invalid signal JSON: {} — {}", e, &line[..line.len().min(120)]);
                }
            }
        }
    }

    /// Core routing logic. <5μs hot path after socket I/O.
    pub async fn route(&self, signal: AgentSignal) -> RouteOutcome {
        let signal_id = Uuid::new_v4();

        // 1. Global risk check (lock-free atomic reads)
        if let Err(reason) = self.risk_bus.check() {
            self.risk_bus.orders_rejected.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return RouteOutcome { signal_id, order_id: None, status: RouteStatus::RiskHalt, reason: Some(reason.into()) };
        }

        // 2. Compute position size via fractional Kelly
        let qty_usd = self.kelly_size(signal.conviction, signal.max_notional_usd);
        if qty_usd < 1.0 {
            return RouteOutcome { signal_id, order_id: None, status: RouteStatus::Error, reason: Some("size too small".into()) };
        }

        // 3. Symbol-level position limit check
        if let Err(reason) = self.risk_bus.check_symbol(&signal.symbol, qty_usd) {
            self.risk_bus.record_order_rejected();
            return RouteOutcome { signal_id, order_id: None, status: RouteStatus::SymbolLimit, reason: Some(reason.into()) };
        }

        // 4. Build Order from signal
        let (side, qty_decimal) = match signal.direction.as_str() {
            "long"  => (Side::Buy,  Decimal::from_str(&format!("{:.4}", qty_usd)).unwrap_or(Decimal::ONE)),
            "short" => (Side::Sell, Decimal::from_str(&format!("{:.4}", qty_usd)).unwrap_or(Decimal::ONE)),
            "flat"  => {
                // Close position — emit sell of full current holding.
                // Portfolio engine resolves actual qty; we send a flat signal.
                (Side::Sell, Decimal::ZERO)
            }
            other => {
                return RouteOutcome {
                    signal_id, order_id: None,
                    status: RouteStatus::Error,
                    reason: Some(format!("unknown direction: {}", other)),
                };
            }
        };

        let order_id = Uuid::new_v4();
        let order = Order::new(
            order_id,
            self.cfg.account_id,
            signal.symbol.clone(),
            side,
            OrderType::Market,
            qty_decimal,
        );

        // 5. Submit to OMS ring buffer
        match self.oms_tx.try_send(order) {
            Ok(_) => {
                self.risk_bus.record_order_submitted();
                info!(
                    agent = %signal.agent_id,
                    symbol = %signal.symbol,
                    direction = %signal.direction,
                    conviction = signal.conviction,
                    qty_usd,
                    "Signal routed → OMS"
                );
                RouteOutcome { signal_id, order_id: Some(order_id), status: RouteStatus::Submitted, reason: None }
            }
            Err(e) => {
                error!("OMS channel full or closed: {}", e);
                RouteOutcome { signal_id, order_id: None, status: RouteStatus::Error, reason: Some("oms channel error".into()) }
            }
        }
    }

    /// Fractional Kelly position sizing.
    /// Kelly fraction: f* = conviction × kelly_fraction
    /// Notional = min(f* × nav, max_notional)
    #[inline]
    fn kelly_size(&self, conviction: f64, max_notional: f64) -> f64 {
        let f = (conviction * self.cfg.kelly_fraction).min(1.0).max(0.0);
        let nav_based = f * self.cfg.portfolio_nav_usd;
        nav_based.min(max_notional)
    }
}
