//! Exposure tracking — per-asset-class, per-strategy, and portfolio-level.
//! All notional values stored in USD base currency.

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicI64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetClass {
    Equity,
    Crypto,
    FX,
    Futures,
    Options,
    FixedIncome,
}

/// Exposure for a single symbol in USD (× 1e4 fixed point).
#[derive(Debug, Default)]
pub struct Exposure {
    /// Gross notional (absolute sum of long+short).
    pub gross_fp: AtomicI64,
    /// Net notional (long - short).
    pub net_fp: AtomicI64,
}

impl Exposure {
    #[inline]
    pub fn gross(&self) -> f64 { self.gross_fp.load(Ordering::Relaxed) as f64 * 1e-4 }
    #[inline]
    pub fn net(&self)   -> f64 { self.net_fp.load(Ordering::Relaxed)   as f64 * 1e-4 }

    /// Update with a delta notional in USD.
    #[inline]
    pub fn apply(&self, delta_notional_usd: f64) {
        let delta_fp = (delta_notional_usd * 1e4) as i64;
        self.gross_fp.fetch_add(delta_fp.abs(), Ordering::Relaxed);
        self.net_fp.fetch_add(delta_fp, Ordering::Relaxed);
    }

    /// Reset exposure (e.g. after position close).
    #[inline]
    pub fn reset(&self) {
        self.gross_fp.store(0, Ordering::Relaxed);
        self.net_fp.store(0, Ordering::Relaxed);
    }
}

/// Multi-asset exposure container.
pub struct ExposureBook {
    /// (strategy_id, symbol) → Exposure
    pub per_symbol: DashMap<(String, String), Exposure>,
    /// (strategy_id, asset_class) → Exposure
    pub per_asset_class: DashMap<(String, AssetClass), Exposure>,
    /// Portfolio-level per-asset-class exposure.
    pub portfolio: DashMap<AssetClass, Exposure>,
}

impl ExposureBook {
    pub fn new() -> Self {
        Self {
            per_symbol: DashMap::new(),
            per_asset_class: DashMap::new(),
            portfolio: DashMap::new(),
        }
    }

    /// Apply a fill to exposure tracking.
    pub fn apply_fill(
        &self,
        strategy_id: &str,
        symbol: &str,
        asset_class: AssetClass,
        delta_notional_usd: f64,
    ) {
        // Update per-symbol
        self.per_symbol
            .entry((strategy_id.to_owned(), symbol.to_owned()))
            .or_default()
            .apply(delta_notional_usd);

        // Update per-asset-class for strategy
        self.per_asset_class
            .entry((strategy_id.to_owned(), asset_class))
            .or_default()
            .apply(delta_notional_usd);

        // Update portfolio-level
        self.portfolio
            .entry(asset_class)
            .or_default()
            .apply(delta_notional_usd);
    }

    /// Get gross exposure for a strategy across all assets.
    pub fn strategy_gross(&self, strategy_id: &str) -> f64 {
        let mut total = 0.0;
        for entry in self.per_symbol.iter() {
            if entry.key().0 == strategy_id {
                total += entry.value().gross();
            }
        }
        total
    }

    /// Get net exposure for a strategy across all assets.
    pub fn strategy_net(&self, strategy_id: &str) -> f64 {
        let mut total = 0.0;
        for entry in self.per_symbol.iter() {
            if entry.key().0 == strategy_id {
                total += entry.value().net();
            }
        }
        total
    }

    /// Get portfolio gross exposure by asset class.
    pub fn portfolio_by_asset(&self) -> Vec<(AssetClass, f64)> {
        self.portfolio
            .iter()
            .map(|entry| (*entry.key(), entry.value().gross()))
            .collect()
    }
}
