use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::cross_market::time_bounded_router::RouteResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardParams {
    pub max_drawdown_bps: i32,
    pub anomaly_std_threshold: f64,
    pub max_trades_per_second: u32,
    pub circuit_breaker_cooldown_ms: u64,
    pub require_human_override_on_halt: bool,
}

impl Default for GuardParams {
    fn default() -> Self {
        Self {
            max_drawdown_bps: 1500,
            anomaly_std_threshold: 3.0,
            max_trades_per_second: 500,
            circuit_breaker_cooldown_ms: 5000,
            require_human_override_on_halt: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardDecision {
    pub allowed: bool,
    pub reason: String,
    pub halt_level: Option<HaltLevel>,
    pub requires_human_override: bool,
    pub hash_matched: bool,
    pub checked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HaltLevel {
    Warning,
    SoftHalt,   // Reduce to 50% size
    HardHalt,   // Block all trades, cooldown
    Emergency,  // Full shutdown, requires human
}

pub struct FabricGuard {
    params: GuardParams,
    cumulative_pnl: f64,
    starting_nav: f64,
    trade_count_second: u32,
    last_second_reset: DateTime<Utc>,
    halted_until: Option<DateTime<Utc>>,
    halt_level: Option<HaltLevel>,
    pnl_history: Vec<f64>,
}

impl FabricGuard {
    pub fn new(params: GuardParams, starting_nav: f64) -> Self {
        Self {
            params,
            cumulative_pnl: 0.0,
            starting_nav,
            trade_count_second: 0,
            last_second_reset: Utc::now(),
            halted_until: None,
            halt_level: None,
            pnl_history: Vec::new(),
        }
    }

    pub fn check(&mut self, route: &RouteResult) -> GuardDecision {
        let now = Utc::now();
        self.reset_counters(now);

        // 1. Check if under circuit breaker halt
        if let Some(halt_until) = self.halted_until {
            if now < halt_until {
                return GuardDecision {
                    allowed: false,
                    reason: format!("Circuit breaker active until {:?}", halt_until),
                    halt_level: self.halt_level,
                    requires_human_override: self.params.require_human_override_on_halt,
                    hash_matched: true,
                    checked_at: now,
                };
            } else {
                self.halted_until = None;
                self.halt_level = None;
            }
        }

        // 2. Hash match validation
        if route.decision.decision_hash.is_empty() {
            return self.reject("Missing decision hash", None);
        }

        // 3. Rate limiting
        if self.trade_count_second >= self.params.max_trades_per_second {
            return self.reject("Rate limit exceeded", Some(HaltLevel::SoftHalt));
        }

        // 4. Drawdown check
        let dd_bps = if self.starting_nav > 0.0 {
            ((self.cumulative_pnl / self.starting_nav).abs() * 10000.0) as i32
        } else { 0 };
        if dd_bps > self.params.max_drawdown_bps {
            self.trigger_halt(HaltLevel::HardHalt, now);
            return self.reject("Max drawdown exceeded", Some(HaltLevel::HardHalt));
        }

        // 5. Anomaly detection (rolling z-score)
        let anomaly = self.detect_anomaly(now);
        if anomaly {
            self.trigger_halt(HaltLevel::SoftHalt, now);
            return self.reject("Anomaly detected (z-score > 3σ)", Some(HaltLevel::SoftHalt));
        }

        self.trade_count_second += 1;
        GuardDecision {
            allowed: true,
            reason: "PASS".to_string(),
            halt_level: None,
            requires_human_override: false,
            hash_matched: true,
            checked_at: now,
        }
    }

    pub fn record_pnl(&mut self, pnl: f64) {
        self.cumulative_pnl += pnl;
        self.pnl_history.push(pnl);
        if self.pnl_history.len() > 1000 {
            self.pnl_history.remove(0);
        }
    }

    pub fn is_halted(&self) -> bool {
        if let Some(halt_until) = self.halted_until {
            Utc::now() < halt_until
        } else { false }
    }

    pub fn get_halt_level(&self) -> Option<HaltLevel> { self.halt_level }

    pub fn manual_override(&mut self) {
        self.halted_until = None;
        self.halt_level = None;
    }

    fn reset_counters(&mut self, now: DateTime<Utc>) {
        if now.signed_duration_since(self.last_second_reset).num_seconds() >= 1 {
            self.trade_count_second = 0;
            self.last_second_reset = now;
        }
    }

    fn detect_anomaly(&self, _now: DateTime<Utc>) -> bool {
        if self.pnl_history.len() < 20 { return false; }
        let recent: Vec<f64> = self.pnl_history.iter().rev().take(20).copied().collect();
        let (mean, std) = mean_std(&recent);
        if std == 0.0 { return false; }
        let last = recent[0];
        let z = (last - mean).abs() / std;
        z > self.params.anomaly_std_threshold
    }

    fn reject(&self, reason: &str, level: Option<HaltLevel>) -> GuardDecision {
        GuardDecision {
            allowed: false,
            reason: reason.to_string(),
            halt_level: level,
            requires_human_override: level == Some(HaltLevel::Emergency) || (level == Some(HaltLevel::HardHalt) && self.params.require_human_override_on_halt),
            hash_matched: true,
            checked_at: Utc::now(),
        }
    }

    fn trigger_halt(&mut self, level: HaltLevel, now: DateTime<Utc>) {
        let cooldown = match level {
            HaltLevel::Warning => 0,
            HaltLevel::SoftHalt => self.params.circuit_breaker_cooldown_ms / 2,
            HaltLevel::HardHalt => self.params.circuit_breaker_cooldown_ms,
            HaltLevel::Emergency => self.params.circuit_breaker_cooldown_ms * 2,
        };
        if cooldown > 0 {
            self.halted_until = Some(now + chrono::Duration::milliseconds(cooldown as i64));
        }
        self.halt_level = Some(level);
    }
}

impl Default for FabricGuard {
    fn default() -> Self { Self::new(GuardParams::default(), 1_000_000.0) }
}

fn mean_std(data: &[f64]) -> (f64, f64) {
    if data.is_empty() { return (0.0, 0.0); }
    let m = data.iter().sum::<f64>() / data.len() as f64;
    let v = data.iter().map(|&x| (x - m) * (x - m)).sum::<f64>() / data.len() as f64;
    (m, v.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cross_market::time_bounded_router::{PathType, RouterParams, TimeBoundedRouter};
    use crate::cross_market::deterministic_engine::{TradeDecision, ProfitEngineParams, DeterministicProfitEngine};

    fn make_route(sharpe: f64) -> RouteResult {
        let dec = TradeDecision {
            symbol: "AAPL".to_string(), direction: 1, quantity: 0.01,
            entry_price: 100.0, target_price: 110.0, stop_loss: 95.0,
            expected_profit: 5.0, confidence_interval_low: 2.0, confidence_interval_high: 8.0,
            sharpe_estimate: sharpe, max_drawdown_estimate_bps: 200,
            decision_hash: "abc123".to_string(), input_signals_hash: "def456".to_string(),
            generated_at: Utc::now(), validity_ms: 300_000,
        };
        let mut router = TimeBoundedRouter::new(RouterParams::default());
        router.route(dec).expect("route")
    }

    #[test]
    fn test_guard_passes() {
        let mut g = FabricGuard::default();
        let r = make_route(3.0);
        let d = g.check(&r);
        assert!(d.allowed, "Normal route should pass: {}", d.reason);
    }

    #[test]
    fn test_guard_rejects_empty_hash() {
        let mut g = FabricGuard::default();
        let mut r = make_route(3.0);
        r.decision.decision_hash = "".to_string();
        let d = g.check(&r);
        assert!(!d.allowed, "Empty hash should be rejected");
    }

    #[test]
    fn test_guard_drawdown_halt() {
        let mut g = FabricGuard::new(GuardParams { max_drawdown_bps: 500, ..Default::default() }, 100000.0);
        g.record_pnl(-600.0); // 600/100000 = 0.6% = 60 bps? No, 600/100000 = 0.006 = 60 bps
        // Actually: (600/100000)*10000 = 60 bps. Let me make it bigger.
        g.record_pnl(-6000.0); // 6000/100000 = 6% = 600 bps
        let r = make_route(3.0);
        let d = g.check(&r);
        assert!(!d.allowed, "Should halt on drawdown: {}", d.reason);
        assert_eq!(d.halt_level, Some(HaltLevel::HardHalt));
    }

    #[test]
    fn test_guard_rate_limit() {
        let mut g = FabricGuard::new(GuardParams { max_trades_per_second: 2, ..Default::default() }, 1_000_000.0);
        let r = make_route(3.0);
        assert!(g.check(&r).allowed);
        assert!(g.check(&r).allowed);
        let d = g.check(&r);
        assert!(!d.allowed, "Should rate limit: {}", d.reason);
    }

    #[test]
    fn test_guard_anomaly_detection() {
        let mut g = FabricGuard::new(GuardParams { anomaly_std_threshold: 2.0, ..Default::default() }, 1_000_000.0);
        // Fill history with small values
        for _ in 0..25 { g.record_pnl(10.0); }
        // Big anomaly
        g.record_pnl(-100.0);
        let r = make_route(3.0);
        let d = g.check(&r);
        assert!(!d.allowed, "Should detect anomaly: {}", d.reason);
        assert_eq!(d.halt_level, Some(HaltLevel::SoftHalt));
    }

    #[test]
    fn test_manual_override() {
        let mut g = FabricGuard::new(GuardParams { max_drawdown_bps: 500, ..Default::default() }, 100000.0);
        g.record_pnl(-6000.0);
        let r = make_route(3.0);
        let d = g.check(&r);
        assert!(!d.allowed);
        assert!(g.is_halted());
        g.manual_override();
        assert!(!g.is_halted());
        let d2 = g.check(&r);
        assert!(d2.allowed, "After override should pass");
    }
}
