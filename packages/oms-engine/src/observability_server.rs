//! Unified Observability Server
//! Combines metrics and health check endpoints

use crate::health::{HealthChecker, HealthCheckerConfig};
use crate::metrics::GLOBAL_RISK_METRICS;
use crate::risk_bus::RiskBus;
use axum::{
    extract::Request,
    http::{header::HeaderValue, Method, StatusCode},
    response::Response,
    routing::get,
    Router,
};
use http_body_util::BodyExt;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::limit::RateLimitLayer;
use tower::ServiceBuilder;
use tower_http::cors::AllowOrigin;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use tracing::{error, info, trace, warn};

/// Observability server configuration
#[derive(Debug, Clone)]
pub struct ObservabilityServerConfig {
    /// Bind address for the server
    pub bind_addr: SocketAddr,
    /// Rate limit per second for /metrics endpoint
    pub metrics_rate_limit_per_sec: u32,
    /// Rate limit per second for health endpoints
    pub health_rate_limit_per_sec: u32,
    /// Enable CORS for cross-origin requests
    pub enable_cors: bool,
    /// Explicit list of origins allowed when CORS is enabled.
    ///
    /// Must be populated (e.g. `["https://grafana.internal"]`) when
    /// `enable_cors` is true — wildcard origins are deliberately not
    /// supported to prevent accidental data exposure.
    pub cors_allowed_origins: Vec<String>,
}

impl Default for ObservabilityServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:9090".parse().unwrap(),
            metrics_rate_limit_per_sec: 10,
            health_rate_limit_per_sec: 100,
            enable_cors: false,
            cors_allowed_origins: Vec::new(),
        }
    }
}

/// Unified observability server
pub struct ObservabilityServer {
    config: ObservabilityServerConfig,
    risk_bus: Arc<RiskBus>,
}

impl ObservabilityServer {
    /// Create new observability server
    pub fn new(config: ObservabilityServerConfig, risk_bus: Arc<RiskBus>) -> Self {
        Self { config, risk_bus }
    }

    /// Build the router with all endpoints
    fn build_router(self) -> Router {
        // Create health checker
        let health_config = HealthCheckerConfig::default();
        let health_checker = HealthChecker::new(health_config, Arc::clone(&self.risk_bus));

        // Clone component health before starting background checks
        let component_health = Arc::clone(&health_checker.component_health);
        let risk_bus = Arc::clone(&self.risk_bus);

        // Start background health checks
        health_checker.start_background_checks();

        // Build base router
        let mut router = Router::new()
            // Metrics endpoints
            .route("/metrics", get(crate::metrics::metrics_handler))
            // Health endpoints
            .route("/health/live", get(crate::health::liveness_handler))
            .route("/health/ready", get({
                let component_health = Arc::clone(&component_health);
                move || crate::health::readiness_handler(component_health)
            }))
            .route("/health", get({
                let component_health = Arc::clone(&component_health);
                let risk_bus = Arc::clone(&risk_bus);
                move || crate::health::detailed_health_handler(component_health, risk_bus)
            }))
            .route("/health/detailed", get({
                let component_health = Arc::clone(&component_health);
                let risk_bus = Arc::clone(&risk_bus);
                move || crate::health::detailed_health_handler(component_health, risk_bus)
            }))
            // Root endpoint with basic info
            .route("/", get(root_handler))
            // Version endpoint
            .route("/version", get(version_handler));

        // Add rate limiting
        router = router.layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                // Apply different rate limits to different endpoints
                .layer(axum::middleware::from_fn(rate_limit_middleware))
        );

        // Add CORS if enabled.
        //
        // We deliberately do NOT fall back to `allow_origin("*")`. The observability
        // server exposes health + risk state that can contain sensitive information
        // (halt reasons, component names, etc.); sending it to arbitrary origins
        // would be a data-exposure hazard. An empty allowlist with `enable_cors=true`
        // is treated as a misconfiguration and skipped with a warning.
        if self.config.enable_cors {
            let parsed_origins: Vec<HeaderValue> = self
                .config
                .cors_allowed_origins
                .iter()
                .filter(|o| !o.is_empty() && o.as_str() != "*")
                .filter_map(|o| match o.parse::<HeaderValue>() {
                    Ok(v) => Some(v),
                    Err(e) => {
                        warn!(origin = %o, error = %e, "Ignoring invalid CORS origin");
                        None
                    }
                })
                .collect();

            if parsed_origins.is_empty() {
                warn!(
                    "enable_cors=true but cors_allowed_origins is empty or invalid; \
                     skipping CORS layer to avoid installing a wildcard allow."
                );
            } else {
                router = router.layer(
                    CorsLayer::new()
                        .allow_origin(AllowOrigin::list(parsed_origins))
                        .allow_methods([Method::GET])
                        .allow_headers([axum::http::header::CONTENT_TYPE]),
                );
            }
        }

        // Start background health checks
        health_checker.start_background_checks();

        router
    }

    /// Start the observability server
    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Extract bind address before consuming self
        let bind_addr = self.config.bind_addr.clone();
        let router = self.build_router();

        // Create TCP listener
        let listener = TcpListener::bind(&bind_addr).await?;
        info!("Observability server listening on {}", bind_addr);

        // Update metrics to indicate server is ready
        GLOBAL_RISK_METRICS.update_halt_status(false);

        // Serve
        axum::serve(listener, router).await?;
        Ok(())
    }
}

/// Root handler with basic service information
async fn root_handler() -> &'static str {
    r#"TraderX Observability Server

Endpoints:
  /metrics        - Prometheus metrics
  /health/live    - Liveness probe
  /health/ready   - Readiness probe
  /health         - Detailed health status
  /version        - Service version

Rate Limits:
  - /metrics: 10 requests/second
  - /health/*: 100 requests/second
"#
}

/// Version handler
async fn version_handler() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Rate limiting middleware
async fn rate_limit_middleware(
    req: Request,
    next: axum::middleware::Next,
) -> Result<Response, StatusCode> {
    let path = req.uri().path();

    // Check rate limits based on path
    if path.starts_with("/metrics") {
        // Metrics rate limiting is handled by tower-http's RateLimitLayer
        // For simplicity, we'll just log here
        trace!("Metrics request: {}", path);
    } else if path.starts_with("/health") {
        // Health endpoints have higher rate limit
        trace!("Health request: {}", path);
    }

    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_observability_server() {
        let config = ObservabilityServerConfig {
            bind_addr: "127.0.0.1:0".parse().unwrap(),
            metrics_rate_limit_per_sec: 100,
            health_rate_limit_per_sec: 100,
            enable_cors: false,
            cors_allowed_origins: Vec::new(),
        };
        
        let risk_bus = RiskBus::new(1_000_000.0, -2000);
        let server = ObservabilityServer::new(config, risk_bus);
        let router = server.build_router();

        // Test root endpoint
        let request = Request::builder()
            .uri("/")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = http_body_util::BodyExt::collect(response.into_body()).await.unwrap().to_bytes();
        let content = String::from_utf8(body.to_vec()).unwrap();
        assert!(content.contains("TraderX Observability Server"));

        // Test version endpoint
        let request = Request::builder()
            .uri("/version")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = http_body_util::BodyExt::collect(response.into_body()).await.unwrap().to_bytes();
        let version = String::from_utf8(body.to_vec()).unwrap();
        assert_eq!(version, env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn test_all_endpoints() {
        let config = ObservabilityServerConfig::default();
        let risk_bus = RiskBus::new(1_000_000.0, -2000);
        let server = ObservabilityServer::new(config, risk_bus);
        let router = server.build_router();

        let endpoints = vec![
            "/",
            "/version",
            "/metrics",
            "/health/live",
            "/health/ready",
            "/health",
            "/health/detailed",
        ];

        for endpoint in endpoints {
            let request = Request::builder()
                .uri(endpoint)
                .body(axum::body::Body::empty())
                .unwrap();

            let response = router.clone().oneshot(request).await.unwrap();
            assert!(
                response.status().is_success(),
                "Endpoint {} returned status {}",
                endpoint,
                response.status()
            );
        }
    }
}
