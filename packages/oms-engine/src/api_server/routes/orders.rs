use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::api_server::{
    error::{ApiError, ApiResult},
    state::AppState,
    types::{CreateOrderRequest, ListOrdersQuery, OrderResponse},
};

pub async fn create_order(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CreateOrderRequest>,
) -> ApiResult<Json<OrderResponse>> {
    let order_id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();

    let response = OrderResponse {
        order_id,
        status: "pending".to_string(),
        symbol: req.symbol,
        side: req.side,
        quantity: req.quantity,
        filled_quantity: 0.0,
        avg_price: None,
        created_at,
    };

    Ok(Json(response))
}

pub async fn list_orders(
    State(_state): State<Arc<AppState>>,
    Query(_params): Query<ListOrdersQuery>,
) -> ApiResult<Json<Vec<OrderResponse>>> {
    Ok(Json(vec![]))
}

pub async fn get_order(
    State(_state): State<Arc<AppState>>,
    Path(order_id): Path<String>,
) -> ApiResult<Json<OrderResponse>> {
    Err(ApiError::OrderNotFound { order_id })
}

pub async fn cancel_order(
    State(_state): State<Arc<AppState>>,
    Path(_order_id): Path<String>,
) -> ApiResult<impl IntoResponse> {
    Ok(())
}
