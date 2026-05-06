// Mesh module
// Phase A: Base Model Foundation - Multi-market grid and schema registry

pub mod multi_market_grid;
pub mod schema_registry;

pub use multi_market_grid::{BamGrid, MarketClass, MultiMarketGridAllocator};
pub use schema_registry::{BamSchema, BamSchemaRegistry};
