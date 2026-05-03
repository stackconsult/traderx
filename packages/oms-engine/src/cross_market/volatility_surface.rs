use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Volatility surface parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilitySurfaceParams {
    pub lookback_window: usize,
    pub decay_factor: f64,
    pub min_volatility: f64,
    pub max_volatility: f64,
}

impl Default for VolatilitySurfaceParams {
    fn default() -> Self {
        Self {
            lookback_window: 20,
            decay_factor: 0.95,
            min_volatility: 0.001,
            max_volatility: 0.10,
        }
    }
}

/// Volatility adjustment for asset classes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityAdjustment {
    pub equity: f64,
    pub fixed_income: f64,
    pub fx: f64,
    pub crypto: f64,
    pub commodity: f64,
}

/// Volatility surface for multi-asset volatility tracking
pub struct VolatilitySurface {
    params: VolatilitySurfaceParams,
    volatility_history: HashMap<String, Vec<f64>>,
    current_volatility: HashMap<String, f64>,
}

impl VolatilitySurface {
    pub fn new(params: VolatilitySurfaceParams) -> Self {
        Self {
            params,
            volatility_history: HashMap::new(),
            current_volatility: HashMap::new(),
        }
    }

    /// Update volatility for an asset
    pub fn update_volatility(&mut self, asset: String, price: f64, previous_price: f64) {
        let return_val = (price - previous_price) / previous_price;
        let vol = return_val.abs();
        
        self.current_volatility.insert(asset.clone(), vol);
        
        let history = self.volatility_history.entry(asset).or_insert_with(Vec::new);
        history.push(vol);
        
        if history.len() > self.params.lookback_window {
            history.remove(0);
        }
    }

    /// Get volatility adjustment for all asset classes
    pub fn get_volatility_adjustment(&self) -> VolatilityAdjustment {
        let equity_vol = self.get_average_volatility("equity");
        let fi_vol = self.get_average_volatility("fixed_income");
        let fx_vol = self.get_average_volatility("fx");
        let crypto_vol = self.get_average_volatility("crypto");
        let commodity_vol = self.get_average_volatility("commodity");

        VolatilityAdjustment {
            equity: self.normalize_volatility(equity_vol),
            fixed_income: self.normalize_volatility(fi_vol),
            fx: self.normalize_volatility(fx_vol),
            crypto: self.normalize_volatility(crypto_vol),
            commodity: self.normalize_volatility(commodity_vol),
        }
    }

    fn get_average_volatility(&self, asset_type: &str) -> f64 {
        self.volatility_history
            .get(asset_type)
            .map(|v| v.iter().sum::<f64>() / v.len() as f64)
            .unwrap_or(self.params.min_volatility)
    }

    fn normalize_volatility(&self, vol: f64) -> f64 {
        vol.clamp(self.params.min_volatility, self.params.max_volatility)
    }
}

impl Default for VolatilitySurface {
    fn default() -> Self {
        Self::new(VolatilitySurfaceParams::default())
    }
}
