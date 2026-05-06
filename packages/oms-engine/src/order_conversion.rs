// Signal to Order Conversion Pipeline
// Converts trading signals into executable orders

use crate::risk_bus::RiskBus;
use crate::signal_router::{AgentSignal, RouteOutcome, RouteStatus};
use crate::state_machine::{Order, OrderState, OrderType, Side};
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

/// Order conversion configuration
#[derive(Debug, Clone)]
pub struct OrderConversionConfig {
    pub kelly_fraction: f64,
    pub max_position_size: f64,
}

impl Default for OrderConversionConfig {
    fn default() -> Self {
        Self {
            kelly_fraction: 0.25,
            max_position_size: 100_000.0,
        }
    }
}

/// Signal to order converter
pub struct SignalToOrderConverter {
    config: OrderConversionConfig,
    risk_bus: Arc<RiskBus>,
}

impl SignalToOrderConverter {
    /// Create a new signal to order converter
    pub fn new(config: OrderConversionConfig, risk_bus: Arc<RiskBus>) -> Self {
        Self { config, risk_bus }
    }

    /// Convert a trading signal to an order
    pub fn convert_signal_to_order(
        &self,
        signal: &AgentSignal,
    ) -> Result<Order, Box<dyn std::error::Error>> {
        // Calculate position size using Kelly criterion
        let position_size =
            signal.max_notional_usd * signal.conviction * self.config.kelly_fraction;

        // Enforce position size limits
        let position_size = position_size.min(self.config.max_position_size);

        // Determine side from direction
        let side = match signal.direction.as_str() {
            "long" => Side::Buy,
            "short" => Side::Sell,
            "flat" => return Err("Flat direction does not generate orders".into()),
            _ => return Err(format!("Invalid direction: {}", signal.direction).into()),
        };

        // Calculate quantity from position size (simplified - in production, use current price)
        let price = Decimal::from(1000); // Placeholder - should fetch from market data
        let quantity = Decimal::from((position_size / 1000.0) as i64); // Simplified

        // Create order
        let order = Order::new(
            Uuid::new_v4(),
            Uuid::new_v4(), // account_id - should be from signal metadata
            signal.symbol.clone(),
            side,
            OrderType::Market,
            quantity,
        );

        // Risk check
        self.risk_bus.check_symbol(&signal.symbol, position_size)?;

        Ok(order)
    }

    /// Batch convert multiple signals to orders
    pub fn convert_signals_to_orders(
        &self,
        signals: &[AgentSignal],
    ) -> Result<Vec<Order>, Box<dyn std::error::Error>> {
        let mut orders = Vec::new();

        for signal in signals {
            match self.convert_signal_to_order(signal) {
                Ok(order) => orders.push(order),
                Err(e) => {
                    log::warn!("Failed to convert signal {}: {}", signal.symbol, e);
                    // Continue with other signals
                }
            }
        }

        Ok(orders)
    }
}

/// Order submission pipeline
pub struct OrderSubmissionPipeline {
    converter: SignalToOrderConverter,
}

impl OrderSubmissionPipeline {
    /// Create a new order submission pipeline
    pub fn new(config: OrderConversionConfig, risk_bus: Arc<RiskBus>) -> Self {
        let converter = SignalToOrderConverter::new(config, risk_bus);
        Self { converter }
    }

    /// Process a single signal and submit order
    pub async fn process_signal(
        &self,
        signal: AgentSignal,
    ) -> Result<RouteOutcome, Box<dyn std::error::Error>> {
        // Convert signal to order
        let order = self.converter.convert_signal_to_order(&signal)?;

        // In production, submit order to exchange adapter
        // For now, just return success
        Ok(RouteOutcome {
            signal_id: Uuid::new_v4(),
            order_id: Some(order.order_id),
            status: RouteStatus::Submitted,
            reason: None,
        })
    }

    /// Process multiple signals in batch
    pub async fn process_signals(
        &self,
        signals: Vec<AgentSignal>,
    ) -> Result<Vec<RouteOutcome>, Box<dyn std::error::Error>> {
        let mut outcomes = Vec::new();

        for signal in signals {
            match self.process_signal(signal).await {
                Ok(outcome) => outcomes.push(outcome),
                Err(e) => {
                    log::error!("Failed to process signal: {}", e);
                    // Continue with other signals
                }
            }
        }

        Ok(outcomes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_to_order_conversion() {
        let config = OrderConversionConfig::default();
        let risk_bus = Arc::new(RiskBus::new(1_000_000.0, -2000));
        let converter = SignalToOrderConverter::new(config, risk_bus);

        let signal = AgentSignal {
            agent_id: "test_agent".to_string(),
            symbol: "AAPL".to_string(),
            direction: "long".to_string(),
            conviction: 0.7,
            max_notional_usd: 10_000.0,
        };

        let result = converter.convert_signal_to_order(&signal);
        assert!(result.is_ok());

        let order = result.unwrap();
        assert_eq!(order.symbol, "AAPL");
        assert_eq!(order.side, Side::Buy);
    }

    #[test]
    fn test_invalid_direction() {
        let config = OrderConversionConfig::default();
        let risk_bus = Arc::new(RiskBus::new(1_000_000.0, -2000));
        let converter = SignalToOrderConverter::new(config, risk_bus);

        let signal = AgentSignal {
            agent_id: "test_agent".to_string(),
            symbol: "AAPL".to_string(),
            direction: "invalid".to_string(),
            conviction: 0.7,
            max_notional_usd: 10_000.0,
        };

        let result = converter.convert_signal_to_order(&signal);
        assert!(result.is_err());
    }
}
