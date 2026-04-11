---
trigger: manual
description: "Deploy and manage quant infrastructure components"
---

# Quant Infrastructure Agent Rules

## QuestDB Deployment

### Detection Protocol
```bash
# Check QuestDB status
docker ps | grep questdb
curl -f http://localhost:9000/exec\?query=SELECT%20NOW()
```

### Reconciliation Actions
- Deploy QuestDB with optimized configuration
- Configure vectorized ingestion buffers
- Enable turboquant compression at ingestion
- Set up connection pooling with auto-recovery

### Self-Healing Procedures
- Monitor container health every 5 seconds
- Auto-restart on OOM with increased memory limit
- Rebuild corrupted indexes automatically
- Failover to replica on primary failure

## Tick Ingestion Pipeline

### Performance Requirements
- 10M ticks/second sustained throughput
- <1ms end-to-end latency
- 99.99% uptime guarantee

### Implementation Checklist
- [ ] Deploy Rust-based ingestion service
- [ ] Configure zero-copy buffers
- [ ] Enable SIMD vectorization
- [ ] Set up adaptive batch sizing
- [ ] Implement backpressure handling

### Optimization Commands
```bash
# Enable turboquant acceleration
export TURBOQUANT_ENABLE=1
export TURBOQUANT_COMPRESSION_LEVEL=8
export TURBOQUANT_PARALLELISM=$(nproc)
```

## Feature Store Setup

### Redis Configuration
```yaml
# redis.conf optimized for quant workloads
maxmemory-policy: allkeys-lru
save: ""
appendonly: yes
appendfsync: no
tcp-keepalive: 300
```

### Parquet Storage Schema
- Partition by symbol/date
- Zstandard compression level 9
- Column pruning for selective reads
- Statistics for query optimization

### Feature Registry API
```python
# Auto-register features with metadata
POST /api/features/register
{
  "name": "volatility_20d",
  "description": "20-day rolling volatility",
  "dtype": "float32",
  "compression": "turboquant",
  "retention": "5y",
  "tags": ["technical", "risk"]
}
```

## Monitoring and Alerting

### Critical Metrics
- QuestDB ingestion rate
- Redis memory usage
- Feature computation latency
- Compression ratios

### Alert Thresholds
- Ingestion rate < 8M ticks/sec for 30s
- Redis memory > 90% capacity
- Feature latency > 100μs
- Compression ratio < 2:1

## Recovery Procedures

### Data Recovery
```bash
# Restore from compressed snapshots
questdb-restore --snapshot latest --decompress turboquant
```

### Service Recovery
- Automatic service restart on crash
- Graceful degradation on resource constraints
- Circuit breaker for failing components
