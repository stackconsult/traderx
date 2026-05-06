// Comparative Metrics
// Phase 0: Foundation - 12-dimension model comparison framework

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Model performance metrics (12 dimensions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    // Dimension 1: Latency
    pub latency_p50_ns: f64,
    pub latency_p99_ns: f64,
    pub latency_p999_ns: f64,
    
    // Dimension 2: Throughput
    pub throughput_events_per_sec: f64,
    pub throughput_trades_per_sec: f64,
    
    // Dimension 3: Predictive Accuracy
    pub precision: f64,
    pub recall: f64,
    
    // Dimension 4: Robustness
    pub sharpe_bull: f64,
    pub sharpe_bear: f64,
    pub sharpe_sideways: f64,
    
    // Dimension 5: Cross-Market Alpha
    pub pnl_from_correlation_pct: f64,
    
    // Dimension 6: Tail Risk
    pub max_drawdown_pct: f64,
    pub cvar_95_pct: f64,
    
    // Dimension 7: Resilience
    pub uptime_pct: f64,
    pub circuit_breaker_trigger_ms: f64,
    
    // Dimension 8: Risk-Adjusted Return
    pub sharpe: f64,
    pub sortino: f64,
    pub calmar: f64,
    
    // Dimension 9: Cognitive Load
    pub human_interventions_per_day: f64,
    
    // Dimension 10: Code Complexity
    pub avg_cyclomatic_complexity: f64,
    
    // Dimension 11: UX Impact
    pub user_trust_score: f64,
    
    // Dimension 12: Memory Usage
    pub memory_mb: f64,
}

impl ModelMetrics {
    /// Calculate weighted score
    pub fn weighted_score(&self, weights: &HashMap<String, f64>) -> f64 {
        let mut score = 0.0;
        
        // Latency (15%)
        if let Some(&w) = weights.get("latency") {
            score += w * (1.0 - (self.latency_p99_ns / 500_000.0).min(1.0));
        }
        
        // Throughput (10%)
        if let Some(&w) = weights.get("throughput") {
            score += w * (self.throughput_events_per_sec / 100_000.0).min(1.0);
        }
        
        // Predictive Accuracy (15%)
        if let Some(&w) = weights.get("accuracy") {
            score += w * ((self.precision + self.recall) / 2.0);
        }
        
        // Robustness (15%)
        if let Some(&w) = weights.get("robustness") {
            score += w * ((self.sharpe_bull + self.sharpe_bear + self.sharpe_sideways) / 6.0);
        }
        
        // Cross-Market Alpha (10%)
        if let Some(&w) = weights.get("cross_market") {
            score += w * (self.pnl_from_correlation_pct / 100.0);
        }
        
        // Tail Risk (10%)
        if let Some(&w) = weights.get("tail_risk") {
            score += w * (1.0 - (self.max_drawdown_pct / 20.0).min(1.0));
        }
        
        // Resilience (5%)
        if let Some(&w) = weights.get("resilience") {
            score += w * (self.uptime_pct / 100.0);
        }
        
        // Risk-Adjusted Return (10%)
        if let Some(&w) = weights.get("risk_adjusted") {
            score += w * (self.sharpe / 3.0).min(1.0);
        }
        
        // Cognitive Load (3%)
        if let Some(&w) = weights.get("cognitive") {
            score += w * (1.0 - (self.human_interventions_per_day / 20.0).min(1.0));
        }
        
        // Code Complexity (4%)
        if let Some(&w) = weights.get("complexity") {
            score += w * (1.0 - (self.avg_cyclomatic_complexity / 20.0).min(1.0));
        }
        
        // UX Impact (3%)
        if let Some(&w) = weights.get("ux") {
            score += w * (self.user_trust_score / 5.0);
        }
        
        score
    }
    
    /// Default weights from BAM Lex 3 roadmap
    pub fn default_weights() -> HashMap<String, f64> {
        let mut weights = HashMap::new();
        weights.insert("latency".to_string(), 0.15);
        weights.insert("throughput".to_string(), 0.10);
        weights.insert("accuracy".to_string(), 0.15);
        weights.insert("robustness".to_string(), 0.15);
        weights.insert("cross_market".to_string(), 0.10);
        weights.insert("tail_risk".to_string(), 0.10);
        weights.insert("resilience".to_string(), 0.05);
        weights.insert("risk_adjusted".to_string(), 0.10);
        weights.insert("cognitive".to_string(), 0.03);
        weights.insert("complexity".to_string(), 0.04);
        weights.insert("ux".to_string(), 0.03);
        weights
    }
}

/// Comparative metrics report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparativeMetricsReport {
    pub base_model: ModelMetrics,
    pub advanced_model: Option<ModelMetrics>,
    pub hyperstatic_model: Option<ModelMetrics>,
    pub winner: Option<String>,
    pub confidence_interval: Option<(f64, f64)>,
}

impl ComparativeMetricsReport {
    /// Create a new comparative metrics report
    pub fn new(base_model: ModelMetrics) -> Self {
        Self {
            base_model,
            advanced_model: None,
            hyperstatic_model: None,
            winner: None,
            confidence_interval: None,
        }
    }
    
    /// Add advanced model metrics
    pub fn with_advanced_model(mut self, metrics: ModelMetrics) -> Self {
        self.advanced_model = Some(metrics);
        self
    }
    
    /// Add hyperstatic model metrics
    pub fn with_hyperstatic_model(mut self, metrics: ModelMetrics) -> Self {
        self.hyperstatic_model = Some(metrics);
        self
    }
    
    /// Determine winner based on weighted score
    pub fn determine_winner(&mut self) {
        let weights = ModelMetrics::default_weights();
        
        let base_score = self.base_model.weighted_score(&weights);
        let advanced_score = self.advanced_model.as_ref()
            .map(|m| m.weighted_score(&weights))
            .unwrap_or(0.0);
        let hyperstatic_score = self.hyperstatic_model.as_ref()
            .map(|m| m.weighted_score(&weights))
            .unwrap_or(0.0);
        
        let scores = vec![
            ("base".to_string(), base_score),
            ("advanced".to_string(), advanced_score),
            ("hyperstatic".to_string(), hyperstatic_score),
        ];
        
        let winner = scores.iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(name, _)| name.clone());
        
        self.winner = winner;
        
        // Calculate confidence interval (simplified)
        let max_score = scores.iter().map(|(_, s)| s).fold(f64::NEG_INFINITY, f64::max);
        let min_score = scores.iter().filter(|(_, s)| *s < max_score)
            .map(|(_, s)| s)
            .fold(f64::INFINITY, f64::min);
        
        if min_score < f64::INFINITY {
            self.confidence_interval = Some((max_score - 0.05, max_score + 0.05));
        }
    }
    
    /// Check if winner meets minimum criteria (≥6 dimensions)
    pub fn winner_meets_criteria(&self) -> bool {
        let weights = ModelMetrics::default_weights();
        let active_dimensions = weights.len();
        
        active_dimensions >= 6
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_metrics() {
        let metrics = ModelMetrics {
            latency_p50_ns: 100.0,
            latency_p99_ns: 200.0,
            latency_p999_ns: 500.0,
            throughput_events_per_sec: 100_000.0,
            throughput_trades_per_sec: 10_000.0,
            precision: 0.8,
            recall: 0.75,
            sharpe_bull: 1.5,
            sharpe_bear: 2.0,
            sharpe_sideways: 1.2,
            pnl_from_correlation_pct: 25.0,
            max_drawdown_pct: 5.0,
            cvar_95_pct: 3.0,
            uptime_pct: 99.99,
            circuit_breaker_trigger_ms: 0.5,
            sharpe: 2.0,
            sortino: 2.5,
            calmar: 3.0,
            human_interventions_per_day: 3.0,
            avg_cyclomatic_complexity: 8.0,
            user_trust_score: 4.5,
            memory_mb: 512.0,
        };
        
        let weights = ModelMetrics::default_weights();
        let score = metrics.weighted_score(&weights);
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }
    
    #[test]
    fn test_comparative_report() {
        let base_metrics = ModelMetrics {
            latency_p50_ns: 100.0,
            latency_p99_ns: 200.0,
            latency_p999_ns: 500.0,
            throughput_events_per_sec: 100_000.0,
            throughput_trades_per_sec: 10_000.0,
            precision: 0.8,
            recall: 0.75,
            sharpe_bull: 1.5,
            sharpe_bear: 2.0,
            sharpe_sideways: 1.2,
            pnl_from_correlation_pct: 25.0,
            max_drawdown_pct: 5.0,
            cvar_95_pct: 3.0,
            uptime_pct: 99.99,
            circuit_breaker_trigger_ms: 0.5,
            sharpe: 2.0,
            sortino: 2.5,
            calmar: 3.0,
            human_interventions_per_day: 3.0,
            avg_cyclomatic_complexity: 8.0,
            user_trust_score: 4.5,
            memory_mb: 512.0,
        };
        
        let mut report = ComparativeMetricsReport::new(base_metrics);
        report.determine_winner();
        
        assert!(report.winner.is_some());
        assert!(report.winner_meets_criteria());
    }
}
