// Correlation Schema Updater (Online)
// Phase B: Advanced Model - Task B6: Correlation schema updater (online)

use std::collections::HashMap;

/// Correlation entry
#[derive(Debug, Clone)]
pub struct CorrelationEntry {
    pub market_a: String,
    pub market_b: String,
    pub correlation: f64,
    pub last_updated: i64,
}

/// Correlation schema updater - maintains online correlation matrix
pub struct CorrelationUpdater {
    correlations: HashMap<(String, String), CorrelationEntry>,
}

impl CorrelationUpdater {
    /// Create a new correlation updater
    pub fn new() -> Self {
        Self {
            correlations: HashMap::new(),
        }
    }
    
    /// Update correlation between two markets
    pub fn update_correlation(&mut self, market_a: String, market_b: String, correlation: f64) {
        let key = if market_a < market_b {
            (market_a.clone(), market_b.clone())
        } else {
            (market_b.clone(), market_a.clone())
        };
        
        self.correlations.insert(key, CorrelationEntry {
            market_a,
            market_b,
            correlation,
            last_updated: chrono::Utc::now().timestamp_millis(),
        });
    }
    
    /// Get correlation between two markets
    pub fn get_correlation(&self, market_a: &str, market_b: &str) -> Option<f64> {
        let key = if market_a < market_b {
            (market_a.to_string(), market_b.to_string())
        } else {
            (market_b.to_string(), market_a.to_string())
        };
        
        self.correlations.get(&key).map(|entry| entry.correlation)
    }
    
    /// Get all correlations for a market
    pub fn get_market_correlations(&self, market: &str) -> Vec<(&String, f64)> {
        self.correlations.iter()
            .filter(|((a, b), _)| a == market || b == market)
            .map(|((a, b), entry)| {
                if a == market {
                    (b, entry.correlation)
                } else {
                    (a, entry.correlation)
                }
            })
            .collect()
    }
    
    /// Get high correlations (>0.7 threshold)
    pub fn get_high_correlations(&self, threshold: f64) -> Vec<CorrelationEntry> {
        self.correlations.values()
            .filter(|entry| entry.correlation.abs() > threshold)
            .cloned()
            .collect()
    }
    
    /// Get total correlation count
    pub fn correlation_count(&self) -> usize {
        self.correlations.len()
    }
}

impl Default for CorrelationUpdater {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_correlation_updater() {
        let mut updater = CorrelationUpdater::new();
        
        // Update correlations
        updater.update_correlation("AAPL".to_string(), "MSFT".to_string(), 0.85);
        updater.update_correlation("AAPL".to_string(), "GOOGL".to_string(), 0.65);
        
        // Get correlation
        let corr = updater.get_correlation("AAPL", "MSFT").unwrap();
        assert_eq!(corr, 0.85);
        
        // Get market correlations
        let apple_corrs = updater.get_market_correlations("AAPL");
        assert_eq!(apple_corrs.len(), 2);
        
        // Get high correlations
        let high_corrs = updater.get_high_correlations(0.7);
        assert_eq!(high_corrs.len(), 1);
    }
    
    #[test]
    fn test_symmetric_correlation() {
        let mut updater = CorrelationUpdater::new();
        
        updater.update_correlation("AAPL".to_string(), "MSFT".to_string(), 0.85);
        
        // Should work regardless of order
        let corr1 = updater.get_correlation("AAPL", "MSFT").unwrap();
        let corr2 = updater.get_correlation("MSFT", "AAPL").unwrap();
        assert_eq!(corr1, corr2);
    }
}
