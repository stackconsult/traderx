// Rust ↔ PyTorch SHM Bridge
// Phase B: Advanced Model - Task B3: Rust ↔ PyTorch SHM bridge

use memmap2::Mmap;
use std::fs::OpenOptions;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// SHM bridge configuration
#[derive(Debug, Clone)]
pub struct ShmBridgeConfig {
    pub shm_size: usize,
    pub shm_path: String,
}

impl Default for ShmBridgeConfig {
    fn default() -> Self {
        Self {
            shm_size: 1024 * 1024, // 1MB
            shm_path: "/tmp/traderx_shm".to_string(),
        }
    }
}

/// SHM bridge statistics
#[derive(Debug, Default)]
pub struct ShmBridgeStats {
    pub read_count: AtomicUsize,
    pub write_count: AtomicUsize,
    pub bytes_transferred: AtomicUsize,
}

impl ShmBridgeStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment_reads(&self) {
        self.read_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_writes(&self) {
        self.write_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add_bytes_transferred(&self, bytes: usize) {
        self.bytes_transferred.fetch_add(bytes, Ordering::Relaxed);
    }
}

/// Rust ↔ PyTorch SHM Bridge - zero-copy shared memory bridge
pub struct PyTorchShmBridge {
    config: ShmBridgeConfig,
    mmap: Option<memmap2::MmapMut>,
    stats: Arc<ShmBridgeStats>,
}

impl PyTorchShmBridge {
    /// Create a new SHM bridge
    pub fn new(config: ShmBridgeConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let path = Path::new(&config.shm_path);

        // Create or open SHM file
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        // Set file size
        file.set_len(config.shm_size as u64)?;

        // Memory-map the file (mutable)
        let mmap = unsafe { memmap2::MmapMut::map_mut(&file)? };

        Ok(Self {
            config,
            mmap: Some(mmap),
            stats: Arc::new(ShmBridgeStats::new()),
        })
    }

    /// Write data to SHM (zero-copy)
    pub fn write(&mut self, data: &[u8], offset: usize) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut mmap) = self.mmap {
            if offset + data.len() > self.config.shm_size {
                return Err("Write exceeds SHM size".into());
            }

            let target = &mut mmap[offset..offset + data.len()];
            target.copy_from_slice(data);

            self.stats.increment_writes();
            self.stats.add_bytes_transferred(data.len());
        }

        Ok(())
    }

    /// Read data from SHM (zero-copy)
    pub fn read(&self, offset: usize, len: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if let Some(ref mmap) = self.mmap {
            if offset + len > self.config.shm_size {
                return Err("Read exceeds SHM size".into());
            }

            let data = mmap[offset..offset + len].to_vec();

            self.stats.increment_reads();
            self.stats.add_bytes_transferred(len);

            Ok(data)
        } else {
            Err("SHM not initialized".into())
        }
    }

    /// Get statistics
    pub fn get_stats(&self) -> Arc<ShmBridgeStats> {
        self.stats.clone()
    }

    /// Get SHM size
    pub fn size(&self) -> usize {
        self.config.shm_size
    }
}

impl Default for PyTorchShmBridge {
    fn default() -> Self {
        Self::new(ShmBridgeConfig::default()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shm_bridge() {
        let config = ShmBridgeConfig {
            shm_size: 4096,
            shm_path: "/tmp/test_shm_bridge".to_string(),
        };

        let mut bridge = PyTorchShmBridge::new(config).unwrap();

        // Write data
        let data = vec![1u8, 2, 3, 4, 5];
        bridge.write(&data, 0).unwrap();

        // Read data
        let read_data = bridge.read(0, 5).unwrap();
        assert_eq!(read_data, data);

        // Check stats
        let stats = bridge.get_stats();
        assert_eq!(stats.write_count.load(Ordering::Relaxed), 1);
        assert_eq!(stats.read_count.load(Ordering::Relaxed), 1);
        assert_eq!(stats.bytes_transferred.load(Ordering::Relaxed), 10);
    }
}
