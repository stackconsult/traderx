use axum::{extract::State, response::IntoResponse, Json};
use std::sync::Arc;

use crate::api_server::{
    error::ApiResult,
    state::AppState,
    types::{PortfolioSummaryResponse, PositionResponse},
};

pub async fn get_positions(
    State(_state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<PositionResponse>>> {
    Ok(Json(vec![]))
}

pub async fn get_portfolio_summary(
    State(_state): State<Arc<AppState>>,
) -> ApiResult<Json<PortfolioSummaryResponse>> {
    let summary = PortfolioSummaryResponse {
        total_value: 0.0,
        cash_balance: 0.0,
        total_exposure: 0.0,
        total_unrealized_pnl: 0.0,
        total_realized_pnl: 0.0,
        buying_power: 0.0,
        margin_used: 0.0,
    };
    Ok(Json(summary))
}
