use model_serving::{
    ModelServer, ModelRegistry, InferenceEngine, FeatureStore, FeatureCache,
    TurboQuantCompressor, metrics,
};
use std::sync::Arc;
use std::net::SocketAddr;
use tokio::signal;
use tracing::{info, error};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    // Configuration
    let mlflow_uri = std::env::var("MLFLOW_TRACKING_URI")
        .unwrap_or_else(|_| "http://localhost:5000".to_string());
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379/0".to_string());
    let host = std::env::var("HOST")
        .unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8000);

    // Initialize components
    info!("Initializing model serving infrastructure...");
    
    // Model registry
    let model_registry = Arc::new(ModelRegistry::new(mlflow_uri));
    
    // Feature store and cache
    let feature_store = Arc::new(FeatureStore::new(&redis_url).await?);
    let feature_cache = Arc::new(FeatureCache::new(50)); // 50ms TTL
    
    // TurboQuant compressor
    let compressor = Arc::new(TurboQuantCompressor::new());
    
    // Inference engine
    let inference_engine = Arc::new(InferenceEngine::new(
        Arc::clone(&model_registry),
        Arc::clone(&feature_store),
        Arc::clone(&feature_cache),
        Arc::clone(&compressor),
    ));
    
    // Create HTTP server
    let app = ModelServer::create_server(
        Arc::clone(&inference_engine),
        Arc::clone(&model_registry),
    );
    
    // Warm up models
    let warmup_models = vec![
        "price_predictor".to_string(),
        "volatility_forecaster".to_string(),
        "signal_generator".to_string(),
    ];
    
    if let Err(e) = inference_engine.warm_up(&warmup_models).await {
        error!("Failed to warm up models: {}", e);
    }
    
    // Set default models for symbols
    inference_engine.set_default_model("BTC-USD", "1").await;
    inference_engine.set_default_model("ETH-USD", "1").await;
    inference_engine.set_default_model("SPY", "2").await;
    
    // Start metrics server
    let metrics_addr = SocketAddr::from(([0, 0, 0, 0], 9090));
    let metrics_app = axum::Router::new().route("/metrics", axum::routing::get(|| async {
        use prometheus::Encoder;
        let encoder = prometheus::TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }));
    
    tokio::spawn(async move {
        axum::Server::bind(&metrics_addr)
            .serve(metrics_app.into_make_service())
            .await
            .unwrap();
    });
    
    // Start main server
    let addr = SocketAddr::new(host.parse()?, port);
    info!("Model server listening on {}", addr);
    info!("Metrics available on http://:9090/metrics");
    
    // Graceful shutdown
    let server = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal());
    
    if let Err(e) = server.await {
        error!("Server error: {}", e);
    }
    
    info!("Model server shut down gracefully");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };
    
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    
    info!("Shutdown signal received");
}
