// Paper Trading Mode
// Simulates trading without real money for testing strategies

use crate::order_conversion::OrderConversionConfig;
use crate::risk_bus::RiskBus;
use crate::state_machine::{Order, OrderState, Side};
use chrono::Utc;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Paper trading configuration
#[derive(Debug, Clone)]
pub struct PaperTradingConfig {
    pub initial_balance: f64,
    pub slippage_bps: i32,   // Basis points of slippage
    pub commission_bps: i32, // Basis points commission
}

impl Default for PaperTradingConfig {
    fn default() -> Self {
        Self {
            initial_balance: 1_000_000.0,
            slippage_bps: 5,    // 0.05% slippage
            commission_bps: 10, // 0.1% commission
        }
    }
}

/// Paper trading account
#[derive(Debug, Clone)]
pub struct PaperAccount {
    pub balance: f64,
    pub positions: HashMap<String, f64>, // symbol -> quantity
    pub unrealized_pnl: f64,
}

impl PaperAccount {
    fn new(initial_balance: f64) -> Self {
        Self {
            balance: initial_balance,
            positions: HashMap::new(),
            unrealized_pnl: 0.0,
        }
    }
}

/// Paper trading engine
pub struct PaperTradingEngine {
    config: PaperTradingConfig,
    account: PaperAccount,
    orders: HashMap<Uuid, Order>,
    fills: Vec<PaperFill>,
}

/// Simulated fill
#[derive(Debug, Clone)]
pub struct PaperFill {
    pub order_id: Uuid,
    pub fill_id: Uuid,
    pub symbol: String,
    pub side: Side,
    pub price: f64,
    pub quantity: f64,
    pub commission: f64,
    pub timestamp: i64,
}

impl PaperTradingEngine {
    /// Create a new paper trading engine
    pub fn new(config: PaperTradingConfig) -> Self {
        let account = PaperAccount::new(config.initial_balance);

        Self {
            config,
            account,
            orders: HashMap::new(),
            fills: Vec::new(),
        }
    }

    /// Submit an order (simulated)
    pub fn submit_order(&mut self, order: Order) -> Result<Uuid, Box<dyn std::error::Error>> {
        let order_id = order.order_id;

        // Store order
        self.orders.insert(order_id, order.clone());

        // Simulate immediate fill for market orders
        if order.order_type == crate::state_machine::OrderType::Market {
            self.simulate_fill(&order)?;
        }

        Ok(order_id)
    }

    /// Simulate a fill
    fn simulate_fill(&mut self, order: &Order) -> Result<(), Box<dyn std::error::Error>> {
        // Get current price (simplified - in production, fetch from market data)
        let price = 1000.0; // Placeholder

        // Apply slippage
        let slippage = price * (self.config.slippage_bps as f64 / 10000.0);
        let fill_price = match order.side {
            Side::Buy => price + slippage,
            Side::Sell => price - slippage,
        };

        // Calculate quantity
        let quantity = order.original_quantity.to_f64().unwrap_or(0.0);

        // Calculate commission
        let notional = fill_price * quantity;
        let commission = notional * (self.config.commission_bps as f64 / 10000.0);

        // Update account
        match order.side {
            Side::Buy => {
                self.account.balance -= notional + commission;
                *self
                    .account
                    .positions
                    .entry(order.symbol.clone())
                    .or_insert(0.0) += quantity;
            }
            Side::Sell => {
                self.account.balance += notional - commission;
                *self
                    .account
                    .positions
                    .entry(order.symbol.clone())
                    .or_insert(0.0) -= quantity;
            }
        }

        // Record fill
        let fill = PaperFill {
            order_id: order.order_id,
            fill_id: Uuid::new_v4(),
            symbol: order.symbol.clone(),
            side: order.side,
            price: fill_price,
            quantity,
            commission,
            timestamp: Utc::now().timestamp_millis(),
        };

        self.fills.push(fill);

        Ok(())
    }

    /// Get account balance
    pub fn get_balance(&self) -> f64 {
        self.account.balance
    }

    /// Get position for a symbol
    pub fn get_position(&self, symbol: &str) -> f64 {
        *self.account.positions.get(symbol).unwrap_or(&0.0)
    }

    /// Get all positions
    pub fn get_positions(&self) -> &HashMap<String, f64> {
        &self.account.positions
    }

    /// Get fills
    pub fn get_fills(&self) -> &[PaperFill] {
        &self.fills
    }

    /// Reset account to initial state
    pub fn reset(&mut self) {
        self.account = PaperAccount::new(self.config.initial_balance);
        self.orders.clear();
        self.fills.clear();
    }

    /// Calculate total P&L
    pub fn calculate_pnl(&self) -> f64 {
        let initial_balance = self.config.initial_balance;
        let current_balance = self.account.balance;

        // Add unrealized P&L from positions (simplified)
        let unrealized: f64 = self
            .account
            .positions
            .iter()
            .map(|(symbol, quantity)| {
                if *quantity != 0.0 {
                    let current_price = 1000.0; // Placeholder
                    quantity * current_price
                } else {
                    0.0
                }
            })
            .sum();

        (current_balance + unrealized) - initial_balance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paper_trading() {
        let config = PaperTradingConfig::default();
        let mut engine = PaperTradingEngine::new(config);

        assert_eq!(engine.get_balance(), 1_000_000.0);

        let order = Order::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "AAPL".to_string(),
            Side::Buy,
            crate::state_machine::OrderType::Market,
            Decimal::from(10),
        );

        let result = engine.submit_order(order);
        assert!(result.is_ok());

        assert!(engine.get_balance() < 1_000_000.0);
        assert_eq!(engine.get_position("AAPL"), 10.0);
    }
}
