//! Observability Server Example
//! Launches the metrics and health endpoints for visual inspection

use oms_engine::{
    observability_server::{ObservabilityServer, ObservabilityServerConfig},
    risk_bus::RiskBus,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║     TraderX Observability Server - Demo Mode             ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create RiskBus for health checks
    let risk_bus = RiskBus::new(10_000_000.0, -2000);
    println!("✅ Risk Bus initialized");

    // Configure observability server
    let config = ObservabilityServerConfig {
        bind_addr: "0.0.0.0:9090".parse()?,
        metrics_rate_limit_per_sec: 100,
        health_rate_limit_per_sec: 1000,
        enable_cors: true,
        cors_allowed_origins: vec!["http://localhost:3000".to_string()],
    };

    println!("📊 Starting Observability Server on http://0.0.0.0:9090");
    println!();
    println!("Available endpoints:");
    println!("  📈 http://0.0.0.0:9090/           - Server info");
    println!("  📈 http://0.0.0.0:9090/metrics    - Prometheus metrics");
    println!("  💚 http://0.0.0.0:9090/health     - Health check");
    println!("  💚 http://0.0.0.0:9090/health/live - Liveness probe");
    println!("  💚 http://0.0.0.0:9090/health/ready - Readiness probe");
    println!("  💚 http://0.0.0.0:9090/health/detailed - Detailed health");
    println!("  📦 http://0.0.0.0:9090/version   - Version info");
    println!();
    println!("Press Ctrl+C to stop");
    println!();

    // Create and run server
    let server = ObservabilityServer::new(config, risk_bus);
    server.serve().await.map_err(|e| {
        eprintln!("Server error: {:?}", e);
        std::io::Error::new(std::io::ErrorKind::Other, "Server failed")
    })?;

    Ok(())
}
