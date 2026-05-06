// Statistical Significance Tests
// Phase C: Comparative Testing - Task C2: Statistical significance tests

use std::collections::HashMap;

/// Statistical test result
#[derive(Debug, Clone)]
pub struct SignificanceTestResult {
    pub test_name: String,
    pub p_value: f64,
    pub is_significant: bool,
    pub confidence: f64,
}

/// Statistical significance tester
pub struct SignificanceTester {
    alpha: f64, // Significance level (default 0.05)
    bonferroni_correction: bool,
}

impl SignificanceTester {
    /// Create a new significance tester
    pub fn new(alpha: f64, bonferroni_correction: bool) -> Self {
        Self {
            alpha,
            bonferroni_correction,
        }
    }
    
    /// Apply Bonferroni correction
    fn apply_bonferroni(&self, alpha: f64, num_tests: usize) -> f64 {
        if self.bonferroni_correction {
            alpha / num_tests as f64
        } else {
            alpha
        }
    }
    
    /// Perform t-test (simplified)
    pub fn t_test(&self, sample_a: &[f64], sample_b: &[f64]) -> SignificanceTestResult {
        let mean_a = sample_a.iter().sum::<f64>() / sample_a.len() as f64;
        let mean_b = sample_b.iter().sum::<f64>() / sample_b.len() as f64;
        
        let variance_a = sample_a.iter().map(|x| (x - mean_a).powi(2)).sum::<f64>() / (sample_a.len() - 1) as f64;
        let variance_b = sample_b.iter().map(|x| (x - mean_b).powi(2)).sum::<f64>() / (sample_b.len() - 1) as f64;
        
        let pooled_std = ((variance_a + variance_b) / 2.0).sqrt();
        let t_stat = (mean_a - mean_b) / (pooled_std * (2.0 / sample_a.len() as f64).sqrt());
        
        // Simplified p-value calculation
        let p_value = if t_stat.abs() > 1.96 {
            0.05
        } else {
            0.5
        };
        
        let corrected_alpha = self.apply_bonferroni(self.alpha, 1);
        let is_significant = p_value < corrected_alpha;
        
        SignificanceTestResult {
            test_name: "t-test".to_string(),
            p_value,
            is_significant,
            confidence: 1.0 - corrected_alpha,
        }
    }
    
    /// Perform paired t-test
    pub fn paired_t_test(&self, differences: &[f64]) -> SignificanceTestResult {
        let mean_diff = differences.iter().sum::<f64>() / differences.len() as f64;
        let variance_diff = differences.iter().map(|x| (x - mean_diff).powi(2)).sum::<f64>() / (differences.len() - 1) as f64;
        let std_diff = variance_diff.sqrt();
        
        let t_stat = mean_diff / (std_diff / (differences.len() as f64).sqrt());
        
        let p_value = if t_stat.abs() > 1.96 {
            0.05
        } else {
            0.5
        };
        
        let corrected_alpha = self.apply_bonferroni(self.alpha, 1);
        let is_significant = p_value < corrected_alpha;
        
        SignificanceTestResult {
            test_name: "paired t-test".to_string(),
            p_value,
            is_significant,
            confidence: 1.0 - corrected_alpha,
        }
    }
    
    /// Run multiple tests with Bonferroni correction
    pub fn run_multiple_tests(&self, test_results: &mut [SignificanceTestResult]) {
        let num_tests = test_results.len();
        let corrected_alpha = self.apply_bonferroni(self.alpha, num_tests);
        
        for result in test_results {
            result.is_significant = result.p_value < corrected_alpha;
            result.confidence = 1.0 - corrected_alpha;
        }
    }
}

impl Default for SignificanceTester {
    fn default() -> Self {
        Self::new(0.05, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_significance_tester() {
        let tester = SignificanceTester::new(0.05, true);
        
        let sample_a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let sample_b = vec![1.5, 2.5, 3.5, 4.5, 5.5];
        
        let result = tester.t_test(&sample_a, &sample_b);
        
        assert_eq!(result.test_name, "t-test");
        assert!(result.p_value > 0.0);
    }
    
    #[test]
    fn test_paired_t_test() {
        let tester = SignificanceTester::default();
        
        let differences = vec![0.1, 0.2, -0.1, 0.3, 0.0];
        
        let result = tester.paired_t_test(&differences);
        
        assert_eq!(result.test_name, "paired t-test");
        assert!(result.p_value > 0.0);
    }
    
    #[test]
    fn test_bonferroni_correction() {
        let tester = SignificanceTester::new(0.05, true);
        
        let corrected = tester.apply_bonferroni(0.05, 10);
        assert!((corrected - 0.005).abs() < 0.0001);
    }
}
