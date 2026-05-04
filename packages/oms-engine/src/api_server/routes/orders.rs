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
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateOrderRequest>,
) -> ApiResult<Json<OrderResponse>> {
    let order_id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();

    let response = OrderResponse {
        order_id: order_id.clone(),
        status: "pending".to_string(),
        symbol: req.symbol.clone(),
        side: req.side.clone(),
        quantity: req.quantity,
        filled_quantity: 0.0,
        avg_price: None,
        created_at,
    };

    state.orders.lock().await.push(response.clone());

    let _ = state
        .market_feed
        .send(crate::api_server::types::WsMessage::OrderUpdate {
            order_id,
            status: "pending".to_string(),
            filled_qty: 0.0,
        });

    Ok(Json(response))
}

pub async fn list_orders(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListOrdersQuery>,
) -> ApiResult<Json<Vec<OrderResponse>>> {
    let orders = state.orders.lock().await;
    let filtered: Vec<OrderResponse> = orders
        .iter()
        .filter(|o| {
            params.status.as_ref().map_or(true, |s| &o.status == s)
                && params.symbol.as_ref().map_or(true, |s| &o.symbol == s)
        })
        .take(params.limit.unwrap_or(100))
        .cloned()
        .collect();
    Ok(Json(filtered))
}

pub async fn get_order(
    State(state): State<Arc<AppState>>,
    Path(order_id): Path<String>,
) -> ApiResult<Json<OrderResponse>> {
    let orders = state.orders.lock().await;
    orders
        .iter()
        .find(|o| o.order_id == order_id)
        .cloned()
        .map(Json)
        .ok_or(ApiError::OrderNotFound { order_id })
}

pub async fn cancel_order(
    State(state): State<Arc<AppState>>,
    Path(order_id): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let mut orders = state.orders.lock().await;
    let order = orders
        .iter_mut()
        .find(|o| o.order_id == order_id && o.status == "pending")
        .ok_or_else(|| ApiError::OrderNotFound {
            order_id: order_id.clone(),
        })?;
    order.status = "cancelled".to_string();
    let _ = state
        .market_feed
        .send(crate::api_server::types::WsMessage::OrderUpdate {
            order_id,
            status: "cancelled".to_string(),
            filled_qty: 0.0,
        });
    Ok(())
}
