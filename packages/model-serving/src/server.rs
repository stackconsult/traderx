//! HTTP API server for model serving with Axum.

use crate::inference::{InferenceEngine, InferenceRequest, InferenceResponse};
use crate::model::ModelRegistry;
use crate::metrics::metrics;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};
use tracing::{error, info};

/// Build a locked-down CORS layer for the model-serving HTTP API.
///
/// Allowed origins come from `MODEL_SERVING_ALLOWED_ORIGINS` (comma-separated).
/// When unset, CORS is closed (no browser origins are allowed) — the API is
/// meant to be called server-to-server. Wildcard origins are never accepted.
fn build_cors_layer() -> CorsLayer {
    let raw = std::env::var("MODEL_SERVING_ALLOWED_ORIGINS").unwrap_or_default();
    let origins: Vec<axum::http::HeaderValue> = raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "*")
        .filter_map(|o| o.parse().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ])
}

/// Model server state.
#[derive(Clone)]
pub struct ModelServer {
    inference_engine: Arc<InferenceEngine>,
    model_registry: Arc<ModelRegistry>,
}

/// Query parameters for model listing.
#[derive(Debug, Deserialize)]
pub struct ListModelsQuery {
    pub stage: Option<String>,
    pub limit: Option<usize>,
}

/// Model registration request.
#[derive(Debug, Deserialize)]
pub struct RegisterModelRequest {
    pub name: String,
    pub version: String,
    pub model_uri: String,
    pub description: Option<String>,
}

/// A/B test configuration.
#[derive(Debug, Deserialize)]
pub struct ABTestConfig {
    pub model_a: String,
    pub model_b: String,
    pub traffic_split: f64, // 0.0-1.0 for model A
}

/// Create and configure the model server.
pub fn create_server(
    inference_engine: Arc<InferenceEngine>,
    model_registry: Arc<ModelRegistry>,
) -> Router {
    let server_state = ModelServer {
        inference_engine,
        model_registry,
    };

    Router::new()
        .route("/health", get(health_check))
        .route("/models", get(list_models).post(register_model))
        .route("/models/:name/versions/:version/predict", post(predict))
        .route("/models/:name/predict", post(predict_production))
        .route("/models/:name/ab-test", post(ab_test_predict))
        .route("/metrics", get(prometheus_metrics))
        .route("/cache/stats", get(cache_stats))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(build_cors_layer()),
        )
        .with_state(server_state)
}

/// Health check endpoint.
async fn health_check() -> Result<Json<HealthStatus>, StatusCode> {
    Ok(Json(HealthStatus {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
    }))
}

#[derive(Debug, Serialize)]
struct HealthStatus {
    status: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// List all registered models.
async fn list_models(
    State(server): State<ModelServer>,
    Query(params): Query<ListModelsQuery>,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    match server.model_registry.list_models().await {
        Ok(models) => {
            let filtered = if let Some(stage) = params.stage {
                models.into_iter()
                    .filter(|m| m.stage == stage)
                    .collect()
            } else {
                models
            };

            let limited = if let Some(limit) = params.limit {
                filtered.into_iter().take(limit).collect()
            } else {
                filtered
            };

            let json_models: Vec<serde_json::Value> = limited
                .into_iter()
                .map(|m| serde_json::to_value(m).unwrap())
                .collect();

            Ok(Json(json_models))
        }
        Err(e) => {
            error!("Failed to list models: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Register a new model.
async fn register_model(
    State(server): State<ModelServer>,
    Json(request): Json<RegisterModelRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // In production, this would validate and register with MLflow
    info!("Registering model {}:{}", request.name, request.version);

    Ok(Json(serde_json::json!({
        "name": request.name,
        "version": request.version,
        "status": "registered"
    })))
}

/// Predict with specific model version.
async fn predict(
    State(server): State<ModelServer>,
    Path((name, version)): Path<(String, String)>,
    Json(request): Json<InferenceRequest>,
) -> Result<Json<InferenceResponse>, StatusCode> {
    let mut req = request;
    req.model_name = name;
    req.model_version = Some(version);

    match server.inference_engine.predict(req).await {
        Ok(response) => {
            metrics().record_inference(response.latency_ns as u64);
            Ok(Json(response))
        }
        Err(e) => {
            error!("Prediction failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Predict with production model version.
async fn predict_production(
    State(server): State<ModelServer>,
    Path(name): Path<String>,
    Json(request): Json<InferenceRequest>,
) -> Result<Json<InferenceResponse>, StatusCode> {
    let mut req = request;
    req.model_name = name;

    match server.inference_engine.predict(req).await {
        Ok(response) => {
            metrics().record_inference(response.latency_ns as u64);
            Ok(Json(response))
        }
        Err(e) => {
            error!("Prediction failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// A/B test prediction endpoint.
async fn ab_test_predict(
    State(server): State<ModelServer>,
    Path(name): Path<String>,
    Json(request): Json<InferenceRequest>,
    Query(config): Query<ABTestConfig>,
) -> Result<Json<ABTestResponse>, StatusCode> {
    // Choose model based on traffic split
    let use_model_a = fastrand::f64() < config.traffic_split;
    let model_version = if use_model_a { &config.model_a } else { &config.model_b };

    let mut req = request;
    req.model_name = name;
    req.model_version = Some(model_version.clone());

    match server.inference_engine.predict(req).await {
        Ok(response) => {
            metrics().record_inference(response.latency_ns as u64);
            Ok(Json(ABTestResponse {
                response,
                model_used: model_version.clone(),
                test_group: if use_model_a { "A" } else { "B" },
            }))
        }
        Err(e) => {
            error!("A/B test prediction failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Serialize)]
struct ABTestResponse {
    #[serde(flatten)]
    response: InferenceResponse,
    model_used: String,
    test_group: &'static str,
}

/// Prometheus metrics endpoint.
async fn prometheus_metrics() -> Result<String, StatusCode> {
    use prometheus::Encoder;

    let encoder = prometheus::TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();

    encoder.encode(&metric_families, &mut buffer).map_err(|e| {
        error!("Failed to encode metrics: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(String::from_utf8(buffer).unwrap_or_default())
}

/// Cache statistics endpoint.
async fn cache_stats(
    State(server): State<ModelServer>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // This would require exposing cache stats from the inference engine
    Ok(Json(serde_json::json!({
        "cache_stats": "not implemented"
    })))
}
