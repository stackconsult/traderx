use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StateMachineError {
    #[error("Invalid state transition: {0} -> {1}")]
    InvalidTransition(String, String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderState {
    New,
    Pending,
    PartialFill { filled: Decimal, price: Decimal },
    Filled,
    Cancelled,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderEvent {
    ValidationPassed,
    CancelRequested,
    PartialFill { filled: Decimal, price: Decimal },
    CompleteFill { price: Decimal },
    Rejected { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: Uuid,
    pub account_id: Uuid,
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub original_quantity: Decimal,
    pub price: Option<Decimal>, // For limit orders
    pub state: OrderState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Order {
    pub fn new(
        order_id: Uuid,
        account_id: Uuid,
        symbol: String,
        side: Side,
        order_type: OrderType,
        quantity: Decimal,
    ) -> Self {
        Self {
            order_id,
            account_id,
            symbol,
            side,
            order_type,
            original_quantity: quantity,
            price: None,
            state: OrderState::New,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    pub fn apply_event(&mut self, event: OrderEvent) -> Result<String, StateMachineError> {
        let from_state = format!("{:?}", self.state);
        
        match (&self.state, event.clone()) {
            (OrderState::New, OrderEvent::ValidationPassed) => {
                self.state = OrderState::Pending;
            }
            (OrderState::Pending, OrderEvent::PartialFill { filled, price }) => {
                self.state = OrderState::PartialFill { filled, price };
            }
            (OrderState::Pending, OrderEvent::CompleteFill { price }) => {
                self.state = OrderState::PartialFill { 
                    filled: self.original_quantity, 
                    price 
                };
            }
            (OrderState::PartialFill { .. }, OrderEvent::PartialFill { filled: new_fill, price: _ }) => {
                if let OrderState::PartialFill { filled, .. } = &mut self.state {
                    *filled += new_fill;
                    if *filled >= self.original_quantity {
                        self.state = OrderState::Filled;
                    }
                }
            }
            (OrderState::PartialFill { .. }, OrderEvent::CompleteFill { price: _ }) => {
                self.state = OrderState::Filled;
            }
            (OrderState::New | OrderState::Pending, OrderEvent::CancelRequested) => {
                self.state = OrderState::Cancelled;
            }
            (OrderState::New | OrderState::Pending, OrderEvent::Rejected { .. }) => {
                self.state = OrderState::Rejected;
            }
            _ => {
                return Err(StateMachineError::InvalidTransition(
                    from_state,
                    format!("{:?}", event)
                ));
            }
        }
        
        self.updated_at = Utc::now();
        Ok(format!("{:?}", self.state))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn opposite(&self) -> Side {
        match self {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}
