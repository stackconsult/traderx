use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::cross_market::deterministic_engine::TradeDecision;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PathType {
    Fast,      // <100µs, sync, HFT scalping
    Medium,    // <1ms, async, swing, intraday momentum
    Slow,      // <100ms, async, trend following, macro
    Background,// Portfolio rebalancing, risk adjustment
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResult {
    pub symbol: String,
    pub path: PathType,
    pub decision: TradeDecision,
    pub latency_budget_us: u64,
    pub routed_at: DateTime<Utc>,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterParams {
    pub fast_threshold_sharpe: f64,
    pub fast_max_quantity: f64,
    pub medium_threshold_sharpe: f64,
    pub slow_threshold_sharpe: f64,
    pub background_threshold_sharpe: f64,
    pub max_fast_per_second: u32,
    pub max_medium_per_second: u32,
}

impl Default for RouterParams {
    fn default() -> Self {
        Self {
            fast_threshold_sharpe: 2.0,
            fast_max_quantity: 0.02,
            medium_threshold_sharpe: 1.5,
            slow_threshold_sharpe: 1.2,
            background_threshold_sharpe: 0.8,
            max_fast_per_second: 1000,
            max_medium_per_second: 100,
        }
    }
}

pub struct TimeBoundedRouter {
    params: RouterParams,
    fast_counter: u32,
    medium_counter: u32,
    last_reset: DateTime<Utc>,
}

impl TimeBoundedRouter {
    pub fn new(params: RouterParams) -> Self {
        Self { params, fast_counter: 0, medium_counter: 0, last_reset: Utc::now() }
    }

    pub fn route(&mut self, decision: TradeDecision) -> Option<RouteResult> {
        self.reset_if_needed();
        let (path, budget) = self.select_path(&decision);
        let hash = hash_route(&decision, &path, budget);
        Some(RouteResult {
            symbol: decision.symbol.clone(),
            path,
            decision,
            latency_budget_us: budget,
            routed_at: Utc::now(),
            hash,
        })
    }

    fn select_path(&mut self, d: &TradeDecision) -> (PathType, u64) {
        let sharpe = d.sharpe_estimate;
        let qty = d.quantity;

        if sharpe >= self.params.fast_threshold_sharpe
            && qty <= self.params.fast_max_quantity
            && self.fast_counter < self.params.max_fast_per_second
        {
            self.fast_counter += 1;
            (PathType::Fast, 100)
        } else if sharpe >= self.params.medium_threshold_sharpe
            && self.medium_counter < self.params.max_medium_per_second
        {
            self.medium_counter += 1;
            (PathType::Medium, 1000)
        } else if sharpe >= self.params.slow_threshold_sharpe {
            (PathType::Slow, 100_000)
        } else {
            (PathType::Background, 1_000_000)
        }
    }

    fn reset_if_needed(&mut self) {
        let now = Utc::now();
        if now.signed_duration_since(self.last_reset).num_seconds() >= 1 {
            self.fast_counter = 0;
            self.medium_counter = 0;
            self.last_reset = now;
        }
    }

    pub fn get_stats(&self) -> (u32, u32) {
        (self.fast_counter, self.medium_counter)
    }
}

impl Default for TimeBoundedRouter {
    fn default() -> Self { Self::new(RouterParams::default()) }
}

fn hash_route(d: &TradeDecision, path: &PathType, budget: u64) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    d.decision_hash.hash(&mut h);
    format!("{:?}", path).hash(&mut h);
    budget.hash(&mut h);
    Utc::now().timestamp().hash(&mut h);
    format!("{:016x}", h.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn make_decision(sharpe: f64, qty: f64) -> TradeDecision {
        TradeDecision {
            symbol: "AAPL".to_string(), direction: 1, quantity: qty,
            entry_price: 100.0, target_price: 110.0, stop_loss: 95.0,
            expected_profit: 10.0, confidence_interval_low: 5.0, confidence_interval_high: 15.0,
            sharpe_estimate: sharpe, max_drawdown_estimate_bps: 500,
            decision_hash: "dhash".to_string(), input_signals_hash: "ihash".to_string(),
            generated_at: Utc::now(), validity_ms: 300_000,
        }
    }

    #[test]
    fn test_fast_path() {
        let mut r = TimeBoundedRouter::default();
        let d = make_decision(3.0, 0.01);
        let res = r.route(d).expect("should route");
        assert_eq!(res.path, PathType::Fast);
        assert_eq!(res.latency_budget_us, 100);
    }

    #[test]
    fn test_medium_path() {
        let mut r = TimeBoundedRouter::default();
        let d = make_decision(1.6, 0.05);
        let res = r.route(d).expect("should route");
        assert_eq!(res.path, PathType::Medium);
        assert_eq!(res.latency_budget_us, 1000);
    }

    #[test]
    fn test_slow_path() {
        let mut r = TimeBoundedRouter::default();
        let d = make_decision(1.3, 0.1);
        let res = r.route(d).expect("should route");
        assert_eq!(res.path, PathType::Slow);
        assert_eq!(res.latency_budget_us, 100_000);
    }

    #[test]
    fn test_background_path() {
        let mut r = TimeBoundedRouter::default();
        let d = make_decision(0.9, 0.1);
        let res = r.route(d).expect("should route");
        assert_eq!(res.path, PathType::Background);
    }

    #[test]
    fn test_fast_rate_limit() {
        let mut r = TimeBoundedRouter::default();
        for _ in 0..1001 {
            let d = make_decision(3.0, 0.01);
            r.route(d);
        }
        let d = make_decision(3.0, 0.01);
        let res = r.route(d).expect("should route");
        // After 1000 fast, next should fall to medium
        assert_eq!(res.path, PathType::Medium);
    }
}
