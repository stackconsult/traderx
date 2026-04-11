//! Multi-venue order book aggregator.
//! Maintains separate books per venue and provides aggregated view.

use crate::book::{OrderBook, BookSnapshot, Side};
use crate::venue::VenueMessage;
use crate::metrics::BookMetrics;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Aggregated order book across all venues.
#[derive(Debug, Clone)]
pub struct AggregatedBook {
    pub symbol: String,
    pub aggregated_bids: Vec<(f64, f64)>, // (price, total_quantity)
    pub aggregated_asks: Vec<(f64, f64)>,
    pub venue_books: Vec<String>, // List of venue names
    pub last_update_ns: i64,
}

/// Core aggregator that maintains order books for all venues.
pub struct BookAggregator {
    /// (venue, symbol) -> OrderBook
    books: Arc<DashMap<(String, String), Arc<OrderBook>>>,
    
    /// symbol -> AggregatedBook
    aggregated: Arc<DashMap<String, AggregatedBook>>,
    
    /// Metrics collector
    metrics: Arc<BookMetrics>,
    
    /// Channel for receiving venue messages
    message_rx: mpsc::Receiver<VenueMessage>,
    
    /// Channel for aggregated book updates
    update_tx: mpsc::Sender<(String, AggregatedBook)>,
}

impl BookAggregator {
    pub fn new(
        message_rx: mpsc::Receiver<VenueMessage>,
        update_tx: mpsc::Sender<(String, AggregatedBook)>,
    ) -> Arc<Self> {
        Arc::new(Self {
            books: Arc::new(DashMap::new()),
            aggregated: Arc::new(DashMap::new()),
            metrics: Arc::new(BookMetrics::new()),
            message_rx,
            update_tx,
        })
    }

    /// Run the aggregation loop.
    pub async fn run(self: Arc<Self>) {
        info!("Book aggregator started");
        
        while let Some(msg) = self.message_rx.recv().await {
            let start = std::time::Instant::now();
            
            match msg {
                VenueMessage::LevelUpdate {
                    venue,
                    symbol,
                    side,
                    price,
                    quantity,
                    sequence,
                    timestamp_ns,
                } => {
                    self.process_level_update(venue, symbol, side, price, quantity, sequence, timestamp_ns);
                }
                VenueMessage::Snapshot {
                    venue,
                    symbol,
                    bids,
                    asks,
                    sequence,
                    timestamp_ns,
                } => {
                    self.process_snapshot(venue, symbol, bids, asks, sequence, timestamp_ns);
                }
                VenueMessage::Heartbeat { venue, timestamp_ns } => {
                    debug!("Heartbeat from {}", venue);
                }
            }
            
            // Track latency
            let latency = start.elapsed().as_nanos() as f64;
            self.metrics.record_update_latency(latency);
        }
    }

    fn process_level_update(
        &self,
        venue: String,
        symbol: String,
        side: Side,
        price: f64,
        quantity: f64,
        sequence: i64,
        timestamp_ns: i64,
    ) {
        let key = (venue.clone(), symbol.clone());
        
        // Get or create order book
        let book = self.books.entry(key.clone()).or_insert_with(|| {
            Arc::new(OrderBook::new(symbol.clone(), venue.clone()))
        });
        
        // Update the book (requires mutable reference - use Arc::get_mut in single-threaded context)
        // In production, this would use lock-free structures or per-venue tasks
        if let Some(book_mut) = Arc::get_mut(book) {
            book_mut.update_level(side, price, quantity, sequence, timestamp_ns);
        }
        
        // Re-aggregate
        self.reaggregate_symbol(&symbol);
        
        self.metrics.increment_updates();
    }

    fn process_snapshot(
        &self,
        venue: String,
        symbol: String,
        bids: Vec<(f64, f64)>,
        asks: Vec<(f64, f64)>,
        sequence: i64,
        timestamp_ns: i64,
    ) {
        let key = (venue.clone(), symbol.clone());
        
        // Create new book with snapshot
        let mut book = OrderBook::new(symbol.clone(), venue.clone());
        book.bulk_update(Side::Bid, bids, sequence, timestamp_ns);
        book.bulk_update(Side::Ask, asks, sequence, timestamp_ns);
        
        self.books.insert(key, Arc::new(book));
        
        // Re-aggregate
        self.reaggregate_symbol(&symbol);
        
        self.metrics.increment_snapshots();
    }

    fn reaggregate_symbol(&self, symbol: &str) {
        let mut all_bids = Vec::new();
        let mut all_asks = Vec::new();
        let mut venues = Vec::new();
        let mut last_update = 0i64;
        
        // Collect all levels from all venues
        for entry in self.books.iter() {
            let ((venue, sym), book) = entry.pair();
            if sym == symbol {
                venues.push(venue.clone());
                
                // Get top 10 levels from each venue
                for level in book.top_levels(Side::Bid, 10) {
                    all_bids.push((level.price, level.quantity));
                }
                
                for level in book.top_levels(Side::Ask, 10) {
                    all_asks.push((level.price, level.quantity));
                }
                
                last_update = last_update.max(book.last_update_ns.load(std::sync::atomic::Ordering::Relaxed));
            }
        }
        
        // Aggregate by price level
        let aggregated_bids = self.aggregate_levels(all_bids, true);
        let aggregated_asks = self.aggregate_levels(all_asks, false);
        
        // Create aggregated book
        let agg_book = AggregatedBook {
            symbol: symbol.to_string(),
            aggregated_bids,
            aggregated_asks,
            venue_books: venues,
            last_update_ns: last_update,
        };
        
        // Store and notify
        self.aggregated.insert(symbol.to_string(), agg_book.clone());
        
        if let Err(e) = self.update_tx.send((symbol.to_string(), agg_book)).await {
            error!("Failed to send aggregated book update: {}", e);
        }
    }

    /// Aggregate levels by price (sum quantities at same price).
    fn aggregate_levels(&self, levels: Vec<(f64, f64)>, is_bid: bool) -> Vec<(f64, f64)> {
        let mut price_map = std::collections::HashMap::new();
        
        for (price, qty) in levels {
            *price_map.entry(price).or_insert(0.0) += qty;
        }
        
        let mut aggregated: Vec<_> = price_map.into_iter().collect();
        
        // Sort appropriately
        if is_bid {
            aggregated.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        } else {
            aggregated.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        }
        
        // Keep top 10 levels
        aggregated.truncate(10);
        aggregated
    }

    /// Get the current aggregated book for a symbol.
    pub fn get_aggregated_book(&self, symbol: &str) -> Option<AggregatedBook> {
        self.aggregated.get(symbol).map(|b| b.clone())
    }

    /// Get all venue books for a symbol.
    pub fn get_venue_books(&self, symbol: &str) -> Vec<BookSnapshot> {
        let mut snapshots = Vec::new();
        
        for entry in self.books.iter() {
            let ((venue, sym), book) = entry.pair();
            if sym == symbol {
                snapshots.push(book.snapshot(10));
            }
        }
        
        snapshots
    }

    /// Get statistics from the aggregator.
    pub fn get_stats(&self) -> AggregatorStats {
        AggregatorStats {
            total_books: self.books.len(),
            total_symbols: self.aggregated.len(),
            updates_per_second: self.metrics.updates_per_second(),
            avg_latency_ns: self.metrics.avg_latency_ns(),
        }
    }
}

/// Aggregator statistics.
#[derive(Debug, Clone)]
pub struct AggregatorStats {
    pub total_books: usize,
    pub total_symbols: usize,
    pub updates_per_second: f64,
    pub avg_latency_ns: f64,
}
