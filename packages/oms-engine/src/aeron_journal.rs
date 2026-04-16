/*
 * Aeron Journal - Stub implementation
 * 
 * Currently using Redis as backend until Aeron API stabilizes
 * TODO: Implement real Aeron integration when API is stable
 */

use crate::oms::{OmsEvent, OmsError};
use crate::journal::{EventJournal, JournalError};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::{info, error, warn};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AeronJournalError {
    #[error("Aeron error: {0}")]
    Aeron(String),
    #[error("Publication error: {0}")]
    Publication(String),
    #[error("Subscription error: {0}")]
    Subscription(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Configuration error: {0}")]
    Configuration(String),
}

/// Aeron-based journal (stub using Redis)
pub struct AeronJournal {
    /// Redis journal backend
    redis_journal: EventJournal,
    
    /// Channel configuration (for future use)
    channel: String,
    
    /// Stream ID (for future use)
    stream_id: i32,
    
    /// Current sequence number
    sequence: Arc<RwLock<u64>>,
}

impl AeronJournal {
    /// Create new Aeron journal (stub using Redis)
    pub fn new(channel: &str, stream_id: i32) -> Result<Self, AeronJournalError> {
        let redis_journal = EventJournal::new()
            .map_err(|e| AeronJournalError::Aeron(e.to_string()))?;
        
        info!("Aeron journal (Redis stub) initialized with channel: {}, stream: {}", channel, stream_id);
        
        Ok(Self {
            redis_journal,
            channel: channel.to_string(),
            stream_id,
            sequence: Arc::new(RwLock::new(0)),
        })
    }
    
    /// Start publication (stub)
    pub async fn start_publication(&mut self) -> Result<(), AeronJournalError> {
        info!("Aeron publication started (Redis stub)");
        Ok(())
    }
    
    /// Start subscription for replay (stub)
    pub async fn start_subscription(&mut self, from_sequence: i64) -> Result<(), AeronJournalError> {
        info!("Aeron subscription started from sequence: {} (Redis stub)", from_sequence);
        Ok(())
    }
    
    /// Append event to Aeron stream (using Redis)
    pub async fn append(&mut self, event: &OmsEvent) -> Result<u64, AeronJournalError> {
        // Convert OmsEvent to JournalEntry
        let entry = crate::journal::JournalEntry {
            entry_id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            event_type: "OmsEvent".to_string(),
            aggregate_id: uuid::Uuid::new_v4(),
            data: serde_json::to_value(event)
                .map_err(|e| AeronJournalError::Serialization(e.to_string()))?,
            sequence: 0,
            correlation_id: None,
            causation_id: None,
        };
        
        // Use Redis backend
        self.redis_journal.append(entry).await
            .map_err(|e| AeronJournalError::Aeron(e.to_string()))?;
        
        let seq = self.redis_journal.get_sequence().await;
        *self.sequence.write().await = seq;
        
        Ok(seq)
    }
    
    /// Replay events from Aeron stream (using Redis)
    pub async fn replay(&mut self, from_sequence: u64, limit: Option<usize>) -> Result<Vec<OmsEvent>, AeronJournalError> {
        let entries = self.redis_journal.replay(Some(from_sequence)).await
            .map_err(|e| AeronJournalError::Aeron(e.to_string()))?;
        
        let mut events = Vec::new();
        for entry in entries.into_iter().take(limit.unwrap_or(usize::MAX)) {
            if let Ok(event) = serde_json::from_value::<OmsEvent>(entry.data) {
                events.push(event);
            }
        }
        
        Ok(events)
    }
    
    /// Get current sequence number
    pub async fn get_sequence(&self) -> u64 {
        self.redis_journal.get_sequence().await
    }
}

/// Aeron message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AeronMessage {
    sequence: u64,
    timestamp: DateTime<Utc>,
    data: Vec<u8>,
}

/// Adapter for AeronJournal to work with existing OMS
impl From<AeronJournalError> for OmsError {
    fn from(err: AeronJournalError) -> Self {
        OmsError::JournalError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_aeron_journal_lifecycle() {
        // This test requires Aeron media driver to be running
        // In production, ensure aeron-driver is started
        
        let mut journal = AeronJournal::new("aeron:udp?endpoint=localhost:40123", 1001).unwrap();
        
        // Start publication
        journal.start_publication().await.unwrap();
        
        // Create test event
        let event = OmsEvent::OrderRejected {
            order_id: Uuid::new_v4(),
            reason: "Test".to_string(),
        };
        
        // Append event
        let sequence = journal.append(&event).await.unwrap();
        assert!(sequence > 0);
        
        println!("✅ Aeron journal test passed, sequence: {}", sequence);
    }
}
