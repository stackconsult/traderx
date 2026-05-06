// Batch Data Ingestion Pipeline
// Phase 0: Foundation - Task P0-1: 10K ticks/sec ingestion

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

/// Market tick data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTick {
    pub timestamp: i64,
    pub market: String,
    pub price: f64,
    pub volume: f64,
    pub bid: f64,
    pub ask: f64,
}

/// Batch ingest configuration
#[derive(Debug, Clone)]
pub struct BatchIngestConfig {
    pub batch_size: usize,
    pub batch_interval_ms: u64,
    pub max_buffer_size: usize,
}

impl Default for BatchIngestConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            batch_interval_ms: 100,
            max_buffer_size: 10_000,
        }
    }
}

/// Batch ingest statistics
#[derive(Debug, Default)]
pub struct IngestStats {
    pub ticks_processed: AtomicU64,
    pub ticks_dropped: AtomicU64,
    pub batches_processed: AtomicU64,
}

impl IngestStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment_ticks(&self) {
        self.ticks_processed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_dropped(&self) {
        self.ticks_dropped.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_batches(&self) {
        self.batches_processed.fetch_add(1, Ordering::Relaxed);
    }
}

/// Batch data ingest pipeline
pub struct BatchIngestPipeline {
    config: BatchIngestConfig,
    tick_rx: mpsc::Receiver<MarketTick>,
    stats: Arc<IngestStats>,
}

impl BatchIngestPipeline {
    /// Create a new batch ingest pipeline
    pub fn new(config: BatchIngestConfig, tick_rx: mpsc::Receiver<MarketTick>) -> Self {
        Self {
            config,
            tick_rx,
            stats: Arc::new(IngestStats::new()),
        }
    }

    /// Get statistics
    pub fn get_stats(&self) -> Arc<IngestStats> {
        self.stats.clone()
    }

    /// Start the ingest pipeline
    pub async fn run(mut self) -> Result<(), anyhow::Error> {
        let mut buffer = Vec::with_capacity(self.config.batch_size);
        let mut ticker = interval(Duration::from_millis(self.config.batch_interval_ms));
        ticker.tick().await; // Skip first tick

        loop {
            tokio::select! {
                // Receive ticks
                tick_result = self.tick_rx.recv() => {
                    match tick_result {
                        Some(tick) => {
                            if buffer.len() < self.config.max_buffer_size {
                                buffer.push(tick);
                                self.stats.increment_ticks();
                            } else {
                                self.stats.increment_dropped();
                            }
                        }
                        None => {
                            // Channel closed, flush remaining buffer and exit
                            if !buffer.is_empty() {
                                self.process_batch(&buffer).await?;
                            }
                            break;
                        }
                    }
                }

                // Batch interval tick
                _ = ticker.tick() => {
                    if !buffer.is_empty() {
                        self.process_batch(&buffer).await?;
                        buffer.clear();
                    }
                }
            }
        }

        Ok(())
    }

    /// Process a batch of ticks
    async fn process_batch(&self, batch: &[MarketTick]) -> Result<()> {
        self.stats.increment_batches();

        // TODO: Store batch to three-tier storage
        // For now, just log the batch size
        log::info!("Processing batch of {} ticks", batch.len());

        // Group by market
        let mut by_market: HashMap<String, Vec<&MarketTick>> = HashMap::new();
        for tick in batch {
            by_market
                .entry(tick.market.clone())
                .or_insert_with(Vec::new)
                .push(tick);
        }

        // TODO: Send to storage layer
        for (market, ticks) in by_market {
            log::debug!("Market {}: {} ticks", market, ticks.len());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_batch_ingest_pipeline() {
        let (tx, rx) = mpsc::channel(1000);
        let config = BatchIngestConfig {
            batch_size: 10,
            batch_interval_ms: 50,
            max_buffer_size: 100,
        };

        let pipeline = BatchIngestPipeline::new(config, rx);
        let stats = pipeline.get_stats();

        // Spawn pipeline in background
        let pipeline_handle = tokio::spawn(async move { pipeline.run().await });

        // Send test ticks
        for i in 0..20 {
            let tick = MarketTick {
                timestamp: i as i64,
                market: "AAPL".to_string(),
                price: 150.0 + (i as f64 * 0.01),
                volume: 100.0,
                bid: 149.99,
                ask: 150.01,
            };
            tx.send(tick).await.unwrap();
        }

        // Wait for batch processing
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Verify stats
        assert!(stats.ticks_processed.load(Ordering::Relaxed) >= 20);
        assert!(stats.batches_processed.load(Ordering::Relaxed) > 0);

        // Cleanup
        drop(tx);
        pipeline_handle.await.unwrap().unwrap();
    }
}
