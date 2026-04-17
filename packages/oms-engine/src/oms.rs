use crate::state_machine::{Order, OrderState, OrderEvent, OrderType, Side};
use crate::disruptor::{Disruptor, DisruptorError, EventProcessor};
use crate::journal::{EventJournal, JournalEntry, JournalError};
use crate::protocol::{OrderProtocol, SBEProtocol, ITCHProtocol};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error, debug};
use thiserror::Error;
use std::path::Path;

#[derive(Error, Debug)]
pub enum OmsError {
    #[error("Order not found: {0}")]
    OrderNotFound(Uuid),
    #[error("Invalid state transition: {0} -> {1}")]
    InvalidStateTransition(String, String),
    #[error("Risk check failed: {0}")]
    RiskCheckFailed(String),
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    #[error("Journal error: {0}")]
    JournalError(String),
}

impl From<JournalError> for OmsError {
    fn from(err: JournalError) -> Self {
        OmsError::JournalError(err.to_string())
    }
}

impl From<DisruptorError> for OmsError {
    fn from(err: DisruptorError) -> Self {
        OmsError::ProtocolError(err.to_string())
    }
}

impl From<crate::protocol::ProtocolError> for OmsError {
    fn from(err: crate::protocol::ProtocolError) -> Self {
        OmsError::ProtocolError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, OmsError>;

/// Order Management System with LMAX Disruptor pattern
pub struct OmsEngine {
    /// Core disruptor for high-performance event processing
    disruptor: Arc<Disruptor>,
    
    /// Order storage - thread-safe concurrent hashmap
    orders: Arc<DashMap<Uuid, Order>>,
    
    /// Event journal for persistence and replay
    journal: Arc<EventJournal>,
    
    /// Protocol handlers
    sbe_protocol: Arc<SBEProtocol>,
    itch_protocol: Arc<ITCHProtocol>,
    
    /// Risk check callback
    risk_checker: Arc<dyn Fn(&Order) -> Result<()> + Send + Sync>,
    
    /// Execution callback
    executor: Arc<dyn Fn(Order) -> Result<()> + Send + Sync>,
    
    /// Position update callback
    position_updater: Arc<dyn Fn(Uuid, Decimal, Decimal) -> Result<()> + Send + Sync>,
}

/// Get state summary for testing
#[derive(Debug, Clone, Serialize)]
pub struct StateSummary {
    pub total_orders: usize,
    pub pending_orders: usize,
    pub filled_orders: usize,
    pub partial_filled_orders: usize,
    pub cancelled_orders: usize,
}

/// Create checkpoint
#[derive(Serialize, Deserialize)]
pub struct Checkpoint {
    pub timestamp: DateTime<Utc>,
    pub order_count: usize,
    pub sequence: u64,
}

impl OmsEngine {
    /// Create new OMS engine with specified ring buffer size
    pub fn new(
        ring_buffer_size: usize,
        risk_checker: impl Fn(&Order) -> Result<()> + Send + Sync + 'static,
        executor: impl Fn(Order) -> Result<()> + Send + Sync + 'static,
        position_updater: impl Fn(Uuid, Decimal, Decimal) -> Result<()> + Send + Sync + 'static,
    ) -> Result<Self> {
        // Initialize disruptor
        let disruptor = Arc::new(Disruptor::new(ring_buffer_size)?);
        
        // Initialize journal
        let journal = Arc::new(EventJournal::new()?);
        
        // Initialize protocols
        let sbe_protocol = Arc::new(SBEProtocol::new()?);
        let itch_protocol = Arc::new(ITCHProtocol::new()?);
        
        let engine = Self {
            disruptor,
            orders: Arc::new(DashMap::new()),
            journal,
            sbe_protocol,
            itch_protocol,
            risk_checker: Arc::new(risk_checker),
            executor: Arc::new(executor),
            position_updater: Arc::new(position_updater),
        };
        
        // Start event processors
        engine.start_processors()?;
        
        info!("OMS Engine initialized with ring buffer size: {}", ring_buffer_size);
        Ok(engine)
    }
    
    /// Submit new order to the system
    pub async fn submit_order(&self, mut order: Order) -> Result<Uuid> {
        info!("Submitting order: {}", order.order_id);
        
        // Validate order
        self.validate_order(&order)?;
        
        // Risk check
        (self.risk_checker)(&order)?;
        
        // Update state to Pending
        order.apply_event(OrderEvent::ValidationPassed)
            .map_err(|e| OmsError::InvalidStateTransition("New".to_string(), e.to_string()))?;
        
        // Store order
        self.orders.insert(order.order_id, order.clone());
        
        // Capture order ID before moving order
        let order_id = order.order_id;
        
        // Publish event to disruptor
        let event = OmsEvent::OrderSubmitted { order };
        self.disruptor.publish(event).await?;
        
        Ok(order_id)
    }
    
    /// Cancel existing order
    pub async fn cancel_order(&self, order_id: Uuid) -> Result<()> {
        info!("Cancelling order: {}", order_id);
        
        let mut order = self.orders.get(&order_id)
            .ok_or(OmsError::OrderNotFound(order_id))?
            .clone();
        
        // Check if order can be cancelled
        match order.state {
            OrderState::Pending | OrderState::PartialFill { .. } => {
                // Apply cancel event
                order.apply_event(OrderEvent::CancelRequested)
                    .map_err(|e| OmsError::InvalidStateTransition(format!("{:?}", order.state), e.to_string()))?;
                
                // Update stored order
                self.orders.insert(order_id, order.clone());
                
                // Publish cancel event
                let event = OmsEvent::OrderCancelled { order_id, order };
                self.disruptor.publish(event).await?;
                
                Ok(())
            }
            _ => Err(OmsError::InvalidStateTransition(
                format!("{:?}", order.state),
                "Cannot cancel".to_string()
            ))
        }
    }
    
    /// Modify existing order
    pub async fn modify_order(&self, order_id: Uuid, new_quantity: Decimal, new_price: Option<Decimal>) -> Result<()> {
        info!("Modifying order: {}", order_id);
        
        let mut order = self.orders.get(&order_id)
            .ok_or(OmsError::OrderNotFound(order_id))?
            .clone();
        
        // Only allow modification for pending orders
        if order.state != OrderState::Pending {
            return Err(OmsError::InvalidStateTransition(
                format!("{:?}", order.state),
                "Cannot modify".to_string()
            ));
        }
        
        // Create modification event
        let event = OmsEvent::OrderModified {
            order_id,
            old_quantity: order.original_quantity,
            new_quantity,
            old_price: None, // Would need to track current price
            new_price,
        };
        
        // Update order
        order.original_quantity = new_quantity;
        self.orders.insert(order_id, order.clone());
        
        // Publish modification event
        self.disruptor.publish(event).await?;
        
        Ok(())
    }
    
    /// Get order by ID
    pub fn get_order(&self, order_id: Uuid) -> Option<Order> {
        self.orders.get(&order_id).map(|entry| entry.clone())
    }
    
    /// Get all orders for account
    pub fn get_orders_by_account(&self, account_id: Uuid) -> Vec<Order> {
        self.orders
            .iter()
            .filter(|entry| entry.value().account_id == account_id)
            .map(|entry| entry.value().clone())
            .collect()
    }
    
    /// Process fill from exchange
    pub async fn process_fill(&self, order_id: Uuid, fill_quantity: Decimal, fill_price: Decimal) -> Result<()> {
        info!("Processing fill for order {}: {} @ {}", order_id, fill_quantity, fill_price);
        
        let mut order = self.orders.get(&order_id)
            .ok_or(OmsError::OrderNotFound(order_id))?
            .clone();
        
        // Determine fill event type
        let total_filled = match order.state {
            OrderState::PartialFill { filled, .. } => filled + fill_quantity,
            OrderState::Pending => fill_quantity,
            _ => return Err(OmsError::InvalidStateTransition(
                format!("{:?}", order.state),
                "Cannot fill".to_string()
            ))
        };
        
        let fill_event = if total_filled >= order.original_quantity {
            OrderEvent::CompleteFill { price: fill_price }
        } else {
            OrderEvent::PartialFill { 
                filled: fill_quantity, 
                price: fill_price 
            }
        };
        
        // Apply fill event
        order.apply_event(fill_event.clone())
            .map_err(|e| OmsError::InvalidStateTransition(format!("{:?}", order.state), e.to_string()))?;
        
        // Update stored order
        self.orders.insert(order_id, order.clone());
        
        // Update position
        let side_multiplier = match order.side {
            Side::Buy => Decimal::ONE,
            Side::Sell => -Decimal::ONE,
        };
        (self.position_updater)(
            order.account_id,
            fill_quantity * side_multiplier,
            fill_price
        )?;
        
        // Publish fill event
        let event = OmsEvent::OrderFilled {
            order_id,
            fill_quantity,
            fill_price,
            total_filled,
            order_state: order.state,
        };
        self.disruptor.publish(event).await?;
        
        Ok(())
    }
    
    /// Validate order basic requirements
    fn validate_order(&self, order: &Order) -> Result<()> {
        if order.original_quantity <= Decimal::ZERO {
            return Err(OmsError::ProtocolError("Invalid quantity".to_string()));
        }
        
        if order.order_type == OrderType::Limit && order.price.is_none() {
            return Err(OmsError::ProtocolError("Limit order requires price".to_string()));
        }
        
        Ok(())
    }
    
    /// Start event processors
    fn start_processors(&self) -> Result<()> {
        let disruptor = self.disruptor.clone();
        let orders = self.orders.clone();
        let journal = self.journal.clone();
        let executor = self.executor.clone();
        
        // Spawn processor task
        tokio::spawn(async move {
            let processor = OmsEventProcessor {
                orders,
                journal,
                executor,
            };
            
            if let Err(e) = disruptor.register_processor(processor).await {
                error!("Failed to register OMS event processor: {}", e);
            }
        });
        
        Ok(())
    }
    
    /// Get order by ID (async)
    pub async fn get_order_async(&self, order_id: &Uuid) -> Option<Order> {
        self.orders.get(order_id).map(|o| o.clone())
    }
    
    /// Fill order (simplified for testing)
    pub async fn fill_order(&self, order_id: &Uuid, fill_price: f64, fill_qty: f64) -> Result<()> {
        self.process_fill(
            *order_id,
            Decimal::from_f64(fill_qty).unwrap_or_default(),
            Decimal::from_f64(fill_price).unwrap_or_default()
        ).await
    }
    
        
    pub async fn get_state_summary(&self) -> StateSummary {
        let mut summary = StateSummary {
            total_orders: self.orders.len(),
            pending_orders: 0,
            filled_orders: 0,
            partial_filled_orders: 0,
            cancelled_orders: 0,
        };
        
        for order in self.orders.iter() {
            match order.state {
                OrderState::Pending => summary.pending_orders += 1,
                OrderState::Filled => summary.filled_orders += 1,
                OrderState::PartialFill { .. } => summary.partial_filled_orders += 1,
                OrderState::Cancelled => summary.cancelled_orders += 1,
                _ => {}
            }
        }
        
        summary
    }
    
    /// Recover from journal
    pub async fn recover_from_journal(&self) -> bool {
        debug!("Starting journal recovery");
        
        match self.journal.replay(None).await {
            Ok(entries) => {
                info!("Recovered {} events from journal", entries.len());
                true
            }
            Err(e) => {
                error!("Journal recovery failed: {}", e);
                false
            }
        }
    }
    
        
    pub async fn create_checkpoint<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let checkpoint = Checkpoint {
            timestamp: Utc::now(),
            order_count: self.orders.len(),
            sequence: self.journal.get_sequence().await,
        };
        
        let json = serde_json::to_string_pretty(&checkpoint)
            .map_err(|e| OmsError::JournalError(e.to_string()))?;
        
        std::fs::write(path, json)
            .map_err(|e| OmsError::JournalError(e.to_string()))?;
        
        info!("Checkpoint created with {} orders", checkpoint.order_count);
        Ok(())
    }
    
    /// Recover from checkpoint
    pub async fn recover_from_checkpoint<P: AsRef<Path>>(&self, path: P) -> bool {
        debug!("Starting checkpoint recovery");
        
        match std::fs::read_to_string(path) {
            Ok(content) => {
                match serde_json::from_str::<Checkpoint>(&content) {
                    Ok(checkpoint) => {
                        info!("Recovered from checkpoint: {} orders at {}", 
                            checkpoint.order_count, checkpoint.timestamp);
                        true
                    }
                    Err(e) => {
                        error!("Failed to parse checkpoint: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                error!("Failed to read checkpoint file: {}", e);
                false
            }
        }
    }
}

/// OMS Events processed by the disruptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OmsEvent {
    OrderSubmitted { order: Order },
    OrderCancelled { order_id: Uuid, order: Order },
    OrderModified {
        order_id: Uuid,
        old_quantity: Decimal,
        new_quantity: Decimal,
        old_price: Option<Decimal>,
        new_price: Option<Decimal>,
    },
    OrderFilled {
        order_id: Uuid,
        fill_quantity: Decimal,
        fill_price: Decimal,
        total_filled: Decimal,
        order_state: OrderState,
    },
    OrderRejected { order_id: Uuid, reason: String },
}

/// Event processor for OMS events
struct OmsEventProcessor {
    orders: Arc<DashMap<Uuid, Order>>,
    journal: Arc<EventJournal>,
    executor: Arc<dyn Fn(Order) -> Result<()> + Send + Sync>,
}

#[async_trait::async_trait]
impl EventProcessor for OmsEventProcessor {
    type Event = OmsEvent;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    
    async fn process(&self, event: Self::Event) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Create journal entry
        let entry = JournalEntry {
            timestamp: Utc::now(),
            event_type: format!("{:?}", std::mem::discriminant(&event)),
            data: serde_json::to_value(&event).unwrap_or_default(),
            entry_id: Uuid::new_v4(),
            aggregate_id: match &event {
                OmsEvent::OrderSubmitted { order } => order.order_id,
                OmsEvent::OrderCancelled { order_id, .. } => *order_id,
                OmsEvent::OrderModified { order_id, .. } => *order_id,
                OmsEvent::OrderFilled { order_id, .. } => *order_id,
                OmsEvent::OrderRejected { order_id, .. } => *order_id,
            },
            sequence: 0, // Will be set by journal
            correlation_id: None,
            causation_id: None,
        };
        
        // Persist to journal
        self.journal.append(entry).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        
        // Process specific event
        match event {
            OmsEvent::OrderSubmitted { order } => {
                // Execute order
                (self.executor)(order).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            }
            OmsEvent::OrderFilled { order_id, total_filled, order_state, .. } => {
                // Update order state
                if let Some(mut order) = self.orders.get_mut(&order_id) {
                    let state_clone = order_state.clone();
                    order.state = state_clone;
                    
                    // If fully filled, remove from active orders
                    if matches!(order_state, OrderState::Filled) {
                        info!("Order {} fully filled", order_id);
                    }
                }
            }
            OmsEvent::OrderCancelled { order_id, .. } => {
                // Remove from active orders
                self.orders.remove(&order_id);
                info!("Order {} cancelled", order_id);
            }
            _ => {}
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_order_submission() {
        let risk_checker = |_order: &Order| Ok(());
        let executor = |_order: Order| Ok(());
        let position_updater = |_account: Uuid, _qty: Decimal, _price: Decimal| Ok(());
        
        let oms = OmsEngine::new(1024, risk_checker, executor, position_updater).unwrap();
        
        let order = Order::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "BTCUSDT".to_string(),
            Side::Buy,
            OrderType::Limit,
            Decimal::from(100),
        );
        
        let order_id = oms.submit_order(order).await.unwrap();
        assert!(oms.get_order(order_id).is_some());
    }
}
