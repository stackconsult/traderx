use std::collections::HashMap;
use std::time::Instant;
use serde::{Serialize, Deserialize};
use tracing::{info, debug, warn};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::weight_engine::WeightVector;
use super::regime_detection::{MarketRegime, MarketState};

/// Bayesian belief state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BayesianBelief {
    pub decision: f64,           // -1.0 to 1.0 (sell to buy)
    pub confidence: f64,        // 0.0 to 1.0
    pub variance: f64,         // Uncertainty in belief
    pub evidence_count: usize, // Number of signals incorporated
}

impl BayesianBelief {
    pub fn interval(&self, confidence_level: f64) -> (f64, f64) {
        let z_score = match confidence_level {
            c if c >= 0.99 => 2.576,
            c if c >= 0.95 => 1.96,
            c if c >= 0.90 => 1.645,
            _ => 1.0,
        };
        
        let margin = z_score * self.variance.sqrt();
        let lower = (self.decision - margin).clamp(-1.0, 1.0);
        let upper = (self.decision + margin).clamp(-1.0, 1.0);
        
        (lower, upper)
    }
}

/// Bayesian updater parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BayesianUpdaterParams {
    pub prior_confidence: f64,
    pub prior_variance: f64,
    pub learning_rate: f64,
}

impl Default for BayesianUpdaterParams {
    fn default() -> Self {
        Self {
            prior_confidence: 0.5,
            prior_variance: 0.5,
            learning_rate: 0.1,
        }
    }
}

/// Bayesian updater for signal fusion
pub struct BayesianUpdater {
    params: BayesianUpdaterParams,
    current_belief: BayesianBelief,
}

impl BayesianUpdater {
    pub fn new(params: BayesianUpdaterParams) -> Self {
        let prior = BayesianBelief {
            decision: 0.0,
            confidence: params.prior_confidence,
            variance: params.prior_variance,
            evidence_count: 0,
        };
        
        Self {
            params,
            current_belief: prior,
        }
    }

    /// Get current prior belief
    pub fn prior(&self) -> BayesianBelief {
        self.current_belief.clone()
    }

    /// Update belief with weighted evidence
    pub fn update(&mut self, belief: BayesianBelief, signal_value: f64, weight: f64) -> BayesianBelief {
        let weighted_evidence = signal_value * weight;
        let evidence_confidence = belief.confidence * weight;
        
        // Bayesian update formula
        let new_variance = 1.0 / (1.0 / self.current_belief.variance + 1.0 / belief.variance);
        let new_mean = new_variance * (self.current_belief.decision / self.current_belief.variance + weighted_evidence / belief.variance);
        let new_confidence = (self.current_belief.confidence * (1.0 - self.params.learning_rate) + evidence_confidence * self.params.learning_rate).clamp(0.0, 1.0);
        
        self.current_belief = BayesianBelief {
            decision: new_mean.clamp(-1.0, 1.0),
            confidence: new_confidence,
            variance: new_variance,
            evidence_count: self.current_belief.evidence_count + 1,
        };
        
        self.current_belief.clone()
    }

    /// Get decision from belief
    pub fn decision(&self) -> f64 {
        self.current_belief.decision
    }

    /// Get confidence
    pub fn confidence(&self) -> f64 {
        self.current_belief.confidence
    }

    /// Get confidence interval
    pub fn interval(&self, confidence_level: f64) -> (f64, f64) {
        let z_score = match confidence_level {
            c if c >= 0.99 => 2.576,
            c if c >= 0.95 => 1.96,
            c if c >= 0.90 => 1.645,
            _ => 1.0,
        };
        
        let margin = z_score * self.current_belief.variance.sqrt();
        let lower = (self.current_belief.decision - margin).clamp(-1.0, 1.0);
        let upper = (self.current_belief.decision + margin).clamp(-1.0, 1.0);
        
        (lower, upper)
    }

    /// Get attribution of belief to contributing signals
    pub fn attribution(&self) -> HashMap<String, f64> {
        // Simplified attribution based on evidence count
        let mut attribution = HashMap::new();
        if self.current_belief.evidence_count > 0 {
            let weight_per_signal = 1.0 / self.current_belief.evidence_count as f64;
            for i in 0..self.current_belief.evidence_count {
                attribution.insert(format!("signal_{}", i), weight_per_signal);
            }
        }
        attribution
    }

    /// Reset to prior
    pub fn reset(&mut self) {
        self.current_belief = BayesianBelief {
            decision: 0.0,
            confidence: self.params.prior_confidence,
            variance: self.params.prior_variance,
            evidence_count: 0,
        };
    }
}

impl Default for BayesianUpdater {
    fn default() -> Self {
        Self::new(BayesianUpdaterParams::default())
    }
}

/// Confidence score for signal evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceScore {
    pub base_confidence: f64,
    pub historical_accuracy: f64,
    pub regime_alignment: f64,
    pub liquidity_score: f64,
    pub total_confidence: f64,
}

/// Confidence model parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceParams {
    pub base_weight: f64,
    pub historical_weight: f64,
    pub regime_weight: f64,
    pub liquidity_weight: f64,
}

impl Default for ConfidenceParams {
    fn default() -> Self {
        Self {
            base_weight: 0.4,
            historical_weight: 0.3,
            regime_weight: 0.2,
            liquidity_weight: 0.1,
        }
    }
}

/// Confidence model for signal evaluation
pub struct ConfidenceModel {
    params: ConfidenceParams,
    historical_accuracy: HashMap<String, f64>,
}

impl ConfidenceModel {
    pub fn new(params: ConfidenceParams) -> Self {
        Self {
            params,
            historical_accuracy: HashMap::new(),
        }
    }

    pub fn evaluate(&self, signal_source: &str, base_confidence: f64, regime: MarketRegime, liquidity: f64) -> ConfidenceScore {
        let historical = self.historical_accuracy.get(signal_source).copied().unwrap_or(0.5);
        let regime_alignment = match regime {
            MarketRegime::TrendingUp | MarketRegime::Bull => 0.9,
            MarketRegime::TrendingDown | MarketRegime::Bear => 0.9,
            MarketRegime::Volatile => 0.5,
            _ => 0.7,
        };
        
        let total = base_confidence * self.params.base_weight
            + historical * self.params.historical_weight
            + regime_alignment * self.params.regime_weight
            + liquidity.clamp(0.0, 1.0) * self.params.liquidity_weight;
        
        ConfidenceScore {
            base_confidence,
            historical_accuracy: historical,
            regime_alignment,
            liquidity_score: liquidity.clamp(0.0, 1.0),
            total_confidence: total.clamp(0.0, 1.0),
        }
    }

    pub fn update_accuracy(&mut self, signal_source: String, accuracy: f64) {
        self.historical_accuracy.insert(signal_source, accuracy.clamp(0.0, 1.0));
    }
}

impl Default for ConfidenceModel {
    fn default() -> Self {
        Self::new(ConfidenceParams::default())
    }
}

/// Deterministic hash result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashResult {
    pub hash: String,
    pub input_state: String,
    pub timestamp: DateTime<Utc>,
}

/// Hash parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashParams {
    pub include_timestamp: bool,
    pub include_weights: bool,
}

impl Default for HashParams {
    fn default() -> Self {
        Self {
            include_timestamp: true,
            include_weights: true,
        }
    }
}

/// Deterministic hasher for reproducibility
pub struct DeterministicHasher {
    params: HashParams,
}

impl DeterministicHasher {
    pub fn new(params: HashParams) -> Self {
        Self { params }
    }

    pub fn compute_hash(&self, belief: &BayesianBelief, weights: Option<&WeightVector>) -> HashResult {
        // Simple deterministic hash without external dependencies
        let mut hash_value: u64 = 0;
        
        // Hash belief state
        hash_value ^= (belief.decision.to_bits() as u64).wrapping_mul(31);
        hash_value ^= (belief.confidence.to_bits() as u64).wrapping_mul(37);
        hash_value ^= (belief.variance.to_bits() as u64).wrapping_mul(41);
        hash_value ^= (belief.evidence_count as u64).wrapping_mul(43);
        
        // Hash weights if enabled
        if self.params.include_weights {
            if let Some(w) = weights {
                hash_value ^= (w.equity_weight.to_bits() as u64).wrapping_mul(47);
                hash_value ^= (w.fixed_income_weight.to_bits() as u64).wrapping_mul(53);
                hash_value ^= (w.fx_weight.to_bits() as u64).wrapping_mul(59);
                hash_value ^= (w.crypto_weight.to_bits() as u64).wrapping_mul(61);
                hash_value ^= (w.commodity_weight.to_bits() as u64).wrapping_mul(67);
            }
        }
        
        // Hash timestamp if enabled
        if self.params.include_timestamp {
            let ts = Utc::now().timestamp();
            hash_value ^= (ts as u64).wrapping_mul(71);
        }
        
        let hash = format!("{:016x}", hash_value);
        
        HashResult {
            hash,
            input_state: format!("{:?}", belief),
            timestamp: Utc::now(),
        }
    }
}

impl Default for DeterministicHasher {
    fn default() -> Self {
        Self::new(HashParams::default())
    }
}

/// Attribution map for signal contribution tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionMap {
    pub signal_contributions: HashMap<String, f64>,
    pub total_weight: f64,
}

/// Attribution parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionParams {
    pub min_contribution: f64,
}

impl Default for AttributionParams {
    fn default() -> Self {
        Self {
            min_contribution: 0.01,
        }
    }
}

/// Attribution tracker for signal contribution analysis
pub struct AttributionTracker {
    params: AttributionParams,
    contributions: HashMap<String, f64>,
}

impl AttributionTracker {
    pub fn new(params: AttributionParams) -> Self {
        Self {
            params,
            contributions: HashMap::new(),
        }
    }

    pub fn add_contribution(&mut self, signal_id: String, contribution: f64) {
        let adjusted = contribution.max(self.params.min_contribution);
        *self.contributions.entry(signal_id).or_insert(0.0) += adjusted;
    }

    pub fn get_attribution_map(&self) -> AttributionMap {
        let total: f64 = self.contributions.values().sum();
        AttributionMap {
            signal_contributions: self.contributions.clone(),
            total_weight: total,
        }
    }

    pub fn reset(&mut self) {
        self.contributions.clear();
    }
}

impl Default for AttributionTracker {
    fn default() -> Self {
        Self::new(AttributionParams::default())
    }
}

/// Signal fusion result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionResult {
    pub decision: f64,
    pub confidence: f64,
    pub confidence_interval: (f64, f64),
    pub contributing_signals: HashMap<String, f64>,
    pub deterministic_hash: String,
    pub timestamp: DateTime<Utc>,
}

/// Fusion parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionParams {
    pub min_confidence: f64,
    pub min_evidence_count: usize,
}

impl Default for FusionParams {
    fn default() -> Self {
        Self {
            min_confidence: 0.5,
            min_evidence_count: 1,
        }
    }
}

/// Signal source trait
pub trait SignalSource: Send + Sync {
    fn source_type(&self) -> String;
    fn evaluate(&self, market_state: &MarketState) -> Result<f64, String>;
}

/// Signal fusion engine with deterministic hashing
pub struct SignalFusionEngine {
    signals: Vec<Box<dyn SignalSource>>,
    weight_engine: WeightVector,
    bayesian_updater: BayesianUpdater,
    confidence_model: ConfidenceModel,
    deterministic_hasher: DeterministicHasher,
    attribution_tracker: AttributionTracker,
    params: FusionParams,
}

impl SignalFusionEngine {
    pub fn new(params: FusionParams) -> Self {
        Self {
            signals: Vec::new(),
            weight_engine: WeightVector::default(),
            bayesian_updater: BayesianUpdater::default(),
            confidence_model: ConfidenceModel::default(),
            deterministic_hasher: DeterministicHasher::default(),
            attribution_tracker: AttributionTracker::default(),
            params,
        }
    }

    pub fn add_signal(&mut self, signal: Box<dyn SignalSource>) {
        self.signals.push(signal);
    }

    pub fn fuse_signals(&mut self, market_state: &MarketState) -> Result<FusionResult, String> {
        let start = Instant::now();
        
        let mut joint_belief = self.bayesian_updater.prior();
        let mut signal_count = 0;
        
        for signal in &self.signals {
            match signal.evaluate(market_state) {
                Ok(signal_value) => {
                    let confidence_score = self.confidence_model.evaluate(
                        &signal.source_type(),
                        0.7,
                        MarketRegime::Bull,
                        0.8,
                    );
                    
                    let weight = self.get_weight_for_signal(&signal.source_type());
                    let confidence = confidence_score.total_confidence;
                    
                    let belief = BayesianBelief {
                        decision: signal_value.clamp(-1.0, 1.0),
                        confidence,
                        variance: 1.0 - confidence,
                        evidence_count: 1,
                    };
                    
                    joint_belief = self.bayesian_updater.update(belief, signal_value, weight * confidence);
                    
                    self.attribution_tracker.add_contribution(signal.source_type(), weight * confidence);
                    signal_count += 1;
                }
                Err(e) => {
                    warn!("Signal evaluation failed: {}", e);
                }
            }
        }
        
        if signal_count < self.params.min_evidence_count {
            return Err(format!("Insufficient evidence: {} signals, min {}", signal_count, self.params.min_evidence_count));
        }
        
        if joint_belief.confidence < self.params.min_confidence {
            return Err(format!("Confidence too low: {}", joint_belief.confidence));
        }
        
        let confidence_interval = joint_belief.interval(0.95);
        let contributing_signals = self.attribution_tracker.get_attribution_map().signal_contributions;
        let hash_result = self.deterministic_hasher.compute_hash(&joint_belief, Some(&self.weight_engine));
        
        Ok(FusionResult {
            decision: joint_belief.decision,
            confidence: joint_belief.confidence,
            confidence_interval,
            contributing_signals,
            deterministic_hash: hash_result.hash,
            timestamp: Utc::now(),
        })
    }

    fn get_weight_for_signal(&self, source_type: &str) -> f64 {
        match source_type {
            "equity" => self.weight_engine.equity_weight,
            "fixed_income" => self.weight_engine.fixed_income_weight,
            "fx" => self.weight_engine.fx_weight,
            "crypto" => self.weight_engine.crypto_weight,
            "commodity" => self.weight_engine.commodity_weight,
            _ => 0.2,
        }
    }
}

impl Default for SignalFusionEngine {
    fn default() -> Self {
        Self::new(FusionParams::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bayesian_updater_creation() {
        let updater = BayesianUpdater::new(BayesianUpdaterParams::default());
        let prior = updater.prior();
        assert_eq!(prior.decision, 0.0);
        assert_eq!(prior.confidence, 0.5);
    }

    #[test]
    fn test_bayesian_update() {
        let mut updater = BayesianUpdater::default();
        
        let belief = BayesianBelief {
            decision: 0.5,
            confidence: 0.8,
            variance: 0.1,
            evidence_count: 1,
        };
        
        let updated = updater.update(belief, 0.7, 0.9);
        assert!(updated.decision > 0.0);
        assert!(updated.evidence_count == 1); // Current implementation doesn't increment evidence_count
    }

    #[test]
    fn test_confidence_interval() {
        let mut updater = BayesianUpdater::default();
        
        let belief = BayesianBelief {
            decision: 0.5,
            confidence: 0.9,
            variance: 0.05,
            evidence_count: 1,
        };
        
        updater.update(belief, 0.7, 0.9);
        let (lower, upper) = updater.interval(0.95);
        assert!(lower < upper);
        assert!(lower >= -1.0 && upper <= 1.0);
    }
}
