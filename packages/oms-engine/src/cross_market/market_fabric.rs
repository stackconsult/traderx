use std::collections::HashMap;
use std::time::Instant;
use serde::{Serialize, Deserialize};
use tracing::debug;
use chrono::{DateTime, Utc};

use super::regime_detection::MarketRegime;
use super::bam_integration::{BamSignal, BamDomain, BamLayer};

/// Per-asset state within the unified market fabric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetFabricState {
    pub symbol: String,
    pub price: f64,
    pub volume_24h: f64,
    pub volatility_short: f64,     // 1-min realized vol
    pub volatility_long: f64,      // 1-day realized vol
    pub liquidity_score: f64,      // 0.0 = frozen, 1.0 = deep liquid
    pub spread_bps: f64,
    pub regime: MarketRegime,
    pub signal_quality: f64,       // Post-noise-filter quality score (0.0–1.0)
    pub predictability_score: f64, // Pattern-based predictability (0.0–1.0)
    pub bam_address: u16,          // 10-bit BAM address
}

impl AssetFabricState {
    /// Create BAM signal for this asset
    pub fn to_bam_signal(&self) -> BamSignal {
        BamSignal {
            domain: BamDomain::CrossAsset,
            layer: BamLayer::Prim,
            type_flag: 0,
            axis_flag: if self.predictability_score > 0.7 { 1 } else { 0 },
        }
    }

    /// Compute composite health score (0.0 = dead, 1.0 = perfect)
    pub fn health_score(&self) -> f64 {
        let vol_health = (1.0 - self.volatility_short.clamp(0.0, 1.0)) * 0.3;
        let liq_health = self.liquidity_score.clamp(0.0, 1.0) * 0.3;
        let sig_health = self.signal_quality.clamp(0.0, 1.0) * 0.2;
        let pred_health = self.predictability_score.clamp(0.0, 1.0) * 0.2;
        (vol_health + liq_health + sig_health + pred_health).clamp(0.0, 1.0)
    }
}

/// Immutable snapshot of the entire market fabric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricState {
    pub timestamp: DateTime<Utc>,
    pub asset_states: HashMap<String, AssetFabricState>,
    pub global_regime: MarketRegime,
    pub noise_level: f64,          // 0.0 = clean, 1.0 = maximum noise
    pub predictability_index: f64, // 0.0 = random, 1.0 = perfectly predictable
    pub deterministic_hash: String,
}

impl FabricState {
    /// Compute deterministic hash of the entire fabric state
    pub fn compute_hash(&self) -> String {
        let mut hash_value: u64 = 0;
        
        // Hash timestamp
        hash_value ^= (self.timestamp.timestamp() as u64).wrapping_mul(31);
        
        // Hash global regime
        let regime_val = match self.global_regime {
            MarketRegime::Bull => 1u64,
            MarketRegime::Bear => 2,
            MarketRegime::Volatile => 3,
            MarketRegime::RangeBound => 4,
            MarketRegime::TrendingUp => 5,
            MarketRegime::TrendingDown => 6,
            MarketRegime::Transition => 7,
        };
        hash_value ^= regime_val.wrapping_mul(37);
        
        // Hash noise level and predictability
        hash_value ^= ((self.noise_level * 1000.0) as u64).wrapping_mul(41);
        hash_value ^= ((self.predictability_index * 1000.0) as u64).wrapping_mul(43);
        
        // Hash each asset state (sorted for determinism)
        let mut sorted_assets: Vec<_> = self.asset_states.iter().collect();
        sorted_assets.sort_by_key(|(k, _)| *k);
        
        for (symbol, state) in sorted_assets {
            hash_value ^= symbol.as_bytes().iter().fold(0u64, |acc, &b| {
                acc.wrapping_mul(47).wrapping_add(b as u64)
            });
            hash_value ^= (state.price.to_bits() as u64).wrapping_mul(53);
            hash_value ^= ((state.health_score() * 1000.0) as u64).wrapping_mul(59);
        }
        
        format!("{:016x}", hash_value)
    }

    /// Get assets sorted by predictability score (highest first)
    pub fn top_predictable_assets(&self, n: usize) -> Vec<(String, f64)> {
        let mut assets: Vec<_> = self.asset_states
            .iter()
            .map(|(k, v)| (k.clone(), v.predictability_score))
            .collect();
        assets.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        assets.into_iter().take(n).collect()
    }

    /// Get assets sorted by signal quality (highest first)
    pub fn top_quality_assets(&self, n: usize) -> Vec<(String, f64)> {
        let mut assets: Vec<_> = self.asset_states
            .iter()
            .map(|(k, v)| (k.clone(), v.signal_quality))
            .collect();
        assets.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        assets.into_iter().take(n).collect()
    }

    /// Get assets sorted by health score (highest first)
    pub fn top_healthy_assets(&self, n: usize) -> Vec<(String, f64)> {
        let mut assets: Vec<_> = self.asset_states
            .iter()
            .map(|(k, v)| (k.clone(), v.health_score()))
            .collect();
        assets.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        assets.into_iter().take(n).collect()
    }

    /// Get assets that should be ignored (noise/illiquidity)
    pub fn ignored_assets(&self, threshold: f64) -> Vec<String> {
        self.asset_states
            .iter()
            .filter(|(_, v)| v.signal_quality < threshold || v.liquidity_score < threshold)
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// Get BAM signals for all assets
    pub fn bam_signals(&self) -> Vec<BamSignal> {
        self.asset_states
            .values()
            .map(|s| s.to_bam_signal())
            .collect()
    }
}

/// Market Fabric Reader — unified multi-asset state capture
pub struct MarketFabric {
    current_state: Option<FabricState>,
    last_update: Option<Instant>,
    update_count: u64,
}

impl MarketFabric {
    pub fn new() -> Self {
        Self {
            current_state: None,
            last_update: None,
            update_count: 0,
        }
    }

    /// Update fabric state from individual asset states — optimized for <50µs
    pub fn update(&mut self, asset_states: HashMap<String, AssetFabricState>, global_regime: MarketRegime) -> FabricState {
        let start = Instant::now();
        
        // Compute aggregate metrics
        let asset_count = asset_states.len().max(1) as f64;
        let total_noise: f64 = asset_states.values()
            .map(|s| 1.0 - s.signal_quality)
            .sum();
        let total_predictability: f64 = asset_states.values()
            .map(|s| s.predictability_score)
            .sum();
        
        let mut state = FabricState {
            timestamp: Utc::now(),
            asset_states,
            global_regime,
            noise_level: (total_noise / asset_count).clamp(0.0, 1.0),
            predictability_index: (total_predictability / asset_count).clamp(0.0, 1.0),
            deterministic_hash: String::new(),
        };
        
        // Compute deterministic hash
        state.deterministic_hash = state.compute_hash();
        
        self.current_state = Some(state.clone());
        self.last_update = Some(Instant::now());
        self.update_count += 1;
        
        let duration = start.elapsed();
        debug!("Fabric update completed in {:?} for {} assets", duration, asset_count);
        
        state
    }

    /// Get current fabric state
    pub fn current_state(&self) -> Option<&FabricState> {
        self.current_state.as_ref()
    }

    /// Get update count
    pub fn update_count(&self) -> u64 {
        self.update_count
    }

    /// Get time since last update
    pub fn time_since_update(&self) -> Option<std::time::Duration> {
        self.last_update.map(|t| t.elapsed())
    }
}

impl Default for MarketFabric {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_asset(symbol: &str, price: f64, quality: f64, predictability: f64) -> AssetFabricState {
        AssetFabricState {
            symbol: symbol.to_string(),
            price,
            volume_24h: 1_000_000.0,
            volatility_short: 0.01,
            volatility_long: 0.15,
            liquidity_score: 0.9,
            spread_bps: 5.0,
            regime: MarketRegime::TrendingUp,
            signal_quality: quality,
            predictability_score: predictability,
            bam_address: 0,
        }
    }

    #[test]
    fn test_market_fabric_update() {
        let mut fabric = MarketFabric::new();
        
        let mut assets = HashMap::new();
        assets.insert("AAPL".to_string(), create_test_asset("AAPL", 150.0, 0.9, 0.8));
        assets.insert("GOOGL".to_string(), create_test_asset("GOOGL", 2800.0, 0.85, 0.75));
        assets.insert("BTC".to_string(), create_test_asset("BTC", 45000.0, 0.6, 0.5));
        
        let state = fabric.update(assets, MarketRegime::Bull);
        
        assert_eq!(state.asset_states.len(), 3);
        assert!(state.noise_level >= 0.0 && state.noise_level <= 1.0);
        assert!(state.predictability_index > 0.0);
        assert!(!state.deterministic_hash.is_empty());
        assert_eq!(fabric.update_count(), 1);
    }

    #[test]
    fn test_fabric_state_hash_determinism() {
        let mut assets = HashMap::new();
        assets.insert("AAPL".to_string(), create_test_asset("AAPL", 150.0, 0.9, 0.8));
        assets.insert("GOOGL".to_string(), create_test_asset("GOOGL", 2800.0, 0.85, 0.75));
        
        let state1 = FabricState {
            timestamp: Utc::now(),
            asset_states: assets.clone(),
            global_regime: MarketRegime::Bull,
            noise_level: 0.1,
            predictability_index: 0.8,
            deterministic_hash: String::new(),
        };
        
        // Small delay to ensure timestamp differs
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let state2 = FabricState {
            timestamp: Utc::now(),
            asset_states: assets,
            global_regime: MarketRegime::Bull,
            noise_level: 0.1,
            predictability_index: 0.8,
            deterministic_hash: String::new(),
        };
        
        let hash1 = state1.compute_hash();
        let hash2 = state2.compute_hash();
        
        // Same asset states should produce same hash regardless of timestamp
        // (timestamp is hashed but we use the same assets)
        // Actually timestamp IS in hash, so they may differ
        // Let's test that same timestamp produces same hash
        let same_time = Utc::now();
        let mut s1 = state1.clone();
        let mut s2 = state2;
        s1.timestamp = same_time;
        s2.timestamp = same_time;
        
        assert_eq!(s1.compute_hash(), s2.compute_hash());
    }

    #[test]
    fn test_top_predictable_assets() {
        let mut assets = HashMap::new();
        assets.insert("AAPL".to_string(), create_test_asset("AAPL", 150.0, 0.9, 0.95));
        assets.insert("GOOGL".to_string(), create_test_asset("GOOGL", 2800.0, 0.85, 0.75));
        assets.insert("BTC".to_string(), create_test_asset("BTC", 45000.0, 0.6, 0.5));
        
        let state = FabricState {
            timestamp: Utc::now(),
            asset_states: assets,
            global_regime: MarketRegime::Bull,
            noise_level: 0.2,
            predictability_index: 0.7,
            deterministic_hash: String::new(),
        };
        
        let top = state.top_predictable_assets(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, "AAPL");
        assert!(top[0].1 > top[1].1);
    }

    #[test]
    fn test_ignored_assets() {
        let mut assets = HashMap::new();
        assets.insert("AAPL".to_string(), create_test_asset("AAPL", 150.0, 0.9, 0.8));
        assets.insert("PENNY".to_string(), create_test_asset("PENNY", 0.05, 0.2, 0.1));
        
        let state = FabricState {
            timestamp: Utc::now(),
            asset_states: assets,
            global_regime: MarketRegime::Bull,
            noise_level: 0.5,
            predictability_index: 0.5,
            deterministic_hash: String::new(),
        };
        
        let ignored = state.ignored_assets(0.5);
        assert!(ignored.contains(&"PENNY".to_string()));
        assert!(!ignored.contains(&"AAPL".to_string()));
    }

    #[test]
    fn test_asset_health_score() {
        let asset = create_test_asset("AAPL", 150.0, 0.9, 0.8);
        let health = asset.health_score();
        assert!(health > 0.0 && health <= 1.0);
    }
}
