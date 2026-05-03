use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Liquidity parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityParams {
    pub lookback_window: usize,
    pub min_liquidity: f64,
    pub max_liquidity: f64,
    pub depth_threshold: f64,
}

impl Default for LiquidityParams {
    fn default() -> Self {
        Self {
            lookback_window: 20,
            min_liquidity: 0.0,
            max_liquidity: 1.0,
            depth_threshold: 0.5,
        }
    }
}

/// Liquidity adjustment for asset classes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityAdjustment {
    pub equity: f64,
    pub fixed_income: f64,
    pub fx: f64,
    pub crypto: f64,
    pub commodity: f64,
}

/// Liquidity tracker for multi-asset liquidity monitoring
pub struct LiquidityTracker {
    params: LiquidityParams,
    liquidity_history: HashMap<String, Vec<f64>>,
    current_liquidity: HashMap<String, f64>,
}

impl LiquidityTracker {
    pub fn new(params: LiquidityParams) -> Self {
        Self {
            params,
            liquidity_history: HashMap::new(),
            current_liquidity: HashMap::new(),
        }
    }

    /// Update liquidity for an asset
    pub fn update_liquidity(&mut self, asset: String, volume: f64, depth: f64) {
        let liquidity = (volume * depth).clamp(self.params.min_liquidity, self.params.max_liquidity);
        
        self.current_liquidity.insert(asset.clone(), liquidity);
        
        let history = self.liquidity_history.entry(asset).or_insert_with(Vec::new);
        history.push(liquidity);
        
        if history.len() > self.params.lookback_window {
            history.remove(0);
        }
    }

    /// Get liquidity adjustment for all asset classes
    pub fn get_liquidity_adjustment(&self) -> LiquidityAdjustment {
        let equity_liq = self.get_average_liquidity("equity");
        let fi_liq = self.get_average_liquidity("fixed_income");
        let fx_liq = self.get_average_liquidity("fx");
        let crypto_liq = self.get_average_liquidity("crypto");
        let commodity_liq = self.get_average_liquidity("commodity");

        LiquidityAdjustment {
            equity: self.normalize_liquidity(equity_liq),
            fixed_income: self.normalize_liquidity(fi_liq),
            fx: self.normalize_liquidity(fx_liq),
            crypto: self.normalize_liquidity(crypto_liq),
            commodity: self.normalize_liquidity(commodity_liq),
        }
    }

    fn get_average_liquidity(&self, asset_type: &str) -> f64 {
        self.liquidity_history
            .get(asset_type)
            .map(|v| v.iter().sum::<f64>() / v.len() as f64)
            .unwrap_or(self.params.min_liquidity)
    }

    fn normalize_liquidity(&self, liquidity: f64) -> f64 {
        liquidity.clamp(self.params.min_liquidity, self.params.max_liquidity)
    }
}

impl Default for LiquidityTracker {
    fn default() -> Self {
        Self::new(LiquidityParams::default())
    }
}
