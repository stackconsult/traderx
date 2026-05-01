use oms_engine::llm::{
    AgentSignal, LlmEngine, LlmClient, PromptEngine, ContextManager, AgentRouter,
};
use oms_engine::ml::{
    MlEngine, MarketData, FeatureExtractor, InferenceEngine,
};
use oms_engine::neural::{
    NeuralEngine, runtime::OnnxRuntime, processor::SignalProcessor,
};
use oms_engine::middleware::{
    message_bus::MessageBus, AgentOrchestrator, AgentMessage,
};
use oms_engine::observability::{
    AgentMetrics, HealthMonitor, HealthStatus, StructuredLogger,
};
use std::sync::Arc;

#[tokio::test]
async fn test_llm_prompt_engine_rendering() {
    let engine = PromptEngine::new();
    let mut req = oms_engine::llm::LlmRequest::default();
    req.metadata.insert("template".to_string(), "signal_analysis".to_string());
    req.metadata.insert("symbol".to_string(), "AAPL".to_string());
    req.metadata.insert("direction".to_string(), "long".to_string());
    req.metadata.insert("conviction".to_string(), "0.75".to_string());
    req.metadata.insert("max_notional".to_string(), "10000.00".to_string());
    
    let result = engine.render(&req).await;
    assert!(result.is_ok());
    let rendered = result.unwrap();
    assert!(rendered.contains("AAPL"));
    assert!(rendered.contains("long"));
}

#[tokio::test]
async fn test_llm_context_manager_store_and_retrieve() {
    let manager = ContextManager::with_defaults();
    let agent_id = "test_agent";
    
    manager.store_interaction(agent_id, "What is the market outlook?", "Bullish on tech.").await.unwrap();
    
    let history = manager.get_conversation(agent_id).await;
    assert_eq!(history.len(), 2);
    
    let enriched = manager.enrich("Current signal?", agent_id).await.unwrap();
    assert!(enriched.contains("Bullish on tech"));
}

#[tokio::test]
async fn test_llm_agent_router_routing() {
    let router = AgentRouter::new();
    let signal = AgentSignal {
        agent_id: "test".to_string(),
        symbol: "AAPL".to_string(),
        direction: "long".to_string(),
        conviction: 0.95,
        max_notional_usd: 5000.0,
        ttl_ms: 5000,
        meta: serde_json::json!({"test": true}),
    };
    
    let request = router.route_signal(&signal).await.unwrap();
    assert_eq!(request.model, "gpt-4");
    assert_eq!(request.temperature, 0.2);
}

#[tokio::test]
async fn test_ml_feature_extraction() {
    let extractor = FeatureExtractor::new();
    let data = vec![
        MarketData {
            symbol: "AAPL".to_string(),
            timestamp: chrono::Utc::now(),
            open: 150.0,
            high: 155.0,
            low: 149.0,
            close: 153.0,
            volume: 1000000,
        },
    ];
    
    let features = extractor.extract_batch(&data).await.unwrap();
    assert_eq!(features.len(), 1);
    assert!(features[0].features.len() > 5);
}

#[tokio::test]
async fn test_ml_inference_mock() {
    let engine = InferenceEngine::new("v1.0");
    let features = vec![oms_engine::ml::FeatureVector {
        symbol: "AAPL".to_string(),
        timestamp: chrono::Utc::now(),
        features: vec![150.0, 155.0, 149.0, 153.0, 1000000.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        feature_names: vec!["open".to_string(), "high".to_string(), "low".to_string(), "close".to_string(), "volume".to_string()],
    }];
    
    let predictions = engine.predict_batch(&features).await.unwrap();
    assert_eq!(predictions.len(), 1);
    assert!(predictions[0].confidence > 0.0);
    assert!(predictions[0].probabilities.contains_key("buy"));
}

#[tokio::test]
async fn test_neural_signal_to_tensor() {
    let processor = SignalProcessor::new();
    let signal = AgentSignal {
        agent_id: "neural_test".to_string(),
        symbol: "TSLA".to_string(),
        direction: "long".to_string(),
        conviction: 0.85,
        max_notional_usd: 10000.0,
        ttl_ms: 5000,
        meta: serde_json::json!({"test": true}),
    };
    
    let tensor = processor.signal_to_tensor(&signal).await.unwrap();
    assert_eq!(tensor.shape, vec![1, 10]);
    assert_eq!(tensor.tensor_data.len(), 10);
}

#[tokio::test]
async fn test_neural_onnx_runtime_infer() {
    let runtime = OnnxRuntime::new();
    let mut runtime = runtime;
    runtime.initialize(vec![1, 10], vec![1, 3]).unwrap();
    
    let input = oms_engine::neural::NeuralInput {
        input_id: uuid::Uuid::new_v4(),
        tensor_data: vec![0.5; 10],
        shape: vec![1, 10],
        dtype: oms_engine::neural::TensorDtype::F32,
        metadata: serde_json::json!({}),
    };
    
    let output = runtime.infer(input).await.unwrap();
    assert_eq!(output.shape, vec![1, 3]);
    assert!(output.confidence > 0.0);
}

#[tokio::test]
async fn test_message_bus_routing() {
    let bus = MessageBus::with_defaults();
    let orchestrator = AgentOrchestrator::new(
        bus.get_llm_sender(),
        bus.get_ml_sender(),
        bus.get_neural_sender(),
    );
    
    let signal = AgentSignal {
        agent_id: "test".to_string(),
        symbol: "BTC".to_string(),
        direction: "long".to_string(),
        conviction: 0.6,
        max_notional_usd: 5000.0,
        ttl_ms: 5000,
        meta: serde_json::json!({}),
    };
    
    let msg = AgentMessage::Signal {
        signal,
        timestamp: chrono::Utc::now(),
    };
    
    let result = orchestrator.route(msg).await;
    assert!(result.is_ok());
    
    let count = orchestrator.get_message_count().await;
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_agent_metrics_initialization() {
    let metrics = AgentMetrics::new();
    assert!(metrics.is_ok());
    
    let m = metrics.unwrap();
    m.llm_requests_total.inc();
    assert_eq!(m.llm_requests_total.get() as u64, 1);
}

#[tokio::test]
async fn test_health_monitor() {
    let monitor = HealthMonitor::new();
    
    monitor.update("llm", HealthStatus::Healthy, 10, "All good").await;
    monitor.update("ml", HealthStatus::Degraded, 50, "Slow inference").await;
    
    let llm_status = monitor.get_status("llm").await.unwrap();
    assert!(matches!(llm_status.status, HealthStatus::Healthy));
    
    let all = monitor.get_all_statuses().await;
    assert_eq!(all.len(), 2);
    
    assert!(!monitor.is_healthy().await);
}

#[tokio::test]
async fn test_structured_logger() {
    let logger = StructuredLogger::new();
    let correlation_id = uuid::Uuid::new_v4();
    let mut meta = std::collections::HashMap::new();
    meta.insert("test".to_string(), "value".to_string());
    
    logger.log(correlation_id, "test_component", oms_engine::observability::LogLevel::Info, "Test message", meta).await;
    
    let trace = logger.get_trace(correlation_id).await;
    assert_eq!(trace.len(), 1);
    assert_eq!(trace[0].component, "test_component");
}

#[tokio::test]
async fn test_end_to_end_agent_orchestra() {
    let llm_client = Arc::new(LlmClient::new());
    let prompt_engine = Arc::new(PromptEngine::new());
    let context_manager = Arc::new(ContextManager::with_defaults());
    let router = Arc::new(AgentRouter::new());
    
    let (llm_engine, _rx) = LlmEngine::new(
        llm_client.clone(),
        prompt_engine,
        context_manager,
        router,
    );
    
    let signal = AgentSignal {
        agent_id: "e2e_test".to_string(),
        symbol: "ETH".to_string(),
        direction: "long".to_string(),
        conviction: 0.7,
        max_notional_usd: 2000.0,
        ttl_ms: 3000,
        meta: serde_json::json!({"test": true}),
    };
    
    let result = llm_engine.process_signal(signal).await;
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert!(!response.content.is_empty());
}
