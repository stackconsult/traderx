//! Metrics HTTP Server
//! Exposes Prometheus metrics at /metrics endpoint

use crate::metrics::{metrics_handler, GLOBAL_RISK_METRICS, MetricsRateLimiter};
use axum::{
    extract::Request,
    http::StatusCode,
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
};
use tracing::{info, error, warn};

/// Metrics server configuration
#[derive(Debug, Clone)]
pub struct MetricsServerConfig {
    /// Bind address for metrics server
    pub bind_addr: SocketAddr,
    /// Rate limit per second for /metrics endpoint
    pub rate_limit_per_sec: u32,
}

impl Default for MetricsServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:9090".parse().unwrap(),
            rate_limit_per_sec: 10,
        }
    }
}

/// Metrics HTTP server
pub struct MetricsServer {
    config: MetricsServerConfig,
    rate_limiter: Arc<MetricsRateLimiter>,
}

impl MetricsServer {
    /// Create new metrics server
    pub fn new(config: MetricsServerConfig) -> Self {
        Self {
            rate_limiter: Arc::new(MetricsRateLimiter::new(config.rate_limit_per_sec)),
            config,
        }
    }

    /// Build the router with rate limiting
    fn build_router(&self) -> Router {
        Router::new()
            .route("/metrics", get(metrics_handler))
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(RateLimitLayer::new(self.config.rate_limit_per_sec, std::time::Duration::from_secs(1)))
            )
            .fallback(handler_404)
    }

    /// Start the metrics server
    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let router = self.build_router();
        
        // Create TCP listener
        let listener = TcpListener::bind(&self.config.bind_addr).await?;
        info!("Metrics server listening on {}", self.config.bind_addr);

        // Update metrics to indicate server is ready
        GLOBAL_RISK_METRICS.update_halt_status(false);

        // Serve
        axum::serve(listener, router).await?;
        Ok(())
    }
}

/// 404 handler
async fn handler_404() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not Found")
}

/// Health check endpoint for metrics server
pub async fn health_check() -> &'static str {
    "OK"
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let config = MetricsServerConfig {
            bind_addr: "127.0.0.1:0".parse().unwrap(),
            rate_limit_per_sec: 100,
        };
        let server = MetricsServer::new(config);
        let router = server.build_router();

        // Test metrics endpoint
        let request = Request::builder()
            .uri("/metrics")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let metrics = String::from_utf8(body.to_vec()).unwrap();
        assert!(metrics.contains("riskbus_"));
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let config = MetricsServerConfig {
            bind_addr: "127.0.0.1:0".parse().unwrap(),
            rate_limit_per_sec: 1,
        };
        let server = MetricsServer::new(config);
        let router = server.build_router();

        // First request should succeed
        let request = Request::builder()
            .uri("/metrics")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Immediate second request should be rate limited
        let request = Request::builder()
            .uri("/metrics")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn test_404_handler() {
        let config = MetricsServerConfig::default();
        let server = MetricsServer::new(config);
        let router = server.build_router();

        let request = Request::builder()
            .uri("/unknown")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
