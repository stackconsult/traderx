// Live Market Data Feed
// WebSocket-based real-time market data ingestion for live trading

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

/// Market tick data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTick {
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
    pub timestamp: i64,
    pub exchange: String,
}

/// WebSocket feed configuration
#[derive(Debug, Clone)]
pub struct WebSocketFeedConfig {
    pub exchange: String,
    pub symbols: Vec<String>,
    pub reconnect_interval_ms: u64,
}

impl Default for WebSocketFeedConfig {
    fn default() -> Self {
        Self {
            exchange: "binance".to_string(),
            symbols: vec!["BTCUSDT".to_string()],
            reconnect_interval_ms: 5000,
        }
    }
}

/// Live market data feed
pub struct LiveMarketDataFeed {
    config: WebSocketFeedConfig,
    tick_sender: mpsc::UnboundedSender<MarketTick>,
}

impl LiveMarketDataFeed {
    /// Create a new live market data feed
    pub fn new(
        config: WebSocketFeedConfig,
        tick_sender: mpsc::UnboundedSender<MarketTick>,
    ) -> Self {
        Self {
            config,
            tick_sender,
        }
    }

    /// Start the WebSocket connection and feed
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = match self.config.exchange.as_str() {
            "binance" => "wss://stream.binance.com:9443/ws/btcusdt@trade",
            "bybit" => "wss://stream.bybit.com/v5/public/spot",
            _ => return Err("Unsupported exchange".into()),
        };

        let (ws_stream, _) = connect_async(url).await?;
        let (_, mut reader) = ws_stream.split();

        while let Some(message) = reader.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    if let Ok(tick) = self.parse_tick(&text) {
                        let _ = self.tick_sender.send(tick);
                    }
                }
                Ok(Message::Close(_)) => {
                    log::warn!("WebSocket connection closed");
                    break;
                }
                Err(e) => {
                    log::error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Parse tick from WebSocket message
    fn parse_tick(&self, text: &str) -> Result<MarketTick, Box<dyn std::error::Error>> {
        // Simplified parsing - in production, use proper JSON deserialization
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
            let price = json["p"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0);
            let volume = json["q"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0);

            Ok(MarketTick {
                symbol: self
                    .config
                    .symbols
                    .first()
                    .unwrap_or(&"UNKNOWN".to_string())
                    .clone(),
                price,
                volume,
                timestamp: chrono::Utc::now().timestamp_millis(),
                exchange: self.config.exchange.clone(),
            })
        } else {
            Err("Failed to parse tick".into())
        }
    }
}

/// Market data aggregator - consolidates feeds from multiple exchanges
pub struct MarketDataAggregator {
    tick_receiver: mpsc::UnboundedReceiver<MarketTick>,
}

impl MarketDataAggregator {
    /// Create a new market data aggregator
    pub fn new() -> (Self, mpsc::UnboundedSender<MarketTick>) {
        let (tick_sender, tick_receiver) = mpsc::unbounded_channel();

        let aggregator = Self { tick_receiver };

        (aggregator, tick_sender)
    }

    /// Get next tick from any feed
    pub async fn next_tick(&mut self) -> Option<MarketTick> {
        self.tick_receiver.recv().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_tick() {
        let tick = MarketTick {
            symbol: "BTCUSDT".to_string(),
            price: 50000.0,
            volume: 1.5,
            timestamp: 1234567890,
            exchange: "binance".to_string(),
        };

        assert_eq!(tick.symbol, "BTCUSDT");
        assert_eq!(tick.price, 50000.0);
    }
}
