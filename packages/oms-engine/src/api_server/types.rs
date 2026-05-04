//! API Request/Response Types (DTOs)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Order Types
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct CreateOrderRequest {
    pub symbol: String,
    pub side: String,  // "buy" | "sell"
    pub order_type: String,  // "market" | "limit" | "iceberg"
    pub quantity: f64,
    pub price: Option<f64>,
    pub time_in_force: Option<String>, // "day" | "gtc" | "ioc"
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderResponse {
    pub order_id: String,
    pub status: String,
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub filled_quantity: f64,
    pub avg_price: Option<f64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListOrdersQuery {
    pub status: Option<String>,
    pub symbol: Option<String>,
    pub limit: Option<usize>,
}

// ============================================================================
// Position Types
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct PositionResponse {
    pub symbol: String,
    pub quantity: f64,
    pub avg_entry_price: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub market_price: f64,
    pub market_value: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PortfolioSummaryResponse {
    pub total_value: f64,
    pub cash_balance: f64,
    pub total_exposure: f64,
    pub total_unrealized_pnl: f64,
    pub total_realized_pnl: f64,
    pub buying_power: f64,
    pub margin_used: f64,
}

// ============================================================================
// Market Data Types
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct MarketDataResponse {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub last_price: f64,
    pub volume_24h: f64,
    pub timestamp: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderBookQuery {
    pub depth: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderBookResponse {
    pub symbol: String,
    pub bids: Vec<(f64, f64)>, // (price, quantity)
    pub asks: Vec<(f64, f64)>, // (price, quantity)
    pub timestamp: String,
}

// ============================================================================
// Signal Types
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct NeuralSignalResponse {
    pub signal_id: String,
    pub symbol: String,
    pub direction: String,  // "long" | "short" | "neutral"
    pub confidence: f64,    // 0.0 - 1.0
    pub strategy: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SignalQuery {
    pub symbol: Option<String>,
    pub strategy: Option<String>,
    pub min_confidence: Option<f64>,
}

// ============================================================================
// Auth Types
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_in: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: String,
}

// ============================================================================
// WebSocket Types
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct WsQuery {
    pub token: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "market_data")]
    MarketData { symbol: String, bid: f64, ask: f64, last: f64 },
    
    #[serde(rename = "order_update")]
    OrderUpdate { order_id: String, status: String, filled_qty: f64 },
    
    #[serde(rename = "position_update")]
    PositionUpdate { symbol: String, quantity: f64, unrealized_pnl: f64 },
    
    #[serde(rename = "signal")]
    Signal { signal_id: String, symbol: String, direction: String, confidence: f64 },
    
    #[serde(rename = "error")]
    Error { message: String },
    
    #[serde(rename = "connected")]
    Connected { session_id: String },
}
