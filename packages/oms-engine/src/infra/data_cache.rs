// Data Cache - Warm Tier (Redis)
// Phase 0: Foundation - Three-tier storage

use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Data cache configuration
#[derive(Debug, Clone)]
pub struct DataCacheConfig {
    pub redis_url: String,
    pub default_ttl_seconds: u64,
}

impl Default for DataCacheConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://127.0.0.1:6379".to_string(),
            default_ttl_seconds: 300, // 5 minutes
        }
    }
}

/// BAM grid cache key format
pub fn bam_grid_key(market: &str) -> String {
    format!("bam:grid:{}", market)
}

/// Market tick cache key format
pub fn market_tick_key(market: &str, timestamp: i64) -> String {
    format!("market:tick:{}:{}", market, timestamp)
}

/// OHLCV cache key format
pub fn ohlcv_key(market: &str, date: &str) -> String {
    format!("market:ohlcv:{}:{}", market, date)
}

/// Data cache - Warm Tier (Redis)
pub struct DataCache {
    client: Client,
    default_ttl: Duration,
}

impl DataCache {
    /// Create a new data cache
    pub async fn new(config: DataCacheConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::open(config.redis_url)?;

        // Test connection
        let mut con = client.get_async_connection().await?;
        let _: String = redis::cmd("PING").query_async(&mut con).await?;

        Ok(Self {
            client,
            default_ttl: Duration::from_secs(config.default_ttl_seconds),
        })
    }

    /// Get BAM grid from cache
    pub async fn get_bam_grid(
        &self,
        market: &str,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut con = self.client.get_async_connection().await?;
        let key = bam_grid_key(market);
        con.get(key)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    /// Set BAM grid in cache
    pub async fn set_bam_grid(
        &self,
        market: &str,
        grid: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut con = self.client.get_async_connection().await?;
        let key = bam_grid_key(market);
        con.set_ex(key, grid, self.default_ttl.as_secs() as usize)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    /// Get market tick from cache
    pub async fn get_market_tick(
        &self,
        market: &str,
        timestamp: i64,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut con = self.client.get_async_connection().await?;
        let key = market_tick_key(market, timestamp);
        con.get(key)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    /// Set market tick in cache
    pub async fn set_market_tick(
        &self,
        market: &str,
        timestamp: i64,
        tick: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut con = self.client.get_async_connection().await?;
        let key = market_tick_key(market, timestamp);
        con.set_ex(key, tick, self.default_ttl.as_secs() as usize)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    /// Get OHLCV data from cache
    pub async fn get_ohlcv(
        &self,
        market: &str,
        date: &str,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut con = self.client.get_async_connection().await?;
        let key = ohlcv_key(market, date);
        con.get(key)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    /// Set OHLCV data in cache
    pub async fn set_ohlcv(
        &self,
        market: &str,
        date: &str,
        ohlcv: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut con = self.client.get_async_connection().await?;
        let key = ohlcv_key(market, date);
        con.set_ex(key, ohlcv, self.default_ttl.as_secs() as usize)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    /// Delete key from cache
    pub async fn delete(&self, key: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut con = self.client.get_async_connection().await?;
        con.del(key)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    /// Flush all data from cache
    pub async fn flush_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut con = self.client.get_async_connection().await?;
        redis::cmd("FLUSHALL")
            .query_async(&mut con)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Redis running
    async fn test_data_cache() {
        let config = DataCacheConfig::default();
        let cache = DataCache::new(config).await.unwrap();

        // Test BAM grid
        let grid = vec![1u8; 600];
        cache.set_bam_grid("AAPL", &grid).await.unwrap();
        let retrieved = cache.get_bam_grid("AAPL").await.unwrap();
        assert_eq!(retrieved, Some(grid));
    }
}
