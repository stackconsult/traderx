//! Advanced Order Types - Benchmark+ Implementation
//! 
//! Implements production-grade order types from NautilusTrader:
//! - IOC: Immediate or Cancel
//! - FOK: Fill or Kill  
//! - GTD: Good Till Date
//! - DAY: Day Order
//! - AT_OPEN: At The Opening
//! - AT_CLOSE: At The Close
//! - Iceberg: Hidden quantity orders
//! - OCO: One Cancels Other
//! - OUO: One Updates Other
//! - OTO: One Triggers Other

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state_machine::{Order, Side};
use uuid::Uuid as OrderId;

/// Extended time-in-force options beyond standard GTC
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeInForce {
    /// Good Till Cancel - remains open until filled or cancelled
    GTC,
    /// Immediate or Cancel - fills immediately, cancels remainder
    IOC,
    /// Fill or Kill - fills completely immediately or cancels
    FOK,
    /// Good Till Date - expires at specified datetime
    GTD(chrono::DateTime<chrono::Utc>),
    /// Day Order - expires at market close
    DAY,
    /// At The Opening - executes at market open
    AT_OPEN,
    /// At The Close - executes at market close
    AT_CLOSE,
}

impl Default for TimeInForce {
    fn default() -> Self {
        TimeInForce::GTC
    }
}

/// Advanced order types with hidden/conditional quantities
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdvancedOrderType {
    /// Standard market order
    Market,
    /// Limit order with price
    Limit { price: Decimal },
    /// Stop order triggers at price
    Stop { trigger_price: Decimal },
    /// Stop-limit combines stop and limit
    StopLimit { 
        trigger_price: Decimal, 
        limit_price: Decimal 
    },
    /// Iceberg order with visible/hidden quantity
    Iceberg { 
        visible_qty: Decimal,
        hidden_qty: Decimal,
    },
    /// Trailing stop with dynamic trigger
    TrailingStop { 
        distance: Decimal,  // Absolute or percentage
        is_percentage: bool,
    },
}

impl Default for AdvancedOrderType {
    fn default() -> Self {
        AdvancedOrderType::Market
    }
}

/// Execution restrictions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionRestriction {
    /// No restrictions
    None,
    /// Post-only - add liquidity, don't take
    PostOnly,
    /// Reduce-only - only reduce position, not increase
    ReduceOnly,
    /// Both post-only and reduce-only
    PostOnlyReduceOnly,
}

impl Default for ExecutionRestriction {
    fn default() -> Self {
        ExecutionRestriction::None
    }
}

/// Contingency order types - linked orders
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContingencyType {
    /// No contingency
    None,
    /// One-Cancels-Other: if one fills, cancel the other
    OCO { 
        partner_order_id: OrderId 
    },
    /// One-Updates-Other: if one fills, update the other's quantity
    OUO { 
        partner_order_id: OrderId,
        ratio: Decimal,  // Ratio to update partner quantity
    },
    /// One-Triggers-Other: if one fills, submit another order
    OTO { 
        child_order: Box<AdvancedOrder>,
    },
    /// Bracket: entry + stop-loss + take-profit
    Bracket {
        stop_loss: Decimal,
        take_profit: Decimal,
    },
}

impl Default for ContingencyType {
    fn default() -> Self {
        ContingencyType::None
    }
}

/// Advanced order with all production features
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdvancedOrder {
    /// Unique order identifier
    pub id: OrderId,
    /// Parent order ID for contingent orders
    pub parent_id: Option<OrderId>,
    /// Symbol being traded
    pub symbol: String,
    /// Order side (buy/sell)
    pub side: Side,
    /// Total order quantity
    pub quantity: Decimal,
    /// Filled quantity
    pub filled_qty: Decimal,
    /// Order type with price/trigger info
    pub order_type: AdvancedOrderType,
    /// Time in force
    pub time_in_force: TimeInForce,
    /// Execution restrictions
    pub restriction: ExecutionRestriction,
    /// Contingency relationship
    pub contingency: ContingencyType,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Expiration timestamp (for GTD)
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Client order ID (for exchange tracking)
    pub client_order_id: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
}

impl AdvancedOrder {
    /// Create a new advanced order
    pub fn new(
        symbol: impl Into<String>,
        side: Side,
        quantity: Decimal,
        order_type: AdvancedOrderType,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            parent_id: None,
            symbol: symbol.into(),
            side,
            quantity,
            filled_qty: Decimal::ZERO,
            order_type,
            time_in_force: TimeInForce::default(),
            restriction: ExecutionRestriction::default(),
            contingency: ContingencyType::default(),
            created_at: chrono::Utc::now(),
            expires_at: None,
            client_order_id: None,
            tags: Vec::new(),
        }
    }

    /// Set time in force (fluent API)
    pub fn with_tif(mut self, tif: TimeInForce) -> Self {
        self.time_in_force = tif;
        if let TimeInForce::GTD(expiry) = tif {
            self.expires_at = Some(expiry);
        }
        self
    }

    /// Set execution restriction (fluent API)
    pub fn with_restriction(mut self, restriction: ExecutionRestriction) -> Self {
        self.restriction = restriction;
        self
    }

    /// Set post-only flag
    pub fn post_only(mut self) -> Self {
        self.restriction = match self.restriction {
            ExecutionRestriction::ReduceOnly | ExecutionRestriction::PostOnlyReduceOnly 
                => ExecutionRestriction::PostOnlyReduceOnly,
            _ => ExecutionRestriction::PostOnly,
        };
        self
    }

    /// Set reduce-only flag
    pub fn reduce_only(mut self) -> Self {
        self.restriction = match self.restriction {
            ExecutionRestriction::PostOnly | ExecutionRestriction::PostOnlyReduceOnly 
                => ExecutionRestriction::PostOnlyReduceOnly,
            _ => ExecutionRestriction::ReduceOnly,
        };
        self
    }

    /// Set OCO contingency
    pub fn oco(mut self, partner_id: OrderId) -> Self {
        self.contingency = ContingencyType::OCO { partner_order_id: partner_id };
        self
    }

    /// Set bracket order (entry + stop + target)
    pub fn bracket(mut self, stop_loss: Decimal, take_profit: Decimal) -> Self {
        self.contingency = ContingencyType::Bracket { stop_loss, take_profit };
        self
    }

    /// Add tag for categorization
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Check if order is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            return chrono::Utc::now() > expires;
        }
        false
    }

    /// Check if order is completely filled
    pub fn is_filled(&self) -> bool {
        self.filled_qty >= self.quantity
    }

    /// Check if order is partially filled
    pub fn is_partially_filled(&self) -> bool {
        self.filled_qty > Decimal::ZERO && self.filled_qty < self.quantity
    }

    /// Get remaining quantity to fill
    pub fn remaining_qty(&self) -> Decimal {
        self.quantity - self.filled_qty
    }

    /// Check if order can be filled at given price (for limit orders)
    pub fn can_fill_at(&self, market_price: Decimal) -> bool {
        match &self.order_type {
            AdvancedOrderType::Market => true,
            AdvancedOrderType::Limit { price } => {
                match self.side {
                    Side::Buy => market_price <= *price,
                    Side::Sell => market_price >= *price,
                }
            }
            _ => false, // Stop orders need trigger logic
        }
    }

    /// Convert to base Order for backward compatibility
    pub fn to_base_order(&self) -> Order {
        Order {
            id: self.id,
            symbol: self.symbol.clone(),
            side: self.side,
            quantity: self.quantity,
            // Map advanced fields to base order as needed
            ..Default::default()
        }
    }
}

/// Order builder for complex order construction
pub struct AdvancedOrderBuilder {
    order: AdvancedOrder,
}

impl AdvancedOrderBuilder {
    /// Start building a market order
    pub fn market(symbol: impl Into<String>, side: Side, quantity: Decimal) -> Self {
        Self {
            order: AdvancedOrder::new(symbol, side, quantity, AdvancedOrderType::Market),
        }
    }

    /// Start building a limit order
    pub fn limit(symbol: impl Into<String>, side: Side, quantity: Decimal, price: Decimal) -> Self {
        Self {
            order: AdvancedOrder::new(symbol, side, quantity, AdvancedOrderType::Limit { price }),
        }
    }

    /// Start building an iceberg order
    pub fn iceberg(
        symbol: impl Into<String>, 
        side: Side, 
        visible_qty: Decimal,
        hidden_qty: Decimal
    ) -> Self {
        Self {
            order: AdvancedOrder::new(
                symbol, 
                side, 
                visible_qty + hidden_qty,
                AdvancedOrderType::Iceberg { visible_qty, hidden_qty }
            ),
        }
    }

    /// Set time in force
    pub fn tif(mut self, tif: TimeInForce) -> Self {
        self.order = self.order.with_tif(tif);
        self
    }

    /// Set IOC
    pub fn ioc(self) -> Self {
        self.tif(TimeInForce::IOC)
    }

    /// Set FOK
    pub fn fok(self) -> Self {
        self.tif(TimeInForce::FOK)
    }

    /// Set DAY
    pub fn day(self) -> Self {
        self.tif(TimeInForce::DAY)
    }

    /// Set GTD
    pub fn gtd(self, expiry: chrono::DateTime<chrono::Utc>) -> Self {
        self.tif(TimeInForce::GTD(expiry))
    }

    /// Set post-only
    pub fn post_only(mut self) -> Self {
        self.order = self.order.post_only();
        self
    }

    /// Set reduce-only
    pub fn reduce_only(mut self) -> Self {
        self.order = self.order.reduce_only();
        self
    }

    /// Set OCO
    pub fn oco(mut self, partner_id: OrderId) -> Self {
        self.order = self.order.oco(partner_id);
        self
    }

    /// Set bracket
    pub fn bracket(mut self, stop_loss: Decimal, take_profit: Decimal) -> Self {
        self.order = self.order.bracket(stop_loss, take_profit);
        self
    }

    /// Add tag
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.order = self.order.with_tag(tag);
        self
    }

    /// Build the order
    pub fn build(self) -> AdvancedOrder {
        self.order
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_order_builder() {
        let order = AdvancedOrderBuilder::market("AAPL", Side::Buy, Decimal::from(100))
            .ioc()
            .post_only()
            .tag("momentum")
            .build();

        assert_eq!(order.symbol, "AAPL");
        assert_eq!(order.side, Side::Buy);
        assert_eq!(order.quantity, Decimal::from(100));
        assert_eq!(order.time_in_force, TimeInForce::IOC);
        assert_eq!(order.restriction, ExecutionRestriction::PostOnly);
        assert!(order.tags.contains(&"momentum".to_string()));
    }

    #[test]
    fn test_limit_order_can_fill() {
        let order = AdvancedOrderBuilder::limit("AAPL", Side::Buy, Decimal::from(100), Decimal::from(150))
            .build();

        assert!(order.can_fill_at(Decimal::from(149)));  // Below limit
        assert!(order.can_fill_at(Decimal::from(150)));  // At limit
        assert!(!order.can_fill_at(Decimal::from(151))); // Above limit
    }

    #[test]
    fn test_iceberg_order() {
        let order = AdvancedOrderBuilder::iceberg("AAPL", Side::Sell, Decimal::from(10), Decimal::from(90))
            .day()
            .build();

        assert_eq!(order.quantity, Decimal::from(100)); // Total
        assert_eq!(order.time_in_force, TimeInForce::DAY);
    }

    #[test]
    fn test_bracket_order() {
        let order = AdvancedOrder::new("AAPL", Side::Buy, Decimal::from(100), AdvancedOrderType::Market)
            .bracket(Decimal::from(90), Decimal::from(110));

        match order.contingency {
            ContingencyType::Bracket { stop_loss, take_profit } => {
                assert_eq!(stop_loss, Decimal::from(90));
                assert_eq!(take_profit, Decimal::from(110));
            }
            _ => panic!("Expected Bracket contingency"),
        }
    }

    #[test]
    fn test_fok_order() {
        let order = AdvancedOrder::new("AAPL", Side::Buy, Decimal::from(100), AdvancedOrderType::Market)
            .with_tif(TimeInForce::FOK);

        assert_eq!(order.time_in_force, TimeInForce::FOK);
    }
}
