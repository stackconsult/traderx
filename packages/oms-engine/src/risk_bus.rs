//! Global Risk Bus — shared atomic state across OMS, signal router, and eBPF router.
//!
//! All reads are lock-free (Relaxed loads from AtomicBool/AtomicI32).
//! Writes happen only from the kill-switch agent (via gRPC) and the portfolio engine.
//! The eBPF router reads `global_halt` on every packet — must be zero-cost.

use dashmap::DashMap;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64, Ordering};
use std::sync::Arc;
use tracing::{error, info, warn};

/// Per-symbol position limit.
#[derive(Debug)]
pub struct PositionLimit {
    /// Maximum absolute notional in USD (× 1e4 fixed point).
    pub max_notional_fp: AtomicI64,
    /// Current gross notional (× 1e4 fixed point).
    pub current_notional_fp: AtomicI64,
}

impl PositionLimit {
    pub fn new(max_notional: f64) -> Self {
        Self {
            max_notional_fp: AtomicI64::new((max_notional * 1e4) as i64),
            current_notional_fp: AtomicI64::new(0),
        }
    }

    /// Returns true if adding `delta_notional` would breach the limit.
    #[inline]
    pub fn would_breach(&self, delta_notional: f64) -> bool {
        let current = self.current_notional_fp.load(Ordering::Relaxed) as f64 / 1e4;
        let max = self.max_notional_fp.load(Ordering::Relaxed) as f64 / 1e4;
        (current + delta_notional.abs()) > max
    }

    #[inline]
    pub fn update(&self, delta_notional: f64) {
        let delta_fp = (delta_notional * 1e4) as i64;
        self.current_notional_fp.fetch_add(delta_fp, Ordering::Relaxed);
    }
}

/// Central risk bus — one instance, shared via `Arc<RiskBus>`.
pub struct RiskBus {
    /// Hard stop: if true, no new orders accepted by any component.
    pub global_halt: AtomicBool,

    /// Kill switch set by kill_switch_agent (Python → gRPC → here).
    pub kill_switch: AtomicBool,

    /// VaR limit breached flag.
    pub var_breach: AtomicBool,

    /// Current portfolio drawdown in basis points (negative = loss).
    pub portfolio_dd_bps: AtomicI32,

    /// Portfolio drawdown halt threshold in bps (e.g. -2000 = -20%).
    pub dd_halt_threshold_bps: i32,

    /// Peak NAV in USD × 1e4.
    pub peak_nav_fp: AtomicI64,

    /// Current NAV in USD × 1e4.
    pub current_nav_fp: AtomicI64,

    /// Per-symbol position limits.
    pub symbol_limits: DashMap<String, PositionLimit>,

    /// Total orders submitted this session.
    pub orders_submitted: AtomicI64,

    /// Total orders rejected by risk.
    pub orders_rejected: AtomicI64,
}

impl RiskBus {
    pub fn new(initial_cash: f64, dd_halt_threshold_bps: i32) -> Arc<Self> {
        let nav_fp = (initial_cash * 1e4) as i64;
        Arc::new(Self {
            global_halt: AtomicBool::new(false),
            kill_switch: AtomicBool::new(false),
            var_breach: AtomicBool::new(false),
            portfolio_dd_bps: AtomicI32::new(0),
            dd_halt_threshold_bps,
            peak_nav_fp: AtomicI64::new(nav_fp),
            current_nav_fp: AtomicI64::new(nav_fp),
            symbol_limits: DashMap::new(),
            orders_submitted: AtomicI64::new(0),
            orders_rejected: AtomicI64::new(0),
        })
    }

    /// Fast path — called on every signal before routing to OMS.
    /// Returns `Err` with reason if any halt condition is active.
    #[inline]
    pub fn check(&self) -> Result<(), &'static str> {
        if self.kill_switch.load(Ordering::SeqCst) {
            return Err("kill_switch active");
        }
        if self.global_halt.load(Ordering::SeqCst) {
            return Err("global_halt active");
        }
        if self.var_breach.load(Ordering::SeqCst) {
            return Err("var_breach active");
        }
        let dd = self.portfolio_dd_bps.load(Ordering::SeqCst);
        if dd < self.dd_halt_threshold_bps {
            return Err("drawdown limit breached");
        }
        Ok(())
    }

    /// Check per-symbol position limit.
    #[inline]
    pub fn check_symbol(&self, symbol: &str, delta_notional: f64) -> Result<(), &'static str> {
        if let Some(limit) = self.symbol_limits.get(symbol) {
            if limit.would_breach(delta_notional) {
                return Err("symbol position limit breached");
            }
        }
        Ok(())
    }

    /// Update NAV and recompute drawdown bps. Called by portfolio engine on every fill.
    pub fn update_nav(&self, current_nav: f64) {
        let nav_fp = (current_nav * 1e4) as i64;
        self.current_nav_fp.store(nav_fp, Ordering::Relaxed);

        // Update peak
        let old_peak = self.peak_nav_fp.load(Ordering::Relaxed);
        if nav_fp > old_peak {
            self.peak_nav_fp.store(nav_fp, Ordering::Relaxed);
        }

        let peak = self.peak_nav_fp.load(Ordering::Relaxed) as f64;
        let dd_bps = ((current_nav - peak / 1e4) / (peak / 1e4) * 10_000.0) as i32;
        self.portfolio_dd_bps.store(dd_bps, Ordering::Relaxed);

        if dd_bps < self.dd_halt_threshold_bps {
            let was_halted = self.global_halt.swap(true, Ordering::SeqCst);
            if !was_halted {
                error!(
                    "GLOBAL HALT triggered: drawdown {}bps exceeds limit {}bps",
                    dd_bps, self.dd_halt_threshold_bps
                );
            }
        }
    }

    /// Called by kill_switch_agent (via gRPC/Unix socket) to assert emergency stop.
    pub fn assert_kill_switch(&self, reason: &str) {
        self.kill_switch.store(true, Ordering::SeqCst);
        self.global_halt.store(true, Ordering::SeqCst);
        error!("KILL SWITCH ASSERTED: {}", reason);
    }

    /// Reset halt state (operator manual override only).
    pub fn reset_halt(&self) {
        warn!("Risk bus halt RESET by operator");
        self.global_halt.store(false, Ordering::SeqCst);
        self.kill_switch.store(false, Ordering::SeqCst);
        self.var_breach.store(false, Ordering::SeqCst);
    }

    pub fn set_symbol_limit(&self, symbol: &str, max_notional: f64) {
        self.symbol_limits
            .insert(symbol.to_owned(), PositionLimit::new(max_notional));
        info!("Position limit set: {} max_notional=${}", symbol, max_notional);
    }

    pub fn is_halted(&self) -> bool {
        self.global_halt.load(Ordering::SeqCst)
    }

    pub fn dd_bps(&self) -> i32 {
        self.portfolio_dd_bps.load(Ordering::SeqCst)
    }
}
