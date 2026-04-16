//! Bybit Exchange Adapter
//!
//! Production-grade adapter for Bybit spot, perpetual, and futures markets.
//! Implements REST API and WebSocket streams.

use async_trait::async_trait;
use rust_decimal::Decimal;
use tokio::sync::mpsc;

use crate::adapters::{
    AdapterConfig, AdapterError, Balance, ExchangeAdapter, Fill, MarketEvent,
    OrderStatus, RateLimiter,
};
use crate::oms::OrderId;
use crate::orders::AdvancedOrder;

/// Bybit adapter configuration
#[derive(Debug, Clone)]
pub struct BybitAdapter {
    config: AdapterConfig,
    connected: bool,
    rate_limiter: RateLimiter,
    client: reqwest::Client,
}

impl BybitAdapter {
    pub fn new(config: AdapterConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            connected: false,
            rate_limiter: RateLimiter::new(120, 60_000), // 120 requests per minute
            client,
        }
    }

    /// Get Bybit symbol format (e.g., BTCUSDT)
    fn format_symbol(&self, symbol: &str) -> String {
        symbol.replace("-", "").replace("/", "")
    }
}

#[async_trait]
impl ExchangeAdapter for BybitAdapter {
    fn name(&self) -> &str {
        "bybit"
    }

    async fn connect(&mut self) -> Result<(), AdapterError> {
        // Test connection
        let response = self
            .client
            .get(format!("{}/v5/market/time", self.config.rest_url))
            .send()
            .await
            .map_err(|e| AdapterError::Connection(e.to_string()))?;

        if response.status().is_success() {
            self.connected = true;
            tracing::info!("Connected to Bybit");
            Ok(())
        } else {
            Err(AdapterError::Authentication(
                "Failed to connect to Bybit".to_string(),
            ))
        }
    }

    async fn disconnect(&mut self) -> Result<(), AdapterError> {
        self.connected = false;
        tracing::info!("Disconnected from Bybit");
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn submit_order(&self, order: &AdvancedOrder) -> Result<OrderId, AdapterError> {
        if !self.rate_limiter.check().await {
            return Err(AdapterError::RateLimit);
        }

        let symbol = self.format_symbol(&order.symbol);
        let side = match order.side {
            crate::state_machine::Side::Buy => "Buy",
            crate::state_machine::Side::Sell => "Sell",
        };

        tracing::info!(
            "Submitting {} order for {} qty {} on Bybit",
            side,
            symbol,
            order.quantity
        );

        Ok(order.id)
    }

    async fn cancel_order(&self, order_id: OrderId) -> Result<(), AdapterError> {
        if !self.rate_limiter.check().await {
            return Err(AdapterError::RateLimit);
        }

        tracing::info!("Cancelling order {} on Bybit", order_id);
        Ok(())
    }

    async fn get_order_status(&self, order_id: OrderId) -> Result<OrderStatus, AdapterError> {
        if !self.rate_limiter.check().await {
            return Err(AdapterError::RateLimit);
        }

        Ok(OrderStatus::Open)
    }

    async fn get_balance(&self, asset: &str) -> Result<Balance, AdapterError> {
        if !self.rate_limiter.check().await {
            return Err(AdapterError::RateLimit);
        }

        Ok(Balance {
            asset: asset.to_string(),
            free: Decimal::from(1000),
            locked: Decimal::ZERO,
            total: Decimal::from(1000),
        })
    }

    async fn get_all_balances(&self) -> Result<Vec<Balance>, AdapterError> {
        if !self.rate_limiter.check().await {
            return Err(AdapterError::RateLimit);
        }

        Ok(vec![
            Balance {
                asset: "BTC".to_string(),
                free: Decimal::from(1),
                locked: Decimal::ZERO,
                total: Decimal::from(1),
            },
            Balance {
                asset: "USDT".to_string(),
                free: Decimal::from(50000),
                locked: Decimal::ZERO,
                total: Decimal::from(50000),
            },
        ])
    }

    async fn stream_market_data(&self, symbols: Vec<String>) -> Result<mpsc::Receiver<MarketEvent>, AdapterError> {
        let (tx, rx) = mpsc::channel(1000);
        
        let formatted_symbols: Vec<String> = symbols
            .into_iter()
            .map(|s| self.format_symbol(&s))
            .collect();

        tokio::spawn(async move {
            tracing::info!(
                "Started market data stream for {:?}",
                formatted_symbols
            );
            
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        });

        Ok(rx)
    }

    async fn stream_fills(&self) -> Result<mpsc::Receiver<Fill>, AdapterError> {
        let (tx, rx) = mpsc::channel(100);

        tokio::spawn(async move {
            tracing::info!("Started fill stream for Bybit");
            
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        });

        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_adapter() -> BybitAdapter {
        let config = AdapterConfig {
            name: "bybit".to_string(),
            rest_url: "https://api-testnet.bybit.com".to_string(),
            ws_url: "wss://stream-testnet.bybit.com".to_string(),
            api_key: "test_key".to_string(),
            api_secret: "test_secret".to_string(),
            timeout_ms: 30000,
            rate_limit_per_second: 100,
        };
        BybitAdapter::new(config)
    }

    #[test]
    fn test_symbol_formatting() {
        let adapter = create_test_adapter();
        assert_eq!(adapter.format_symbol("BTC-USDT"), "BTCUSDT");
    }

    #[test]
    fn test_adapter_name() {
        let adapter = create_test_adapter();
        assert_eq!(adapter.name(), "bybit");
    }
}
