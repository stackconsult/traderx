//! Feature store client with hybrid caching strategy.

use anyhow::{Context, Result};
use redis::AsyncCommands;
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Redis-backed feature store client.
pub struct FeatureStore {
    client: Arc<redis::aio::MultiplexedConnection>,
    key_prefix: String,
}

impl FeatureStore {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)
            .context("Failed to create Redis client")?;
        
        let conn = client.get_multiplexed_async_connection().await
            .context("Failed to get Redis connection")?;
        
        Ok(Self {
            client: Arc::new(conn),
            key_prefix: "features:".to_string(),
        })
    }

    /// Get all features for a symbol.
    pub async fn get_all(&self, symbol: &str) -> Result<Option<HashMap<String, f64>>> {
        let key = format!("{}{}:__all__", self.key_prefix, symbol);
        let mut conn = self.client.clone();
        
        let data: Option<Vec<u8>> = conn.get(&key).await
            .context("Failed to get features from Redis")?;
        
        if let Some(data) = data {
            let features: HashMap<String, serde_json::Value> = serde_json::from_slice(&data)
                .context("Failed to deserialize features")?;
            
            let mut float_features = HashMap::new();
            for (k, v) in features {
                if let Some(f) = v.as_f64() {
                    float_features.insert(k, f);
                }
            }
            
            Ok(Some(float_features))
        } else {
            Ok(None)
        }
    }

    /// Get specific features for a symbol.
    pub async fn get(&self, symbol: &str, feature_names: &[String]) -> Result<HashMap<String, f64>> {
        let keys: Vec<String> = feature_names.iter()
            .map(|name| format!("{}{}:{}", self.key_prefix, symbol, name))
            .collect();
        
        let mut conn = self.client.clone();
        let values: Vec<Option<Vec<u8>>> = conn.mget(&keys).await
            .context("Failed to mget features from Redis")?;
        
        let mut features = HashMap::new();
        for (name, value) in feature_names.iter().zip(values) {
            if let Some(data) = value {
                if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&data) {
                    if let Some(f) = v.as_f64() {
                        features.insert(name.clone(), f);
                    }
                }
            }
        }
        
        Ok(features)
    }
}

/// TTL-based feature cache to reduce Redis round-trips.
pub struct FeatureCache {
    cache: Arc<RwLock<HashMap<String, CachedFeatures>>>,
    ttl: Duration,
}

#[derive(Debug, Clone)]
struct CachedFeatures {
    features: HashMap<String, f64>,
    timestamp: Instant,
}

impl FeatureCache {
    pub fn new(ttl_ms: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_millis(ttl_ms),
        }
    }

    /// Get cached features if not expired.
    pub async fn get(&self, symbol: &str) -> Option<HashMap<String, f64>> {
        let cache = self.cache.read().await;
        if let Some(cached) = cache.get(symbol) {
            if cached.timestamp.elapsed() < self.ttl {
                return Some(cached.features.clone());
            }
        }
        None
    }

    /// Set features in cache.
    pub async fn set(&self, symbol: &str, features: HashMap<String, f64>) {
        let mut cache = self.cache.write().await;
        cache.insert(symbol.to_string(), CachedFeatures {
            features,
            timestamp: Instant::now(),
        });
        
        // Periodic cleanup of expired entries
        if cache.len() > 1000 {
            let now = Instant::now();
            cache.retain(|_, cached| now.duration_since(cached.timestamp) < self.ttl);
        }
    }

    /// Clear cache for a symbol.
    pub async fn invalidate(&self, symbol: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(symbol);
    }

    /// Clear entire cache.
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get cache statistics.
    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let now = Instant::now();
        let (valid, expired) = cache.values()
            .partition(|cached| now.duration_since(cached.timestamp) < self.ttl);
        
        CacheStats {
            total_entries: cache.len(),
            valid_entries: valid.len(),
            expired_entries: expired.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub valid_entries: usize,
    pub expired_entries: usize,
}
