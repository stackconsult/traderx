use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Correlation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationParams {
    pub lookback_window: usize,
    pub min_correlation: f64,
    pub max_correlation: f64,
    pub update_threshold: f64,
}

impl Default for CorrelationParams {
    fn default() -> Self {
        Self {
            lookback_window: 20,
            min_correlation: -1.0,
            max_correlation: 1.0,
            update_threshold: 0.05,
        }
    }
}

/// Correlation adjustment for asset pairs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationAdjustment {
    pub equity_bond: f64,
    pub equity_fx: f64,
    pub fx_commodity: f64,
    pub crypto_equity: f64,
    pub crypto_fx: f64,
}

/// Cross correlation matrix
pub struct CrossCorrelationMatrix {
    params: CorrelationParams,
    price_history: HashMap<String, Vec<f64>>,
    correlations: HashMap<String, f64>,
}

impl CrossCorrelationMatrix {
    pub fn new(params: CorrelationParams) -> Self {
        Self {
            params,
            price_history: HashMap::new(),
            correlations: HashMap::new(),
        }
    }

    /// Update price history for an asset
    pub fn update_price(&mut self, asset: String, price: f64) {
        let history = self.price_history.entry(asset).or_insert_with(Vec::new);
        history.push(price);
        
        if history.len() > self.params.lookback_window {
            history.remove(0);
        }
    }

    /// Get cross-market correlation adjustment
    pub fn get_cross_adjustment(&self) -> CorrelationAdjustment {
        let equity_bond = self.calculate_correlation("equity", "fixed_income");
        let equity_fx = self.calculate_correlation("equity", "fx");
        let fx_commodity = self.calculate_correlation("fx", "commodity");
        let crypto_equity = self.calculate_correlation("crypto", "equity");
        let crypto_fx = self.calculate_correlation("crypto", "fx");

        CorrelationAdjustment {
            equity_bond,
            equity_fx,
            fx_commodity,
            crypto_equity,
            crypto_fx,
        }
    }

    fn calculate_correlation(&self, asset1: &str, asset2: &str) -> f64 {
        let history1 = self.price_history.get(asset1);
        let history2 = self.price_history.get(asset2);
        
        match (history1, history2) {
            (Some(h1), Some(h2)) if h1.len() >= 2 && h2.len() >= 2 => {
                let returns1: Vec<f64> = h1.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect();
                let returns2: Vec<f64> = h2.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect();
                
                let mean1: f64 = returns1.iter().sum::<f64>() / returns1.len() as f64;
                let mean2: f64 = returns2.iter().sum::<f64>() / returns2.len() as f64;
                
                let cov: f64 = returns1.iter()
                    .zip(returns2.iter())
                    .map(|(r1, r2)| (r1 - mean1) * (r2 - mean2))
                    .sum::<f64>() / (returns1.len() - 1) as f64;
                
                let var1: f64 = returns1.iter()
                    .map(|r| (r - mean1).powi(2))
                    .sum::<f64>() / (returns1.len() - 1) as f64;
                
                let var2: f64 = returns2.iter()
                    .map(|r| (r - mean2).powi(2))
                    .sum::<f64>() / (returns2.len() - 1) as f64;
                
                let std1 = var1.sqrt();
                let std2 = var2.sqrt();
                
                if std1 > 0.0 && std2 > 0.0 {
                    (cov / (std1 * std2)).clamp(self.params.min_correlation, self.params.max_correlation)
                } else {
                    0.0
                }
            },
            _ => 0.0,
        }
    }
}

impl Default for CrossCorrelationMatrix {
    fn default() -> Self {
        Self::new(CorrelationParams::default())
    }
}
