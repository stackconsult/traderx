// Position Reconciliation
// Syncs local positions with exchange positions

use crate::adapters::{ExchangeAdapter, Balance};
use crate::state_machine::{Order, Side};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Position reconciliation configuration
#[derive(Debug, Clone)]
pub struct ReconciliationConfig {
    pub reconcile_interval_seconds: u64,
    pub tolerance_bps: i32, // Basis points tolerance for position differences
}

impl Default for ReconciliationConfig {
    fn default() -> Self {
        Self {
            reconcile_interval_seconds: 60,
            tolerance_bps: 10, // 0.1% tolerance
        }
    }
}

/// Local position
#[derive(Debug, Clone)]
pub struct LocalPosition {
    pub symbol: String,
    pub quantity: Decimal,
    pub avg_price: Decimal,
    pub unrealized_pnl: Decimal,
}

/// Exchange position
#[derive(Debug, Clone)]
pub struct ExchangePosition {
    pub symbol: String,
    pub quantity: Decimal,
    pub avg_price: Decimal,
}

/// Reconciliation result
#[derive(Debug, Clone)]
pub struct ReconciliationResult {
    pub symbol: String,
    pub local_quantity: Decimal,
    pub exchange_quantity: Decimal,
    pub difference: Decimal,
    pub within_tolerance: bool,
    pub requires_correction: bool,
}

/// Position reconciler
pub struct PositionReconciler {
    config: ReconciliationConfig,
    adapter: Arc<dyn ExchangeAdapter>,
    local_positions: HashMap<String, LocalPosition>,
}

impl PositionReconciler {
    /// Create a new position reconciler
    pub fn new(
        config: ReconciliationConfig,
        adapter: Arc<dyn ExchangeAdapter>,
    ) -> Self {
        Self {
            config,
            adapter,
            local_positions: HashMap::new(),
        }
    }
    
    /// Update local position
    pub fn update_local_position(&mut self, position: LocalPosition) {
        self.local_positions.insert(position.symbol.clone(), position);
    }
    
    /// Get local position
    pub fn get_local_position(&self, symbol: &str) -> Option<&LocalPosition> {
        self.local_positions.get(symbol)
    }
    
    /// Reconcile positions with exchange
    pub async fn reconcile(&self) -> Result<Vec<ReconciliationResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        
        // Get all balances from exchange
        let balances = self.adapter.get_all_balances().await?;
        
        // Convert balances to positions
        let mut exchange_positions: HashMap<String, ExchangePosition> = HashMap::new();
        for balance in balances {
            if balance.total > Decimal::ZERO {
                exchange_positions.insert(
                    balance.asset.clone(),
                    ExchangePosition {
                        symbol: balance.asset.clone(),
                        quantity: balance.total,
                        avg_price: Decimal::ZERO, // Exchange may not provide avg price
                    },
                );
            }
        }
        
        // Compare local vs exchange positions
        for (symbol, local_pos) in &self.local_positions {
            let exchange_pos = exchange_positions.get(symbol);
            
            let exchange_quantity = exchange_pos.map(|p| p.quantity).unwrap_or(Decimal::ZERO);
            let difference = local_pos.quantity - exchange_quantity;
            
            // Calculate tolerance
            let tolerance = local_pos.quantity * Decimal::from(self.config.tolerance_bps) / Decimal::from(10000);
            let within_tolerance = difference.abs() <= tolerance;
            
            results.push(ReconciliationResult {
                symbol: symbol.clone(),
                local_quantity: local_pos.quantity,
                exchange_quantity,
                difference,
                within_tolerance,
                requires_correction: !within_tolerance && difference.abs() > Decimal::ZERO,
            });
        }
        
        // Check for positions on exchange not in local
        for (symbol, exchange_pos) in &exchange_positions {
            if !self.local_positions.contains_key(symbol) {
                results.push(ReconciliationResult {
                    symbol: symbol.clone(),
                    local_quantity: Decimal::ZERO,
                    exchange_quantity: exchange_pos.quantity,
                    difference: -exchange_pos.quantity,
                    within_tolerance: exchange_pos.quantity.abs() <= Decimal::from(1), // Small tolerance for dust
                    requires_correction: exchange_pos.quantity.abs() > Decimal::from(1),
                });
            }
        }
        
        Ok(results)
    }
    
    /// Auto-correct position discrepancies
    pub async fn auto_correct(&self, results: &[ReconciliationResult]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut corrections = Vec::new();
        
        for result in results {
            if result.requires_correction {
                let correction = format!(
                    "Correcting {}: local={}, exchange={}, diff={}",
                    result.symbol, result.local_quantity, result.exchange_quantity, result.difference
                );
                corrections.push(correction);
                
                // In production, would submit correction orders here
                log::warn!("Position discrepancy detected for {}", result.symbol);
            }
        }
        
        Ok(corrections)
    }
    
    /// Start periodic reconciliation
    pub async fn start_periodic_reconciliation(&self) {
        let interval = tokio::time::Duration::from_secs(self.config.reconcile_interval_seconds);
        
        loop {
            tokio::time::sleep(interval).await;
            
            match self.reconcile().await {
                Ok(results) => {
                    let discrepancies: Vec<_> = results.iter()
                        .filter(|r| r.requires_correction)
                        .collect();
                    
                    if !discrepancies.is_empty() {
                        log::warn!("Found {} position discrepancies", discrepancies.len());
                        if let Err(e) = self.auto_correct(&results).await {
                            log::error!("Auto-correction failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    log::error!("Reconciliation failed: {}", e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_position_reconciliation() {
        let config = ReconciliationConfig::default();
        
        // This would require a mock adapter
        // For now, just test the config
        assert_eq!(config.reconcile_interval_seconds, 60);
        assert_eq!(config.tolerance_bps, 10);
    }
}
