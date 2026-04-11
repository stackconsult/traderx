use order_book_aggregator::{
    BookAggregator, VenueAdapter, VenueMessage, MockAdapter, DatabentoAdapter,
};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, error};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    // Configuration
    let symbols = vec!["BTC-USD".to_string(), "ETH-USD".to_string()];
    let venues: Vec<Box<dyn VenueAdapter + Send + Sync>> = vec![
        Box::new(MockAdapter::new(
            "mock1".to_string(),
            symbols.clone(),
            100, // 100ms update interval
        )),
        Box::new(MockAdapter::new(
            "mock2".to_string(),
            symbols.clone(),
            150,
        )),
        // Uncomment to use real Databento
        // Box::new(DatabentoAdapter::new(
        //     std::env::var("DATABENTO_API_KEY")?,
        //     "XNAS.ITCH".to_string(),
        //     symbols.clone(),
        // )),
    ];

    // Create channels
    let (message_tx, message_rx) = mpsc::channel(10_000);
    let (update_tx, mut update_rx) = mpsc::channel(1_000);

    // Start aggregator
    let aggregator = BookAggregator::new(message_rx, update_tx);
    let aggregator_task = tokio::spawn(Arc::clone(&aggregator).run());

    // Start venue adapters
    let mut venue_tasks = Vec::new();
    for mut adapter in venues {
        let tx = message_tx.clone();
        let task = tokio::spawn(async move {
            if let Err(e) = adapter.connect().await {
                error!("Failed to connect to {}: {}", adapter.name(), e);
                return;
            }
            info!("Started venue adapter: {}", adapter.name());
            if let Err(e) = adapter.run(tx).await {
                error!("Venue adapter {} error: {}", adapter.name(), e);
            }
            let _ = adapter.disconnect().await;
        });
        venue_tasks.push(task);
    }

    // Task to handle aggregated book updates
    let update_task = tokio::spawn(async move {
        while let Some((symbol, book)) = update_rx.recv().await {
            info!(
                "Aggregated book update for {}: {} venues, best_bid={:?}, best_ask={:?}",
                symbol,
                book.venue_books.len(),
                book.aggregated_bids.first(),
                book.aggregated_asks.first(),
            );
        }
    });

    // Periodic stats reporting
    let stats_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
        loop {
            interval.tick().await;
            let stats = aggregator.get_stats();
            info!(
                "Aggregator stats: {} books, {} symbols, {:.1} updates/sec, {:.0}ns avg latency",
                stats.total_books,
                stats.total_symbols,
                stats.updates_per_second,
                stats.avg_latency_ns,
            );
        }
    });

    // Wait for shutdown
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Shutting down...");
        }
        _ = aggregator_task => {
            error!("Aggregator task exited unexpectedly");
        }
    }

    // Cleanup
    drop(message_tx);
    for task in venue_tasks {
        let _ = task.await;
    }
    let _ = aggregator_task.await;
    let _ = update_task.await;
    stats_task.abort();

    Ok(())
}
