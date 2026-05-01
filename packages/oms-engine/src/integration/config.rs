//! Integration Configuration - Canonical System Configuration
//! 
//! This module defines the standard configuration structures for all TraderX
//! components to ensure consistent initialization and prevent configuration drift.

use std::path::PathBuf;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

/// Complete system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub oms: OmsConfig,
    pub router: RouterConfig,
    pub risk: RiskConfig,
    pub observability: ObservabilityConfig,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            oms: OmsConfig::default(),
            router: RouterConfig::default(),
            risk: RiskConfig::default(),
            observability: ObservabilityConfig::default(),
        }
    }
}

/// OMS Engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmsConfig {
    /// Ring buffer size for order processing
    pub ring_buffer_size: usize,
    /// Account ID for this OMS instance
    pub account_id: Uuid,
    /// Channel capacities
    pub channel_capacity: usize,
}

impl Default for OmsConfig {
    fn default() -> Self {
        Self {
            ring_buffer_size: 1024,
            account_id: Uuid::new_v4(),
            channel_capacity: 1000,
        }
    }
}

/// Signal Router configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    /// Unix socket path for signal reception
    pub socket_path: String,
    /// Account ID for order routing
    pub account_id: Uuid,
    /// Kelly fraction for position sizing
    pub kelly_fraction: f64,
    /// Portfolio NAV for calculations
    pub portfolio_nav_usd: f64,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            socket_path: "/tmp/traderx_signals.sock".to_string(),
            account_id: Uuid::new_v4(),
            kelly_fraction: 0.25,
            portfolio_nav_usd: 10_000_000.0,
        }
    }
}

/// Risk Bus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    /// Initial capital in USD
    pub initial_capital_usd: f64,
    /// Maximum drawdown in basis points
    pub max_drawdown_bps: i32,
    /// Position limits per symbol
    pub symbol_limits: std::collections::HashMap<String, f64>,
}

impl Default for RiskConfig {
    fn default() -> Self {
        let mut symbol_limits = std::collections::HashMap::new();
        symbol_limits.insert("AAPL".to_string(), 1_000_000.0);
        symbol_limits.insert("BTCUSDT".to_string(), 500_000.0);
        symbol_limits.insert("ETHUSDT".to_string(), 300_000.0);
        
        Self {
            initial_capital_usd: 10_000_000.0,
            max_drawdown_bps: -2000,
            symbol_limits,
        }
    }
}

/// Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// Metrics server bind address
    pub metrics_bind_addr: String,
    /// Health check bind address
    pub health_bind_addr: String,
    /// Metrics rate limit per second
    pub metrics_rate_limit: u32,
    /// Health check rate limit per second
    pub health_rate_limit: u32,
    /// Enable CORS for metrics
    pub enable_cors: bool,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            metrics_bind_addr: "127.0.0.1:9090".to_string(),
            health_bind_addr: "127.0.0.1:8080".to_string(),
            metrics_rate_limit: 10,
            health_rate_limit: 100,
            enable_cors: false,
        }
    }
}

/// Environment-based configuration builder
impl SystemConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            oms: OmsConfig {
                ring_buffer_size: std::env::var("OMS_RING_BUFFER_SIZE")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1024),
                account_id: std::env::var("OMS_ACCOUNT_ID")
                    .ok()
                    .and_then(|s| Uuid::parse_str(&s).ok())
                    .unwrap_or_else(Uuid::new_v4),
                channel_capacity: std::env::var("OMS_CHANNEL_CAPACITY")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1000),
            },
            router: RouterConfig {
                socket_path: std::env::var("ROUTER_SOCKET_PATH")
                    .unwrap_or_else(|_| "/tmp/traderx_signals.sock".to_string()),
                account_id: std::env::var("ROUTER_ACCOUNT_ID")
                    .ok()
                    .and_then(|s| Uuid::parse_str(&s).ok())
                    .unwrap_or_else(Uuid::new_v4),
                kelly_fraction: std::env::var("ROUTER_KELLY_FRACTION")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.25),
                portfolio_nav_usd: std::env::var("ROUTER_PORTFOLIO_NAV")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(10_000_000.0),
            },
            risk: RiskConfig {
                initial_capital_usd: std::env::var("RISK_INITIAL_CAPITAL")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(10_000_000.0),
                max_drawdown_bps: std::env::var("RISK_MAX_DRAWDOWN_BPS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(-2000),
                symbol_limits: std::collections::HashMap::new(), // TODO: Parse from env
            },
            observability: ObservabilityConfig {
                metrics_bind_addr: std::env::var("METRICS_BIND_ADDR")
                    .unwrap_or_else(|_| "127.0.0.1:9090".to_string()),
                health_bind_addr: std::env::var("HEALTH_BIND_ADDR")
                    .unwrap_or_else(|_| "127.0.0.1:8080".to_string()),
                metrics_rate_limit: std::env::var("METRICS_RATE_LIMIT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(10),
                health_rate_limit: std::env::var("HEALTH_RATE_LIMIT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(100),
                enable_cors: std::env::var("ENABLE_CORS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(false),
            },
        })
    }
}
