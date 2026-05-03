use std::time::Duration;
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use tracing::{info, warn, error};
use crate::observability::{AgentMetrics, StructuredLogger};
use super::{LlmRequest, AgentResponse, LlmResult, LlmError, ResponseType};
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;
use std::collections::HashMap;

pub struct LlmClient {
    http_client: Client,
    openai_api_key: Option<String>,
    anthropic_api_key: Option<String>,
    base_url_openai: String,
    base_url_anthropic: String,
    max_retries: u32,
    metrics: Option<Arc<AgentMetrics>>,
    logger: Option<Arc<StructuredLogger>>,
}

impl LlmClient {
    pub fn new() -> Self {
        Self {
            http_client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client"),
            openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
            anthropic_api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
            base_url_openai: "https://api.openai.com/v1".to_string(),
            base_url_anthropic: "https://api.anthropic.com/v1".to_string(),
            max_retries: 3,
            metrics: None,
            logger: None,
        }
    }

    pub fn with_metrics(mut self, metrics: Arc<AgentMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    pub fn with_logger(mut self, logger: Arc<StructuredLogger>) -> Self {
        self.logger = Some(logger);
        self
    }

    pub async fn complete(&self, request: LlmRequest) -> LlmResult<AgentResponse> {
        let start = std::time::Instant::now();
        let correlation_id = Uuid::new_v4();

        if let Some(logger) = &self.logger {
            let mut meta = HashMap::new();
            meta.insert("model".to_string(), request.model.clone());
            meta.insert("max_tokens".to_string(), request.max_tokens.to_string());
            logger.log(correlation_id, "llm_client", crate::observability::LogLevel::Info, "LLM request started", meta).await;
        }

        let result = self.execute_with_retry(request).await;

        let latency = start.elapsed();
        if let Some(metrics) = &self.metrics {
            metrics.llm_requests_total.inc();
            metrics.llm_request_duration.observe(latency.as_secs_f64());
        }

        if let Some(logger) = &self.logger {
            let mut meta = HashMap::new();
            meta.insert("latency_ms".to_string(), latency.as_millis().to_string());
            meta.insert("success".to_string(), result.is_ok().to_string());
            logger.log(correlation_id, "llm_client", crate::observability::LogLevel::Info, "LLM request completed", meta).await;
        }

        result
    }

    async fn execute_with_retry(&self, request: LlmRequest) -> LlmResult<AgentResponse> {
        let mut last_error = None;
        
        for attempt in 0..self.max_retries {
            match self.execute_once(&request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e.clone());
                    if matches!(e, LlmError::RateLimitExceeded) {
                        let delay = Duration::from_secs(2u64.pow(attempt + 1));
                        warn!(attempt = attempt + 1, "Rate limited, backing off for {:?}", delay);
                        tokio::time::sleep(delay).await;
                    } else {
                        warn!(attempt = attempt + 1, error = %e, "LLM request failed, retrying");
                        tokio::time::sleep(Duration::from_millis(500 * (attempt as u64 + 1))).await;
                    }
                }
            }
        }

        error!(retries = self.max_retries, "All LLM retries exhausted");
        Err(last_error.unwrap_or_else(|| LlmError::RequestFailed("Unknown error".to_string())))
    }

    async fn execute_once(&self, request: &LlmRequest) -> LlmResult<AgentResponse> {
        if let Some(key) = &self.openai_api_key {
            self.call_openai(request, key).await
        } else if let Some(key) = &self.anthropic_api_key {
            self.call_anthropic(request, key).await
        } else {
            self.mock_response(request).await
        }
    }

    async fn call_openai(&self, request: &LlmRequest, api_key: &str) -> LlmResult<AgentResponse> {
        let url = format!("{}/chat/completions", self.base_url_openai);
        let body = json!({
            "model": request.model,
            "messages": [{"role": "user", "content": request.prompt}],
            "max_tokens": request.max_tokens,
            "temperature": request.temperature,
        });

        let response = self.http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::RequestFailed(e.to_string()))?;

        match response.status() {
            StatusCode::OK => {
                let data: Value = response.json().await
                    .map_err(|e| LlmError::InvalidResponse(e.to_string()))?;
                
                let content = data["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                
                let tokens_used = data["usage"]["total_tokens"].as_u64().unwrap_or(0);
                if let Some(metrics) = &self.metrics {
                    metrics.llm_tokens_used.inc_by(tokens_used as f64);
                }
                
                Ok(AgentResponse {
                    request_id: request.request_id,
                    agent_id: request.metadata.get("agent_id").cloned().unwrap_or_default(),
                    response_type: ResponseType::Analysis,
                    content,
                    confidence: 0.85,
                    timestamp: Utc::now(),
                    metadata: request.metadata.clone(),
                })
            }
            StatusCode::TOO_MANY_REQUESTS => Err(LlmError::RateLimitExceeded),
            status => Err(LlmError::RequestFailed(format!("HTTP {}: {}", status, status.canonical_reason().unwrap_or("Unknown")))),
        }
    }

    async fn call_anthropic(&self, request: &LlmRequest, api_key: &str) -> LlmResult<AgentResponse> {
        let url = format!("{}/messages", self.base_url_anthropic);
        let body = json!({
            "model": "claude-3-opus-20240229",
            "max_tokens": request.max_tokens,
            "messages": [{"role": "user", "content": request.prompt}],
            "temperature": request.temperature,
        });

        let response = self.http_client
            .post(&url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::RequestFailed(e.to_string()))?;

        match response.status() {
            StatusCode::OK => {
                let data: Value = response.json().await
                    .map_err(|e| LlmError::InvalidResponse(e.to_string()))?;
                
                let content = data["content"][0]["text"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                
                Ok(AgentResponse {
                    request_id: request.request_id,
                    agent_id: request.metadata.get("agent_id").cloned().unwrap_or_default(),
                    response_type: ResponseType::Analysis,
                    content,
                    confidence: 0.88,
                    timestamp: Utc::now(),
                    metadata: request.metadata.clone(),
                })
            }
            StatusCode::TOO_MANY_REQUESTS => Err(LlmError::RateLimitExceeded),
            status => Err(LlmError::RequestFailed(format!("HTTP {}: {}", status, status.canonical_reason().unwrap_or("Unknown")))),
        }
    }

    async fn mock_response(&self, request: &LlmRequest) -> LlmResult<AgentResponse> {
        info!("No API keys configured, returning mock LLM response");
        
        let content = format!(
            "Mock analysis for: {}. Based on the signal, the agent suggests careful evaluation. \
             No real LLM was called — this is a fallback response when API keys are not configured.",
            request.prompt.chars().take(50).collect::<String>()
        );
        
        Ok(AgentResponse {
            request_id: request.request_id,
            agent_id: request.metadata.get("agent_id").cloned().unwrap_or_default(),
            response_type: ResponseType::Analysis,
            content,
            confidence: 0.5,
            timestamp: Utc::now(),
            metadata: {
                let mut m = request.metadata.clone();
                m.insert("mock".to_string(), "true".to_string());
                m
            },
        })
    }
}
