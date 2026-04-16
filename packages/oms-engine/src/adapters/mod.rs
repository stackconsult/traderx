//! Exchange Adapter Framework - Benchmark+ Implementation
//!
//! Modular adapter system for integrating with any exchange
//! via REST/WebSocket APIs. Inspired by NautilusTrader's adapter
//! architecture for clean, testable exchange integrations.

use async_trait::async_trait;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::oms::{Order, OrderId, Side};
use crate::orders::AdvancedOrder;

/// Exchange-agnostic error type
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("Connection failed: {0}")]
    Connection(String),
    #[error("Authentication failed: {0}")]
    Authentication(String),
    #[error("Rate limit exceeded")]
    RateLimit,
    #[error("Order rejected: {0}")]
    OrderRejected(String),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("WebSocket error: {0}")]
    WebSocket(String),
    #[error("Timeout")]
    Timeout,
}

/// Balance for an asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub asset: String,
    pub free: Decimal,
    pub locked: Decimal,
    pub total: Decimal,
}

/// Market data event from exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketEvent {
    Trade {
        symbol: String,
        price: Decimal,
        quantity: Decimal,
        side: Side,
        timestamp: u64,
    },
    OrderBookUpdate {
        symbol: String,
        bids: Vec<(Decimal, Decimal)>,
        asks: Vec<(Decimal, Decimal)>,
        timestamp: u64,
    },
    Ticker {
        symbol: String,
        bid: Decimal,
        ask: Decimal,
        last: Decimal,
        volume: Decimal,
        timestamp: u64,
    },
    Kline {
        symbol: String,
        open: Decimal,
        high: Decimal,
        low: Decimal,
        close: Decimal,
        volume: Decimal,
        interval: String,
        timestamp: u64,
    },
}

/// Order fill notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    pub order_id: OrderId,
    pub fill_id: String,
    pub symbol: String,
    pub side: Side,
    pub price: Decimal,
    pub quantity: Decimal,
    pub fee: Decimal,
    pub fee_asset: String,
    pub timestamp: u64,
}

/// Exchange adapter trait - implement for each exchange
#[async_trait]
pub trait ExchangeAdapter: Send + Sync {
    /// Get adapter name
    fn name(&self) -> &str;

    /// Connect to exchange
    async fn connect(&mut self) -> Result<(), AdapterError>;

    /// Disconnect from exchange
    async fn disconnect(&mut self) -> Result<(), AdapterError>;

    /// Check if connected
    fn is_connected(&self) -> bool;

    /// Submit an order
    async fn submit_order(&self, order: &AdvancedOrder) -> Result<OrderId, AdapterError>;

    /// Cancel an order
    async fn cancel_order(&self, order_id: OrderId) -> Result<(), AdapterError>;

    /// Get order status
    async fn get_order_status(&self, order_id: OrderId) -> Result<OrderStatus, AdapterError>;

    /// Get account balance for an asset
    async fn get_balance(&self, asset: &str) -> Result<Balance, AdapterError>;

    /// Get all balances
    async fn get_all_balances(&self) -> Result<Vec<Balance>, AdapterError>;

    /// Stream market data for symbols
    async fn stream_market_data(&self, symbols: Vec<String>) -> Result<mpsc::Receiver<MarketEvent>, AdapterError>;

    /// Stream fills
    async fn stream_fills(&self) -> Result<mpsc::Receiver<Fill>, AdapterError>;
}

/// Order status from exchange
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
    Expired,
}

/// Adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    pub name: String,
    pub rest_url: String,
    pub ws_url: String,
    pub api_key: String,
    pub api_secret: String,
    pub timeout_ms: u64,
    pub rate_limit_per_second: u32,
}

impl Default for AdapterConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            rest_url: "https://api.example.com".to_string(),
            ws_url: "wss://stream.example.com".to_string(),
            api_key: String::new(),
            api_secret: String::new(),
            timeout_ms: 30000,
            rate_limit_per_second: 100,
        }
    }
}

/// Adapter manager - manages multiple exchange connections
pub struct AdapterManager {
    adapters: HashMap<String, Box<dyn ExchangeAdapter>>,
}

impl AdapterManager {
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    /// Register an adapter
    pub fn register(&mut self, adapter: Box<dyn ExchangeAdapter>) {
        let name = adapter.name().to_string();
        self.adapters.insert(name, adapter);
    }

    /// Get adapter by name
    pub fn get(&self, name: &str) -> Option<&dyn ExchangeAdapter> {
        self.adapters.get(name).map(|a| a.as_ref())
    }

    /// Get mutable adapter by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Box<dyn ExchangeAdapter>> {
        self.adapters.get_mut(name)
    }

    /// Connect all adapters
    pub async fn connect_all(&mut self) -> Vec<(String, Result<(), AdapterError>)> {
        let mut results = Vec::new();
        for (name, adapter) in &mut self.adapters {
            let result = adapter.connect().await;
            results.push((name.clone(), result));
        }
        results
    }

    /// Disconnect all adapters
    pub async fn disconnect_all(&mut self) -> Vec<(String, Result<(), AdapterError>)> {
        let mut results = Vec::new();
        for (name, adapter) in &mut self.adapters {
            let result = adapter.disconnect().await;
            results.push((name.clone(), result));
        }
        results
    }

    /// List connected adapters
    pub fn list_connected(&self) -> Vec<&str> {
        self.adapters
            .iter()
            .filter(|(_, a)| a.is_connected())
            .map(|(n, _)| n.as_str())
            .collect()
    }
}

/// Rate limiter for API calls
pub struct RateLimiter {
    max_requests: u32,
    interval_ms: u64,
    requests: Vec<std::time::Instant>,
}

impl RateLimiter {
    pub fn new(max_requests: u32, interval_ms: u64) -> Self {
        Self {
            max_requests,
            interval_ms,
            requests: Vec::new(),
        }
    }

    /// Check if a request can be made
    pub async fn check(&mut self) -> bool {
        let now = std::time::Instant::now();
        let window = std::time::Duration::from_millis(self.interval_ms);

        // Remove old requests outside window
        self.requests.retain(|&t| now - t < window);

        if self.requests.len() < self.max_requests as usize {
            self.requests.push(now);
            true
        } else {
            // Wait until oldest request expires
            if let Some(&oldest) = self.requests.first() {
                let wait_time = window - (now - oldest);
                tokio::time::sleep(wait_time).await;
                self.requests.push(std::time::Instant::now());
                true
            } else {
                false
            }
        }
    }
}

// Exchange-specific implementations
pub mod binance;
pub mod bybit;

// Re-exports
pub use binance::BinanceAdapter;
pub use bybit::BybitAdapter;
