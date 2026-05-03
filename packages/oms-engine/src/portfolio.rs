//! Portfolio engine — tracks live P&L, NAV, and feeds the risk bus.
//! This module lives in oms-engine to share the Order / Fill types directly.

use crate::risk_bus::RiskBus;
use crate::state_machine::Side;
use dashmap::DashMap;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use tracing::debug;

/// Live position in a single symbol.
#[derive(Debug, Default)]
pub struct LivePosition {
    pub quantity:     AtomicI64, // × 1e8
    pub avg_entry:    AtomicI64, // × 1e8 USD
    pub realized_pnl: AtomicI64, // × 1e4 USD
}

#[inline]
fn fp8(v: f64) -> i64  { (v * 1e8).round() as i64 }
#[inline]
fn ff8(fp: i64) -> f64 { fp as f64 * 1e-8 }
#[inline]
fn fp4(v: f64) -> i64  { (v * 1e4).round() as i64 }
#[inline]
fn ff4(fp: i64) -> f64 { fp as f64 * 1e-4 }

/// Portfolio engine — call `on_fill` for every execution, `on_price` for mark-to-market.
pub struct PortfolioEngine {
    pub positions:   Arc<DashMap<String, LivePosition>>,
    pub cash_fp:     AtomicI64,  // × 1e4
    pub realized_fp: AtomicI64,  // × 1e4
    pub risk_bus:    Arc<RiskBus>,
    initial_cash:    f64,
}

impl PortfolioEngine {
    pub fn new(initial_cash: f64, risk_bus: Arc<RiskBus>) -> Arc<Self> {
        Arc::new(Self {
            positions:   Arc::new(DashMap::new()),
            cash_fp:     AtomicI64::new(fp4(initial_cash)),
            realized_fp: AtomicI64::new(0),
            risk_bus,
            initial_cash,
        })
    }

    /// Called by OMS on every fill confirmation.
    pub fn on_fill(
        &self,
        symbol: &str,
        side: &Side,
        qty: Decimal,
        fill_price: Decimal,
        commission: f64,
    ) {
        let qty_f = qty.to_f64().unwrap_or(0.0);
        let price_f = fill_price.to_f64().unwrap_or(0.0);
        let signed_qty = if *side == Side::Buy { qty_f } else { -qty_f };

        let entry = self.positions.entry(symbol.to_owned()).or_default();
        let old_qty   = ff8(entry.quantity.load(Ordering::Relaxed));
        let old_entry = ff8(entry.avg_entry.load(Ordering::Relaxed));

        let new_qty = old_qty + signed_qty;

        if signed_qty > 0.0 {
            // Buy — update average entry
            let new_avg = if new_qty.abs() < 1e-12 {
                0.0
            } else {
                (old_qty * old_entry + signed_qty * price_f) / new_qty
            };
            entry.quantity.store(fp8(new_qty), Ordering::Relaxed);
            entry.avg_entry.store(fp8(new_avg), Ordering::Relaxed);
            let cost = qty_f * price_f + commission;
            self.cash_fp.fetch_sub(fp4(cost), Ordering::Relaxed);
        } else {
            // Sell — realize P&L
            let sell_qty = signed_qty.abs();
            let pnl = sell_qty * (price_f - old_entry) - commission;
            entry.quantity.fetch_add(fp8(signed_qty), Ordering::Relaxed);
            entry.realized_pnl.fetch_add(fp4(pnl), Ordering::Relaxed);
            self.realized_fp.fetch_add(fp4(pnl), Ordering::Relaxed);
            let proceeds = sell_qty * price_f - commission;
            self.cash_fp.fetch_add(fp4(proceeds), Ordering::Relaxed);
        }

        debug!(symbol, signed_qty, price_f, commission, "Fill applied");

        // Update risk bus symbol limit
        let _notional = qty_f * price_f;
        if let Some(limit) = self.risk_bus.symbol_limits.get(symbol) {
            limit.update(signed_qty * price_f);
        }
    }

    /// Called on every market data tick to mark-to-market.
    pub fn on_price(&self, _symbol: &str, price: f64) {
        // Compute total NAV: cash + Σ(position MtM)
        let cash = ff4(self.cash_fp.load(Ordering::Relaxed));
        let mut nav = cash;

        for entry in self.positions.iter() {
            let qty = ff8(entry.quantity.load(Ordering::Relaxed));
            let entry_price = ff8(entry.avg_entry.load(Ordering::Relaxed));
            let realized = ff4(entry.realized_pnl.load(Ordering::Relaxed));
            let unrealized = qty * (price - entry_price);
            nav += unrealized + realized;
        }

        // Feed risk bus
        self.risk_bus.update_nav(nav);
    }

    pub fn cash(&self) -> f64 {
        ff4(self.cash_fp.load(Ordering::Relaxed))
    }

    pub fn realized_pnl(&self) -> f64 {
        ff4(self.realized_fp.load(Ordering::Relaxed))
    }

    pub fn position_quantity(&self, symbol: &str) -> f64 {
        self.positions
            .get(symbol)
            .map(|p| ff8(p.quantity.load(Ordering::Relaxed)))
            .unwrap_or(0.0)
    }

    pub fn position_avg_entry(&self, symbol: &str) -> f64 {
        self.positions
            .get(symbol)
            .map(|p| ff8(p.avg_entry.load(Ordering::Relaxed)))
            .unwrap_or(0.0)
    }
}
