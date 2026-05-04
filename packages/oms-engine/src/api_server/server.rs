use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::{
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    trace::TraceLayer,
};
use tracing::info;

use crate::api_server::{
    auth::{jwt_middleware, login},
    routes::{auth as auth_routes, health, market_data, orders, positions, signals},
    state::AppState,
    websocket::ws_handler,
};

pub async fn run_server(app_state: Arc<AppState>, port: u16) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/api/v1/health", get(health::health_check))
        .route("/api/v1/auth/login", post(login))
        .route(
            "/api/v1/orders",
            get(orders::list_orders).post(orders::create_order),
        )
        .route(
            "/api/v1/orders/:id",
            get(orders::get_order).delete(orders::cancel_order),
        )
        .route("/api/v1/positions", get(positions::get_positions))
        .route(
            "/api/v1/portfolio/summary",
            get(positions::get_portfolio_summary),
        )
        .route("/api/v1/market-data/:symbol", get(market_data::get_market_data))
        .route(
            "/api/v1/market-data/order-book/:symbol",
            get(market_data::get_order_book),
        )
        .route("/api/v1/signals", get(signals::get_signals))
        .route("/ws", get(ws_handler))
        .layer(middleware::from_fn_with_state(app_state.clone(), jwt_middleware))
        .layer(CorsLayer::permissive())
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    info!("API server listening on {}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
