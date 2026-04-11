//! Venue adapters for different market data feeds.
//! Each adapter runs in its own task and sends messages to the aggregator.

use crate::book::{Side, OrderBook};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// Message from venue to aggregator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VenueMessage {
    /// Single level update.
    LevelUpdate {
        venue: String,
        symbol: String,
        side: Side,
        price: f64,
        quantity: f64,
        sequence: i64,
        timestamp_ns: i64,
    },
    /// Full snapshot (for initial sync).
    Snapshot {
        venue: String,
        symbol: String,
        bids: Vec<(f64, f64)>,
        asks: Vec<(f64, f64)>,
        sequence: i64,
        timestamp_ns: i64,
    },
    /// Heartbeat to keep connection alive.
    Heartbeat {
        venue: String,
        timestamp_ns: i64,
    },
}

/// Trait for venue-specific adapters.
pub trait VenueAdapter {
    fn name(&self) -> &str;
    async fn connect(&mut self) -> anyhow::Result<()>;
    async fn run(&mut self, tx: mpsc::Sender<VenueMessage>) -> anyhow::Result<()>;
    async fn disconnect(&mut self) -> anyhow::Result<()>;
}

/// Example Databento venue adapter.
pub struct DatabentoAdapter {
    api_key: String,
    dataset: String,
    symbols: Vec<String>,
    client: Option<databento::Live>,
}

impl DatabentoAdapter {
    pub fn new(api_key: String, dataset: String, symbols: Vec<String>) -> Self {
        Self {
            api_key,
            dataset,
            symbols,
            client: None,
        }
    }
}

impl VenueAdapter for DatabentoAdapter {
    fn name(&self) -> &str {
        "databento"
    }

    async fn connect(&mut self) -> anyhow::Result<()> {
        self.client = Some(databento::Live::new(&self.api_key));
        info!("Databento adapter connected");
        Ok(())
    }

    async fn run(&mut self, tx: mpsc::Sender<VenueMessage>) -> anyhow::Result<()> {
        let client = self.client.as_mut().ok_or_else(|| anyhow::anyhow!("Not connected"))?;

        // Subscribe to quotes for all symbols
        for symbol in &self.symbols {
            let job = client.subscribe(
                &self.dataset,
                &[symbol],
                databento::Definition::Quotes,
                databento::PublisherTimestamp::Now(),
            )?;

            let tx = tx.clone();
            let symbol = symbol.clone();
            tokio::spawn(async move {
                for msg in job {
                    match msg {
                        databento::Record::Quote(quote) => {
                            let msg = VenueMessage::LevelUpdate {
                                venue: "databento".to_string(),
                                symbol: symbol.clone(),
                                side: Side::Bid,
                                price: quote.bid_px,
                                quantity: quote.bid_sz,
                                sequence: quote.quote_seq as i64,
                                timestamp_ns: quote.ts_event,
                            };
                            if let Err(e) = tx.send(msg).await {
                                error!("Failed to send bid update: {}", e);
                                break;
                            }

                            let msg = VenueMessage::LevelUpdate {
                                venue: "databento".to_string(),
                                symbol: symbol.clone(),
                                side: Side::Ask,
                                price: quote.ask_px,
                                quantity: quote.ask_sz,
                                sequence: quote.quote_seq as i64,
                                timestamp_ns: quote.ts_event,
                            };
                            if let Err(e) = tx.send(msg).await {
                                error!("Failed to send ask update: {}", e);
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            });
        }

        // Send periodic heartbeats
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let msg = VenueMessage::Heartbeat {
                        venue: "databento".to_string(),
                        timestamp_ns: chrono::Utc::now().timestamp_nanos(),
                    };
                    if tx.send(msg).await.is_err() {
                        break;
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    info!("Databento adapter stopping");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn disconnect(&mut self) -> anyhow::Result<()> {
        if let Some(client) = self.client.take() {
            client.stop().await?;
        }
        info!("Databento adapter disconnected");
        Ok(())
    }
}

/// Mock venue adapter for testing.
pub struct MockAdapter {
    name: String,
    symbols: Vec<String>,
    update_interval_ms: u64,
}

impl MockAdapter {
    pub fn new(name: String, symbols: Vec<String>, update_interval_ms: u64) -> Self {
        Self {
            name,
            symbols,
            update_interval_ms,
        }
    }
}

impl VenueAdapter for MockAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    async fn connect(&mut self) -> anyhow::Result<()> {
        info!("Mock adapter {} connected", self.name);
        Ok(())
    }

    async fn run(&mut self, tx: mpsc::Sender<VenueMessage>) -> anyhow::Result<()> {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(self.update_interval_ms));
        let mut sequence = 0i64;

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    for symbol in &self.symbols {
                        // Generate random bid/ask
                        let base_price = 100.0 + fastrand::f64() * 900.0;
                        let spread = 0.01 + fastrand::f64() * 0.09;
                        let bid = base_price - spread / 2.0;
                        let ask = base_price + spread / 2.0;

                        // Bid update
                        let msg = VenueMessage::LevelUpdate {
                            venue: self.name.clone(),
                            symbol: symbol.clone(),
                            side: Side::Bid,
                            price: bid,
                            quantity: 10.0 + fastrand::f64() * 90.0,
                            sequence,
                            timestamp_ns: chrono::Utc::now().timestamp_nanos(),
                        };
                        if tx.send(msg).await.is_err() {
                            return Ok(());
                        }

                        // Ask update
                        let msg = VenueMessage::LevelUpdate {
                            venue: self.name.clone(),
                            symbol: symbol.clone(),
                            side: Side::Ask,
                            price: ask,
                            quantity: 10.0 + fastrand::f64() * 90.0,
                            sequence: sequence + 1,
                            timestamp_ns: chrono::Utc::now().timestamp_nanos(),
                        };
                        if tx.send(msg).await.is_err() {
                            return Ok(());
                        }

                        sequence += 2;
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    info!("Mock adapter {} stopping", self.name);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn disconnect(&mut self) -> anyhow::Result<()> {
        info!("Mock adapter {} disconnected", self.name);
        Ok(())
    }
}
