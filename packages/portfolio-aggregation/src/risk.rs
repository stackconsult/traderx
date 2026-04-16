//! Risk metrics — VaR and concentration tracking.

use crate::exposure::AssetClass;
use dashmap::DashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use tracing::debug;

/// Simple parametric VaR engine (historical simulation approach).
pub struct VarEngine {
    /// Current prices by symbol.
    prices: DashMap<String, f64>,
    /// Position data: (strategy, symbol) -> (quantity, avg_entry)
    positions: DashMap<(String, String), (f64, f64)>,
    /// Historical returns for VaR calculation (last 252 trading days).
    returns_history: DashMap<String, Vec<f64>>,
}

impl VarEngine {
    pub fn new() -> Self {
        Self {
            prices: DashMap::new(),
            positions: DashMap::new(),
            returns_history: DashMap::new(),
        }
    }

    pub fn update_price(&self, symbol: &str, price: f64) {
        if let Some(old_price) = self.prices.get(symbol) {
            let ret = (price / *old_price - 1.0).ln();
            self.returns_history
                .entry(symbol.to_owned())
                .or_insert_with(Vec::new)
                .push(ret);
            
            // Keep only last 252 returns
            if let Some(history) = self.returns_history.get(symbol) {
                if history.len() > 252 {
                    let mut h = history.clone();
                    h.remove(0);
                    self.returns_history.insert(symbol.to_owned(), h);
                }
            }
        }
        self.prices.insert(symbol.to_owned(), price);
    }

    pub fn update_position(&self, strategy: &str, symbol: &str, quantity: f64, avg_entry: f64) {
        self.positions.insert((strategy.to_owned(), symbol.to_owned()), (quantity, avg_entry));
    }

    pub fn get_price(&self, symbol: &str) -> f64 {
        self.prices.get(symbol).map(|p| *p).unwrap_or(0.0)
    }

    /// Calculate 1-day 95% VaR for a strategy using historical simulation.
    pub fn calculate_var(&self, strategy: &str, confidence: f64) -> f64 {
        let mut portfolio_value = 0.0;
        let mut weights = Vec::new();
        let mut returns = Vec::new();

        // Collect all positions for the strategy
        for entry in self.positions.iter() {
            if entry.key().0 == strategy {
                let (qty, avg) = entry.value();
                let current_price = self.get_price(&entry.key().1);
                let value = qty.abs() * current_price;
                portfolio_value += value;
                
                if let Some(history) = self.returns_history.get(&entry.key().1) {
                    weights.push(value / portfolio_value);
                    returns.push(history.clone());
                }
            }
        }

        if portfolio_value == 0.0 || returns.is_empty() {
            return 0.0;
        }

        // Simple historical VaR (weighted sum of individual VaRs)
        let mut portfolio_returns = vec![0.0; 252];
        for (weight, asset_returns) in weights.iter().zip(returns.iter()) {
            for (i, &r) in asset_returns.iter().enumerate() {
                if i < portfolio_returns.len() {
                    portfolio_returns[i] += weight * r;
                }
            }
        }

        portfolio_returns.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let var_index = ((1.0 - confidence) * portfolio_returns.len() as f64) as usize;
        -portfolio_returns[var_index.min(portfolio_returns.len() - 1)] * portfolio_value
    }

    pub fn reset_strategy(&self, strategy: &str) {
        self.positions.retain(|(s, _), _| s != strategy);
    }
}

/// Concentration risk tracker.
pub struct ConcentrationEngine {
    /// Strategy-level exposure by asset class: (strategy, asset_class) -> exposure_usd
    exposures: DashMap<(String, AssetClass), AtomicI64>,
}

impl ConcentrationEngine {
    pub fn new() -> Self {
        Self {
            exposures: DashMap::new(),
        }
    }

    pub fn update_exposure(&self, strategy: &str, asset_class: AssetClass, delta_usd: f64) {
        let delta_fp = (delta_usd * 1e4) as i64;
        self.exposures
            .entry((strategy.to_owned(), asset_class))
            .or_default()
            .fetch_add(delta_fp, Ordering::Relaxed);
    }

    /// Get concentration report for a strategy.
    pub fn get_concentration(&self, strategy: &str) -> ConcentrationReport {
        let mut total_exposure = 0.0;
        let mut by_asset = Vec::new();

        for entry in self.exposures.iter() {
            if entry.key().0 == strategy {
                let exposure = entry.value().load(Ordering::Relaxed) as f64 * 1e-4;
                total_exposure += exposure.abs();
                by_asset.push((entry.key().1.clone(), exposure.abs()));
            }
        }

        // Calculate percentages
        let by_asset_pct = by_asset
            .into_iter()
            .map(|(asset, exp)| {
                let pct = if total_exposure > 0.0 { exp / total_exposure * 100.0 } else { 0.0 };
                (asset, exp, pct)
            })
            .collect();

        ConcentrationReport {
            strategy_id: strategy.to_owned(),
            total_exposure_usd: total_exposure,
            by_asset: by_asset_pct,
        }
    }

    pub fn reset_strategy(&self, strategy: &str) {
        self.exposures.retain(|(s, _), _| s != strategy);
    }
}

#[derive(Debug, Clone)]
pub struct ConcentrationReport {
    pub strategy_id: String,
    pub total_exposure_usd: f64,
    pub by_asset: Vec<(AssetClass, f64, f64)>, // (asset, exposure, percentage)
}

#[derive(Debug, Clone)]
pub struct VarReport {
    pub strategy_id: String,
    pub var_95_1d: f64,
    pub var_99_1d: f64,
    pub var_95_5d: f64,
    pub var_99_5d: f64,
}
