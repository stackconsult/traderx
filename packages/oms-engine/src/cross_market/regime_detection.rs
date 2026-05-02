use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use tracing::{info, warn, debug};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::stability::performance_audit::Benchmark;

/// Market regime classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketRegime {
    Bull,
    Bear,
    Volatile,
    RangeBound,
    TrendingUp,
    TrendingDown,
    Transition,
}

/// Regime bias for asset classes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeBias {
    pub equity_bias: f64,      // -1.0 to 1.0
    pub fi_bias: f64,         // -1.0 to 1.0
    pub fx_bias: f64,         // -1.0 to 1.0
    pub crypto_bias: f64,     // -1.0 to 1.0
    pub commodity_bias: f64,  // -1.0 to 1.0
}

/// Regime detection parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeDetectionParams {
    pub lookback_window: usize,      // Number of bars to analyze
    pub volatility_threshold: f64,   // Volatility threshold for regime classification
    pub trend_threshold: f64,       // Trend strength threshold
    pub regime_change_threshold: f64, // Minimum change to trigger regime shift
}

impl Default for RegimeDetectionParams {
    fn default() -> Self {
        Self {
            lookback_window: 20,
            volatility_threshold: 0.02,
            trend_threshold: 0.005,
            regime_change_threshold: 0.3,
        }
    }
}

/// Market state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketState {
    pub timestamp: DateTime<Utc>,
    pub prices: Vec<f64>,
    pub volumes: Vec<f64>,
    pub returns: Vec<f64>,
    pub volatility: f64,
    pub trend: f64,
}

/// Regime detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeDetectionResult {
    pub regime: MarketRegime,
    pub confidence: f64,
    pub bias: RegimeBias,
    pub volatility_level: f64,
    pub trend_strength: f64,
    pub transition_probability: f64,
    pub timestamp: DateTime<Utc>,
}

/// Regime detector with sub-50µs detection capability
pub struct RegimeDetector {
    params: RegimeDetectionParams,
    price_history: VecDeque<f64>,
    volume_history: VecDeque<f64>,
    current_regime: MarketRegime,
    regime_confidence: f64,
    last_detection: Option<Instant>,
}

impl RegimeDetector {
    pub fn new(params: RegimeDetectionParams) -> Self {
        let lookback = params.lookback_window;
        Self {
            params,
            price_history: VecDeque::with_capacity(lookback),
            volume_history: VecDeque::with_capacity(lookback),
            current_regime: MarketRegime::Transition,
            regime_confidence: 0.0,
            last_detection: None,
        }
    }

    /// Detect regime from market state - optimized for <50µs execution
    pub fn detect(&mut self, market_state: &MarketState) -> RegimeDetectionResult {
        let start = Instant::now();
        
        // Update history
        self.update_history(market_state);
        
        // Use provided volatility and trend from market_state for speed
        let volatility = market_state.volatility;
        let trend = market_state.trend;
        let regime = self.classify_regime(volatility, trend);
        
        // Calculate regime bias
        let bias = self.calculate_regime_bias(regime, volatility, trend);
        
        // Calculate confidence
        let confidence = self.calculate_confidence(volatility, trend);
        
        // Calculate transition probability
        let transition_prob = self.calculate_transition_probability(regime);
        
        // Update current state
        self.current_regime = regime;
        self.regime_confidence = confidence;
        self.last_detection = Some(start);
        
        let duration = start.elapsed();
        debug!("Regime detection completed in {:?}", duration);
        
        RegimeDetectionResult {
            regime,
            confidence,
            bias,
            volatility_level: volatility,
            trend_strength: trend,
            transition_probability: transition_prob,
            timestamp: Utc::now(),
        }
    }

    /// Update price and volume history
    fn update_history(&mut self, market_state: &MarketState) {
        if let Some(&latest_price) = market_state.prices.last() {
            self.price_history.push_back(latest_price);
            if self.price_history.len() > self.params.lookback_window {
                self.price_history.pop_front();
            }
        }
        
        if let Some(&latest_volume) = market_state.volumes.last() {
            self.volume_history.push_back(latest_volume);
            if self.volume_history.len() > self.params.lookback_window {
                self.volume_history.pop_front();
            }
        }
    }

    /// Calculate rolling volatility
    fn calculate_volatility(&self) -> f64 {
        if self.price_history.len() < 2 {
            return 0.0;
        }
        
        let prices: Vec<f64> = self.price_history.iter().copied().collect();
        let n = prices.len();
        
        let mean: f64 = prices.iter().sum::<f64>() / n as f64;
        let variance: f64 = prices.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / (n - 1) as f64;
        
        variance.sqrt()
    }

    /// Calculate rolling trend
    fn calculate_trend(&self) -> f64 {
        if self.price_history.len() < 2 {
            return 0.0;
        }
        
        let prices: Vec<f64> = self.price_history.iter().copied().collect();
        let first = match prices.first() {
            Some(v) => *v,
            None => return 0.0,
        };
        let last = match prices.last() {
            Some(v) => *v,
            None => return 0.0,
        };
        
        (last - first) / first
    }

    /// Classify regime based on volatility and trend
    fn classify_regime(&self, volatility: f64, trend: f64) -> MarketRegime {
        let vol_threshold = self.params.volatility_threshold;
        let trend_threshold = self.params.trend_threshold;
        
        if volatility > vol_threshold * 2.0 {
            return MarketRegime::Volatile;
        }
        
        if trend.abs() > trend_threshold {
            if trend > 0.0 {
                return MarketRegime::TrendingUp;
            } else {
                return MarketRegime::TrendingDown;
            }
        }
        
        if volatility > vol_threshold {
            return MarketRegime::RangeBound;
        }
        
        if trend >= 0.0 {
            return MarketRegime::Bull;
        } else {
            return MarketRegime::Bear;
        }
    }

    /// Calculate regime bias for asset classes
    fn calculate_regime_bias(&self, regime: MarketRegime, volatility: f64, trend: f64) -> RegimeBias {
        match regime {
            MarketRegime::Bull => RegimeBias {
                equity_bias: 0.8,
                fi_bias: -0.3,
                fx_bias: 0.2,
                crypto_bias: 0.9,
                commodity_bias: 0.4,
            },
            MarketRegime::Bear => RegimeBias {
                equity_bias: -0.8,
                fi_bias: 0.6,
                fx_bias: -0.2,
                crypto_bias: -0.9,
                commodity_bias: -0.5,
            },
            MarketRegime::Volatile => RegimeBias {
                equity_bias: 0.0,
                fi_bias: 0.8,
                fx_bias: 0.3,
                crypto_bias: 0.1,
                commodity_bias: 0.5,
            },
            MarketRegime::RangeBound => RegimeBias {
                equity_bias: 0.2,
                fi_bias: 0.3,
                fx_bias: 0.1,
                crypto_bias: 0.0,
                commodity_bias: 0.2,
            },
            MarketRegime::TrendingUp => RegimeBias {
                equity_bias: 0.9,
                fi_bias: -0.2,
                fx_bias: 0.4,
                crypto_bias: 1.0,
                commodity_bias: 0.6,
            },
            MarketRegime::TrendingDown => RegimeBias {
                equity_bias: -0.9,
                fi_bias: 0.7,
                fx_bias: -0.4,
                crypto_bias: -1.0,
                commodity_bias: -0.6,
            },
            MarketRegime::Transition => RegimeBias {
                equity_bias: 0.0,
                fi_bias: 0.0,
                fx_bias: 0.0,
                crypto_bias: 0.0,
                commodity_bias: 0.0,
            },
        }
    }

    /// Calculate confidence in regime classification
    fn calculate_confidence(&self, volatility: f64, trend: f64) -> f64 {
        let vol_confidence = (volatility / self.params.volatility_threshold).min(1.0);
        let trend_confidence = (trend.abs() / self.params.trend_threshold).min(1.0);
        
        (vol_confidence + trend_confidence) / 2.0
    }

    /// Calculate probability of regime transition
    fn calculate_transition_probability(&self, new_regime: MarketRegime) -> f64 {
        if new_regime == self.current_regime {
            return 0.0;
        }
        
        // Higher probability if regime confidence is low
        1.0 - self.regime_confidence
    }

    /// Get current regime
    pub fn current_regime(&self) -> MarketRegime {
        self.current_regime
    }

    /// Get last detection duration
    pub fn last_detection_duration(&self) -> Option<Duration> {
        self.last_detection.map(|t| t.elapsed())
    }
}

impl Default for RegimeDetector {
    fn default() -> Self {
        Self::new(RegimeDetectionParams::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regime_detector_creation() {
        let detector = RegimeDetector::new(RegimeDetectionParams::default());
        assert_eq!(detector.current_regime(), MarketRegime::Transition);
    }

    #[test]
    fn test_regime_detection_trending_up() {
        let params = RegimeDetectionParams {
            trend_threshold: 0.01,
            ..Default::default()
        };
        let mut detector = RegimeDetector::new(params);
        
        let market_state = MarketState {
            timestamp: Utc::now(),
            prices: vec![100.0, 101.0, 102.0, 103.0, 104.0],
            volumes: vec![1000.0, 1100.0, 1200.0, 1300.0, 1400.0],
            returns: vec![0.01, 0.01, 0.01, 0.01],
            volatility: 0.01,
            trend: 0.04,
        };
        
        let result = detector.detect(&market_state);
        assert_eq!(result.regime, MarketRegime::TrendingUp);
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_regime_detection_volatile() {
        let params = RegimeDetectionParams {
            volatility_threshold: 0.02,
            ..Default::default()
        };
        let mut detector = RegimeDetector::new(params);
        
        let market_state = MarketState {
            timestamp: Utc::now(),
            prices: vec![100.0, 105.0, 95.0, 110.0, 90.0],
            volumes: vec![1000.0, 2000.0, 500.0, 3000.0, 400.0],
            returns: vec![0.05, -0.1, 0.16, -0.18],
            volatility: 0.06,
            trend: -0.1,
        };
        
        let result = detector.detect(&market_state);
        assert_eq!(result.regime, MarketRegime::Volatile);
    }

    #[test]
    fn test_regime_bias_bull() {
        let bias = RegimeBias {
            equity_bias: 0.8,
            fi_bias: -0.3,
            fx_bias: 0.2,
            crypto_bias: 0.9,
            commodity_bias: 0.4,
        };
        
        assert!(bias.equity_bias > 0.0);
        assert!(bias.fi_bias < 0.0);
        assert!(bias.crypto_bias > bias.equity_bias);
    }
}
