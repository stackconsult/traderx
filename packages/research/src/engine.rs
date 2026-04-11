use crate::metrics::{compute, BacktestMetrics};
use crate::portfolio::Portfolio;
use crate::slippage::{CommissionModel, SlippageModel};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// A single OHLCV bar as input to the engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bar {
    pub ts_nanos: i64,
    pub symbol:   String,
    pub open:     f64,
    pub high:     f64,
    pub low:      f64,
    pub close:    f64,
    pub volume:   f64,
    pub vwap:     f64,
}

/// Signal emitted by a strategy for a given bar.
#[derive(Debug, Clone)]
pub struct Signal {
    pub symbol:    String,
    pub direction: Direction,
    /// Fraction of portfolio to allocate, 0.0–1.0.
    pub size_frac: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction { Long, Short, Flat }

/// Fill record for post-analysis.
#[derive(Debug, Clone, Serialize)]
pub struct Fill {
    pub ts_nanos:   i64,
    pub symbol:     String,
    pub qty:        f64,
    pub price:      f64,
    pub commission: f64,
    pub pnl:        f64,
}

/// Trait every strategy must implement.
pub trait Strategy: Send {
    fn on_bar(&mut self, bar: &Bar, portfolio: &Portfolio) -> Option<Signal>;
    fn name(&self) -> &str;
}

/// Engine configuration.
pub struct EngineConfig {
    pub initial_cash:    f64,
    pub bars_per_year:   f64,
    pub slippage:        SlippageModel,
    pub commission:      CommissionModel,
    pub max_drawdown_halt: Option<f64>, // halt if DD > this (e.g. 0.20)
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            initial_cash: 1_000_000.0,
            bars_per_year: 252.0 * 390.0, // 1-min bars
            slippage: SlippageModel::default(),
            commission: CommissionModel::default(),
            max_drawdown_halt: Some(0.25),
        }
    }
}

/// Backtest result.
pub struct BacktestResult {
    pub metrics:  BacktestMetrics,
    pub nav_curve: Vec<f64>,
    pub fills:    Vec<Fill>,
}

/// Core vectorized backtesting engine.
///
/// Process loop: O(n_bars × n_symbols) — pure Rust, no allocations per bar.
/// Target: ≥10M bars/sec on a single core (Arrow columnar input removes copies).
pub struct BacktestEngine {
    cfg:      EngineConfig,
    portfolio: Portfolio,
    nav_curve: Vec<f64>,
    fills:    Vec<Fill>,
    trade_pnls: Vec<f64>,
}

impl BacktestEngine {
    pub fn new(cfg: EngineConfig) -> Self {
        let portfolio = Portfolio::new(cfg.initial_cash);
        Self { cfg, portfolio, nav_curve: Vec::new(), fills: Vec::new(), trade_pnls: Vec::new() }
    }

    /// Run strategy over a slice of bars. Single-threaded hot path.
    pub fn run<S: Strategy>(&mut self, bars: &[Bar], strategy: &mut S, adv_map: &[(String, f64)]) -> BacktestResult {
        self.portfolio.reset(self.cfg.initial_cash);
        self.nav_curve.clear();
        self.fills.clear();
        self.trade_pnls.clear();

        for bar in bars {
            // Let strategy emit a signal
            if let Some(signal) = strategy.on_bar(bar, &self.portfolio) {
                self.execute_signal(&signal, bar, adv_map);
            }

            // Record NAV at bar close
            let prices = [(bar.symbol.clone(), bar.close)];
            let nav = self.portfolio.nav(&prices);
            self.nav_curve.push(nav);

            // Check drawdown halt
            if let Some(limit) = self.cfg.max_drawdown_halt {
                if self.portfolio.drawdown_pct(nav) < -limit {
                    tracing::warn!("Max drawdown halt triggered at bar {}", bar.ts_nanos);
                    break;
                }
            }
        }

        let metrics = compute(&self.nav_curve, &self.trade_pnls, self.cfg.bars_per_year, 0.05);
        BacktestResult {
            metrics,
            nav_curve: self.nav_curve.clone(),
            fills: self.fills.clone(),
        }
    }

    fn execute_signal(&mut self, signal: &Signal, bar: &Bar, adv_map: &[(String, f64)]) {
        let adv = adv_map.iter()
            .find(|(s, _)| *s == signal.symbol)
            .map(|(_, v)| *v)
            .unwrap_or(1e6);

        let nav = {
            let prices = [(signal.symbol.clone(), bar.close)];
            self.portfolio.nav(&prices)
        };

        let target_notional = nav * signal.size_frac;

        let (is_buy, qty) = match signal.direction {
            Direction::Long  => (true, target_notional / bar.close),
            Direction::Short => (false, target_notional / bar.close),
            Direction::Flat  => {
                // Close existing position
                if let Some(pos) = self.portfolio.positions.get(&signal.symbol) {
                    let existing = pos.quantity();
                    if existing.abs() < 1e-10 { return; }
                    let is_sell = existing > 0.0;
                    let fp = self.cfg.slippage.apply(bar.close, existing.abs(), adv, !is_sell);
                    let comm = self.cfg.commission.calculate(existing, fp);
                    let prev_entry = pos.avg_entry();
                    let pnl = existing.abs() * (fp - prev_entry) * if is_sell { 1.0 } else { -1.0 } - comm;
                    self.portfolio.apply_fill(&signal.symbol, -existing, fp, comm);
                    self.fills.push(Fill { ts_nanos: bar.ts_nanos, symbol: signal.symbol.clone(), qty: -existing, price: fp, commission: comm, pnl });
                    self.trade_pnls.push(pnl);
                }
                return;
            }
        };

        let fill_price = self.cfg.slippage.apply(bar.close, qty, adv, is_buy);
        let comm = self.cfg.commission.calculate(qty, fill_price);
        let signed_qty = if is_buy { qty } else { -qty };
        self.portfolio.apply_fill(&signal.symbol, signed_qty, fill_price, comm);
        self.fills.push(Fill { ts_nanos: bar.ts_nanos, symbol: signal.symbol.clone(), qty: signed_qty, price: fill_price, commission: comm, pnl: 0.0 });
    }

    /// Parallel parameter sweep: run same strategy factory over multiple param sets.
    /// Returns metrics for each param set. Uses Rayon for parallelism.
    pub fn parameter_sweep<F, S>(
        param_sets: Vec<(String, F)>,
        bars: &[Bar],
        adv_map: &[(String, f64)],
        initial_cash: f64,
        bars_per_year: f64,
    ) -> Vec<(String, BacktestMetrics)>
    where
        F: Fn() -> S + Send + Sync,
        S: Strategy,
    {
        param_sets
            .into_par_iter()
            .map(|(label, factory)| {
                let mut engine = BacktestEngine::new(EngineConfig {
                    initial_cash,
                    bars_per_year,
                    ..Default::default()
                });
                let mut strategy = factory();
                let result = engine.run(bars, &mut strategy, adv_map);
                (label, result.metrics)
            })
            .collect()
    }
}
