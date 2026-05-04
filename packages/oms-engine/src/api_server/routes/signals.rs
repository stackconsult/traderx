use axum::{
    extract::{Query, State},
    Json,
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::api_server::{
    error::ApiResult,
    state::AppState,
    types::{NeuralSignalResponse, SignalQuery},
};

pub async fn get_signals(
    State(_state): State<Arc<AppState>>,
    Query(_params): Query<SignalQuery>,
) -> ApiResult<Json<Vec<NeuralSignalResponse>>> {
    let signal = NeuralSignalResponse {
        signal_id: Uuid::new_v4().to_string(),
        symbol: "AAPL".to_string(),
        direction: "long".to_string(),
        confidence: 0.85,
        strategy: "default".to_string(),
        timestamp: Utc::now().to_rfc3339(),
    };
    Ok(Json(vec![signal]))
}
