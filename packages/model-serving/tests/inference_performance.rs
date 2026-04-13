//! Model Serving Inference Performance Tests
//! Validates <1ms inference at 10k RPS requirement

use model_serving::server::create_server;
use model_serving::inference::{InferenceEngine, InferenceRequest, InferenceResponse};
use model_serving::model::ModelRegistry;
use model_serving::features::FeatureStore;
use model_serving::compression::TurboQuantCompressor;
use axum::{
    body::Body,
    extract::Request,
    http::{StatusCode, Method},
    response::Response,
    Router,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tower::ServiceExt;
use tempfile::TempDir;

#[tokio::test]
async fn test_inference_latency_direct() {
    // Test inference engine directly without HTTP overhead
    let temp_dir = TempDir::new().unwrap();
    let model_registry = Arc::new(ModelRegistry::new(temp_dir.path().to_str().unwrap()));
    let feature_store = Arc::new(FeatureStore::new());
    let compressor = Arc::new(TurboQuantCompressor::new());
    
    let inference_engine = InferenceEngine::new(
        model_registry.clone(),
        feature_store.clone(),
        feature_store.get_cache(),
        compressor,
    );
    
    // Create a simple test request
    let mut features = HashMap::new();
    features.insert("price".to_string(), 150.0);
    features.insert("volume".to_string(), 1000000.0);
    features.insert("rsi".to_string(), 0.5);
    
    let request = InferenceRequest {
        model_name: "test_model".to_string(),
        model_version: Some("1".to_string()),
        symbol: "AAPL".to_string(),
        timestamp_ns: chrono::Utc::now().timestamp_nanos(),
        features: Some(features),
    };
    
    // Measure inference latency
    let num_samples = 1000;
    let mut latencies = Vec::new();
    
    for _ in 0..num_samples {
        let start = Instant::now();
        
        // Direct inference call
        let _response = inference_engine.predict(request.clone()).await;
        
        let latency = start.elapsed();
        latencies.push(latency);
        
        // Small delay between samples
        tokio::time::sleep(Duration::from_micros(10)).await;
    }
    
    // Calculate statistics
    latencies.sort();
    let avg_latency = latencies.iter().sum::<Duration>() / num_samples as u32;
    let p95_latency = latencies[(num_samples as f64 * 0.95) as usize];
    let p99_latency = latencies[(num_samples as f64 * 0.99) as usize];
    
    println!("Direct Inference Latency:");
    println!("  Average: {:?}", avg_latency);
    println!("  P95: {:?}", p95_latency);
    println!("  P99: {:?}", p99_latency);
    
    // Validate latency requirement (should be very fast without HTTP)
    assert!(p95_latency.as_millis() < 1,
        "P95 latency {:?} > 1ms requirement", p95_latency);
}

#[tokio::test]
async fn test_http_handler_latency() {
    // Test HTTP handler latency using oneshot (no server spawn)
    let temp_dir = TempDir::new().unwrap();
    let model_registry = Arc::new(ModelRegistry::new(temp_dir.path().to_str().unwrap()));
    let feature_store = Arc::new(FeatureStore::new());
    let compressor = Arc::new(TurboQuantCompressor::new());
    
    let inference_engine = InferenceEngine::new(
        model_registry.clone(),
        feature_store.clone(),
        feature_store.get_cache(),
        compressor,
    );
    
    let app = create_server(inference_engine, model_registry);
    
    // Test request
    let request_body = json!({
        "model_name": "test_model",
        "model_version": "1",
        "symbol": "AAPL",
        "timestamp_ns": chrono::Utc::now().timestamp_nanos(),
        "features": {
            "price": 150.0,
            "volume": 1000000.0,
            "rsi": 0.5
        }
    });
    
    let num_samples = 1000;
    let mut latencies = Vec::new();
    
    for _ in 0..num_samples {
        let request = Request::builder()
            .method(Method::POST)
            .uri("/models/test_model/predict")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
            .unwrap();
        
        let start = Instant::now();
        
        // Use oneshot to test handler directly
        let response = app.clone().oneshot(request).await.unwrap();
        
        let latency = start.elapsed();
        latencies.push(latency);
        
        assert_eq!(response.status(), StatusCode::OK);
        
        // Small delay between samples
        tokio::time::sleep(Duration::from_micros(10)).await;
    }
    
    // Calculate statistics
    latencies.sort();
    let avg_latency = latencies.iter().sum::<Duration>() / num_samples as u32;
    let p95_latency = latencies[(num_samples as f64 * 0.95) as usize];
    let p99_latency = latencies[(num_samples as f64 * 0.99) as usize];
    
    println!("HTTP Handler Latency:");
    println!("  Average: {:?}", avg_latency);
    println!("  P95: {:?}", p95_latency);
    println!("  P99: {:?}", p99_latency);
    
    // Validate latency requirement (includes HTTP processing)
    assert!(p95_latency.as_millis() < 1,
        "P95 latency {:?} > 1ms requirement", p95_latency);
}

#[tokio::test]
async fn test_concurrent_inference_throughput() {
    // Test concurrent inference throughput
    let temp_dir = TempDir::new().unwrap();
    let model_registry = Arc::new(ModelRegistry::new(temp_dir.path().to_str().unwrap()));
    let feature_store = Arc::new(FeatureStore::new());
    let compressor = Arc::new(TurboQuantCompressor::new());
    
    let inference_engine = InferenceEngine::new(
        model_registry.clone(),
        feature_store.clone(),
        feature_store.get_cache(),
        compressor,
    );
    
    let app = create_server(inference_engine, model_registry);
    
    // Test parameters
    let total_requests = 10_000;
    let concurrent_limit = 100;
    let semaphore = Arc::new(Semaphore::new(concurrent_limit));
    
    let request_body = json!({
        "model_name": "test_model",
        "model_version": "1",
        "symbol": "AAPL",
        "timestamp_ns": chrono::Utc::now().timestamp_nanos(),
        "features": {
            "price": 150.0,
            "volume": 1000000.0
        }
    });
    
    let start = Instant::now();
    let mut tasks = Vec::new();
    
    for i in 0..total_requests {
        let semaphore = Arc::clone(&semaphore);
        let app = app.clone();
        let request_body = request_body.clone();
        
        let task = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            
            // Vary the symbol slightly
            let mut body = request_body;
            body["symbol"] = json!(format!("SYMBOL_{}", i % 100));
            
            let request = Request::builder()
                .method(Method::POST)
                .uri("/models/test_model/predict")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap();
            
            let response = app.oneshot(request).await.unwrap();
            response.status() == StatusCode::OK
        });
        tasks.push(task);
    }
    
    // Wait for all requests
    let mut successful = 0;
    for task in tasks {
        if task.await.unwrap() {
            successful += 1;
        }
    }
    
    let duration = start.elapsed();
    let rps = total_requests as f64 / duration.as_secs_f64();
    
    println!("Concurrent Inference Throughput:");
    println!("  Total requests: {}", total_requests);
    println!("  Successful: {}", successful);
    println!("  Duration: {:?}", duration);
    println!("  RPS: {:.0}", rps);
    
    // Validate throughput requirement
    assert!(rps >= 10_000.0,
        "Throughput {:.0} < 10,000 RPS requirement", rps);
    
    assert_eq!(successful, total_requests,
        "Some requests failed: {}/{}", successful, total_requests);
}

#[tokio::test]
async fn test_feature_cache_performance() {
    // Test feature cache performance
    let temp_dir = TempDir::new().unwrap();
    let model_registry = Arc::new(ModelRegistry::new(temp_dir.path().to_str().unwrap()));
    let feature_store = Arc::new(FeatureStore::new());
    let compressor = Arc::new(TurboQuantCompressor::new());
    
    // Pre-populate feature cache
    let cache = feature_store.get_cache();
    for i in 0..1000 {
        cache.set(
            &format!("FEATURE_AAPL_{}", i),
            &(i as f64 * 1.5),
        ).await;
    }
    
    let inference_engine = InferenceEngine::new(
        model_registry.clone(),
        feature_store.clone(),
        cache,
        compressor,
    );
    
    // Test request with cached features
    let request = InferenceRequest {
        model_name: "test_model".to_string(),
        model_version: Some("1".to_string()),
        symbol: "AAPL".to_string(),
        timestamp_ns: chrono::Utc::now().timestamp_nanos(),
        features: None, // Will use cached features
    };
    
    // Measure cached inference performance
    let num_requests = 1000;
    let start = Instant::now();
    
    for _ in 0..num_requests {
        let _response = inference_engine.predict(request.clone()).await;
    }
    
    let duration = start.elapsed();
    let avg_cached_latency = duration / num_requests as u32;
    
    println!("Feature Cache Performance:");
    println!("  Cached requests: {}", num_requests);
    println!("  Total time: {:?}", duration);
    println!("  Avg cached latency: {:?}", avg_cached_latency);
    
    // Cached inference should be very fast
    assert!(avg_cached_latency.as_micros() < 100,
        "Cached inference latency {:?} > 100μs", avg_cached_latency);
}

#[tokio::test]
async fn test_turboquant_compression() {
    // Test TurboQuant compression performance
    let compressor = TurboQuantCompressor::new();
    
    // Create test context
    let context = vec![1.0; 1000]; // 1KB of floats
    
    let start = Instant::now();
    let compressed = compressor.compress(&context).unwrap();
    let compression_time = start.elapsed();
    
    let start = Instant::now();
    let decompressed = compressor.decompress(&compressed).unwrap();
    let decompression_time = start.elapsed();
    
    assert_eq!(context, decompressed);
    
    let compression_ratio = context.len() as f64 / compressed.len() as f64;
    
    println!("TurboQuant Compression:");
    println!("  Original size: {} bytes", context.len() * 8);
    println!("  Compressed size: {} bytes", compressed.len());
    println!("  Compression ratio: {:.2}x", compression_ratio);
    println!("  Compression time: {:?}", compression_time);
    println!("  Decompression time: {:?}", decompression_time);
    
    // Compression should be fast and effective
    assert!(compression_ratio > 2.0, "Compression ratio too low: {:.2}x", compression_ratio);
    assert!(compression_time.as_micros() < 100, "Compression too slow: {:?}", compression_time);
    assert!(decompression_time.as_micros() < 50, "Decompression too slow: {:?}", decompression_time);
}

#[tokio::test]
async fn test_memory_usage_scaling() {
    // Test memory usage with increasing inference load
    let temp_dir = TempDir::new().unwrap();
    let model_registry = Arc::new(ModelRegistry::new(temp_dir.path().to_str().unwrap()));
    let feature_store = Arc::new(FeatureStore::new());
    let compressor = Arc::new(TurboQuantCompressor::new());
    
    let inference_engine = InferenceEngine::new(
        model_registry.clone(),
        feature_store.clone(),
        feature_store.get_cache(),
        compressor,
    );
    
    // Measure memory before
    let memory_before = get_memory_usage();
    
    // Generate many inference requests
    let num_requests = 10_000;
    for i in 0..num_requests {
        let mut features = HashMap::new();
        features.insert("price".to_string(), 150.0 + (i as f64 * 0.01));
        features.insert("volume".to_string(), 1000000.0);
        
        let request = InferenceRequest {
            model_name: "test_model".to_string(),
            model_version: Some("1".to_string()),
            symbol: format!("SYMBOL_{}", i % 100),
            timestamp_ns: chrono::Utc::now().timestamp_nanos(),
            features: Some(features),
        };
        
        let _response = inference_engine.predict(request).await;
        
        if i % 1000 == 0 {
            tokio::task::yield_now().await;
        }
    }
    
    // Measure memory after
    let memory_after = get_memory_usage();
    let memory_per_request = (memory_after - memory_before) as f64 / num_requests as f64;
    
    println!("Memory Usage Scaling:");
    println!("  Requests: {}", num_requests);
    println!("  Memory before: {} KB", memory_before / 1024);
    println!("  Memory after: {} KB", memory_after / 1024);
    println!("  Memory per request: {:.2} bytes", memory_per_request);
    
    // Each request should use minimal memory (<1KB)
    assert!(memory_per_request < 1024.0,
        "Memory per request {:.2} > 1KB", memory_per_request);
}

#[cfg(unix)]
fn get_memory_usage() -> usize {
    use std::fs;
    let status = fs::read_to_string("/proc/self/status").unwrap();
    for line in status.lines() {
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            return parts[1].parse::<usize>().unwrap() * 1024; // Convert KB to bytes
        }
    }
    0
}

#[cfg(not(unix))]
fn get_memory_usage() -> usize {
    // Placeholder for non-Unix systems
    0
}
