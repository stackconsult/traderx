//! API Error Types and Handling

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug)]
pub enum ApiError {
    // Authentication errors
    Unauthorized { message: String },
    InvalidToken { message: String },
    
    // Validation errors
    ValidationError { field: String, message: String },
    InvalidRequest { message: String },
    
    // Not found errors
    OrderNotFound { order_id: String },
    SymbolNotFound { symbol: String },
    
    // Business logic errors
    RiskCheckFailed { reason: String },
    OrderRejected { order_id: String, reason: String },
    InsufficientFunds { required: f64, available: f64 },
    
    // System errors
    InternalError { message: String },
    OmsError { message: String },
    
    // Rate limiting
    RateLimited { retry_after: u64 },
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unauthorized { message } => write!(f, "Unauthorized: {}", message),
            Self::InvalidToken { message } => write!(f, "Invalid token: {}", message),
            Self::ValidationError { field, message } => write!(f, "Validation error for {}: {}", field, message),
            Self::InvalidRequest { message } => write!(f, "Invalid request: {}", message),
            Self::OrderNotFound { order_id } => write!(f, "Order not found: {}", order_id),
            Self::SymbolNotFound { symbol } => write!(f, "Symbol not found: {}", symbol),
            Self::RiskCheckFailed { reason } => write!(f, "Risk check failed: {}", reason),
            Self::OrderRejected { order_id, reason } => write!(f, "Order {} rejected: {}", order_id, reason),
            Self::InsufficientFunds { required, available } => write!(f, "Insufficient funds: required {}, available {}", required, available),
            Self::InternalError { message } => write!(f, "Internal error: {}", message),
            Self::OmsError { message } => write!(f, "OMS error: {}", message),
            Self::RateLimited { retry_after } => write!(f, "Rate limited. Retry after {} seconds", retry_after),
        }
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            Self::Unauthorized { message } => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", message.clone()),
            Self::InvalidToken { message } => (StatusCode::UNAUTHORIZED, "INVALID_TOKEN", message.clone()),
            Self::ValidationError { field: _, message } => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", message.clone()),
            Self::InvalidRequest { message } => (StatusCode::BAD_REQUEST, "INVALID_REQUEST", message.clone()),
            Self::OrderNotFound { order_id } => (StatusCode::NOT_FOUND, "ORDER_NOT_FOUND", format!("Order {} not found", order_id)),
            Self::SymbolNotFound { symbol } => (StatusCode::NOT_FOUND, "SYMBOL_NOT_FOUND", format!("Symbol {} not found", symbol)),
            Self::RiskCheckFailed { reason } => (StatusCode::FORBIDDEN, "RISK_CHECK_FAILED", reason.clone()),
            Self::OrderRejected { order_id: _, reason } => (StatusCode::FORBIDDEN, "ORDER_REJECTED", reason.clone()),
            Self::InsufficientFunds { required: _, available: _ } => (StatusCode::FORBIDDEN, "INSUFFICIENT_FUNDS", self.to_string()),
            Self::InternalError { message } => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", message.clone()),
            Self::OmsError { message } => (StatusCode::INTERNAL_SERVER_ERROR, "OMS_ERROR", message.clone()),
            Self::RateLimited { retry_after } => (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMITED", format!("Retry after {} seconds", retry_after)),
        };

        let body = Json(json!({
            "error": {
                "code": error_code,
                "message": message,
            }
        }));

        (status, body).into_response()
    }
}
