use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::schema_registry::{BamSchemaRegistry, MarketClass};

/// 26-byte packed BAM cell — same layout across all market classes,
/// interpretation varies per schema registry
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BamCell {
    pub bid_price: u32, // scaled by schema tick_size
    pub ask_price: u32,
    pub quantity: u32, // scaled by schema lot_size
    pub volume: u32,
    pub timestamp: u32, // milliseconds within the 60s bucket
    pub flags: u16,     // market-specific flags
}

impl BamCell {
    pub const SIZE: usize = 26;

    /// Decode price using schema tick size
    pub fn price_f64(&self, schema: &super::schema_registry::MarketSchema) -> f64 {
        (self.bid_price as f64 + self.ask_price as f64) / 2.0 * schema.tick_size
    }

    /// Decode spread in basis points
    pub fn spread_bps(&self, schema: &super::schema_registry::MarketSchema) -> f64 {
        let mid = (self.bid_price as f64 + self.ask_price as f64) / 2.0;
        if mid == 0.0 {
            return 0.0;
        }
        let spread = (self.ask_price as f64 - self.bid_price as f64).abs();
        (spread / mid) * 10_000.0 * schema.tick_size
    }

    /// Decode quantity in natural units
    pub fn quantity_f64(&self, schema: &super::schema_registry::MarketSchema) -> f64 {
        self.quantity as f64 * schema.lot_size
    }
}

/// 10 price levels × 60 time buckets = 600 cells per market
/// Total: ~15.6KB per market (600 × 26B)
pub const GRID_LEVELS: usize = 10;
pub const GRID_TIME_BUCKETS: usize = 60;
pub const GRID_CELLS: usize = GRID_LEVELS * GRID_TIME_BUCKETS;

/// Single-market BAM grid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BamGrid {
    pub market_class: MarketClass,
    pub symbol: String,
    pub cells: Vec<BamCell>,
    pub timestamp: DateTime<Utc>,
    /// Overall predictability computed from pattern detection pass
    pub overall_predictability: f64,
}

impl BamGrid {
    pub fn new(market_class: MarketClass, symbol: String) -> Self {
        let empty_cell = BamCell {
            bid_price: 0,
            ask_price: 0,
            quantity: 0,
            volume: 0,
            timestamp: 0,
            flags: 0,
        };
        Self {
            market_class,
            symbol,
            cells: vec![empty_cell; GRID_CELLS],
            timestamp: Utc::now(),
            overall_predictability: 0.0,
        }
    }

    /// Index into cells: level (0–9) × time_bucket (0–59)
    #[inline]
    pub fn cell(&self, level: usize, time_bucket: usize) -> &BamCell {
        &self.cells[level * GRID_TIME_BUCKETS + time_bucket]
    }

    #[inline]
    pub fn cell_mut(&mut self, level: usize, time_bucket: usize) -> &mut BamCell {
        &mut self.cells[level * GRID_TIME_BUCKETS + time_bucket]
    }
}

/// All 6 market classes simultaneously
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiMarketBamGrids {
    pub markets: [BamGrid; 6],
    pub snapshot_time: DateTime<Utc>,
}

impl MultiMarketBamGrids {
    pub fn new(symbols_per_class: [&str; 6]) -> Self {
        Self {
            markets: [
                BamGrid::new(MarketClass::Equities, symbols_per_class[0].to_string()),
                BamGrid::new(MarketClass::FX, symbols_per_class[1].to_string()),
                BamGrid::new(MarketClass::Metals, symbols_per_class[2].to_string()),
                BamGrid::new(MarketClass::Commodities, symbols_per_class[3].to_string()),
                BamGrid::new(MarketClass::Crypto, symbols_per_class[4].to_string()),
                BamGrid::new(MarketClass::Indices, symbols_per_class[5].to_string()),
            ],
            snapshot_time: Utc::now(),
        }
    }

    pub fn get(&self, class: MarketClass) -> &BamGrid {
        &self.markets[class as usize]
    }

    pub fn get_mut(&mut self, class: MarketClass) -> &mut BamGrid {
        &mut self.markets[class as usize]
    }

    /// Weighted average predictability across all markets
    pub fn blended_predictability(&self) -> f64 {
        let sum: f64 = self.markets.iter().map(|g| g.overall_predictability).sum();
        sum / 6.0
    }
}

/// Allocator: assigns capital across market classes based on grid predictability
#[derive(Debug, Clone)]
pub struct PortfolioFabricAllocator {
    pub allocation: [f64; 6],
    pub risk_budget: [f64; 6],
    pub target_sharpe: f64,
    pub rebalance_threshold: f64,
    pub schema: Arc<BamSchemaRegistry>,
}

impl PortfolioFabricAllocator {
    pub fn new(schema: Arc<BamSchemaRegistry>, target_sharpe: f64) -> Self {
        Self {
            allocation: [1.0 / 6.0; 6],
            risk_budget: [1.0 / 6.0; 6],
            target_sharpe,
            rebalance_threshold: 0.05,
            schema,
        }
    }

    /// Kelly Criterion simplified: f* ≈ (2p - 1) / 2 for 2:1 payoff assumption
    /// Then correlation penalty applied
    pub fn rebalance(&mut self, grids: &MultiMarketBamGrids) -> Vec<RebalanceOrder> {
        let predictability: [f64; 6] = [
            grids.markets[0].overall_predictability,
            grids.markets[1].overall_predictability,
            grids.markets[2].overall_predictability,
            grids.markets[3].overall_predictability,
            grids.markets[4].overall_predictability,
            grids.markets[5].overall_predictability,
        ];

        let mut kelly: [f64; 6] = [0.0; 6];
        for i in 0..6 {
            let p = predictability[i].clamp(0.0, 1.0);
            kelly[i] = ((2.0 * p - 1.0) / 2.0).max(0.0);
        }

        let adjusted = self.apply_correlation_penalty(&kelly);
        let sum: f64 = adjusted.iter().sum();
        if sum < 1e-9 {
            return vec![];
        }

        let target: Vec<f64> = adjusted.iter().map(|x| x / sum).collect();

        target
            .iter()
            .zip(self.allocation.iter())
            .enumerate()
            .filter(|(_, (t, a))| (*t - *a).abs() > self.rebalance_threshold)
            .map(|(i, (t, a))| RebalanceOrder {
                market_class: MarketClass::from(i as u8),
                target_weight: *t,
                current_weight: *a,
                delta: *t - *a,
            })
            .collect()
    }

    fn apply_correlation_penalty(&self, kelly: &[f64; 6]) -> [f64; 6] {
        let mut result = *kelly;
        for i in 0..6 {
            for j in (i + 1)..6 {
                let corr = self
                    .schema
                    .correlation(MarketClass::from(i as u8), MarketClass::from(j as u8));
                if corr > 0.7 {
                    // Reduce both allocations when highly correlated
                    let penalty = 1.0 - (corr - 0.7) * 2.0;
                    result[i] *= penalty;
                    result[j] *= penalty;
                }
            }
        }
        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RebalanceOrder {
    pub market_class: MarketClass,
    pub target_weight: f64,
    pub current_weight: f64,
    pub delta: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_bam_cell_size() {
        assert_eq!(std::mem::size_of::<BamCell>(), BamCell::SIZE);
    }

    #[test]
    fn test_grid_indexing() {
        let grid = BamGrid::new(MarketClass::Equities, "AAPL".to_string());
        let cell = grid.cell(5, 30);
        assert_eq!(cell.bid_price, 0); // empty grid
    }

    #[test]
    fn test_multi_market_predictability() {
        let mut grids =
            MultiMarketBamGrids::new(["SPY", "EURUSD", "XAUUSD", "CL=F", "BTCUSD", "SPX"]);
        grids.markets[0].overall_predictability = 0.8;
        grids.markets[1].overall_predictability = 0.6;
        assert!((grids.blended_predictability() - (0.8 + 0.6) / 6.0).abs() < 0.001);
    }

    #[test]
    fn test_rebalance_generates_orders() {
        let schema = Arc::new(BamSchemaRegistry::new());
        let mut allocator = PortfolioFabricAllocator::new(schema, 1.5);
        let mut grids =
            MultiMarketBamGrids::new(["SPY", "EURUSD", "XAUUSD", "CL=F", "BTCUSD", "SPX"]);
        grids.markets[0].overall_predictability = 0.9; // Equities very predictable
        grids.markets[1].overall_predictability = 0.1; // FX not predictable

        let orders = allocator.rebalance(&grids);
        assert!(!orders.is_empty());

        let eq_order = orders
            .iter()
            .find(|o| o.market_class == MarketClass::Equities);
        assert!(eq_order.is_some());
        assert!(eq_order.unwrap().delta > 0.0); // should increase equity allocation
    }
}
