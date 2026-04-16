//! Health Check Endpoints
//! Provides liveness, readiness, and detailed health status

use crate::risk_bus::RiskBus;
use axum::{
    extract::Request,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// Health check status
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Component health information
#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    pub message: Option<String>,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub response_time_ms: Option<u64>,
}

/// Overall health response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub uptime_seconds: u64,
    pub checks: HashMap<String, ComponentHealth>,
    pub version: String,
}

/// Health checker configuration
#[derive(Debug, Clone)]
pub struct HealthCheckerConfig {
    /// Component check timeout
    pub check_timeout: Duration,
    /// How often to run background checks
    pub check_interval: Duration,
    /// How long before a component is considered stale
    pub stale_threshold: Duration,
}

impl Default for HealthCheckerConfig {
    fn default() -> Self {
        Self {
            check_timeout: Duration::from_millis(500),
            check_interval: Duration::from_secs(5),
            stale_threshold: Duration::from_secs(30),
        }
    }
}

/// Health checker for system components
pub struct HealthChecker {
    config: HealthCheckerConfig,
    start_time: Instant,
    pub component_health: Arc<RwLock<HashMap<String, ComponentHealth>>>,
    risk_bus: Arc<RiskBus>,
}

impl HealthChecker {
    /// Create new health checker
    pub fn new(config: HealthCheckerConfig, risk_bus: Arc<RiskBus>) -> Self {
        let component_health = Arc::new(RwLock::new(HashMap::new()));
        
        Self {
            config,
            start_time: Instant::now(),
            component_health,
            risk_bus,
        }
    }

    /// Build health check router
    pub fn build_router(self) -> Router {
        let component_health = Arc::clone(&self.component_health);
        let risk_bus = Arc::clone(&self.risk_bus);
        
        // Start background health checks
        self.start_background_checks();

        Router::new()
            .route("/health/live", get(liveness_handler))
            .route("/health/ready", get({
                let component_health = Arc::clone(&component_health);
                move || readiness_handler(component_health)
            }))
            .route("/health", get({
                let component_health = Arc::clone(&component_health);
                let risk_bus = Arc::clone(&risk_bus);
                move || detailed_health_handler(component_health, risk_bus)
            }))
            .route("/health/detailed", get({
                let component_health = Arc::clone(&component_health);
                let risk_bus = Arc::clone(&risk_bus);
                move || detailed_health_handler(component_health, risk_bus)
            }))
    }

    /// Start background health checking
    pub fn start_background_checks(&self) {
        let component_health = Arc::clone(&self.component_health);
        let config = self.config.clone();
        let risk_bus = Arc::clone(&self.risk_bus);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.check_interval);
            
            loop {
                interval.tick().await;
                
                // Check Risk Bus
                let start = Instant::now();
                let risk_bus_health = check_risk_bus(&risk_bus).await;
                let response_time = start.elapsed().as_millis() as u64;
                
                let health = ComponentHealth {
                    status: risk_bus_health,
                    message: None,
                    last_check: chrono::Utc::now(),
                    response_time_ms: Some(response_time),
                };
                
                component_health.write().await
                    .insert("risk_bus".to_string(), health);

                // Check other components here (Redis, database, etc.)
                // For now, mark them as healthy
                check_component(&component_health, "redis", HealthStatus::Healthy).await;
                check_component(&component_health, "aeron_journal", HealthStatus::Healthy).await;
                check_component(&component_health, "signal_router", HealthStatus::Healthy).await;
            }
        });
    }
}

/// Check Risk Bus health
async fn check_risk_bus(risk_bus: &RiskBus) -> HealthStatus {
    // Check if risk bus is halted
    if risk_bus.is_halted() {
        warn!("Risk bus is halted");
        return HealthStatus::Degraded;
    }

    // Check drawdown
    let dd_bps = risk_bus.dd_bps();
    if dd_bps < -1000 { // More than 10% drawdown
        error!("Critical drawdown: {}bps", dd_bps);
        return HealthStatus::Unhealthy;
    }

    HealthStatus::Healthy
}

/// Update component health
async fn check_component(
    component_health: &Arc<RwLock<HashMap<String, ComponentHealth>>>,
    name: &str,
    status: HealthStatus,
) {
    let health = ComponentHealth {
        status,
        message: None,
        last_check: chrono::Utc::now(),
        response_time_ms: None,
    };
    
    component_health.write().await
        .insert(name.to_string(), health);
}

/// Liveness probe - simple check if process is running
pub async fn liveness_handler() -> StatusCode {
    StatusCode::OK
}

/// Readiness probe - check if all components are ready
pub async fn readiness_handler(
    component_health: Arc<RwLock<HashMap<String, ComponentHealth>>>,
) -> Result<Json<HealthResponse>, StatusCode> {
    let checks = component_health.read().await;
    let timestamp = chrono::Utc::now();
    
    // Determine overall status
    let mut overall_status = HealthStatus::Healthy;
    for health in checks.values() {
        match health.status {
            HealthStatus::Unhealthy => {
                overall_status = HealthStatus::Unhealthy;
                break;
            }
            HealthStatus::Degraded => {
                overall_status = HealthStatus::Degraded;
            }
            HealthStatus::Healthy => {}
        }
    }

    let response = HealthResponse {
        status: overall_status,
        timestamp,
        uptime_seconds: 0, // TODO: Track actual uptime
        checks: checks.clone(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    match overall_status {
        HealthStatus::Healthy => Ok(Json(response)),
        HealthStatus::Degraded => Ok(Json(response)),
        HealthStatus::Unhealthy => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}

/// Detailed health check with all component status
pub async fn detailed_health_handler(
    component_health: Arc<RwLock<HashMap<String, ComponentHealth>>>,
    risk_bus: Arc<RiskBus>,
) -> Json<HealthResponse> {
    let checks = component_health.read().await;
    let timestamp = chrono::Utc::now();
    
    // Add risk bus specific details
    let mut detailed_checks = checks.clone();
    
    // Add risk bus metrics
    let risk_bus_details = ComponentHealth {
        status: check_risk_bus(&risk_bus).await,
        message: Some(format!(
            "halted={}, orders_submitted={}, orders_rejected={}",
            risk_bus.is_halted(),
            risk_bus.orders_submitted_count(),
            risk_bus.orders_rejected_count()
        )),
        last_check: chrono::Utc::now(),
        response_time_ms: None,
    };
    
    detailed_checks.insert("risk_bus_detailed".to_string(), risk_bus_details);

    // Determine overall status
    let mut overall_status = HealthStatus::Healthy;
    for health in detailed_checks.values() {
        match health.status {
            HealthStatus::Unhealthy => {
                overall_status = HealthStatus::Unhealthy;
                break;
            }
            HealthStatus::Degraded => {
                overall_status = HealthStatus::Degraded;
            }
            HealthStatus::Healthy => {}
        }
    }

    Json(HealthResponse {
        status: overall_status,
        timestamp,
        uptime_seconds: 0, // TODO: Track actual uptime
        checks: detailed_checks,
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_liveness_endpoint() {
        let response = liveness_handler().await;
        assert_eq!(response, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_readiness_endpoint() {
        let component_health = Arc::new(RwLock::new(HashMap::new()));
        
        // Add healthy component
        component_health.write().await.insert(
            "test".to_string(),
            ComponentHealth {
                status: HealthStatus::Healthy,
                message: None,
                last_check: chrono::Utc::now(),
                response_time_ms: None,
            }
        );

        let response = readiness_handler(component_health).await;
        assert!(response.is_ok());
        
        let health = response.unwrap().0;
        assert_eq!(health.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_readiness_with_unhealthy_component() {
        let component_health = Arc::new(RwLock::new(HashMap::new()));
        
        // Add unhealthy component
        component_health.write().await.insert(
            "test".to_string(),
            ComponentHealth {
                status: HealthStatus::Unhealthy,
                message: Some("Failed".to_string()),
                last_check: chrono::Utc::now(),
                response_time_ms: None,
            }
        );

        let response = readiness_handler(component_health).await;
        assert!(response.is_err() && response.unwrap_err() == StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_risk_bus_health_check() {
        let risk_bus = RiskBus::new(1_000_000.0, -2000);
        
        // Normal state should be healthy
        let status = check_risk_bus(&risk_bus).await;
        assert_eq!(status, HealthStatus::Healthy);
        
        // Halted state should be degraded
        risk_bus.assert_kill_switch("test");
        let status = check_risk_bus(&risk_bus).await;
        assert_eq!(status, HealthStatus::Degraded);
    }

    #[tokio::test]
    async fn test_health_checker_router() {
        let config = HealthCheckerConfig::default();
        let risk_bus = RiskBus::new(1_000_000.0, -2000);
        let health_checker = HealthChecker::new(config, risk_bus);
        let router = health_checker.build_router();

        // Test liveness endpoint
        let request = Request::builder()
            .uri("/health/live")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Test readiness endpoint
        let request = Request::builder()
            .uri("/health/ready")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = router.clone().oneshot(request).await.unwrap();
        assert!(response.status().is_success());

        // Test detailed health endpoint
        let request = Request::builder()
            .uri("/health/detailed")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = router.clone().oneshot(request).await.unwrap();
        assert!(response.status().is_success());
    }
}
