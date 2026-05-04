//! API Server Module - REST API and WebSocket endpoints for TraderX
//!
//! Provides:
//! - REST API on /api/v1/* for orders, positions, market data
//! - WebSocket on /ws for real-time updates
//! - JWT authentication middleware

pub mod auth;
pub mod error;
pub mod routes;
pub mod server;
pub mod state;
pub mod types;
pub mod websocket;

pub use server::run_server;
pub use state::AppState;
