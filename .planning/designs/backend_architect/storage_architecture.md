# Three-Tier Storage Architecture Design

**Designer**: Backend Engineer (HOARD-ROLE-1)  
**Date**: 2026-05-05  
**Status**: DESIGN COMPLETE  
**Target**: BAM grid storage for multi-market HFT system

---

## Overview

Three-tier storage architecture for TraderX BAM grids and market data:
- **Hot Tier**: In-memory NVMe pool (sub-100ns access)
- **Warm Tier**: Redis cache (sub-1ms access)
- **Cold Tier**: QuestDB time-series database (sub-10ms access)

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                     Trading Engine                      │
└───────────────────┬─────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────────┐
│              Hot Tier (NVMe Pool)                        │
│  - 1TB NVMe for BAM grids                              │
│  - Zero-copy memory-mapped files                        │
│  - 10×60 grid per market (6 markets = 3.6KB each)      │
│  - Sub-100ns access time                               │
│  - Location: /mnt/nvme/bam_grids/                      │
└───────────────────┬─────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────────┐
│               Warm Tier (Redis Cache)                     │
│  - LRU cache for frequently accessed BAM grids           │
│  - Sub-1ms access time                                  │
│  - TTL: 5 minutes for market data                       │
│  - Location: redis://localhost:6379                   │
└───────────────────┬─────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────────┐
│           Cold Tier (QuestDB Time-Series)                 │
│  - Historical BAM grids (90+ days)                      │
│  - Market data time-series                              │
│  - Sub-10ms access time                                 │
│  - Location: postgresql://localhost:8812/traderx      │
└─────────────────────────────────────────────────────────┘
```

---

## Hot Tier: NVMe Pool

### Specifications
- **Capacity**: 1TB NVMe SSD
- **Access Pattern**: Memory-mapped files (mmap)
- **Latency Target**: <100ns
- **File Format**: Binary BAM grid (600 bytes per grid)
- **Concurrency**: Lock-free atomic reads

### Data Layout
```
/mnt/nvme/bam_grids/
├── equities/
│   ├── current_grid.bin (600 bytes)
│   └── history/
├── fx/
│   ├── current_grid.bin (600 bytes)
│   └── history/
├── metals/
│   ├── current_grid.bin (600 bytes)
│   └── history/
├── commodities/
│   ├── current_grid.bin (600 bytes)
│   └── history/
├── crypto/
│   ├── current_grid.bin (600 bytes)
│   └── history/
└── indices/
    ├── current_grid.bin (600 bytes)
    └── history/
```

### Implementation Strategy
```rust
// packages/oms-engine/src/infra/nvme_pool.rs
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

pub struct NvmePool {
    // Memory-mapped BAM grids
    grids: HashMap<String, Mmap>,
    // File handles for memory mapping
    files: HashMap<String, File>,
}

impl NvmePool {
    pub fn new(base_path: &Path) -> Result<Self> {
        // Open files with O_DIRECT for direct I/O
        // Memory-map for zero-copy access
        // Pre-allocate space for 6 markets
    }
    
    pub fn get_grid(&self, market: &str) -> Result<&[u8]> {
        // Lock-free atomic read
        // Return memory-mapped slice
    }
    
    pub fn update_grid(&mut self, market: &str, grid: &[u8]) -> Result<()> {
        // Atomic write with memory barrier
        // Update memory-mapped region
    }
}
```

---

## Warm Tier: Redis Cache

### Specifications
- **Cache Type**: LRU (Least Recently Used)
- **Capacity**: 16GB RAM
- **Latency Target**: <1ms
- **TTL**: 5 minutes for market data, 1 hour for BAM grids
- **Concurrency**: Pipelined commands

### Data Schema
```
Redis Keys:
- bam:grid:{market} → Binary BAM grid (600 bytes)
- market:tick:{market}:{timestamp} → Tick data
- market:ohlcv:{market}:{date} → OHLCV data
```

### Implementation Strategy
```rust
// packages/oms-engine/src/infra/data_cache.rs
use redis::Commands;

pub struct DataCache {
    client: redis::Client,
}

impl DataCache {
    pub fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { client })
    }
    
    pub fn get_bam_grid(&self, market: &str) -> Result<Option<Vec<u8>>> {
        let mut con = self.client.get_connection()?;
        let key = format!("bam:grid:{}", market);
        con.get(key)
    }
    
    pub fn set_bam_grid(&self, market: &str, grid: &[u8]) -> Result<()> {
        let mut con = self.client.get_connection()?;
        let key = format!("bam:grid:{}", market);
        con.set_ex(key, grid, 3600) // 1 hour TTL
    }
}
```

---

## Cold Tier: QuestDB Time-Series

### Specifications
- **Database**: QuestDB (PostgreSQL-compatible)
- **Latency Target**: <10ms
- **Retention**: 90 days for market data, 1 year for BAM grids
- **Compression**: LZ4 for time-series data
- **Query**: SQL with time-series extensions

### Schema Design
```sql
-- BAM grid history table
CREATE TABLE bam_grid_history (
    timestamp TIMESTAMP,
    market SYMBOL,
    grid BINARY,
    watermark TIMESTAMP
) TIMESTAMP(timestamp) PARTITION BY DAY;

-- Market data ticks table
CREATE TABLE market_ticks (
    timestamp TIMESTAMP,
    market SYMBOL,
    price DOUBLE,
    volume LONG,
    bid DOUBLE,
    ask DOUBLE
) TIMESTAMP(timestamp) PARTITION BY DAY;
```

### Implementation Strategy
```rust
// packages/oms-engine/src/infra/time_series.rs
use postgres::{Client, NoTls};

pub struct TimeSeriesStore {
    client: Client,
}

impl TimeSeriesStore {
    pub fn new(url: &str) -> Result<Self> {
        let client = Client::connect(url, NoTls)?;
        Ok(Self { client })
    }
    
    pub fn insert_bam_grid(&mut self, market: &str, grid: &[u8]) -> Result<()> {
        self.client.execute(
            "INSERT INTO bam_grid_history VALUES (now(), $1, $2, now())",
            &[&market, &grid]
        )?;
        Ok(())
    }
    
    pub fn query_bam_grid_history(&self, market: &str, start: DateTime, end: DateTime) -> Result<Vec<BamGrid>> {
        self.client.query(
            "SELECT timestamp, grid FROM bam_grid_history 
             WHERE market = $1 AND timestamp BETWEEN $2 AND $3",
            &[&market, &start, &end]
        )?
        .iter()
        .map(|row| BamGrid { ... })
        .collect()
    }
}
```

---

## Tier Coordination

### Read Path
1. Check Hot Tier (NVMe pool) → Found? Return
2. Not found? Check Warm Tier (Redis) → Found? Return, promote to Hot
3. Not found? Check Cold Tier (QuestDB) → Found? Return, promote to Warm, then Hot

### Write Path
1. Write to Hot Tier (atomic update)
2. Async write to Warm Tier (background)
3. Async write to Cold Tier (batched every 1 second)

### Cache Eviction
- Hot Tier: LRU when NVMe pool > 80% capacity
- Warm Tier: Redis LRU with TTL
- Cold Tier: QuestDB retention policy (90 days)

---

## Performance Targets

| Tier | Read Latency | Write Latency | Throughput |
|------|--------------|---------------|------------|
| Hot (NVMe) | <100ns | <500ns | >1M ops/sec |
| Warm (Redis) | <1ms | <2ms | >100K ops/sec |
| Cold (QuestDB) | <10ms | <20ms | >10K ops/sec |

---

## Failure Modes

### Hot Tier Failure
- **Detection**: Memory-mapped file I/O error
- **Fallback**: Read from Warm Tier (Redis)
- **Recovery**: Remap NVMe files, restore from Cold Tier

### Warm Tier Failure
- **Detection**: Redis connection error
- **Fallback**: Read from Cold Tier (QuestDB)
- **Recovery**: Restart Redis, rebuild cache from Hot Tier

### Cold Tier Failure
- **Detection**: QuestDB connection error
- **Fallback**: Continue with Hot + Warm tiers only
- **Recovery**: Restart QuestDB, backfill from Warm Tier

---

## Implementation Order

1. **Phase 1**: Hot Tier (NVMe pool) - 4 hours
   - Implement NvmePool struct
   - Memory-mapped file I/O
   - Lock-free atomic reads

2. **Phase 2**: Warm Tier (Redis cache) - 3 hours
   - Implement DataCache struct
   - Redis client integration
   - LRU cache logic

3. **Phase 3**: Cold Tier (QuestDB) - 3 hours
   - Implement TimeSeriesStore struct
   - QuestDB client integration
   - Batch write optimization

4. **Phase 4**: Tier coordination - 4 hours
   - Read path with tier fallback
   - Write path with async promotion
   - Cache eviction logic

5. **Phase 5**: Testing - 4 hours
   - Unit tests for each tier
   - Integration tests for tier coordination
   - Performance benchmarks

---

**Total Estimated Time**: 18 hours  
**Next Step**: Implement Phase 1 (Hot Tier - NVMe pool)
