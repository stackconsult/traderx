//! High-performance order book implementation.
//! Uses BTreeMap with ordered-float for sorted price levels.
//! Lock-free reads; single-writer per venue.

use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Bid,
    Ask,
}

/// Single price level in the order book.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level {
    pub price: f64,
    pub quantity: f64,
    pub orders: i64, // Number of orders at this level
}

impl Level {
    pub fn new(price: f64, quantity: f64) -> Self {
        Self {
            price,
            quantity,
            orders: 1,
        }
    }
}

/// Order book for a single symbol on a single venue.
/// Thread-safe for reads; updates must be sequential per venue.
#[derive(Debug)]
pub struct OrderBook {
    pub symbol: String,
    pub venue: String,
    pub bids: BTreeMap<OrderedFloat<f64>, Level>,
    pub asks: BTreeMap<OrderedFloat<f64>, Level>,
    pub sequence: AtomicI64,
    pub last_update_ns: AtomicI64,
}

impl OrderBook {
    pub fn new(symbol: String, venue: String) -> Self {
        Self {
            symbol,
            venue,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            sequence: AtomicI64::new(0),
            last_update_ns: AtomicI64::new(0),
        }
    }

    /// Get best bid price and quantity.
    pub fn best_bid(&self) -> Option<(f64, f64)> {
        self.bids
            .iter()
            .rev()
            .next()
            .map(|(_, level)| (level.price, level.quantity))
    }

    /// Get best ask price and quantity.
    pub fn best_ask(&self) -> Option<(f64, f64)> {
        self.asks
            .iter()
            .next()
            .map(|(_, level)| (level.price, level.quantity))
    }

    /// Get spread in basis points.
    pub fn spread_bps(&self) -> Option<f64> {
        if let (Some((bid, _)), Some((ask, _))) = (self.best_bid(), self.best_ask()) {
            let mid = (bid + ask) / 2.0;
            if mid > 0.0 {
                Some((ask - bid) / mid * 10_000.0)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get top N levels for a side.
    pub fn top_levels(&self, side: Side, n: usize) -> Vec<&Level> {
        let iter = match side {
            Side::Bid => self.bids.iter().rev(),
            Side::Ask => self.asks.iter(),
        };
        iter.take(n).map(|(_, level)| level).collect()
    }

    /// Update order book with a new level.
    pub fn update_level(&mut self, side: Side, price: f64, quantity: f64, sequence: i64, ts_ns: i64) {
        let price_key = OrderedFloat(price);
        let book = match side {
            Side::Bid => &mut self.bids,
            Side::Ask => &mut self.asks,
        };

        if quantity > 0.0 {
            // Add or update level
            let level = book.entry(price_key).or_insert_with(|| Level::new(price, quantity));
            level.quantity = quantity;
            level.orders += 1;
        } else {
            // Remove level
            book.remove(&price_key);
        }

        self.sequence.store(sequence, Ordering::Relaxed);
        self.last_update_ns.store(ts_ns, Ordering::Relaxed);
    }

    /// Bulk update multiple levels (for snapshot sync).
    pub fn bulk_update(&mut self, side: Side, updates: Vec<(f64, f64)>, sequence: i64, ts_ns: i64) {
        let book = match side {
            Side::Bid => &mut self.bids,
            Side::Ask => &mut self.asks,
        };

        for (price, quantity) in updates {
            let price_key = OrderedFloat(price);
            if quantity > 0.0 {
                let level = book.entry(price_key).or_insert_with(|| Level::new(price, quantity));
                level.quantity = quantity;
            } else {
                book.remove(&price_key);
            }
        }

        self.sequence.store(sequence, Ordering::Relaxed);
        self.last_update_ns.store(ts_ns, Ordering::Relaxed);
    }

    /// Get a snapshot of the top N levels on both sides.
    pub fn snapshot(&self, depth: usize) -> BookSnapshot {
        BookSnapshot {
            symbol: self.symbol.clone(),
            venue: self.venue.clone(),
            bids: self.top_levels(Side::Bid, depth).into_iter().cloned().collect(),
            asks: self.top_levels(Side::Ask, depth).into_iter().cloned().collect(),
            sequence: self.sequence.load(Ordering::Relaxed),
            timestamp_ns: self.last_update_ns.load(Ordering::Relaxed),
        }
    }
}

/// Serializable snapshot of an order book.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookSnapshot {
    pub symbol: String,
    pub venue: String,
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
    pub sequence: i64,
    pub timestamp_ns: i64,
}

impl BookSnapshot {
    /// Merge with another snapshot (for aggregation).
    pub fn merge(&mut self, other: &BookSnapshot) {
        // For now, simple concatenation - real implementation would aggregate by price
        self.bids.extend_from_slice(&other.bids);
        self.bids.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
        self.bids.truncate(10); // Keep top 10

        self.asks.extend_from_slice(&other.asks);
        self.asks.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
        self.asks.truncate(10); // Keep top 10

        self.sequence = self.sequence.max(other.sequence);
        self.timestamp_ns = self.timestamp_ns.max(other.timestamp_ns);
    }
}
