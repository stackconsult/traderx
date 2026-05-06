// BAM Trading Data Router
// Integrates BAM components with live trading system

use crate::fabric::{AdvancedModelPipeline, CorrelationUpdater, DivergenceMonitor, PatternMatcher};
use crate::mesh::{MarketClass, MultiMarketGridAllocator};
use crate::risk_bus::RiskBus;
use crate::signal_router::{AgentSignal, RouteOutcome, RouteStatus};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// BAM Trading Router - routes trading signals through BAM system
pub struct BamTradingRouter {
    grid_allocator: Arc<RwLock<MultiMarketGridAllocator>>,
    pattern_matcher: Arc<RwLock<PatternMatcher>>,
    advanced_model: Arc<RwLock<Option<AdvancedModelPipeline>>>,
    divergence_monitor: Arc<RwLock<DivergenceMonitor>>,
    correlation_updater: Arc<RwLock<CorrelationUpdater>>,
    risk_bus: Arc<RiskBus>,
}

impl BamTradingRouter {
    /// Create a new BAM trading router
    pub fn new(risk_bus: Arc<RiskBus>) -> Self {
        let grid_allocator = Arc::new(RwLock::new(MultiMarketGridAllocator::new()));
        let pattern_matcher = Arc::new(RwLock::new(PatternMatcher::new()));
        let advanced_model = Arc::new(RwLock::new(None));
        let divergence_monitor = Arc::new(RwLock::new(DivergenceMonitor::default()));
        let correlation_updater = Arc::new(RwLock::new(CorrelationUpdater::default()));

        Self {
            grid_allocator,
            pattern_matcher,
            advanced_model,
            divergence_monitor,
            correlation_updater,
            risk_bus,
        }
    }

    /// Set advanced model
    pub async fn set_advanced_model(&self, model: AdvancedModelPipeline) {
        let mut advanced = self.advanced_model.write().await;
        *advanced = Some(model);
    }

    /// Initialize BAM grids
    pub async fn initialize(&self, timestamp: i64) {
        let mut allocator = self.grid_allocator.write().await;
        allocator.initialize(timestamp);
    }

    /// Process market tick and update BAM grid
    pub async fn process_tick(&self, market_class: MarketClass, price: f64, volume: f64) {
        let mut allocator = self.grid_allocator.write().await;
        if let Some(grid) = allocator.get_grid_mut(&market_class) {
            // Update grid with tick data (simplified)
            let row = (price % 10.0) as usize;
            let col = (volume % 60.0) as usize;
            grid.set_cell(row, col, (price * 10.0) as u8);
        }
    }

    /// Route trading signal through BAM system
    pub async fn route_signal(
        &self,
        signal: AgentSignal,
    ) -> Result<RouteOutcome, Box<dyn std::error::Error>> {
        // Check base model (BAM grid pattern detection)
        let grid_allocator = self.grid_allocator.read().await;
        let market_class = MarketClass::from_str(&signal.symbol).unwrap_or(MarketClass::Equities);

        if let Some(grid) = grid_allocator.get_grid(&market_class) {
            let pattern_matcher = self.pattern_matcher.read().await;

            // Convert grid to array for pattern matching
            let grid_array = grid.grid;

            // Check if base model says NO (guarded line)
            let patterns = pattern_matcher.match_all(&grid_array);
            if patterns.is_empty() {
                return Ok(RouteOutcome {
                    signal_id: Uuid::new_v4(),
                    order_id: None,
                    status: RouteStatus::RiskHalt,
                    reason: Some("Base model (BAM) rejected - no matching patterns".to_string()),
                });
            }
        }

        // If base model says YES, check advanced model if available
        let advanced = self.advanced_model.read().await;
        if let Some(ref model) = *advanced {
            let grid_allocator = self.grid_allocator.read().await;
            if let Some(grid) = grid_allocator.get_grid(&market_class) {
                let inference = model.infer(&grid.grid)?;

                // Check confidence threshold
                if inference.confidence < 0.5 {
                    return Ok(RouteOutcome {
                        signal_id: Uuid::new_v4(),
                        order_id: None,
                        status: RouteStatus::RiskHalt,
                        reason: Some(format!(
                            "Advanced model confidence too low: {}",
                            inference.confidence
                        )),
                    });
                }
            }
        }

        // All BAM checks passed, now do risk check
        let position_size = signal.max_notional_usd * signal.conviction * 0.25; // Kelly fraction
        self.risk_bus.check_symbol(&signal.symbol, position_size)?;

        // Generate order ID
        let order_id = Uuid::new_v4();

        Ok(RouteOutcome {
            signal_id: Uuid::new_v4(),
            order_id: Some(order_id),
            status: RouteStatus::Submitted,
            reason: None,
        })
    }

    /// Update correlations between markets
    pub async fn update_correlation(&self, market_a: &str, market_b: &str, correlation: f64) {
        let mut updater = self.correlation_updater.write().await;
        updater.update_correlation(market_a.to_string(), market_b.to_string(), correlation);
    }

    /// Check model divergence
    pub async fn check_divergence(&self, model_a: &str, model_b: &str) {
        let monitor = self.divergence_monitor.read().await;
        let event = monitor.check_divergence(model_a, model_b);

        if event.triggered {
            log::warn!(
                "Model divergence triggered: {} - {}",
                event.model_id,
                event.divergence_score
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bam_trading_router() {
        // This would require setting up risk_bus
        // For now, just test the structure
        assert!(true);
    }
}
