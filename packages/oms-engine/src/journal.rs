use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::{info, error, warn};
use thiserror::Error;
use redis::AsyncCommands;
use serde_json::Value;

#[derive(Error, Debug)]
pub enum JournalError {
    #[error("Redis error: {0}")]
    Redis(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Storage error: {0}")]
    Storage(String),
}

/// Journal entry for event sourcing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    /// Timestamp when event occurred
    pub timestamp: DateTime<Utc>,
    
    /// Event type identifier
    pub event_type: String,
    
    /// Event data
    pub data: Value,
    
    /// Unique entry ID (generated if not provided)
    #[serde(default = "Uuid::new_v4")]
    pub entry_id: Uuid,
    
    /// Aggregate ID (e.g., order_id) - extracted from data if not provided
    #[serde(default)]
    pub aggregate_id: Uuid,
    
    /// Event sequence number
    pub sequence: u64,
    
    /// Correlation ID for tracing
    #[serde(default)]
    pub correlation_id: Option<Uuid>,
    
    /// Causation ID (what caused this event)
    #[serde(default)]
    pub causation_id: Option<Uuid>,
}

/// Event journal for persistence and replay
pub struct EventJournal {
    /// Redis client for persistence
    redis_client: Arc<redis::Client>,
    
    /// Current sequence number
    sequence: Arc<RwLock<u64>>,
    
    /// Journal configuration
    config: JournalConfig,
}

#[derive(Debug, Clone)]
pub struct JournalConfig {
    /// Redis key prefix
    pub key_prefix: String,
    
    /// Retention period in seconds
    pub retention_seconds: u64,
    
    /// Batch size for writes
    pub batch_size: usize,
    
    /// Enable compression
    pub compression: bool,
}

impl Default for JournalConfig {
    fn default() -> Self {
        Self {
            key_prefix: "traderx:journal".to_string(),
            retention_seconds: 86400 * 30, // 30 days
            batch_size: 100,
            compression: true,
        }
    }
}

impl EventJournal {
    /// Create new event journal
    pub fn new() -> Result<Self, JournalError> {
        Self::with_config(JournalConfig::default())
    }
    
    /// Create new event journal with custom config
    pub fn with_config(config: JournalConfig) -> Result<Self, JournalError> {
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());
        
        let client = redis::Client::open(redis_url.as_str())
            .map_err(|e| JournalError::Redis(e.to_string()))?;
        
        info!("Event journal initialized with Redis: {}", redis_url);
        
        Ok(Self {
            redis_client: Arc::new(client),
            sequence: Arc::new(RwLock::new(0)),
            config,
        })
    }
    
    /// Append single event to journal
    pub async fn append(&self, mut entry: JournalEntry) -> Result<(), JournalError> {
        // Get next sequence number
        {
            let mut seq = self.sequence.write().await;
            *seq += 1;
            entry.sequence = *seq;
        }
        
        // Serialize entry
        let serialized = serde_json::to_string(&entry)
            .map_err(|e| JournalError::Serialization(e.to_string()))?;
        
        // Store in Redis
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| JournalError::Redis(e.to_string()))?;
        
        let key = format!("{}:{}", self.config.key_prefix, entry.sequence);
        
        // Store with expiration
        conn.set_ex::<String, String, usize>(key, serialized, self.config.retention_seconds as usize).await
            .map_err(|e| JournalError::Redis(e.to_string()))?;
        
        // Update aggregate index
        let aggregate_key = format!("{}:aggregate:{}", self.config.key_prefix, entry.aggregate_id);
        conn.zadd(&aggregate_key, entry.sequence.to_string(), entry.sequence as f64).await
            .map_err(|e| JournalError::Redis(e.to_string()))?;
        
        // Set expiration on aggregate index
        conn.expire::<String, usize>(aggregate_key, self.config.retention_seconds as usize).await
            .map_err(|e| JournalError::Redis(e.to_string()))?;
        
        info!("Appended journal entry: {} (seq: {})", entry.entry_id, entry.sequence);
        
        Ok::<_, JournalError>(())
    }
    
    /// Append multiple events in batch
    pub async fn append_batch(&self, mut entries: Vec<JournalEntry>) -> Result<(), JournalError> {
        if entries.is_empty() {
            return Ok(());
        }
        
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| JournalError::Redis(e.to_string()))?;
        
        // Use Redis pipeline for batch operations
        let mut pipe = redis::pipe();
        
        for mut entry in &mut entries {
            // Get sequence number
            let seq = {
                let mut seq_lock = self.sequence.write().await;
                *seq_lock += 1;
                *seq_lock
            };
            entry.sequence = seq;
            
            // Serialize
            let serialized = serde_json::to_string(&entry)
                .map_err(|e| JournalError::Serialization(e.to_string()))?;
            
            // Add to pipeline
            let key = format!("{}:{}", self.config.key_prefix, entry.sequence);
            pipe.set_ex(&key, serialized, self.config.retention_seconds as usize);
            
            // Update aggregate index
            let aggregate_key = format!("{}:aggregate:{}", self.config.key_prefix, entry.aggregate_id);
            pipe.zadd(&aggregate_key, entry.sequence as f64, entry.sequence.to_string());
            pipe.expire(&aggregate_key, self.config.retention_seconds as usize);
        }
        
        // Execute pipeline
        pipe.query_async(&mut conn).await
            .map_err(|e| JournalError::Redis(e.to_string()))?;
        
        info!("Appended {} journal entries in batch", entries.len());
        
        Ok(())
    }
    
    /// Get events for an aggregate
    pub async fn get_events(&self, aggregate_id: Uuid, from_sequence: Option<u64>) -> Result<Vec<JournalEntry>, JournalError> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| JournalError::Redis(e.to_string()))?;
        
        let aggregate_key = format!("{}:aggregate:{}", self.config.key_prefix, aggregate_id);
        
        // Get sequence numbers for aggregate
        let sequences: Vec<String> = if let Some(from) = from_sequence {
            conn.zrangebyscore(&aggregate_key, from as f64, "+inf").await
                .map_err(|e| JournalError::Redis(e.to_string()))?
        } else {
            conn.zrange(&aggregate_key, 0, -1).await
                .map_err(|e| JournalError::Redis(e.to_string()))?
        };
        
        // Get actual entries
        let mut entries = Vec::with_capacity(sequences.len());
        
        for seq_str in sequences {
            let key = format!("{}:{}", self.config.key_prefix, seq_str);
            let serialized: String = conn.get(&key).await
                .map_err(|e| JournalError::Redis(e.to_string()))?;
            
            let entry: JournalEntry = serde_json::from_str(&serialized)
                .map_err(|e| JournalError::Serialization(e.to_string()))?;
            
            entries.push(entry);
        }
        
        Ok(entries)
    }
    
    /// Get all events from sequence
    pub async fn get_events_from(&self, from_sequence: u64, limit: Option<usize>) -> Result<Vec<JournalEntry>, JournalError> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| JournalError::Redis(e.to_string()))?;
        
        // Get current max sequence
        let max_seq = *self.sequence.read().await;
        
        let to_sequence = limit.map(|l| from_sequence + l as u64 - 1).unwrap_or(max_seq);
        
        let mut entries = Vec::with_capacity((to_sequence - from_sequence + 1) as usize);
        
        for seq in from_sequence..=to_sequence {
            let key = format!("{}:{}", self.config.key_prefix, seq);
            
            if let Ok(serialized) = conn.get::<_, String>(&key).await {
                let entry: JournalEntry = serde_json::from_str(&serialized)
                    .map_err(|e| JournalError::Serialization(e.to_string()))?;
                
                entries.push(entry);
            }
        }
        
        Ok(entries)
    }
    
    /// Replay events for aggregate
    pub async fn replay_aggregate(&self, aggregate_id: Uuid) -> Result<Vec<JournalEntry>, JournalError> {
        self.get_events(aggregate_id, None).await
    }
    
    /// Get current sequence number
    pub async fn current_sequence(&self) -> u64 {
        *self.sequence.read().await
    }
    
    /// Create snapshot of aggregate state
    pub async fn create_snapshot<T>(&self, aggregate_id: Uuid, state: &T, sequence: u64) -> Result<(), JournalError>
    where
        T: Serialize,
    {
        let serialized = serde_json::to_string(state)
            .map_err(|e| JournalError::Serialization(e.to_string()))?;
        
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| JournalError::Redis(e.to_string()))?;
        
        let snapshot_key = format!("{}:snapshot:{}", self.config.key_prefix, aggregate_id);
        
        conn.set_ex::<String, String, usize>(snapshot_key, serialized, self.config.retention_seconds as usize).await
            .map_err(|e| JournalError::Redis(e.to_string()))?;
        
        // Store snapshot sequence
        let seq_key = format!("{}:snapshot_seq:{}", self.config.key_prefix, aggregate_id);
        conn.set_ex::<String, u64, usize>(seq_key, sequence, self.config.retention_seconds as usize).await
            .map_err(|e| JournalError::Redis(e.to_string()))?;
        
        info!("Created snapshot for aggregate {} at sequence {}", aggregate_id, sequence);
        
        Ok(())
    }
    
    /// Get latest snapshot for aggregate
    pub async fn get_snapshot<T>(&self, aggregate_id: Uuid) -> Result<Option<(T, u64)>, JournalError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| JournalError::Redis(e.to_string()))?;
        
        let snapshot_key = format!("{}:snapshot:{}", self.config.key_prefix, aggregate_id);
        let seq_key = format!("{}:snapshot_seq:{}", self.config.key_prefix, aggregate_id);
        
        // Get snapshot and sequence
        let snapshot_result: Result<Option<String>, redis::RedisError> = conn.get::<String, Option<String>>(snapshot_key).await;
        let sequence_result: Result<Option<u64>, redis::RedisError> = conn.get::<String, Option<u64>>(&seq_key).await;
        
        let serialized = snapshot_result.map_err(|e| JournalError::Redis(e.to_string()))?;
        let sequence = sequence_result.map_err(|e| JournalError::Redis(e.to_string()))?;
        
        match (serialized, sequence) {
            (Some(data), Some(seq)) => {
                let state: T = serde_json::from_str(&data)
                    .map_err(|e| JournalError::Serialization(e.to_string()))?;
                
                Ok(Some((state, seq)))
            }
            _ => Ok(None),
        }
    }
    
    /// Replay events from journal
    pub async fn replay(&self, from_sequence: Option<u64>) -> Result<Vec<JournalEntry>, JournalError> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| JournalError::Redis(e.to_string()))?;
        
        let pattern = format!("{}:*", self.config.key_prefix);
        let keys: Vec<String> = conn.keys(pattern.as_str()).await
            .map_err(|e| JournalError::Redis(e.to_string()))?;
        
        let mut entries = Vec::new();
        for key in keys {
            if let Some(data) = conn.get::<String, Option<String>>(&key).await
                .map_err(|e| JournalError::Redis(e.to_string()))? {
                let entry: JournalEntry = serde_json::from_str(&data)
                    .map_err(|e| JournalError::Serialization(e.to_string()))?;
                
                if let Some(from) = from_sequence {
                    if entry.sequence >= from {
                        entries.push(entry);
                    }
                } else {
                    entries.push(entry);
                }
            }
        }
        
        Ok(entries)
    }
    
    /// Get current sequence number
    pub async fn get_sequence(&self) -> u64 {
        *self.sequence.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_journal_append_and_retrieve() {
        let journal = EventJournal::new().unwrap();
        
        let entry = JournalEntry {
            entry_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: "OrderSubmitted".to_string(),
            aggregate_id: Uuid::new_v4(),
            data: json!({"test": "data"}),
            sequence: 0,
            correlation_id: None,
            causation_id: None,
        };
        
        // Append entry
        journal.append(entry.clone()).await.unwrap();
        
        // Retrieve events
        let events = journal.get_events(entry.aggregate_id, None).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "OrderSubmitted");
    }
}
