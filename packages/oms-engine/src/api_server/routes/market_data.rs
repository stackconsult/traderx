use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::Utc;
use std::sync::Arc;

use crate::api_server::{
    error::{ApiError, ApiResult},
    state::AppState,
    types::{MarketDataResponse, OrderBookQuery, OrderBookResponse},
};

pub async fn get_market_data(
    State(_state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
) -> ApiResult<Json<MarketDataResponse>> {
    let response = MarketDataResponse {
        symbol,
        bid: 0.0,
        ask: 0.0,
        last_price: 0.0,
        volume_24h: 0.0,
        timestamp: Utc::now().to_rfc3339(),
    };
    Ok(Json(response))
}

pub async fn get_order_book(
    State(_state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
    Query(params): Query<OrderBookQuery>,
) -> ApiResult<Json<OrderBookResponse>> {
    let depth = params.depth.unwrap_or(10);
    let bids = vec![(100.0, 1.0); depth];
    let asks = vec![(101.0, 1.0); depth];

    let response = OrderBookResponse {
        symbol,
        bids,
        asks,
        timestamp: Utc::now().to_rfc3339(),
    };
    Ok(Json(response))
}
