use dashmap::DashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

/// Fixed-point representation: price stored as integer × 1e8 to avoid f64 drift.
pub type PriceFP = i64;

#[inline]
pub fn to_fp(price: f64) -> PriceFP { (price * 1e8).round() as i64 }
#[inline]
pub fn from_fp(fp: PriceFP) -> f64 { fp as f64 * 1e-8 }

/// Per-symbol position.
#[derive(Debug, Default)]
pub struct Position {
    pub quantity_fp: AtomicI64,     // shares/contracts × 1e8
    pub avg_entry_fp: AtomicI64,    // average entry price × 1e8
    pub realized_pnl_fp: AtomicI64,
}

impl Position {
    pub fn quantity(&self) -> f64  { from_fp(self.quantity_fp.load(Ordering::Relaxed)) }
    pub fn avg_entry(&self) -> f64 { from_fp(self.avg_entry_fp.load(Ordering::Relaxed)) }
    pub fn realized_pnl(&self) -> f64 { from_fp(self.realized_pnl_fp.load(Ordering::Relaxed)) }

    pub fn unrealized_pnl(&self, current_price: f64) -> f64 {
        let qty = self.quantity();
        let entry = self.avg_entry();
        qty * (current_price - entry)
    }
}

/// Portfolio state — designed to be updated from a single-threaded backtest loop
/// but readable from any thread (Prometheus scrape etc.)
pub struct Portfolio {
    pub positions: Arc<DashMap<String, Position>>,
    pub cash_fp: AtomicI64,
    pub realized_pnl_fp: AtomicI64,
    pub commission_paid_fp: AtomicI64,
    pub peak_nav_fp: AtomicI64,
    pub initial_cash_fp: i64,
}

impl Portfolio {
    pub fn new(initial_cash: f64) -> Self {
        let fp = to_fp(initial_cash);
        Self {
            positions: Arc::new(DashMap::new()),
            cash_fp: AtomicI64::new(fp),
            realized_pnl_fp: AtomicI64::new(0),
            commission_paid_fp: AtomicI64::new(0),
            peak_nav_fp: AtomicI64::new(fp),
            initial_cash_fp: fp,
        }
    }

    pub fn cash(&self) -> f64 { from_fp(self.cash_fp.load(Ordering::Relaxed)) }

    pub fn nav(&self, prices: &[(String, f64)]) -> f64 {
        let mut nav = self.cash();
        for (symbol, price) in prices {
            if let Some(pos) = self.positions.get(symbol.as_str()) {
                nav += pos.unrealized_pnl(*price) + pos.quantity() * pos.avg_entry();
            }
        }
        nav
    }

    pub fn drawdown_pct(&self, current_nav: f64) -> f64 {
        let peak = from_fp(self.peak_nav_fp.load(Ordering::Relaxed));
        if peak == 0.0 { return 0.0; }
        (current_nav - peak) / peak
    }

    /// Apply a fill. `qty` positive = buy, negative = sell.
    pub fn apply_fill(&self, symbol: &str, qty: f64, fill_price: f64, commission: f64) {
        let mut entry = self.positions.entry(symbol.to_owned()).or_default();
        let old_qty = entry.quantity();
        let old_entry = entry.avg_entry();

        if qty > 0.0 {
            // Buy: update average entry
            let new_qty = old_qty + qty;
            let new_entry = if new_qty.abs() < 1e-12 {
                0.0
            } else {
                (old_qty * old_entry + qty * fill_price) / new_qty
            };
            entry.quantity_fp.store(to_fp(new_qty), Ordering::Relaxed);
            entry.avg_entry_fp.store(to_fp(new_entry), Ordering::Relaxed);
            // Debit cash
            let cost = qty * fill_price + commission;
            self.cash_fp.fetch_sub(to_fp(cost), Ordering::Relaxed);
        } else {
            // Sell: realize P&L on |qty| units
            let sell_qty = qty.abs();
            let pnl = sell_qty * (fill_price - old_entry) - commission;
            entry.quantity_fp.fetch_add(to_fp(qty), Ordering::Relaxed);
            entry.realized_pnl_fp.fetch_add(to_fp(pnl), Ordering::Relaxed);
            self.realized_pnl_fp.fetch_add(to_fp(pnl), Ordering::Relaxed);
            // Credit cash
            let proceeds = sell_qty * fill_price - commission;
            self.cash_fp.fetch_add(to_fp(proceeds), Ordering::Relaxed);
        }
        self.commission_paid_fp.fetch_add(to_fp(commission), Ordering::Relaxed);
    }

    pub fn reset(&self, initial_cash: f64) {
        self.positions.clear();
        self.cash_fp.store(to_fp(initial_cash), Ordering::Relaxed);
        self.realized_pnl_fp.store(0, Ordering::Relaxed);
        self.commission_paid_fp.store(0, Ordering::Relaxed);
        self.peak_nav_fp.store(to_fp(initial_cash), Ordering::Relaxed);
    }
}
