// NVMe Pool - Hot Tier (Memory-mapped files)
// Phase 0: Foundation - Three-tier storage

use memmap2::Mmap;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// NVMe pool configuration
#[derive(Debug, Clone)]
pub struct NvmePoolConfig {
    pub base_path: PathBuf,
    pub pool_size_gb: usize,
    pub grid_size_bytes: usize,
}

impl Default for NvmePoolConfig {
    fn default() -> Self {
        Self {
            base_path: PathBuf::from("/mnt/nvme/bam_grids"),
            pool_size_gb: 1024, // 1TB
            grid_size_bytes: 600, // 10×60 grid = 600 bytes
        }
    }
}

/// NVMe pool statistics
#[derive(Debug, Default)]
pub struct NvmePoolStats {
    pub total_grids: AtomicUsize,
    pub cache_hits: AtomicUsize,
    pub cache_misses: AtomicUsize,
}

impl NvmePoolStats {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn increment_total(&self) {
        self.total_grids.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn increment_hits(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn increment_misses(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }
}

/// NVMe Pool - Hot Tier (Memory-mapped files)
pub struct NvmePool {
    config: NvmePoolConfig,
    grids: HashMap<String, Arc<Mmap>>,
    files: HashMap<String, File>,
    stats: Arc<NvmePoolStats>,
}

impl NvmePool {
    /// Create a new NVMe pool
    pub fn new(config: NvmePoolConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // Create base directory if it doesn't exist
        std::fs::create_dir_all(&config.base_path)?;
        
        // Create market subdirectories
        let markets = ["equities", "fx", "metals", "commodities", "crypto", "indices"];
        for market in markets {
            std::fs::create_dir_all(config.base_path.join(market))?;
            std::fs::create_dir_all(config.base_path.join(market).join("history"))?;
        }
        
        Ok(Self {
            config,
            grids: HashMap::new(),
            files: HashMap::new(),
            stats: Arc::new(NvmePoolStats::new()),
        })
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> Arc<NvmePoolStats> {
        self.stats.clone()
    }
    
    /// Get BAM grid for a market
    pub fn get_grid(&self, market: &str) -> Result<&[u8], Box<dyn std::error::Error>> {
        if let Some(mmap) = self.grids.get(market) {
            self.stats.increment_hits();
            Ok(&**mmap)
        } else {
            self.stats.increment_misses();
            Err(format!("Grid for {} not loaded", market).into())
        }
    }
    
    /// Load BAM grid for a market
    pub fn load_grid(&mut self, market: &str) -> Result<(), Box<dyn std::error::Error>> {
        let grid_path = self.config.base_path.join(market).join("current_grid.bin");
        
        // Open file with O_DIRECT for direct I/O (if supported)
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&grid_path)?;
        
        // Ensure file is the right size
        file.set_len(self.config.grid_size_bytes as u64)?;
        
        // Memory-map the file
        let mmap = unsafe { Mmap::map(&file)? };
        
        self.grids.insert(market.to_string(), Arc::new(mmap));
        self.files.insert(market.to_string(), file);
        self.stats.increment_total();
        
        Ok(())
    }
    
    /// Update BAM grid for a market
    pub fn update_grid(&mut self, market: &str, grid: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let grid_path = self.config.base_path.join(market).join("current_grid.bin");
        
        // Ensure grid is the right size
        if grid.len() != self.config.grid_size_bytes {
            return Err(format!("Grid size mismatch: expected {}, got {}", 
                              self.config.grid_size_bytes, grid.len()).into());
        }
        
        // Write to file
        let mut file = File::create(&grid_path)?;
        file.write_all(grid)?;
        file.sync_all()?;
        
        // Reload memory map
        self.load_grid(market)?;
        
        Ok(())
    }
    
    /// Archive current grid to history
    pub fn archive_grid(&self, market: &str, timestamp: i64) -> Result<(), Box<dyn std::error::Error>> {
        let current_path = self.config.base_path.join(market).join("current_grid.bin");
        let history_path = self.config.base_path.join(market)
            .join("history")
            .join(format!("grid_{}.bin", timestamp));
        
        std::fs::copy(current_path, history_path)?;
        
        Ok(())
    }
    
    /// Get pool capacity
    pub fn capacity(&self) -> usize {
        self.config.pool_size_gb * 1024 * 1024 * 1024
    }
    
    /// Get pool usage
    pub fn usage(&self) -> usize {
        self.stats.total_grids.load(Ordering::Relaxed) * self.config.grid_size_bytes
    }
    
    /// Get pool usage percentage
    pub fn usage_percent(&self) -> f64 {
        (self.usage() as f64 / self.capacity() as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_nvme_pool() {
        let temp_dir = TempDir::new().unwrap();
        let config = NvmePoolConfig {
            base_path: temp_dir.path().to_path_buf(),
            pool_size_gb: 1,
            grid_size_bytes: 600,
        };
        
        let mut pool = NvmePool::new(config).unwrap();
        
        // Load grid for AAPL
        pool.load_grid("AAPL").unwrap();
        
        // Create test grid
        let grid = vec![42u8; 600];
        pool.update_grid("AAPL", &grid).unwrap();
        
        // Get grid
        let retrieved = pool.get_grid("AAPL").unwrap();
        assert_eq!(retrieved, &grid[..]);
        
        // Archive grid
        pool.archive_grid("AAPL", 1234567890).unwrap();
        
        // Check stats
        let stats = pool.get_stats();
        assert_eq!(stats.total_grids.load(Ordering::Relaxed), 1);
        assert_eq!(stats.cache_hits.load(Ordering::Relaxed), 1);
    }
}
