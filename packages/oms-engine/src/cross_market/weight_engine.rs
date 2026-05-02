use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use tracing::{info, warn, debug, error};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::regime_detection::{RegimeDetector, MarketRegime, RegimeBias, MarketState, RegimeDetectionResult};
use super::volatility_surface::{VolatilitySurface, VolatilityAdjustment};
use super::correlation_matrix::{CrossCorrelationMatrix, CorrelationAdjustment};
use super::liquidity_tracker::{LiquidityTracker, LiquidityAdjustment};
use super::time_decay::{TimeDecayModel, DecayAdjustment};

/// Weight engine parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightEngineParams {
    pub latency_target_us: u64,  // Target latency in microseconds
}

impl Default for WeightEngineParams {
    fn default() -> Self {
        Self {
            latency_target_us: 100,  // 100µs target
        }
    }
}

/// Weight vector for asset classes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightVector {
    pub equity_weight: f64,
    pub fixed_income_weight: f64,
    pub fx_weight: f64,
    pub crypto_weight: f64,
    pub commodity_weight: f64,
    pub timestamp: DateTime<Utc>,
}

impl Default for WeightVector {
    fn default() -> Self {
        Self {
            equity_weight: 0.4,
            fixed_income_weight: 0.25,
            fx_weight: 0.15,
            crypto_weight: 0.1,
            commodity_weight: 0.1,
            timestamp: Utc::now(),
        }
    }
}

/// Weight engine - sub-100µs dynamic weight recalculation
pub struct WeightEngine {
    regime_detector: RegimeDetector,
    volatility_surface: VolatilitySurface,
    correlation_matrix: CrossCorrelationMatrix,
    liquidity_tracker: LiquidityTracker,
    time_decay: TimeDecayModel,
    params: WeightEngineParams,
    last_calculation: Option<Instant>,
}

impl WeightEngine {
    pub fn new(params: WeightEngineParams) -> Self {
        Self {
            regime_detector: RegimeDetector::default(),
            volatility_surface: VolatilitySurface::default(),
            correlation_matrix: CrossCorrelationMatrix::default(),
            liquidity_tracker: LiquidityTracker::default(),
            time_decay: TimeDecayModel::default(),
            params,
            last_calculation: None,
        }
    }

    /// Recalculate weights with sub-100µs latency target
    pub async fn recalculate_weights(&mut self, market_state: &MarketState) -> WeightVector {
        let start = Instant::now();
        
        // Step 1: Detect regime (sub-50µs)
        let regime_result = self.regime_detector.detect(market_state);
        
        // Step 2: Get volatility adjustment (sub-10µs)
        let vol_adj = self.volatility_surface.get_volatility_adjustment();
        
        // Step 3: Get correlation adjustment (sub-25µs)
        let corr_adj = self.correlation_matrix.get_cross_adjustment();
        
        // Step 4: Get liquidity adjustment (sub-10µs)
        let liq_adj = self.liquidity_tracker.get_liquidity_adjustment();
        
        // Step 5: Get time decay adjustment (sub-5µs)
        let decay_adj = self.time_decay.get_decay_adjustment();
        
        // Step 6: Fuse weights (sub-10µs)
        let weights = self.fuse_weights(
            &regime_result.bias,
            &vol_adj,
            &corr_adj,
            &liq_adj,
            &decay_adj,
        );
        
        let duration = start.elapsed();
        self.last_calculation = Some(start);
        
        debug!("Weight recalculation completed in {:?}", duration);
        
        if duration.as_micros() as u64 > self.params.latency_target_us {
            warn!("Weight recalculation exceeded target: {:?}", duration);
        }
        
        weights
    }

    fn fuse_weights(
        &self,
        regime_bias: &RegimeBias,
        vol_adj: &VolatilityAdjustment,
        corr_adj: &CorrelationAdjustment,
        liq_adj: &LiquidityAdjustment,
        decay_adj: &DecayAdjustment,
    ) -> WeightVector {
        let equity_weight = 0.4 * regime_bias.equity_bias * vol_adj.equity * corr_adj.equity_bond * liq_adj.equity * decay_adj.equity;
        let fixed_income_weight = 0.25 * regime_bias.fi_bias * vol_adj.fixed_income * corr_adj.equity_fx * liq_adj.fixed_income * decay_adj.fixed_income;
        let fx_weight = 0.15 * regime_bias.fx_bias * vol_adj.fx * corr_adj.fx_commodity * liq_adj.fx * decay_adj.fx;
        let crypto_weight = 0.1 * regime_bias.crypto_bias * vol_adj.crypto * corr_adj.crypto_equity * liq_adj.crypto * decay_adj.crypto;
        let commodity_weight = 0.1 * regime_bias.commodity_bias * vol_adj.commodity * liq_adj.commodity * decay_adj.commodity;
        
        // Normalize to sum to 1.0
        let total = equity_weight + fixed_income_weight + fx_weight + crypto_weight + commodity_weight;
        
        WeightVector {
            equity_weight: equity_weight / total,
            fixed_income_weight: fixed_income_weight / total,
            fx_weight: fx_weight / total,
            crypto_weight: crypto_weight / total,
            commodity_weight: commodity_weight / total,
            timestamp: Utc::now(),
        }
    }

    /// Update market data for all components
    pub fn update_market_data(&mut self, asset: String, price: f64, volume: f64, depth: f64) {
        self.volatility_surface.update_volatility(asset.clone(), price, price * 0.99);
        self.correlation_matrix.update_price(asset.clone(), price);
        self.liquidity_tracker.update_liquidity(asset.clone(), volume, depth);
        self.time_decay.update_signal(asset, Utc::now());
    }

    /// Get last calculation duration
    pub fn last_calculation_duration(&self) -> Option<std::time::Duration> {
        self.last_calculation.map(|t| t.elapsed())
    }
}

impl Default for WeightEngine {
    fn default() -> Self {
        Self::new(WeightEngineParams::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_weight_engine_creation() {
        let engine = WeightEngine::new(WeightEngineParams::default());
        assert_eq!(engine.params.latency_target_us, 100);
    }

    #[tokio::test]
    async fn test_weight_recalculation() {
        let mut engine = WeightEngine::default();
        
        let market_state = MarketState {
            timestamp: Utc::now(),
            prices: vec![100.0, 101.0, 102.0, 103.0, 104.0],
            volumes: vec![1000.0, 1100.0, 1200.0, 1300.0, 1400.0],
            returns: vec![0.01, 0.01, 0.01, 0.01],
            volatility: 0.01,
            trend: 0.04,
        };
        
        let weights = engine.recalculate_weights(&market_state).await;
        
        assert!(weights.equity_weight > 0.0);
        assert!(weights.equity_weight + weights.fixed_income_weight + weights.fx_weight + weights.crypto_weight + weights.commodity_weight - 1.0 < 0.01);
    }
}
