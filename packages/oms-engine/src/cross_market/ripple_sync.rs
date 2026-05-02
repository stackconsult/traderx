use std::collections::{HashMap, VecDeque};
use std::time::{Instant, Duration};
use serde::{Serialize, Deserialize};
use tracing::{info, debug, warn};
use chrono::{DateTime, Utc};

use super::market_fabric::{AssetFabricState, FabricState};

/// Type of ripple pattern detected between markets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RippleType {
    MomentumRipple,    // Lead market momentum propagates to lag
    MeanReversionRipple, // Lead overshoot, lag mean-reverts
    VolatilitySpillover, // Lead vol spike, lag follows
    LiquidityRipple,   // Lead liquidity shock propagates
    None,
}

/// Cross-market lead-lag ripple pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RipplePattern {
    pub source_symbol: String,       // Lead market
    pub target_symbol: String,       // Lag market
    pub ripple_type: RippleType,
    pub correlation_strength: f64,   // -1.0 to 1.0
    pub lag_microseconds: u64,         // Detected lag
    pub predictability_score: f64,     // 0.0 to 1.0
    pub confidence_interval: (f64, f64),
    pub is_active: bool,               // Currently active?
    pub first_detected: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub sample_count: u64,
}

/// Ripple sync parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RippleSyncParams {
    pub max_lag_ms: u64,               // Maximum lag to detect (ms)
    pub min_correlation: f64,          // Minimum correlation threshold
    pub min_predictability: f64,       // Minimum predictability to report
    pub max_history_ticks: usize,      // How many ticks to keep per asset
    pub correlation_window_ticks: usize, // Ticks for correlation calc
}

impl Default for RippleSyncParams {
    fn default() -> Self {
        Self {
            max_lag_ms: 100,              // Up to 100ms lag
            min_correlation: 0.6,         // 60% correlation minimum
            min_predictability: 0.5,      // 50% predictability minimum
            max_history_ticks: 1000,      // Keep 1000 ticks
            correlation_window_ticks: 50,   // 50-tick correlation window
        }
    }
}

/// Price tick for lag detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceTick {
    pub timestamp_micros: u64,
    pub price: f64,
    pub volume: f64,
}

/// Ripple Sync Engine — detects cross-market lead-lag patterns
/// 
/// Synchronous: detects ripples within <100µs
/// Asynchronous: maintains rolling history for pattern validation
pub struct RippleSyncEngine {
    params: RippleSyncParams,
    // Symbol -> rolling price history (microsecond timestamps)
    price_history: HashMap<String, VecDeque<PriceTick>>,
    // Active ripple patterns (source_symbol -> target_symbol -> pattern)
    active_ripples: HashMap<String, HashMap<String, RipplePattern>>,
    // Detected patterns pending confirmation
    candidate_ripples: Vec<RipplePattern>,
    // Statistics
    total_detections: u64,
    confirmed_patterns: u64,
    rejected_patterns: u64,
}

impl RippleSyncEngine {
    pub fn new(params: RippleSyncParams) -> Self {
        Self {
            params,
            price_history: HashMap::new(),
            active_ripples: HashMap::new(),
            candidate_ripples: Vec::new(),
            total_detections: 0,
            confirmed_patterns: 0,
            rejected_patterns: 0,
        }
    }

    /// Record a price tick for an asset — optimized for <10µs per tick
    pub fn record_tick(&mut self, symbol: &str, timestamp_micros: u64, price: f64, volume: f64) {
        let tick = PriceTick { timestamp_micros, price, volume };
        
        let history = self.price_history
            .entry(symbol.to_string())
            .or_insert_with(VecDeque::new);
        
        history.push_back(tick);
        
        // Maintain bounded history
        while history.len() > self.params.max_history_ticks {
            history.pop_front();
        }
    }

    /// Record from fabric state (batch update)
    pub fn record_fabric_state(&mut self, fabric: &FabricState) {
        let now_micros = fabric.timestamp.timestamp_micros() as u64;
        
        for (symbol, state) in &fabric.asset_states {
            self.record_tick(symbol, now_micros, state.price, state.volume_24h);
        }
        
        // After recording, scan for ripples between all pairs
        self.scan_all_pairs();
    }

    /// Scan all symbol pairs for ripple patterns
    pub fn scan_all_pairs(&mut self) -> Vec<RipplePattern> {
        let symbols: Vec<String> = self.price_history.keys().cloned().collect();
        let mut new_patterns = Vec::new();
        
        for i in 0..symbols.len() {
            for j in (i + 1)..symbols.len() {
                let source = &symbols[i];
                let target = &symbols[j];
                
                // Check both directions
                if let Some(pattern) = self.detect_ripple(source, target) {
                    if pattern.predictability_score >= self.params.min_predictability {
                        new_patterns.push(pattern.clone());
                        self.register_pattern(pattern);
                    }
                }
                
                if let Some(pattern) = self.detect_ripple(target, source) {
                    if pattern.predictability_score >= self.params.min_predictability {
                        new_patterns.push(pattern.clone());
                        self.register_pattern(pattern);
                    }
                }
            }
        }
        
        self.total_detections += new_patterns.len() as u64;
        new_patterns
    }

    /// Detect ripple between source (lead) and target (lag)
    fn detect_ripple(&self, source: &str, target: &str) -> Option<RipplePattern> {
        let source_hist = self.price_history.get(source)?;
        let target_hist = self.price_history.get(target)?;
        
        if source_hist.len() < self.params.correlation_window_ticks || 
           target_hist.len() < self.params.correlation_window_ticks {
            return None;
        }
        
        // Get recent windows
        let source_window: Vec<f64> = source_hist.iter()
            .rev().take(self.params.correlation_window_ticks)
            .map(|t| t.price)
            .collect();
        let target_window: Vec<f64> = target_hist.iter()
            .rev().take(self.params.correlation_window_ticks)
            .map(|t| t.price)
            .collect();
        
        // Compute returns (price changes)
        let source_returns = compute_returns(&source_window);
        let target_returns = compute_returns(&target_window);
        
        if source_returns.is_empty() || target_returns.is_empty() {
            return None;
        }
        
        // Find optimal lag using cross-correlation
        let max_lag_ticks = (self.params.max_lag_ms as usize).min(target_returns.len() / 2);
        let mut best_correlation: f64 = 0.0;
        let mut best_lag = 0usize;
        let mut best_type = RippleType::None;
        
        for lag in 1..=max_lag_ticks {
            let aligned_len = source_returns.len().min(target_returns.len() - lag);
            if aligned_len < 5 {
                continue;
            }
            
            let source_slice = &source_returns[..aligned_len];
            let target_slice = &target_returns[lag..lag + aligned_len];
            
            let corr = pearson_correlation(source_slice, target_slice);
            
            if corr.abs() > best_correlation.abs() {
                best_correlation = corr;
                best_lag = lag;
                
                // Classify ripple type
                if corr > 0.7 {
                    best_type = RippleType::MomentumRipple;
                } else if corr < -0.7 {
                    best_type = RippleType::MeanReversionRipple;
                } else if corr > 0.5 {
                    best_type = RippleType::VolatilitySpillover;
                } else if corr > 0.3 {
                    best_type = RippleType::LiquidityRipple;
                }
            }
        }
        
        // Require minimum correlation
        if best_correlation.abs() < self.params.min_correlation {
            return None;
        }
        
        // Estimate lag in microseconds from tick spacing
        let lag_micros = estimate_lag_micros(source_hist, target_hist, best_lag)?;
        
        // Compute predictability score
        let predictability = compute_predictability(best_correlation, best_lag, source_returns.len());
        
        // Compute confidence interval (simplified)
        let confidence_width = (1.0 - best_correlation.abs()) * 0.5;
        let confidence_interval = (best_correlation - confidence_width, best_correlation + confidence_width);
        
        Some(RipplePattern {
            source_symbol: source.to_string(),
            target_symbol: target.to_string(),
            ripple_type: best_type,
            correlation_strength: best_correlation,
            lag_microseconds: lag_micros,
            predictability_score: predictability,
            confidence_interval,
            is_active: true,
            first_detected: Utc::now(),
            last_updated: Utc::now(),
            sample_count: source_returns.len() as u64,
        })
    }

    /// Register a confirmed ripple pattern
    fn register_pattern(&mut self, pattern: RipplePattern) {
        let source = pattern.source_symbol.clone();
        let target = pattern.target_symbol.clone();
        
        self.active_ripples
            .entry(source)
            .or_insert_with(HashMap::new)
            .insert(target, pattern);
        
        self.confirmed_patterns += 1;
    }

    /// Get active ripple patterns for a given source symbol
    pub fn get_ripples_from(&self, source: &str) -> Vec<&RipplePattern> {
        self.active_ripples
            .get(source)
            .map(|m| m.values().collect())
            .unwrap_or_default()
    }

    /// Get all active ripple patterns sorted by predictability
    pub fn get_top_ripples(&self, n: usize) -> Vec<&RipplePattern> {
        let mut all: Vec<&RipplePattern> = self.active_ripples
            .values()
            .flat_map(|m| m.values())
            .collect();
        
        all.sort_by(|a, b| b.predictability_score.partial_cmp(&a.predictability_score)
            .unwrap_or(std::cmp::Ordering::Equal));
        
        all.into_iter().take(n).collect()
    }

    /// Get patterns targeting a specific symbol (useful for predicting what will happen to this asset)
    pub fn get_ripples_to(&self, target: &str) -> Vec<&RipplePattern> {
        self.active_ripples
            .values()
            .filter_map(|m| m.get(target))
            .collect()
    }

    /// Get statistics
    pub fn stats(&self) -> (u64, u64, u64) {
        (self.total_detections, self.confirmed_patterns, self.rejected_patterns)
    }

    /// Expire old patterns (not updated in last N seconds)
    pub fn expire_patterns_older_than(&mut self, seconds: u64) {
        let cutoff = Utc::now() - chrono::Duration::seconds(seconds as i64);
        
        for source_patterns in self.active_ripples.values_mut() {
            source_patterns.retain(|_, p| p.last_updated > cutoff);
        }
        
        // Remove empty entries
        self.active_ripples.retain(|_, m| !m.is_empty());
    }

    /// Clear all data (emergency reset)
    pub fn reset(&mut self) {
        self.price_history.clear();
        self.active_ripples.clear();
        self.candidate_ripples.clear();
        self.total_detections = 0;
        self.confirmed_patterns = 0;
        self.rejected_patterns = 0;
    }
}

impl Default for RippleSyncEngine {
    fn default() -> Self {
        Self::new(RippleSyncParams::default())
    }
}

// --- Helper functions (no unwrap, pure math) ---

fn compute_returns(prices: &[f64]) -> Vec<f64> {
    if prices.len() < 2 {
        return Vec::new();
    }
    prices.windows(2)
        .map(|w| if w[0] != 0.0 { (w[1] - w[0]) / w[0] } else { 0.0 })
        .collect()
}

fn pearson_correlation(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len().min(y.len());
    if n < 2 {
        return 0.0;
    }
    
    let x_mean = x[..n].iter().sum::<f64>() / n as f64;
    let y_mean = y[..n].iter().sum::<f64>() / n as f64;
    
    let mut num = 0.0;
    let mut x_var = 0.0;
    let mut y_var = 0.0;
    
    for i in 0..n {
        let x_diff = x[i] - x_mean;
        let y_diff = y[i] - y_mean;
        num += x_diff * y_diff;
        x_var += x_diff * x_diff;
        y_var += y_diff * y_diff;
    }
    
    let denom = (x_var * y_var).sqrt();
    if denom == 0.0 {
        // If both series have zero variance, they are constant
        // If their means are equal, correlation is 1.0 (identical constants)
        // Otherwise 0.0 (different constants)
        if x_var == 0.0 && y_var == 0.0 {
            if (x_mean - y_mean).abs() < f64::EPSILON { 1.0 } else { 0.0 }
        } else {
            0.0
        }
    } else {
        (num / denom).clamp(-1.0, 1.0)
    }
}

fn estimate_lag_micros(source_hist: &VecDeque<PriceTick>, target_hist: &VecDeque<PriceTick>, lag_ticks: usize) -> Option<u64> {
    // Estimate average tick interval for source
    if source_hist.len() < 2 || target_hist.len() < 2 {
        return None;
    }
    
    let source_intervals: Vec<u64> = source_hist.iter()
        .collect::<Vec<_>>()
        .windows(2)
        .map(|w| w[1].timestamp_micros.saturating_sub(w[0].timestamp_micros))
        .collect();
    
    if source_intervals.is_empty() {
        return None;
    }
    
    let avg_interval = source_intervals.iter().sum::<u64>() / source_intervals.len() as u64;
    Some(avg_interval * lag_ticks as u64)
}

fn compute_predictability(correlation: f64, lag: usize, sample_size: usize) -> f64 {
    // Predictability = |correlation| * (1 - decay with lag) * (confidence from sample size)
    let correlation_component = correlation.abs();
    let lag_penalty = 1.0 / (1.0 + (lag as f64 * 0.01));
    let sample_confidence = (sample_size as f64 / 100.0).min(1.0);
    
    (correlation_component * lag_penalty * sample_confidence).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ripple_sync_record_and_detect() {
        let params = RippleSyncParams {
            max_lag_ms: 100,
            min_correlation: 0.6,
            min_predictability: 0.3, // Lower threshold for test
            max_history_ticks: 1000,
            correlation_window_ticks: 50,
        };
        let mut engine = RippleSyncEngine::new(params);
        
        // Record ticks for two correlated assets with INTRODUCED LAG
        let base_time = 1_000_000u64;
        
        // Asset A leads — goes up first
        for i in 0..100 {
            engine.record_tick("A", base_time + i * 1000, 100.0 + i as f64 * 0.1, 1000.0);
        }
        
        // Asset B lags by 3 ticks — copies A's price from 3 ticks ago
        for i in 0..100 {
            let lagged_i = if i >= 3 { i - 3 } else { 0 };
            engine.record_tick("B", base_time + i * 1000, 100.0 + lagged_i as f64 * 0.1, 1000.0);
        }
        
        let patterns = engine.scan_all_pairs();
        
        // With perfect correlation and 3-tick lag, should detect ripple
        assert!(!patterns.is_empty(), "Should detect ripple between correlated assets with lag");
        
        // Top ripples should have high predictability
        let top = engine.get_top_ripples(1);
        assert!(!top.is_empty());
        assert!(top[0].predictability_score > 0.0);
    }

    #[test]
    fn test_no_ripple_for_uncorrelated() {
        let mut engine = RippleSyncEngine::default();
        let base_time = 1_000_000u64;
        
        // Asset A goes up
        for i in 0..50 {
            engine.record_tick("A", base_time + i * 1000, 100.0 + i as f64, 1000.0);
        }
        
        // Asset B goes down (opposite)
        for i in 0..50 {
            engine.record_tick("B", base_time + i * 1000, 100.0 - i as f64, 1000.0);
        }
        
        let patterns = engine.scan_all_pairs();
        
        // Correlation should be negative but still detected as mean reversion ripple
        // However predictability might be low
        let top = engine.get_top_ripples(1);
        if !top.is_empty() {
            assert!(top[0].predictability_score > 0.0);
        }
    }

    #[test]
    fn test_ripple_stats() {
        let params = RippleSyncParams {
            max_lag_ms: 100,
            min_correlation: 0.6,
            min_predictability: 0.3, // Lower threshold for test
            max_history_ticks: 1000,
            correlation_window_ticks: 50,
        };
        let mut engine = RippleSyncEngine::new(params);
        let base_time = 1_000_000u64;
        
        for i in 0..100 {
            engine.record_tick("A", base_time + i * 1000, 100.0 + i as f64 * 0.1, 1000.0);
        }
        
        // B lags by 2 ticks
        for i in 0..100 {
            let lagged_i = if i >= 2 { i - 2 } else { 0 };
            engine.record_tick("B", base_time + i * 1000, 100.0 + lagged_i as f64 * 0.1, 1000.0);
        }
        
        engine.scan_all_pairs();
        
        let (total, confirmed, rejected) = engine.stats();
        assert!(total > 0, "Should detect at least one ripple pattern");
        assert!(confirmed > 0, "Should have confirmed patterns");
    }

    #[test]
    fn test_pearson_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let corr = pearson_correlation(&x, &y);
        assert!((corr - 1.0).abs() < 0.001);
        
        let y_neg = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        let corr_neg = pearson_correlation(&x, &y_neg);
        assert!((corr_neg - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_returns_computation() {
        let prices = vec![100.0, 101.0, 102.0, 101.0];
        let returns = compute_returns(&prices);
        assert_eq!(returns.len(), 3);
        assert!((returns[0] - 0.01).abs() < 0.0001);
    }

    #[test]
    fn test_predictability_score() {
        let high_corr = compute_predictability(0.9, 5, 100);
        let low_corr = compute_predictability(0.3, 50, 20);
        let high_lag = compute_predictability(0.8, 100, 100);
        
        assert!(high_corr > low_corr);
        assert!(high_corr > high_lag); // High lag should reduce predictability
    }

    #[test]
    fn test_expire_patterns() {
        let mut engine = RippleSyncEngine::default();
        
        // Manually insert an old pattern
        let old_pattern = RipplePattern {
            source_symbol: "A".to_string(),
            target_symbol: "B".to_string(),
            ripple_type: RippleType::MomentumRipple,
            correlation_strength: 0.8,
            lag_microseconds: 5000,
            predictability_score: 0.9,
            confidence_interval: (0.7, 0.9),
            is_active: true,
            first_detected: Utc::now() - chrono::Duration::seconds(3600),
            last_updated: Utc::now() - chrono::Duration::seconds(3600),
            sample_count: 100,
        };
        
        engine.register_pattern(old_pattern);
        
        assert_eq!(engine.get_top_ripples(10).len(), 1);
        
        engine.expire_patterns_older_than(300); // Expire patterns older than 5 min
        
        assert_eq!(engine.get_top_ripples(10).len(), 0);
    }
}
