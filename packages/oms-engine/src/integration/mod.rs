//! Integration Module - Canonical System Wiring
//! 
//! This module provides the single source of truth for how all TraderX components
//! are wired together. All binaries MUST use these factory functions to ensure
//! consistent API usage and prevent architectural drift.
//! 
//! # ARCHITECTURAL RULES
//! 
//! 1. **Always use factory functions** - Never construct components directly
//! 2. **Use canonical channel types** - Defined in this module
//! 3. **Follow error handling patterns** - Use Result<(), OmsError> consistently
//! 4. **Maintain async boundaries** - No blocking operations in async contexts
//! 
//! # EXAMPLE USAGE
//! 
//! ```rust
//! use oms_engine::integration::factory::create_trading_system;
//! 
//! let system = create_trading_system(config).await?;
//! let outcome = system.route_signal(signal).await?;
//! ```

pub mod factory;
pub mod types;
pub mod config;

// Re-export commonly used types for convenience
pub use factory::{
    create_trading_system,
    create_oms_engine,
    create_signal_router,
    create_risk_bus,
};
pub use types::{
    TradingSystem,
    SignalRouterChannels,
    OmsChannels,
    SystemChannels,
};
pub use config::{
    SystemConfig,
    OmsConfig,
    RouterConfig,
    RiskConfig,
};
