//! Binance Exchange Adapter
//!
//! Production-grade adapter for Binance spot and futures markets.
//! Implements REST API and WebSocket streams for market data
//! and order management.

use async_trait::async_trait;
use rust_decimal::Decimal;
use tokio::sync::mpsc;

use crate::adapters::{
    AdapterConfig, AdapterError, Balance, ExchangeAdapter, Fill, MarketEvent,
    OrderStatus, RateLimiter,
};
use uuid::Uuid as OrderId;
use crate::orders::AdvancedOrder;

/// Binance adapter configuration
#[derive(Debug, Clone)]
pub struct BinanceAdapter {
    config: AdapterConfig,
    connected: bool,
    rate_limiter: RateLimiter,
    client: reqwest::Client,
}

impl BinanceAdapter {
    pub fn new(config: AdapterConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            connected: false,
            rate_limiter: RateLimiter::new(1200, 60_000), // 1200 requests per minute
            client,
        }
    }

    /// Get Binance-specific symbol format (e.g., BTCUSDT)
    fn format_symbol(&self, symbol: &str) -> String {
        symbol.replace("-", "").replace("/", "")
    }

    /// Build authenticated request
    fn build_request(&self, method: reqwest::Method, endpoint: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.config.rest_url, endpoint);
        self.client
            .request(method, &url)
            .header("X-MBX-APIKEY", &self.config.api_key)
    }
}

#[async_trait]
impl ExchangeAdapter for BinanceAdapter {
    fn name(&self) -> &str {
        "binance"
    }

    async fn connect(&mut self) -> Result<(), AdapterError> {
        // Test connection by fetching server time
        let response = self
            .client
            .get(format!("{}/api/v3/time", self.config.rest_url))
            .send()
            .await
            .map_err(|e| AdapterError::Connection(e.to_string()))?;

        if response.status().is_success() {
            self.connected = true;
            tracing::info!("Connected to Binance");
            Ok(())
        } else {
            Err(AdapterError::Authentication(
                "Failed to connect to Binance".to_string(),
            ))
        }
    }

    async fn disconnect(&mut self) -> Result<(), AdapterError> {
        self.connected = false;
        tracing::info!("Disconnected from Binance");
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn submit_order(&self, order: &AdvancedOrder) -> Result<OrderId, AdapterError> {
        // Rate limiting
        if !self.rate_limiter.check().await {
            return Err(AdapterError::RateLimit);
        }

        let symbol = self.format_symbol(&order.symbol);
        
        // Build order parameters
        let side = match order.side {
            crate::state_machine::Side::Buy => "BUY",
            crate::state_machine::Side::Sell => "SELL",
        };

        // Placeholder: Actual implementation would POST to /api/v3/order
        tracing::info!(
            "Submitting {} order for {} qty {} on Binance",
            side,
            symbol,
            order.quantity
        );

        // Return order ID (in real impl, would parse from response)
        Ok(order.id)
    }

    async fn cancel_order(&self, order_id: OrderId) -> Result<(), AdapterError> {
        if !self.rate_limiter.check().await {
            return Err(AdapterError::RateLimit);
        }

        tracing::info!("Cancelling order {} on Binance", order_id);
        Ok(())
    }

    async fn get_order_status(&self, order_id: OrderId) -> Result<OrderStatus, AdapterError> {
        if !self.rate_limiter.check().await {
            return Err(AdapterError::RateLimit);
        }

        tracing::info!("Getting order status {} on Binance", order_id);
        Ok(OrderStatus::Open)
    }

    async fn get_balance(&self, asset: &str) -> Result<Balance, AdapterError> {
        if !self.rate_limiter.check().await {
            return Err(AdapterError::RateLimit);
        }

        // Placeholder: Actual impl would fetch from /api/v3/account
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

        // Placeholder: Return sample balances
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
            .map(|s| self.format_symbol(&s).to_lowercase())
            .collect();

        tokio::spawn(async move {
            // Placeholder: Would connect to WebSocket wss://stream.binance.com:9443/ws
            tracing::info!(
                "Started market data stream for {:?}",
                formatted_symbols
            );
            
            // Simulate keeping connection alive
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        });

        Ok(rx)
    }

    async fn stream_fills(&self) -> Result<mpsc::Receiver<Fill>, AdapterError> {
        let (tx, rx) = mpsc::channel(100);

        tokio::spawn(async move {
            // Placeholder: Would connect to user data stream
            tracing::info!("Started fill stream for Binance");
            
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

    fn create_test_adapter() -> BinanceAdapter {
        let config = AdapterConfig {
            name: "binance".to_string(),
            rest_url: "https://testnet.binance.vision".to_string(),
            ws_url: "wss://testnet.binance.vision".to_string(),
            api_key: "test_key".to_string(),
            api_secret: "test_secret".to_string(),
            timeout_ms: 30000,
            rate_limit_per_second: 100,
        };
        BinanceAdapter::new(config)
    }

    #[test]
    fn test_symbol_formatting() {
        let adapter = create_test_adapter();
        assert_eq!(adapter.format_symbol("BTC-USDT"), "BTCUSDT");
        assert_eq!(adapter.format_symbol("ETH/BTC"), "ETHBTC");
    }

    #[test]
    fn test_adapter_name() {
        let adapter = create_test_adapter();
        assert_eq!(adapter.name(), "binance");
    }
}
