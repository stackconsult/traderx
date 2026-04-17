//! Orders Module - Advanced Order Types and Management
//!
//! This module provides production-grade order types and management
//! capabilities that exceed industry benchmarks from NautilusTrader
//! and other leading trading platforms.

pub mod advanced;

pub use advanced::{
    AdvancedOrder,
    AdvancedOrderBuilder,
    AdvancedOrderType,
    ContingencyType,
    ExecutionRestriction,
    TimeInForce,
};
