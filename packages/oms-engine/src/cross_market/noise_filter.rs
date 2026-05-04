use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

use super::market_fabric::AssetFabricState;
use super::regime_detection::MarketRegime;

/// Type of noise detected in market data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoiseType {
    VolatilitySpike,
    Illiquidity,
    FlashCrash,
    FatFinger,
    Normal,
}

/// Recommended action after noise detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterAction {
    Ignore,
    ReduceSize,
    Pause,
    Proceed,
}

/// Result of noise filtering for a single asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseFilterResult {
    pub symbol: String,
    pub is_noise: bool,
    pub noise_type: NoiseType,
    pub confidence: f64,
    pub recommended_action: FilterAction,
    pub details: String,
}

/// Noise filter parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseFilterParams {
    pub volatility_threshold: f64, // Short vol > threshold * long vol = spike
    pub illiquidity_spread_bps: f64, // Spread > threshold = illiquid
    pub flash_crash_price_change: f64, // Price change > threshold in single tick
    pub fat_finger_sigma: f64,     // Single tick move > N sigma
    pub min_liquidity_score: f64,  // Minimum liquidity to proceed
}

impl Default for NoiseFilterParams {
    fn default() -> Self {
        Self {
            volatility_threshold: 3.0,      // 3x normal vol = spike
            illiquidity_spread_bps: 50.0,   // 50 bps spread = illiquid
            flash_crash_price_change: 0.05, // 5% single-tick move = flash crash
            fat_finger_sigma: 5.0,          // 5 sigma = fat finger
            min_liquidity_score: 0.3,       // <30% liquidity = pause
        }
    }
}

/// Noise filter — real-time discrimination of market noise vs signal
///
/// What to IGNORE due to volatility or illiquidity:
/// - Volatility spikes (>3x normal)
/// - Illiquidity events (>50bps spread or <30% liquidity score)
/// - Flash crashes (>5% single-tick moves)
/// - Fat-finger errors (>5 sigma single-tick moves)
pub struct NoiseFilter {
    params: NoiseFilterParams,
    historical_volatility: HashMap<String, Vec<f64>>,
    historical_spread: HashMap<String, Vec<f64>>,
    last_prices: HashMap<String, f64>,
    filter_stats: FilterStats,
}

/// Filter statistics for monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilterStats {
    pub total_checks: u64,
    pub noise_detected: u64,
    pub volatility_spikes: u64,
    pub illiquidity_events: u64,
    pub flash_crashes: u64,
    pub fat_fingers: u64,
    pub signals_passed: u64,
}

impl NoiseFilter {
    pub fn new(params: NoiseFilterParams) -> Self {
        Self {
            params,
            historical_volatility: HashMap::new(),
            historical_spread: HashMap::new(),
            last_prices: HashMap::new(),
            filter_stats: FilterStats::default(),
        }
    }

    /// Filter a single asset — optimized for <10µs
    pub fn filter(&mut self, asset: &AssetFabricState) -> NoiseFilterResult {
        let _start = Instant::now();
        self.filter_stats.total_checks += 1;

        // Check 1: Volatility spike
        let vol_result = self.check_volatility_spike(asset);
        if vol_result.is_noise {
            self.filter_stats.noise_detected += 1;
            self.filter_stats.volatility_spikes += 1;
            return vol_result;
        }

        // Check 2: Illiquidity
        let liq_result = self.check_illiquidity(asset);
        if liq_result.is_noise {
            self.filter_stats.noise_detected += 1;
            self.filter_stats.illiquidity_events += 1;
            return liq_result;
        }

        // Check 3: Flash crash (requires price history)
        let crash_result = self.check_flash_crash(asset);
        if crash_result.is_noise {
            self.filter_stats.noise_detected += 1;
            self.filter_stats.flash_crashes += 1;
            return crash_result;
        }

        // Check 4: Fat finger (requires price history)
        let ff_result = self.check_fat_finger(asset);
        if ff_result.is_noise {
            self.filter_stats.noise_detected += 1;
            self.filter_stats.fat_fingers += 1;
            return ff_result;
        }

        // Update historical data
        self.update_history(asset);

        self.filter_stats.signals_passed += 1;

        NoiseFilterResult {
            symbol: asset.symbol.clone(),
            is_noise: false,
            noise_type: NoiseType::Normal,
            confidence: 1.0,
            recommended_action: FilterAction::Proceed,
            details: "Clean signal — all checks passed".to_string(),
        }
    }

    /// Filter multiple assets in batch
    pub fn filter_batch(&mut self, assets: &[AssetFabricState]) -> Vec<NoiseFilterResult> {
        assets.iter().map(|a| self.filter(a)).collect()
    }

    /// Get clean assets only (those that passed filtering) — returns cloned owned values
    pub fn get_clean_assets(&mut self, assets: &[AssetFabricState]) -> Vec<AssetFabricState> {
        assets
            .iter()
            .filter(|a| !self.filter(a).is_noise)
            .cloned()
            .collect()
    }

    /// Get filter statistics
    pub fn stats(&self) -> &FilterStats {
        &self.filter_stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.filter_stats = FilterStats::default();
    }

    // --- Private check methods ---

    fn check_volatility_spike(&self, asset: &AssetFabricState) -> NoiseFilterResult {
        let vol_ratio = if asset.volatility_long > 0.0 {
            asset.volatility_short / asset.volatility_long
        } else {
            asset.volatility_short
        };

        if vol_ratio > self.params.volatility_threshold {
            let confidence = (vol_ratio / self.params.volatility_threshold).min(1.0);
            return NoiseFilterResult {
                symbol: asset.symbol.clone(),
                is_noise: true,
                noise_type: NoiseType::VolatilitySpike,
                confidence,
                recommended_action: FilterAction::Ignore,
                details: format!(
                    "Volatility spike: short={:.4} vs long={:.4} (ratio={:.2}x, threshold={:.1}x)",
                    asset.volatility_short,
                    asset.volatility_long,
                    vol_ratio,
                    self.params.volatility_threshold
                ),
            };
        }

        NoiseFilterResult {
            symbol: asset.symbol.clone(),
            is_noise: false,
            noise_type: NoiseType::Normal,
            confidence: 0.0,
            recommended_action: FilterAction::Proceed,
            details: String::new(),
        }
    }

    fn check_illiquidity(&self, asset: &AssetFabricState) -> NoiseFilterResult {
        // Check spread
        if asset.spread_bps > self.params.illiquidity_spread_bps {
            let confidence = (asset.spread_bps / self.params.illiquidity_spread_bps).min(1.0);
            return NoiseFilterResult {
                symbol: asset.symbol.clone(),
                is_noise: true,
                noise_type: NoiseType::Illiquidity,
                confidence,
                recommended_action: FilterAction::Pause,
                details: format!(
                    "Illiquidity: spread={:.1}bps > threshold={:.1}bps",
                    asset.spread_bps, self.params.illiquidity_spread_bps
                ),
            };
        }

        // Check liquidity score
        if asset.liquidity_score < self.params.min_liquidity_score {
            let confidence = 1.0 - asset.liquidity_score;
            return NoiseFilterResult {
                symbol: asset.symbol.clone(),
                is_noise: true,
                noise_type: NoiseType::Illiquidity,
                confidence,
                recommended_action: FilterAction::Pause,
                details: format!(
                    "Low liquidity score: {:.2} < threshold={:.2}",
                    asset.liquidity_score, self.params.min_liquidity_score
                ),
            };
        }

        NoiseFilterResult {
            symbol: asset.symbol.clone(),
            is_noise: false,
            noise_type: NoiseType::Normal,
            confidence: 0.0,
            recommended_action: FilterAction::Proceed,
            details: String::new(),
        }
    }

    fn check_flash_crash(&self, asset: &AssetFabricState) -> NoiseFilterResult {
        if let Some(last_price) = self.last_prices.get(&asset.symbol) {
            if *last_price > 0.0 {
                let price_change = (asset.price - last_price).abs() / *last_price;
                if price_change > self.params.flash_crash_price_change {
                    let confidence = (price_change / self.params.flash_crash_price_change).min(1.0);
                    return NoiseFilterResult {
                        symbol: asset.symbol.clone(),
                        is_noise: true,
                        noise_type: NoiseType::FlashCrash,
                        confidence,
                        recommended_action: FilterAction::Pause,
                        details: format!(
                            "Flash crash: price change={:.2}% > threshold={:.1}%",
                            price_change * 100.0,
                            self.params.flash_crash_price_change * 100.0
                        ),
                    };
                }
            }
        }

        NoiseFilterResult {
            symbol: asset.symbol.clone(),
            is_noise: false,
            noise_type: NoiseType::Normal,
            confidence: 0.0,
            recommended_action: FilterAction::Proceed,
            details: String::new(),
        }
    }

    fn check_fat_finger(&self, asset: &AssetFabricState) -> NoiseFilterResult {
        // Simple fat-finger check: if price change is > 5 sigma from recent history
        if let Some(vol_history) = self.historical_volatility.get(&asset.symbol) {
            if vol_history.len() >= 10 {
                let recent_vols: Vec<f64> = vol_history.iter().rev().take(20).copied().collect();
                let mean = recent_vols.iter().sum::<f64>() / recent_vols.len() as f64;
                let variance = recent_vols.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
                    / recent_vols.len() as f64;
                let std_dev = variance.sqrt();

                if std_dev > 0.0 {
                    let z_score = (asset.volatility_short - mean).abs() / std_dev;
                    if z_score > self.params.fat_finger_sigma {
                        let confidence = (z_score / self.params.fat_finger_sigma).min(1.0);
                        return NoiseFilterResult {
                            symbol: asset.symbol.clone(),
                            is_noise: true,
                            noise_type: NoiseType::FatFinger,
                            confidence,
                            recommended_action: FilterAction::Ignore,
                            details: format!(
                                "Fat finger: z-score={:.2} > threshold={:.1} sigma",
                                z_score, self.params.fat_finger_sigma
                            ),
                        };
                    }
                }
            }
        }

        NoiseFilterResult {
            symbol: asset.symbol.clone(),
            is_noise: false,
            noise_type: NoiseType::Normal,
            confidence: 0.0,
            recommended_action: FilterAction::Proceed,
            details: String::new(),
        }
    }

    fn update_history(&mut self, asset: &AssetFabricState) {
        self.last_prices.insert(asset.symbol.clone(), asset.price);

        self.historical_volatility
            .entry(asset.symbol.clone())
            .or_insert_with(Vec::new)
            .push(asset.volatility_short);

        // Keep only last 100 values
        if let Some(vols) = self.historical_volatility.get_mut(&asset.symbol) {
            if vols.len() > 100 {
                vols.remove(0);
            }
        }

        self.historical_spread
            .entry(asset.symbol.clone())
            .or_insert_with(Vec::new)
            .push(asset.spread_bps);

        if let Some(spreads) = self.historical_spread.get_mut(&asset.symbol) {
            if spreads.len() > 100 {
                spreads.remove(0);
            }
        }
    }
}

impl Default for NoiseFilter {
    fn default() -> Self {
        Self::new(NoiseFilterParams::default())
    }
}

#[cfg(test)]
mod tests {
    use super::super::market_fabric::AssetFabricState;
    use super::*;

    fn create_test_asset(
        symbol: &str,
        price: f64,
        vol_short: f64,
        vol_long: f64,
        spread: f64,
        liq: f64,
    ) -> AssetFabricState {
        AssetFabricState {
            symbol: symbol.to_string(),
            price,
            volume_24h: 1_000_000.0,
            volatility_short: vol_short,
            volatility_long: vol_long,
            liquidity_score: liq,
            spread_bps: spread,
            regime: MarketRegime::TrendingUp,
            signal_quality: 0.8,
            predictability_score: 0.7,
            bam_address: 0,
        }
    }

    #[test]
    fn test_volatility_spike_detection() {
        let mut filter = NoiseFilter::default();

        // Normal volatility
        let normal = create_test_asset("AAPL", 150.0, 0.01, 0.15, 5.0, 0.9);
        let result = filter.filter(&normal);
        assert!(!result.is_noise);

        // Volatility spike (short vol > 3x long vol)
        let spike = create_test_asset("AAPL", 150.0, 0.5, 0.15, 5.0, 0.9);
        let result = filter.filter(&spike);
        assert!(result.is_noise);
        assert_eq!(result.noise_type, NoiseType::VolatilitySpike);
    }

    #[test]
    fn test_illiquidity_detection() {
        let mut filter = NoiseFilter::default();

        // Normal liquidity
        let normal = create_test_asset("AAPL", 150.0, 0.01, 0.15, 5.0, 0.9);
        let result = filter.filter(&normal);
        assert!(!result.is_noise);

        // High spread
        let illiquid = create_test_asset("PENNY", 0.05, 0.01, 0.15, 100.0, 0.2);
        let result = filter.filter(&illiquid);
        assert!(result.is_noise);
        assert_eq!(result.noise_type, NoiseType::Illiquidity);

        // Low liquidity score
        let low_liq = create_test_asset("AAPL", 150.0, 0.01, 0.15, 5.0, 0.1);
        let result = filter.filter(&low_liq);
        assert!(result.is_noise);
        assert_eq!(result.noise_type, NoiseType::Illiquidity);
    }

    #[test]
    fn test_flash_crash_detection() {
        let mut filter = NoiseFilter::default();

        // First update establishes price history
        let normal = create_test_asset("AAPL", 150.0, 0.01, 0.15, 5.0, 0.9);
        let _ = filter.filter(&normal);

        // Flash crash (>5% drop)
        let crash = create_test_asset("AAPL", 135.0, 0.01, 0.15, 5.0, 0.9);
        let result = filter.filter(&crash);
        assert!(result.is_noise);
        assert_eq!(result.noise_type, NoiseType::FlashCrash);
    }

    #[test]
    fn test_filter_stats() {
        let mut filter = NoiseFilter::default();

        let normal = create_test_asset("AAPL", 150.0, 0.01, 0.15, 5.0, 0.9);
        let spike = create_test_asset("VIX", 20.0, 0.5, 0.15, 5.0, 0.9);

        filter.filter(&normal);
        filter.filter(&spike);

        let stats = filter.stats();
        assert_eq!(stats.total_checks, 2);
        assert_eq!(stats.noise_detected, 1);
        assert_eq!(stats.volatility_spikes, 1);
        assert_eq!(stats.signals_passed, 1);
    }

    #[test]
    fn test_filter_batch() {
        let mut filter = NoiseFilter::default();

        let assets = vec![
            create_test_asset("AAPL", 150.0, 0.01, 0.15, 5.0, 0.9),
            create_test_asset("VIX", 20.0, 0.5, 0.15, 5.0, 0.9),
            create_test_asset("GOOGL", 2800.0, 0.01, 0.15, 5.0, 0.9),
        ];

        let results = filter.filter_batch(&assets);
        assert_eq!(results.len(), 3);
        assert!(!results[0].is_noise); // AAPL
        assert!(results[1].is_noise); // VIX spike
        assert!(!results[2].is_noise); // GOOGL
    }
}
