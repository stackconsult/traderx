use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::cross_market::cross_layer_fusion::FusedSignal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfitEngineParams {
    pub min_fused_predictability: f64,
    pub min_sharpe_threshold: f64,
    pub max_drawdown_bps: i32,
    pub kelly_fraction: f64,
    pub hash_salt: String,
}

impl Default for ProfitEngineParams {
    fn default() -> Self {
        Self {
            min_fused_predictability: 0.55,
            min_sharpe_threshold: 1.2,
            max_drawdown_bps: 1500,
            kelly_fraction: 0.25,
            hash_salt: "TRADERX_V1".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeDecision {
    pub symbol: String,
    pub direction: i8, // +1 long, -1 short, 0 neutral
    pub quantity: f64,
    pub entry_price: f64,
    pub target_price: f64,
    pub stop_loss: f64,
    pub expected_profit: f64,
    pub confidence_interval_low: f64,
    pub confidence_interval_high: f64,
    pub sharpe_estimate: f64,
    pub max_drawdown_estimate_bps: i32,
    pub decision_hash: String,
    pub input_signals_hash: String,
    pub generated_at: DateTime<Utc>,
    pub validity_ms: u64,
}

pub struct DeterministicProfitEngine {
    params: ProfitEngineParams,
    decision_log: Vec<TradeDecision>,
}

impl DeterministicProfitEngine {
    pub fn new(params: ProfitEngineParams) -> Self {
        Self { params, decision_log: Vec::new() }
    }

    pub fn decide(&mut self, signals: &[FusedSignal]) -> Vec<TradeDecision> {
        let mut out = Vec::new();
        for sig in signals {
            if let Some(d) = self.evaluate(sig) {
                out.push(d);
            }
        }
        self.decision_log.extend(out.clone());
        out
    }

    fn evaluate(&self, sig: &FusedSignal) -> Option<TradeDecision> {
        if sig.fused_predictability < self.params.min_fused_predictability {
            return None;
        }

        let risk = (sig.entry_price - sig.stop_loss).abs();
        let reward = (sig.target_price - sig.entry_price).abs();
        if risk == 0.0 { return None; }
        let rr = reward / risk;

        // Bayesian profit estimate: E[P] = predictability * expected_return * direction
        let expected_profit = sig.fused_predictability * sig.expected_return * sig.direction;

        // Confidence interval: +/- 1.96 * SE where SE ~ (1-predictability) * expected_return
        let se = (1.0 - sig.fused_predictability) * sig.expected_return;
        let ci_low = expected_profit - 1.96 * se;
        let ci_high = expected_profit + 1.96 * se;

        // Sharpe estimate: E[R] / σ_R where σ_R ~ (1-p) * expected_return * 2 (approx)
        let sharpe = if se > 0.0 { expected_profit.abs() / (se * 2.0) } else { 0.0 };
        if sharpe < self.params.min_sharpe_threshold {
            return None;
        }

        // Drawdown estimate: max adverse excursion ~ 2 * risk / entry
        let dd_bps = ((2.0 * risk / sig.entry_price) * 10000.0) as i32;
        if dd_bps > self.params.max_drawdown_bps {
            return None;
        }

        // Kelly sizing: f* = (p*b - q) / b, then fractional
        let p_win = sig.fused_predictability;
        let b = rr;
        let q = 1.0 - p_win;
        let kelly = if b > 0.0 { (p_win * b - q) / b } else { 0.0 };
        let size_frac = kelly.max(0.0) * self.params.kelly_fraction;

        // Deterministic hashes
        let input_hash = hash_signal(sig, &self.params.hash_salt);
        let decision_hash = hash_decision(sig, size_frac, &self.params.hash_salt);

        Some(TradeDecision {
            symbol: sig.symbol.clone(),
            direction: sig.direction as i8,
            quantity: size_frac,
            entry_price: sig.entry_price,
            target_price: sig.target_price,
            stop_loss: sig.stop_loss,
            expected_profit,
            confidence_interval_low: ci_low,
            confidence_interval_high: ci_high,
            sharpe_estimate: sharpe,
            max_drawdown_estimate_bps: dd_bps,
            decision_hash,
            input_signals_hash: input_hash,
            generated_at: Utc::now(),
            validity_ms: 300_000, // 5 minutes
        })
    }

    pub fn decisions(&self) -> &[TradeDecision] { &self.decision_log }
    pub fn reset_log(&mut self) { self.decision_log.clear(); }
}

impl Default for DeterministicProfitEngine {
    fn default() -> Self { Self::new(ProfitEngineParams::default()) }
}

fn hash_signal(sig: &FusedSignal, salt: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    salt.hash(&mut h);
    sig.symbol.hash(&mut h);
    sig.fused_predictability.to_bits().hash(&mut h);
    sig.fused_confidence.to_bits().hash(&mut h);
    sig.direction.to_bits().hash(&mut h);
    sig.entry_price.to_bits().hash(&mut h);
    format!("{:016x}", h.finish())
}

fn hash_decision(sig: &FusedSignal, size: f64, salt: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    salt.hash(&mut h);
    sig.symbol.hash(&mut h);
    sig.direction.to_bits().hash(&mut h);
    size.to_bits().hash(&mut h);
    sig.entry_price.to_bits().hash(&mut h);
    Utc::now().timestamp().hash(&mut h);
    format!("{:016x}", h.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_signal(sym: &str, pred: f64, conf: f64, dir: f64, entry: f64, target: f64, stop: f64) -> FusedSignal {
        FusedSignal {
            symbol: sym.to_string(),
            fused_predictability: pred,
            fused_confidence: conf,
            direction: dir,
            expected_return: (target - entry).abs() / entry,
            entry_price: entry,
            target_price: target,
            stop_loss: stop,
            contributing_layers: vec![],
            hash: "test".to_string(),
            generated_at: Utc::now(),
        }
    }

    #[test]
    fn test_decision_long() {
        let params = ProfitEngineParams { min_fused_predictability: 0.5, min_sharpe_threshold: 0.5, ..Default::default() };
        let engine = DeterministicProfitEngine::new(params);
        let sig = make_signal("AAPL", 0.8, 0.85, 1.0, 100.0, 110.0, 95.0);
        let d = engine.evaluate(&sig).expect("should produce decision");
        assert_eq!(d.direction, 1);
        assert!(d.expected_profit > 0.0);
        assert!(d.sharpe_estimate > 0.5);
        assert!(!d.decision_hash.is_empty());
        assert!(!d.input_signals_hash.is_empty());
    }

    #[test]
    fn test_decision_filtered_by_predictability() {
        let params = ProfitEngineParams { min_fused_predictability: 0.9, ..Default::default() };
        let engine = DeterministicProfitEngine::new(params);
        let sig = make_signal("AAPL", 0.5, 0.6, 1.0, 100.0, 110.0, 95.0);
        assert!(engine.evaluate(&sig).is_none(), "Low predictability should be filtered");
    }

    #[test]
    fn test_decision_filtered_by_sharpe() {
        let params = ProfitEngineParams { min_fused_predictability: 0.5, min_sharpe_threshold: 5.0, ..Default::default() };
        let engine = DeterministicProfitEngine::new(params);
        let sig = make_signal("AAPL", 0.6, 0.7, 1.0, 100.0, 101.0, 99.0); // tiny RR
        assert!(engine.evaluate(&sig).is_none(), "Low sharpe should be filtered");
    }

    #[test]
    fn test_decision_hash_determinism() {
        let sig = make_signal("AAPL", 0.8, 0.85, 1.0, 100.0, 110.0, 95.0);
        let h1 = hash_signal(&sig, "salt");
        let h2 = hash_signal(&sig, "salt");
        assert_eq!(h1, h2, "Same inputs must produce same hash");
    }

    #[test]
    fn test_kelly_sizing_positive() {
        let params = ProfitEngineParams { min_fused_predictability: 0.5, min_sharpe_threshold: 0.1, ..Default::default() };
        let engine = DeterministicProfitEngine::new(params);
        let sig = make_signal("AAPL", 0.8, 0.9, 1.0, 100.0, 120.0, 90.0); // 2:1 RR
        let d = engine.evaluate(&sig).expect("should produce decision");
        assert!(d.quantity > 0.0, "Kelly sizing should be positive");
        assert!(d.quantity < 1.0, "Fractional Kelly should be < 100%");
    }
}
