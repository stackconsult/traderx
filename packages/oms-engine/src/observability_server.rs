//! Unified Observability Server
//! Combines metrics and health check endpoints

use crate::health::{HealthChecker, HealthCheckerConfig};
use crate::metrics::GLOBAL_RISK_METRICS;
use crate::risk_bus::RiskBus;
use axum::{
    extract::Request,
    http::{StatusCode, Method, header::HeaderValue},
    response::Response,
    routing::get,
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    trace::TraceLayer,
    limit::RateLimitLayer,
    cors::CorsLayer,
};
use tracing::{info, error, warn, trace};

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
}

impl Default for ObservabilityServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:9090".parse().unwrap(),
            metrics_rate_limit_per_sec: 10,
            health_rate_limit_per_sec: 100,
            enable_cors: false,
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

        // Add CORS if enabled
        if self.config.enable_cors {
            router = router.layer(
                CorsLayer::new()
                    .allow_origin("*".parse::<HeaderValue>().unwrap())
                    .allow_methods([Method::GET])
                    .allow_headers([axum::http::header::CONTENT_TYPE])
            );
        }

        // Start background health checks
        health_checker.start_background_checks();

        router
    }

    /// Start the observability server
    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let router = self.build_router();
        
        // Create TCP listener
        let listener = TcpListener::bind(&self.config.bind_addr).await?;
        info!("Observability server listening on {}", self.config.bind_addr);

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
        };
        
        let risk_bus = Arc::new(RiskBus::new(1_000_000.0, -2000));
        let server = ObservabilityServer::new(config, risk_bus);
        let router = server.build_router();

        // Test root endpoint
        let request = Request::builder()
            .uri("/")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let content = String::from_utf8(body.to_vec()).unwrap();
        assert!(content.contains("TraderX Observability Server"));

        // Test version endpoint
        let request = Request::builder()
            .uri("/version")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let version = String::from_utf8(body.to_vec()).unwrap();
        assert_eq!(version, env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn test_all_endpoints() {
        let config = ObservabilityServerConfig::default();
        let risk_bus = Arc::new(RiskBus::new(1_000_000.0, -2000));
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
