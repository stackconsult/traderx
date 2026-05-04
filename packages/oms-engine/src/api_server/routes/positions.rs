use axum::{extract::State, Json};
use std::sync::Arc;

use crate::api_server::{
    error::ApiResult,
    state::AppState,
    types::{PortfolioSummaryResponse, PositionResponse},
};

pub async fn get_positions(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<PositionResponse>>> {
    let positions = state.positions.lock().await;
    Ok(Json(positions.clone()))
}

pub async fn get_portfolio_summary(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<PortfolioSummaryResponse>> {
    let positions = state.positions.lock().await;
    let total_unrealized_pnl: f64 = positions.iter().map(|p| p.unrealized_pnl).sum();
    let total_exposure: f64 = positions.iter().map(|p| p.market_value).sum();
    let cash_balance = 100_000.0_f64; // paper trading starting balance
    let summary = PortfolioSummaryResponse {
        total_value: cash_balance + total_exposure,
        cash_balance,
        total_exposure,
        total_unrealized_pnl,
        total_realized_pnl: positions.iter().map(|p| p.realized_pnl).sum(),
        buying_power: cash_balance,
        margin_used: 0.0,
    };
    Ok(Json(summary))
}
