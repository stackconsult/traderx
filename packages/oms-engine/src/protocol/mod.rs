pub mod sbe;
pub mod itch;

pub use sbe::SBEProtocol;
pub use itch::ITCHProtocol;

use crate::state_machine::{Order, OrderType, Side};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Invalid message format")]
    InvalidFormat,
    #[error("Unsupported message type")]
    UnsupportedType,
    #[error("Encoding error: {0}")]
    Encoding(String),
    #[error("Decoding error: {0}")]
    Decoding(String),
}

/// Common protocol trait
pub trait OrderProtocol: Send + Sync {
    /// Encode order to binary format
    fn encode_order(&self, order: &Order) -> Result<Vec<u8>, ProtocolError>;
    
    /// Decode order from binary format
    fn decode_order(&self, data: &[u8]) -> Result<Order, ProtocolError>;
    
    /// Get protocol name
    fn name(&self) -> &'static str;
}

/// Standard order frame for network transmission
#[derive(Debug, Clone)]
pub struct OrderFrame {
    /// Order ID
    pub order_id: Uuid,
    
    /// Account ID
    pub account_id: Uuid,
    
    /// Symbol
    pub symbol: String,
    
    /// Side
    pub side: Side,
    
    /// Order type
    pub order_type: OrderType,
    
    /// Quantity
    pub quantity: Decimal,
    
    /// Price (for limit orders)
    pub price: Option<Decimal>,
    
    /// Time in force
    pub time_in_force: TimeInForce,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Checksum for integrity
    pub checksum: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeInForce {
    Gtc,  // Good Till Canceled
    Ioc,  // Immediate or Cancel
    Fok,  // Fill or Kill
}

impl From<Order> for OrderFrame {
    fn from(order: Order) -> Self {
        Self {
            order_id: order.order_id,
            account_id: order.account_id,
            symbol: order.symbol,
            side: order.side,
            order_type: order.order_type,
            quantity: order.original_quantity,
            price: None, // Would need to track current price
            time_in_force: TimeInForce::Gtc,
            timestamp: order.created_at,
            checksum: 0, // Would calculate actual checksum
        }
    }
}

impl From<OrderFrame> for Order {
    fn from(frame: OrderFrame) -> Self {
        Self {
            order_id: frame.order_id,
            account_id: frame.account_id,
            symbol: frame.symbol,
            side: frame.side,
            order_type: frame.order_type,
            original_quantity: frame.quantity,
            state: crate::state_machine::OrderState::New,
            created_at: frame.timestamp,
            updated_at: frame.timestamp,
        }
    }
}
