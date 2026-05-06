// Mesh module
// Phase A: Base Model Foundation - Multi-market grid and schema registry
// Phase B: Advanced Model - Rust ↔ PyTorch SHM bridge

pub mod multi_market_grid;
pub mod pytorch_bridge;
pub mod schema_registry;

pub use multi_market_grid::{BamGrid, MarketClass, MultiMarketGridAllocator};
pub use pytorch_bridge::{PyTorchShmBridge, ShmBridgeConfig, ShmBridgeStats};
pub use schema_registry::{BamSchema, BamSchemaRegistry};
