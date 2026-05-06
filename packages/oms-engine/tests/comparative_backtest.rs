// Side-by-side Backtest Harness
// Phase C: Comparative Testing - Task C1: Side-by-side backtest harness

use crate::fabric::base_model::BaseModel;
use crate::fabric::advanced_model::AdvancedModelPipeline;
use std::collections::HashMap;

/// Backtest result for a single model
#[derive(Debug, Clone)]
pub struct ModelBacktestResult {
    pub model_name: String,
    pub total_return: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
    pub trades_executed: u32,
}

/// Comparative backtest results
#[derive(Debug, Clone)]
pub struct ComparativeBacktestResult {
    pub base_model: ModelBacktestResult,
    pub advanced_model: Option<ModelBacktestResult>,
    pub hyperstatic_model: Option<ModelBacktestResult>,
    pub winner: Option<String>,
}

/// Side-by-side backtest harness
pub struct ComparativeBacktestHarness {
    base_model: BaseModel,
    advanced_model: Option<AdvancedModelPipeline>,
}

impl ComparativeBacktestHarness {
    /// Create a new comparative backtest harness
    pub fn new(base_model: BaseModel) -> Self {
        Self {
            base_model,
            advanced_model: None,
        }
    }
    
    /// Set advanced model
    pub fn with_advanced_model(mut self, model: AdvancedModelPipeline) -> Self {
        self.advanced_model = Some(model);
        self
    }
    
    /// Run backtest on a single model
    pub fn run_single_model_backtest(
        &self,
        model_name: &str,
        market_data: &[f64],
    ) -> ModelBacktestResult {
        // Simulate backtest
        let total_return = market_data.iter().sum::<f64>() / market_data.len() as f64;
        let sharpe_ratio = total_return / 0.15; // Simplified
        let max_drawdown = -0.05; // Simplified
        let win_rate = 0.55; // Simplified
        let trades_executed = 100; // Simplified
        
        ModelBacktestResult {
            model_name: model_name.to_string(),
            total_return,
            sharpe_ratio,
            max_drawdown,
            win_rate,
            trades_executed,
        }
    }
    
    /// Run comparative backtest on all models
    pub fn run_comparative_backtest(
        &self,
        market_data: &[f64],
    ) -> ComparativeBacktestResult {
        let base_result = self.run_single_model_backtest("base", market_data);
        
        let advanced_result = if self.advanced_model.is_some() {
            Some(self.run_single_model_backtest("advanced", market_data))
        } else {
            None
        };
        
        let hyperstatic_result = None; // TODO: Implement hyperstatic model
        
        // Determine winner based on Sharpe ratio
        let winner = vec![
            ("base".to_string(), base_result.sharpe_ratio),
            (advanced_result.as_ref().map(|r| ("advanced".to_string(), r.sharpe_ratio))),
            (hyperstatic_result.as_ref().map(|r| ("hyperstatic".to_string(), r.sharpe_ratio))),
        ]
        .into_iter()
        .filter_map(|x| x)
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(name, _)| name);
        
        ComparativeBacktestResult {
            base_model: base_result,
            advanced_model: advanced_result,
            hyperstatic_model: hyperstatic_result,
            winner,
        }
    }
    
    /// Check if base model says NO
    pub fn base_model_says_no(&self, market_data: &[f64]) -> bool {
        let result = self.run_single_model_backtest("base", market_data);
        result.sharpe_ratio < 0.5 // Guarded line: threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_comparative_backtest() {
        let base_model = BaseModel::default();
        let harness = ComparativeBacktestHarness::new(base_model);
        
        let market_data = vec![0.5, 0.6, 0.4, 0.7, 0.3];
        let result = harness.run_comparative_backtest(&market_data);
        
        assert_eq!(result.base_model.model_name, "base");
        assert!(result.base_model.total_return > 0.0);
    }
    
    #[test]
    fn test_base_model_guard() {
        let base_model = BaseModel::default();
        let harness = ComparativeBacktestHarness::new(base_model);
        
        let good_data = vec![0.8, 0.9, 0.7, 0.85];
        let bad_data = vec![0.1, 0.05, 0.02, 0.03];
        
        assert!(!harness.base_model_says_no(&good_data));
        assert!(harness.base_model_says_no(&bad_data));
    }
}
