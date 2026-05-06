// Portfolio Fabric Allocator
// Phase B: Advanced Model - Task B7: Portfolio fabric allocator

use crate::fabric::correlation_updater::CorrelationUpdater;

/// Portfolio allocation configuration
#[derive(Debug, Clone)]
pub struct PortfolioConfig {
    pub total_capital: f64,
    pub max_position_pct: f64,
    pub use_kelly_criterion: bool,
}

impl Default for PortfolioConfig {
    fn default() -> Self {
        Self {
            total_capital: 1_000_000.0,
            max_position_pct: 0.2, // 20% max per position
            use_kelly_criterion: true,
        }
    }
}

/// Position allocation
#[derive(Debug, Clone)]
pub struct PositionAllocation {
    pub market: String,
    pub allocation: f64,
    pub weight: f64,
}

/// Portfolio fabric allocator - Kelly criterion + correlation adjustment
pub struct PortfolioFabricAllocator {
    config: PortfolioConfig,
    correlation_updater: CorrelationUpdater,
}

impl PortfolioFabricAllocator {
    /// Create a new portfolio allocator
    pub fn new(config: PortfolioConfig, correlation_updater: CorrelationUpdater) -> Self {
        Self {
            config,
            correlation_updater,
        }
    }
    
    /// Calculate Kelly criterion allocation
    fn calculate_kelly(&self, expected_return: f64, variance: f64) -> f64 {
        if !self.config.use_kelly_criterion {
            return 1.0 / 6.0; // Equal weight for 6 markets
        }
        
        if variance <= 0.0 {
            return 0.0;
        }
        
        // Kelly criterion: f* = μ/σ²
        let kelly = expected_return / variance;
        
        // Cap at max position
        kelly.min(self.config.max_position_pct)
    }
    
    /// Adjust allocation based on correlations
    fn adjust_for_correlation(&self, base_allocation: f64, market: &str) -> f64 {
        let correlations = self.correlation_updater.get_market_correlations(market);
        
        if correlations.is_empty() {
            return base_allocation;
        }
        
        // Reduce allocation if highly correlated
        let avg_correlation: f64 = correlations.iter().map(|(_, c)| c).sum::<f64>() / correlations.len() as f64;
        let adjustment = 1.0 - (avg_correlation.abs() * 0.5);
        
        base_allocation * adjustment
    }
    
    /// Allocate portfolio across markets
    pub fn allocate(&self, market_data: &[(String, f64, f64)]) -> Vec<PositionAllocation> {
        // market_data: (market, expected_return, variance)
        let mut allocations = Vec::new();
        let mut total_weight = 0.0;
        
        for (market, expected_return, variance) in market_data {
            let kelly = self.calculate_kelly(*expected_return, *variance);
            let adjusted = self.adjust_for_correlation(kelly, market);
            
            allocations.push(PositionAllocation {
                market: market.clone(),
                allocation: adjusted * self.config.total_capital,
                weight: adjusted,
            });
            
            total_weight += adjusted;
        }
        
        // Normalize weights to sum to 1.0
        if total_weight > 0.0 {
            for allocation in &mut allocations {
                allocation.weight /= total_weight;
                allocation.allocation = allocation.weight * self.config.total_capital;
            }
        }
        
        allocations
    }
    
    /// Get configuration
    pub fn get_config(&self) -> &PortfolioConfig {
        &self.config
    }
}

impl Default for PortfolioFabricAllocator {
    fn default() -> Self {
        Self::new(
            PortfolioConfig::default(),
            CorrelationUpdater::default(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_portfolio_allocator() {
        let config = PortfolioConfig::default();
        let correlation_updater = CorrelationUpdater::default();
        let allocator = PortfolioFabricAllocator::new(config, correlation_updater);
        
        let market_data = vec![
            ("AAPL".to_string(), 0.15, 0.1),
            ("MSFT".to_string(), 0.12, 0.08),
            ("GOOGL".to_string(), 0.10, 0.12),
        ];
        
        let allocations = allocator.allocate(&market_data);
        
        assert_eq!(allocations.len(), 3);
        
        // Check allocations sum to total capital
        let total_allocation: f64 = allocations.iter().map(|a| a.allocation).sum();
        assert!((total_allocation - allocator.config.total_capital).abs() < 1.0);
    }
    
    #[test]
    fn test_kelly_criterion() {
        let config = PortfolioConfig::default();
        let correlation_updater = CorrelationUpdater::default();
        let allocator = PortfolioFabricAllocator::new(config, correlation_updater);
        
        // High return, low variance -> high allocation
        let kelly1 = allocator.calculate_kelly(0.2, 0.05);
        
        // Low return, high variance -> low allocation
        let kelly2 = allocator.calculate_kelly(0.05, 0.2);
        
        assert!(kelly1 > kelly2);
    }
}
