// Infrastructure module
// Phase 0: Foundation - Three-tier storage, monitoring, model registry

pub mod batch_ingest;
pub mod data_cache;
pub mod model_registry;
pub mod monitoring;
pub mod nvme_pool;

pub use batch_ingest::{BatchIngestConfig, BatchIngestPipeline, IngestStats, MarketTick};
