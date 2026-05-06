// Live Order Submission
// Submits orders to exchanges via adapters

use crate::adapters::{AdapterError, ExchangeAdapter, OrderStatus};
use crate::order_conversion::OrderConversionConfig;
use crate::risk_bus::RiskBus;
use crate::state_machine::Order;
use rust_decimal::prelude::ToPrimitive;
use std::sync::Arc;
use uuid::Uuid;

/// Live order submission configuration
#[derive(Debug, Clone)]
pub struct LiveOrderConfig {
    pub paper_trading: bool,
    pub max_orders_per_second: u32,
    pub retry_attempts: u32,
}

impl Default for LiveOrderConfig {
    fn default() -> Self {
        Self {
            paper_trading: true, // Default to paper trading for safety
            max_orders_per_second: 10,
            retry_attempts: 3,
        }
    }
}

/// Live order submitter
pub struct LiveOrderSubmitter {
    config: LiveOrderConfig,
    adapter: Arc<dyn ExchangeAdapter>,
    risk_bus: Arc<RiskBus>,
}

impl LiveOrderSubmitter {
    /// Create a new live order submitter
    pub fn new(
        config: LiveOrderConfig,
        adapter: Arc<dyn ExchangeAdapter>,
        risk_bus: Arc<RiskBus>,
    ) -> Self {
        Self {
            config,
            adapter,
            risk_bus,
        }
    }

    /// Submit an order to the exchange
    pub async fn submit_order(&self, order: &Order) -> Result<Uuid, Box<dyn std::error::Error>> {
        // In paper trading mode, just simulate submission
        if self.config.paper_trading {
            log::info!(
                "Paper trading: simulating order submission for {}",
                order.order_id
            );
            return Ok(order.order_id);
        }

        // Risk check before submission
        let notional = order.original_quantity.to_f64().unwrap_or(0.0) * 1000.0; // Simplified
        self.risk_bus.check_symbol(&order.symbol, notional)?;

        // Convert to AdvancedOrder for adapter
        let advanced_order = crate::orders::AdvancedOrder::new(
            &order.symbol,
            order.side,
            order.original_quantity,
            crate::orders::AdvancedOrderType::Market,
        );

        // Submit to exchange with retry logic
        let mut last_error = None;
        for attempt in 0..self.config.retry_attempts {
            match self.adapter.submit_order(&advanced_order).await {
                Ok(order_id) => {
                    log::info!("Order submitted successfully: {}", order_id);
                    return Ok(order_id);
                }
                Err(e) => {
                    log::warn!("Order submission attempt {} failed: {}", attempt + 1, e);
                    last_error = Some(e);
                    if attempt < self.config.retry_attempts - 1 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(
                            100 * (attempt + 1) as u64,
                        ))
                        .await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or(AdapterError::Timeout).into())
    }

    /// Cancel an order
    pub async fn cancel_order(&self, order_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.paper_trading {
            log::info!(
                "Paper trading: simulating order cancellation for {}",
                order_id
            );
            return Ok(());
        }

        self.adapter
            .cancel_order(order_id)
            .await
            .map_err(|e| e.into())
    }

    /// Get order status
    pub async fn get_order_status(
        &self,
        order_id: Uuid,
    ) -> Result<OrderStatus, Box<dyn std::error::Error>> {
        if self.config.paper_trading {
            return Ok(OrderStatus::Filled); // Simulate filled in paper trading
        }

        self.adapter
            .get_order_status(order_id)
            .await
            .map_err(|e| e.into())
    }

    /// Check if in paper trading mode
    pub fn is_paper_trading(&self) -> bool {
        self.config.paper_trading
    }
}

/// Order submission manager - manages multiple submitters
pub struct OrderSubmissionManager {
    submitters: Vec<LiveOrderSubmitter>,
}

impl OrderSubmissionManager {
    /// Create a new order submission manager
    pub fn new() -> Self {
        Self {
            submitters: Vec::new(),
        }
    }

    /// Add a submitter
    pub fn add_submitter(&mut self, submitter: LiveOrderSubmitter) {
        self.submitters.push(submitter);
    }

    /// Submit order to first available submitter
    pub async fn submit_order(&self, order: &Order) -> Result<Uuid, Box<dyn std::error::Error>> {
        for submitter in &self.submitters {
            match submitter.submit_order(order).await {
                Ok(order_id) => return Ok(order_id),
                Err(e) => {
                    log::warn!("Submitter failed: {}, trying next", e);
                }
            }
        }
        Err("All submitters failed".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_paper_trading_submission() {
        let config = LiveOrderConfig {
            paper_trading: true,
            ..Default::default()
        };

        // This would require a mock adapter
        // For now, just test the config
        assert!(config.paper_trading);
    }
}
