// Model A/B Testing Framework
// Phase C: Comparative Testing - Task C3: Model A/B testing framework

use crate::tests::comparative_backtest::ModelBacktestResult;
use crate::tests::significance_tests::SignificanceTester;
use std::collections::HashMap;

/// A/B test variant
#[derive(Debug, Clone)]
pub struct AbTestVariant {
    pub name: String,
    pub model_result: ModelBacktestResult,
}

/// A/B test result
#[derive(Debug, Clone)]
pub struct AbTestResult {
    pub variant_a: AbTestVariant,
    pub variant_b: AbTestVariant,
    pub winner: String,
    pub lift: f64,
    pub is_significant: bool,
}

/// A/B testing framework
pub struct AbTestFramework {
    significance_tester: SignificanceTester,
}

impl AbTestFramework {
    /// Create a new A/B testing framework
    pub fn new(significance_tester: SignificanceTester) -> Self {
        Self {
            significance_tester,
        }
    }
    
    /// Run A/B test between two models
    pub fn run_ab_test(
        &self,
        variant_a: AbTestVariant,
        variant_b: AbTestVariant,
    ) -> AbTestResult {
        // Calculate lift (percentage improvement)
        let lift = if variant_a.model_result.total_return != 0.0 {
            (variant_b.model_result.total_return - variant_a.model_result.total_return)
                / variant_a.model_result.total_return.abs()
        } else {
            0.0
        };
        
        // Determine winner based on Sharpe ratio
        let winner = if variant_b.model_result.sharpe_ratio > variant_a.model_result.sharpe_ratio {
            variant_b.name.clone()
        } else {
            variant_a.name.clone()
        };
        
        // Perform significance test on returns
        let returns_a = vec![variant_a.model_result.total_return];
        let returns_b = vec![variant_b.model_result.total_return];
        let test_result = self.significance_tester.t_test(&returns_a, &returns_b);
        
        AbTestResult {
            variant_a,
            variant_b,
            winner,
            lift,
            is_significant: test_result.is_significant,
        }
    }
    
    /// Run multi-arm A/B test (A/B/C)
    pub fn run_multi_arm_test(&self, variants: Vec<AbTestVariant>) -> Vec<AbTestResult> {
        let mut results = Vec::new();
        
        for i in 0..variants.len() {
            for j in (i + 1)..variants.len() {
                let result = self.run_ab_test(
                    variants[i].clone(),
                    variants[j].clone(),
                );
                results.push(result);
            }
        }
        
        results
    }
}

impl Default for AbTestFramework {
    fn default() -> Self {
        Self::new(SignificanceTester::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ab_test_framework() {
        let framework = AbTestFramework::default();
        
        let variant_a = AbTestVariant {
            name: "model_a".to_string(),
            model_result: ModelBacktestResult {
                model_name: "model_a".to_string(),
                total_return: 0.10,
                sharpe_ratio: 1.5,
                max_drawdown: -0.05,
                win_rate: 0.55,
                trades_executed: 100,
            },
        };
        
        let variant_b = AbTestVariant {
            name: "model_b".to_string(),
            model_result: ModelBacktestResult {
                model_name: "model_b".to_string(),
                total_return: 0.12,
                sharpe_ratio: 1.8,
                max_drawdown: -0.04,
                win_rate: 0.60,
                trades_executed: 95,
            },
        };
        
        let result = framework.run_ab_test(variant_a, variant_b);
        
        assert_eq!(result.winner, "model_b");
        assert!(result.lift > 0.0);
    }
    
    #[test]
    fn test_multi_arm_test() {
        let framework = AbTestFramework::default();
        
        let variants = vec![
            AbTestVariant {
                name: "model_a".to_string(),
                model_result: ModelBacktestResult {
                    model_name: "model_a".to_string(),
                    total_return: 0.10,
                    sharpe_ratio: 1.5,
                    max_drawdown: -0.05,
                    win_rate: 0.55,
                    trades_executed: 100,
                },
            },
            AbTestVariant {
                name: "model_b".to_string(),
                model_result: ModelBacktestResult {
                    model_name: "model_b".to_string(),
                    total_return: 0.12,
                    sharpe_ratio: 1.8,
                    max_drawdown: -0.04,
                    win_rate: 0.60,
                    trades_executed: 95,
                },
            },
            AbTestVariant {
                name: "model_c".to_string(),
                model_result: ModelBacktestResult {
                    model_name: "model_c".to_string(),
                    total_return: 0.11,
                    sharpe_ratio: 1.6,
                    max_drawdown: -0.06,
                    win_rate: 0.58,
                    trades_executed: 98,
                },
            },
        ];
        
        let results = framework.run_multi_arm_test(variants);
        
        assert_eq!(results.len(), 3); // A vs B, A vs C, B vs C
    }
}
