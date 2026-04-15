//! Global Risk Bus — shared atomic state across OMS, signal router, and eBPF router.
//!
//! All reads are lock-free (Relaxed loads from AtomicBool/AtomicI32).
//! Writes happen only from the kill-switch agent (via gRPC) and the portfolio engine.
//! The eBPF router reads `global_halt` on every packet — must be zero-cost.

use dashmap::DashMap;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64, Ordering};
use std::sync::Arc;
use tracing::{error, info, warn, trace};

use crate::metrics::GLOBAL_RISK_METRICS;

/// Per-symbol position limit.
#[derive(Debug)]
pub struct PositionLimit {
    /// Maximum absolute notional in USD (× 1e4 fixed point).
    pub max_notional_fp: AtomicI64,
    /// Current gross notional (× 1e4 fixed point).
    pub current_notional_fp: AtomicI64,
}

/// Errors that can occur during position limit operations
#[derive(Debug, Clone, PartialEq)]
pub enum RiskError {
    PositionLimitExceeded,
    ConcurrentModification,
}

impl std::fmt::Display for RiskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskError::PositionLimitExceeded => write!(f, "Position limit would be exceeded"),
            RiskError::ConcurrentModification => write!(f, "Concurrent modification detected"),
        }
    }
}

impl std::error::Error for RiskError {}

impl PositionLimit {
    pub fn new(max_notional: f64) -> Self {
        Self {
            max_notional_fp: AtomicI64::new((max_notional * 1e4) as i64),
            current_notional_fp: AtomicI64::new(0),
        }
    }

    /// Returns true if adding `delta_notional` would breach the limit.
    /// Uses SeqCst ordering for consistency with check_and_update.
    #[inline]
    pub fn would_breach(&self, delta_notional: f64) -> bool {
        let current = self.current_notional_fp.load(Ordering::SeqCst) as f64 / 1e4;
        let max = self.max_notional_fp.load(Ordering::SeqCst) as f64 / 1e4;
        (current + delta_notional.abs()) > max
    }

    /// Atomically check if adding delta would breach limit AND update if it wouldn't.
    /// This prevents the TOCTOU race condition between check and update.
    /// 
    /// # Arguments
    /// * `delta_notional` - The change in position (can be positive or negative)
    /// 
    /// # Returns
    /// * `Ok(())` - Position was updated, within limit
    /// * `Err(RiskError::PositionLimitExceeded)` - Adding delta would exceed limit
    /// 
    /// # Example
    /// ```
    /// let limit = PositionLimit::new(1000.0);
    /// 
    /// // Thread-safe: concurrent calls are serialized via CAS
    /// let result = limit.check_and_update(100.0);
    /// assert!(result.is_ok());
    /// ```
    #[inline]
    pub fn check_and_update(&self, delta_notional: f64) -> Result<(), RiskError> {
        let delta_fp = (delta_notional * 1e4) as i64;
        
        loop {
            // Load current values with SeqCst for total ordering
            let current = self.current_notional_fp.load(Ordering::SeqCst);
            let max = self.max_notional_fp.load(Ordering::SeqCst);
            
            // Calculate new position
            let new_position = current + delta_fp;
            
            // Check if would breach
            if new_position.abs() > max {
                return Err(RiskError::PositionLimitExceeded);
            }
            
            // Attempt atomic update with compare-and-swap
            match self.current_notional_fp.compare_exchange(
                current,
                new_position,
                Ordering::SeqCst,  // Success ordering
                Ordering::SeqCst,  // Failure ordering
            ) {
                Ok(_) => return Ok(()),  // CAS succeeded, update complete
                Err(_) => continue,       // CAS failed, retry with new current value
            }
        }
    }

    /// Legacy update method - kept for backward compatibility
    /// Prefer check_and_update for new code to avoid race conditions
    #[inline]
    pub fn update(&self, delta_notional: f64) {
        let delta_fp = (delta_notional * 1e4) as i64;
        self.current_notional_fp.fetch_add(delta_fp, Ordering::SeqCst);
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
        let _timer = GLOBAL_RISK_METRICS.start_risk_check_timer();
        
        if self.kill_switch.load(Ordering::SeqCst) {
            GLOBAL_RISK_METRICS.record_risk_check_failure();
            trace!("Risk check failed: kill_switch active");
            return Err("kill_switch active");
        }
        if self.global_halt.load(Ordering::SeqCst) {
            GLOBAL_RISK_METRICS.record_risk_check_failure();
            trace!("Risk check failed: global_halt active");
            return Err("global_halt active");
        }
        if self.var_breach.load(Ordering::SeqCst) {
            GLOBAL_RISK_METRICS.record_risk_check_failure();
            trace!("Risk check failed: var_breach active");
            return Err("var_breach active");
        }
        let dd = self.portfolio_dd_bps.load(Ordering::SeqCst);
        if dd < self.dd_halt_threshold_bps {
            GLOBAL_RISK_METRICS.record_risk_check_failure();
            trace!("Risk check failed: drawdown limit breached");
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
        
        // Update metrics
        GLOBAL_RISK_METRICS.update_nav(nav_fp);

        // Update peak
        let old_peak = self.peak_nav_fp.load(Ordering::Relaxed);
        if nav_fp > old_peak {
            self.peak_nav_fp.store(nav_fp, Ordering::Relaxed);
        }

        let peak = self.peak_nav_fp.load(Ordering::Relaxed) as f64;
        let dd_bps = ((current_nav - peak / 1e4) / (peak / 1e4) * 10_000.0) as i32;
        self.portfolio_dd_bps.store(dd_bps, Ordering::Relaxed);
        
        // Update drawdown metrics
        GLOBAL_RISK_METRICS.update_drawdown(dd_bps as i64);

        if dd_bps < self.dd_halt_threshold_bps {
            let was_halted = self.global_halt.swap(true, Ordering::SeqCst);
            if !was_halted {
                error!(
                    "GLOBAL HALT triggered: drawdown {}bps exceeds limit {}bps",
                    dd_bps, self.dd_halt_threshold_bps
                );
            }
            // Update halt status metric
            GLOBAL_RISK_METRICS.update_halt_status(true);
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

    /// Record order submission - called by signal router
    #[inline]
    pub fn record_order_submitted(&self) {
        self.orders_submitted.fetch_add(1, Ordering::SeqCst);
        GLOBAL_RISK_METRICS.record_order_submitted();
        trace!("Order submitted metric updated");
    }

    /// Record order rejection - called by signal router
    #[inline]
    pub fn record_order_rejected(&self) {
        self.orders_rejected.fetch_add(1, Ordering::SeqCst);
        GLOBAL_RISK_METRICS.record_order_rejected();
        trace!("Order rejected metric updated");
    }

    /// Get order submission count
    pub fn orders_submitted_count(&self) -> i64 {
        self.orders_submitted.load(Ordering::SeqCst)
    }

    /// Get order rejection count
    pub fn orders_rejected_count(&self) -> i64 {
        self.orders_rejected.load(Ordering::SeqCst)
    }

    /// Update position utilization metric
    pub fn update_position_utilization(&self, ratio: f64) {
        GLOBAL_RISK_METRICS.update_position_utilization(ratio);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::Arc;

    /// Test: PositionLimit basic check_and_update
    #[test]
    fn test_position_limit_check_and_update() {
        let limit = PositionLimit::new(1000.0);
        
        // Add 500 - should succeed
        assert!(limit.check_and_update(500.0).is_ok());
        
        // Add another 400 - should succeed (900 total)
        assert!(limit.check_and_update(400.0).is_ok());
        
        // Add another 200 - should fail (would be 1100 > 1000)
        assert!(matches!(
            limit.check_and_update(200.0),
            Err(RiskError::PositionLimitExceeded)
        ));
    }

    /// Test: PositionLimit atomicity - the critical race condition fix
    /// This test verifies that concurrent updates are properly serialized
    /// and the position limit is never exceeded, even under contention.
    #[test]
    fn test_position_limit_atomicity() {
        let limit = Arc::new(PositionLimit::new(1000.0));
        let mut handles = vec![];
        
        // Spawn 100 threads, each trying to add 20 (potential total: 2000 > 1000)
        for _ in 0..100 {
            let limit_clone = Arc::clone(&limit);
            handles.push(thread::spawn(move || {
                // Each thread attempts to add 20
                let _ = limit_clone.check_and_update(20.0);
            }));
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify position never exceeded limit
        let final_position = limit.current_notional_fp.load(Ordering::SeqCst) as f64 / 1e4;
        assert!(
            final_position.abs() <= 1000.0,
            "Position {} exceeded limit 1000 - race condition detected!",
            final_position
        );
        
        // Verify some operations succeeded (not all failed due to contention)
        assert!(final_position > 0.0, "No operations succeeded");
    }

    /// Test: PositionLimit concurrent stress test
    /// High-contention scenario simulating real HFT load
    #[test]
    fn test_position_limit_concurrent_stress() {
        let limit = Arc::new(PositionLimit::new(10000.0));
        let iterations = 1000;
        let threads = 50;
        let mut handles = vec![];
        
        for _ in 0..threads {
            let limit_clone = Arc::clone(&limit);
            handles.push(thread::spawn(move || {
                for _ in 0..iterations {
                    // Alternate between adding and subtracting
                    let delta = if rand::random::<bool>() { 100.0 } else { -100.0 };
                    let _ = limit_clone.check_and_update(delta);
                }
            }));
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify position stayed within bounds
        let final_position = limit.current_notional_fp.load(Ordering::SeqCst) as f64 / 1e4;
        assert!(
            final_position.abs() <= 10000.0,
            "Position {} exceeded limit after stress test",
            final_position
        );
    }

    /// Test: PositionLimit would_breach consistency
    /// Ensures would_breach and check_and_update agree
    #[test]
    fn test_would_breach_consistency() {
        let limit = PositionLimit::new(1000.0);
        
        // Initially at 0, adding 1500 should breach
        assert!(limit.would_breach(1500.0));
        
        // Add 500
        assert!(limit.check_and_update(500.0).is_ok());
        
        // Now at 500, adding 600 should not breach (1100 > 1000 = breach)
        assert!(limit.would_breach(600.0));
        
        // But adding 400 should not breach (900 < 1000)
        assert!(!limit.would_breach(400.0));
    }

    /// Test: PositionLimit negative positions (short selling)
    #[test]
    fn test_position_limit_short_positions() {
        let limit = PositionLimit::new(1000.0);
        
        // Short 500
        assert!(limit.check_and_update(-500.0).is_ok());
        
        // Short another 400 (total -900)
        assert!(limit.check_and_update(-400.0).is_ok());
        
        // Short another 200 - should fail (would be -1100, abs = 1100 > 1000)
        assert!(matches!(
            limit.check_and_update(-200.0),
            Err(RiskError::PositionLimitExceeded)
        ));
    }

    /// Test: PositionLimit legacy update method (backward compatibility)
    #[test]
    fn test_position_limit_legacy_update() {
        let limit = PositionLimit::new(1000.0);
        
        // Use legacy update method
        limit.update(300.0);
        limit.update(200.0);
        
        let position = limit.current_notional_fp.load(Ordering::SeqCst) as f64 / 1e4;
        assert_eq!(position, 500.0);
    }
}
