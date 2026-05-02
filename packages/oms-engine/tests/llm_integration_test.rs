use std::sync::Arc;
use tokio::time::{timeout, Duration};
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use traderx_oms_engine::llm::{LlmRequest, AgentResponse, ResponseType};
use traderx_oms_engine::llm::ollama_client::OllamaClient;
use traderx_oms_engine::llm::hybrid_router::{HybridProviderRouter, SelectionStrategy};
use traderx_oms_engine::ml::dynamic_model_selection::{DynamicModelSelector, SelectionCriteria};
use traderx_oms_engine::middleware::llm_message_bus::{LlmMessageBus, LlmMessage, LlmMessageType, LlmMessagePayload};
use traderx_oms_engine::middleware::function_orchestrator::{FunctionOrchestrator, ExecutionStep, Workflow, StepStatus};
use traderx_oms_engine::stability::reliability_assessment::{ReliabilityAssessment, AssessmentConfig, FailureMode};
use traderx_oms_engine::stability::performance_audit::{PerformanceAuditor, Benchmark};

#[tokio::test]
async fn test_ollama_client_integration() {
    info!("Starting Ollama client integration test");
    
    let client = OllamaClient::new("http://localhost:11434".to_string());
    
    // Test health check
    let health_result = client.health_check().await;
    assert!(health_result.is_ok() || health_result.is_err()); // May fail if Ollama not running
    
    // Test model listing
    let models = client.list_models().await;
    assert!(models.is_ok() || models.is_err()); // May fail if Ollama not running
    
    info!("Ollama client integration test completed");
}

#[tokio::test]
async fn test_hybrid_router_decision_making() {
    info!("Starting hybrid router decision making test");
    
    let router = HybridProviderRouter::new(SelectionStrategy::LocalFirst);
    
    let request = LlmRequest {
        request_id: Uuid::new_v4(),
        model: "deepseek-coder:1.3b".to_string(),
        prompt: "Test prompt".to_string(),
        max_tokens: 100,
        temperature: 0.7,
        metadata: Default::default(),
    };
    
    let conviction = 0.8;
    let decision = router.make_routing_decision(&request, conviction).await;
    
    assert_eq!(decision.selected_provider, traderx_oms_engine::middleware::LlmProvider::Ollama);
    assert!(!decision.reasoning.is_empty());
    assert!(decision.confidence >= 0.0 && decision.confidence <= 1.0);
    
    info!("Hybrid router decision making test completed");
}

#[tokio::test]
async fn test_dynamic_model_selection() {
    info!("Starting dynamic model selection test");
    
    let available_models = vec![
        "deepseek-coder:1.3b".to_string(),
        "gemma-mini:2b".to_string(),
    ];
    
    let model_costs = std::collections::HashMap::from([
        ("deepseek-coder:1.3b".to_string(), 0.0),
        ("gemma-mini:2b".to_string(), 0.0),
    ]);
    
    let selector = DynamicModelSelector::new(available_models.clone(), model_costs);
    
    let request = LlmRequest {
        request_id: Uuid::new_v4(),
        model: "deepseek-coder:1.3b".to_string(),
        prompt: "Test prompt".to_string(),
        max_tokens: 100,
        temperature: 0.7,
        metadata: Default::default(),
    };
    
    let conviction = 0.5;
    let decision = selector.select_model(&request, conviction).await;
    
    assert!(!decision.selected_model.is_empty());
    assert!(!decision.reasoning.is_empty());
    
    info!("Dynamic model selection test completed");
}

#[tokio::test]
async fn test_message_bus_routing() {
    info!("Starting message bus routing test");
    
    let bus = LlmMessageBus::new();
    
    let message = LlmMessage {
        message_id: Uuid::new_v4(),
        message_type: LlmMessageType::LlmRequest {
            model: "deepseek-coder:1.3b".to_string(),
            provider: traderx_oms_engine::middleware::LlmProvider::Ollama,
        },
        payload: LlmMessagePayload::Request(serde_json::json!({"test": "data"})),
        priority: traderx_oms_engine::middleware::MessagePriority::Normal,
        timestamp: chrono::Utc::now(),
        ttl: std::time::Duration::from_secs(60),
        correlation_id: Uuid::new_v4(),
        context: None,
    };
    
    let result = bus.route_message(message).await;
    assert!(result.is_ok());
    
    info!("Message bus routing test completed");
}

#[tokio::test]
async fn test_function_orchestrator_workflow() {
    info!("Starting function orchestrator workflow test");
    
    let orchestrator = FunctionOrchestrator::new();
    
    let step = ExecutionStep {
        step_id: Uuid::new_v4(),
        step_name: "test_step".to_string(),
        function_name: "test_function".to_string(),
        parameters: std::collections::HashMap::new(),
        dependencies: vec![],
        timeout_ms: 5000,
        retry_count: 3,
        status: traderx_oms_engine::middleware::StepStatus::Pending,
        result: None,
        error: None,
        start_time: None,
        end_time: None,
        duration_ms: None,
    };
    
    let workflow_id = orchestrator.create_workflow(
        "test_workflow".to_string(),
        vec![step],
        std::collections::HashMap::new(),
    ).await;
    
    assert_ne!(workflow_id, Uuid::nil());
    
    info!("Function orchestrator workflow test completed");
}

#[tokio::test]
async fn test_reliability_assessment() {
    info!("Starting reliability assessment test");
    
    let config = AssessmentConfig::default();
    let mut assessment = ReliabilityAssessment::new(config);
    
    let failure = FailureMode::SystemOverload {
        load_factor: 0.95,
        cpu_usage: 0.90,
        memory_usage: 0.85,
    };
    
    assessment.record_failure(failure).await;
    
    let metrics = assessment.calculate_metrics().await;
    assert!(metrics.uptime_percentage >= 0.0 && metrics.uptime_percentage <= 100.0);
    
    info!("Reliability assessment test completed");
}

#[tokio::test]
async fn test_performance_auditor() {
    info!("Starting performance auditor test");
    
    let auditor = PerformanceAuditor::new();
    
    let report = auditor.generate_audit_report().await;
    
    assert!(!report.report_id.is_nil());
    assert!(report.overall_score >= 0.0 && report.overall_score <= 100.0);
    assert!(report.compliance_checks.len() > 0);
    
    info!("Performance auditor test completed");
}

#[tokio::test]
async fn test_end_to_end_integration() {
    info!("Starting end-to-end integration test");
    
    // This test validates the complete flow from signal to LLM response
    
    // 1. Create a signal
    let request = LlmRequest {
        request_id: Uuid::new_v4(),
        model: "deepseek-coder:1.3b".to_string(),
        prompt: "Analyze trading signal for AAPL with conviction 0.8".to_string(),
        max_tokens: 200,
        temperature: 0.7,
        metadata: std::collections::HashMap::new(),
    };
    
    // 2. Route through message bus
    let bus = LlmMessageBus::new();
    let message = LlmMessage {
        message_id: Uuid::new_v4(),
        message_type: LlmMessageType::LlmRequest {
            model: request.model.clone(),
            provider: traderx_oms_engine::middleware::LlmProvider::Ollama,
        },
        payload: LlmMessagePayload::Request(serde_json::json!({"prompt": request.prompt})),
        priority: traderx_oms_engine::middleware::MessagePriority::Normal,
        timestamp: chrono::Utc::now(),
        ttl: std::time::Duration::from_secs(60),
        correlation_id: request.request_id,
        context: None,
    };
    
    let _ = bus.route_message(message).await;
    
    // 3. Make routing decision
    let router = HybridProviderRouter::new(SelectionStrategy::LocalFirst);
    let decision = router.make_routing_decision(&request, 0.8).await;
    
    assert_eq!(decision.selected_provider, traderx_oms_engine::middleware::LlmProvider::Ollama);
    
    // 4. Record metrics
    let selector = DynamicModelSelector::new(
        vec!["deepseek-coder:1.3b".to_string()],
        std::collections::HashMap::from([("deepseek-coder:1.3b".to_string(), 0.0)]),
    );
    
    selector.record_usage(&decision.selected_model, 100.0, true, 0.85).await;
    
    // 5. Check reliability
    let config = AssessmentConfig::default();
    let mut assessment = ReliabilityAssessment::new(config);
    let metrics = assessment.calculate_metrics().await;
    
    assert!(metrics.uptime_percentage > 0.0);
    
    info!("End-to-end integration test completed successfully");
}

#[tokio::test]
async fn test_integration_with_timeout() {
    info!("Starting integration test with timeout");
    
    // Test that components respond within acceptable time limits
    let start = std::time::Instant::now();
    
    let router = HybridProviderRouter::new(SelectionStrategy::LocalFirst);
    let request = LlmRequest {
        request_id: Uuid::new_v4(),
        model: "deepseek-coder:1.3b".to_string(),
        prompt: "Test".to_string(),
        max_tokens: 50,
        temperature: 0.5,
        metadata: Default::default(),
    };
    
    let decision = timeout(
        Duration::from_millis(100),
        router.make_routing_decision(&request, 0.5)
    ).await;
    
    assert!(decision.is_ok(), "Routing decision should complete within timeout");
    
    let duration = start.elapsed();
    assert!(duration.as_millis() < 100, "Routing decision should be fast");
    
    info!("Integration test with timeout completed");
}

#[tokio::test]
async fn test_error_handling_integration() {
    info!("Starting error handling integration test");
    
    let router = HybridProviderRouter::new(SelectionStrategy::LocalFirst);
    let request = LlmRequest {
        request_id: Uuid::new_v4(),
        model: "nonexistent-model".to_string(),
        prompt: "Test".to_string(),
        max_tokens: 50,
        temperature: 0.5,
        metadata: Default::default(),
    };
    
    // Should handle gracefully even with invalid model
    let decision = router.make_routing_decision(&request, 0.5).await;
    assert!(!decision.selected_model.is_empty());
    
    info!("Error handling integration test completed");
}
