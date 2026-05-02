use std::time::Duration;
use tokio::time::timeout;
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use tracing::{info, warn, error, debug};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;

use super::{LlmRequest, AgentResponse, LlmResult, LlmError, ResponseType};
use crate::observability::{AgentMetrics, StructuredLogger};

/// Ollama model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub size: u64,           // bytes
    pub digest: String,       // model checksum
    pub details: ModelDetails,
    pub status: ModelStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDetails {
    pub format: String,
    pub family: String,
    pub families: Option<Vec<String>>,
    pub parameter_size: String,
    pub quantization_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStatus {
    pub name: String,
    pub size: u64,
    pub digest: String,
    pub status: String,      // "ready", "loading", "error"
}

/// Ollama generation request
#[derive(Debug, Serialize)]
pub struct OllamaGenerateRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    pub options: Option<GenerationOptions>,
}

#[derive(Debug, Serialize)]
pub struct GenerationOptions {
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<i32>,
    pub num_predict: Option<i32>,
    pub repeat_penalty: Option<f64>,
    pub stop: Option<Vec<String>>,
}

/// Ollama generation response
#[derive(Debug, Deserialize)]
pub struct OllamaGenerateResponse {
    pub model: String,
    pub created_at: String,
    pub response: String,
    pub done: bool,
    pub total_duration: Option<u64>,
    pub load_duration: Option<u64>,
    pub prompt_eval_count: Option<u32>,
    pub prompt_eval_duration: Option<u64>,
    pub eval_count: Option<u32>,
    pub eval_duration: Option<u64>,
}

/// Ollama API client
pub struct OllamaClient {
    http_client: Client,
    base_url: String,
    default_timeout: Duration,
    max_retries: u32,
    metrics: Option<std::sync::Arc<AgentMetrics>>,
    logger: Option<std::sync::Arc<StructuredLogger>>,
    available_models: HashMap<String, OllamaModel>,
}

impl OllamaClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            http_client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .expect("Failed to build HTTP client for Ollama"),
            base_url: base_url.trim_end_matches('/').to_string(),
            default_timeout: Duration::from_secs(30),
            max_retries: 3,
            metrics: None,
            logger: None,
            available_models: HashMap::new(),
        }
    }
    
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }
    
    pub fn with_metrics(mut self, metrics: std::sync::Arc<AgentMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }
    
    pub fn with_logger(mut self, logger: std::sync::Arc<StructuredLogger>) -> Self {
        self.logger = Some(logger);
        self
    }
    
    /// Check if Ollama server is available
    pub async fn health_check(&self) -> LlmResult<bool> {
        let url = format!("{}/api/tags", self.base_url);
        
        match timeout(Duration::from_secs(5), self.http_client.get(&url).send()).await {
            Ok(Ok(response)) => Ok(response.status().is_success()),
            Ok(Err(e)) => {
                error!("Ollama health check failed: {}", e);
                Err(LlmError::NetworkError(format!("Health check failed: {}", e)))
            },
            Err(_) => {
                warn!("Ollama health check timed out");
                Err(LlmError::Timeout("Health check timed out".to_string()))
            }
        }
    }
    
    /// List available models
    pub async fn list_models(&mut self) -> LlmResult<Vec<OllamaModel>> {
        let url = format!("{}/api/tags", self.base_url);
        
        let response = self.retry_request(|| async {
            self.http_client.get(&url).send().await
        }).await?;
        
        if !response.status().is_success() {
            return Err(LlmError::ApiError(
                format!("Failed to list models: {}", response.status())
            ));
        }
        
        let body: Value = response.json().await
            .map_err(|e| LlmError::ParseError(format!("Failed to parse model list: {}", e)))?;
        
        let models: Vec<OllamaModel> = body.get("models")
            .and_then(|m| m.as_array())
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|m| serde_json::from_value(m.clone()).ok())
            .collect();
        
        // Update available models cache
        self.available_models.clear();
        for model in &models {
            self.available_models.insert(model.name.clone(), model.clone());
        }
        
        info!("Ollama: {} models available", models.len());
        Ok(models)
    }
    
    /// Pull a model from Ollama registry
    pub async fn pull_model(&self, model_name: &str) -> LlmResult<()> {
        info!("Ollama: Pulling model {}", model_name);
        
        let url = format!("{}/api/pull", self.base_url);
        let request_body = json!({
            "name": model_name
        });
        
        let response = self.http_client.post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| LlmError::NetworkError(format!("Failed to pull model: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(LlmError::ApiError(
                format!("Failed to pull model {}: {}", model_name, response.status())
            ));
        }
        
        // Note: Model pulling is asynchronous, we should monitor status
        info!("Ollama: Model pull initiated for {}", model_name);
        Ok(())
    }
    
    /// Check if a model is available and ready
    pub fn is_model_ready(&self, model_name: &str) -> bool {
        self.available_models.contains_key(model_name)
    }
    
    /// Generate text using Ollama
    pub async fn generate(&self, request: &LlmRequest) -> LlmResult<AgentResponse> {
        let start_time = std::time::Instant::now();
        
        // Check if model is available
        if !self.is_model_ready(&request.model) {
            return Err(LlmError::ModelNotFound(format!("Model {} not available", request.model)));
        }
        
        let ollama_request = OllamaGenerateRequest {
            model: request.model.clone(),
            prompt: request.prompt.clone(),
            stream: false,
            options: Some(GenerationOptions {
                temperature: Some(0.7),
                top_p: Some(0.9),
                top_k: Some(40),
                num_predict: Some(request.max_tokens as i32),
                repeat_penalty: Some(1.1),
                stop: None,
            }),
        };
        
        let url = format!("{}/api/generate", self.base_url);
        
        let response = self.retry_request(|| async {
            self.http_client.post(&url)
                .json(&ollama_request)
                .send()
                .await
        }).await?;
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::ApiError(
                format!("Ollama generation failed: {} - {}", status, error_text)
            ));
        }
        
        let ollama_response: OllamaGenerateResponse = response.json().await
            .map_err(|e| LlmError::ParseError(format!("Failed to parse Ollama response: {}", e)))?;
        
        let duration = start_time.elapsed();
        
        // Record metrics
        if let Some(metrics) = &self.metrics {
            metrics.llm_request_duration.observe(duration.as_secs_f64());
            metrics.llm_requests_total.inc();
        }
        
        // Log the interaction
        if let Some(logger) = &self.logger {
            let metadata = HashMap::from([
                ("model".to_string(), request.model.clone()),
                ("prompt_length".to_string(), request.prompt.len().to_string()),
                ("response_length".to_string(), ollama_response.response.len().to_string()),
                ("duration_ms".to_string(), duration.as_millis().to_string()),
                ("eval_count".to_string(), ollama_response.eval_count.unwrap_or(0).to_string()),
            ]);
            
            logger.log(
                request.request_id,
                "ollama_client",
                crate::observability::LogLevel::Info,
                "Generation completed",
                metadata
            ).await;
        }
        
        let agent_response = AgentResponse {
            request_id: request.request_id,
            agent_id: request.metadata.get("agent_id").cloned().unwrap_or_default(),
            response_type: ResponseType::Analysis,
            content: ollama_response.response.clone(),
            confidence: 0.8, // Ollama doesn't provide confidence, use default
            timestamp: Utc::now(),
            metadata: {
                let mut meta = request.metadata.clone();
                meta.insert("provider".to_string(), "ollama".to_string());
                meta.insert("model".to_string(), request.model.clone());
                meta.insert("tokens_used".to_string(), ollama_response.eval_count.unwrap_or(0).to_string());
                meta.insert("duration_ms".to_string(), duration.as_millis().to_string());
                meta
            },
        };
        
        debug!("Ollama generation completed in {:?} for model {}", duration, request.model);
        Ok(agent_response)
    }
    
    /// Get model information
    pub async fn get_model_info(&self, model_name: &str) -> Option<&OllamaModel> {
        self.available_models.get(model_name)
    }
    
    /// Get all available models
    pub fn get_available_models(&self) -> Vec<&OllamaModel> {
        self.available_models.values().collect()
    }
    
    /// Retry request with exponential backoff
    async fn retry_request<F, Fut>(&self, operation: F) -> LlmResult<reqwest::Response>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<reqwest::Response, reqwest::Error>> + Send,
    {
        let mut last_error = None;
        
        for attempt in 1..=self.max_retries {
            match timeout(self.default_timeout, operation()).await {
                Ok(Ok(response)) => return Ok(response),
                Ok(Err(e)) => {
                    warn!("Ollama request attempt {} failed: {}", attempt, e);
                    last_error = Some(LlmError::NetworkError(format!("Request failed: {}", e)));
                },
                Err(_) => {
                    warn!("Ollama request attempt {} timed out", attempt);
                    last_error = Some(LlmError::Timeout("Request timed out".to_string()));
                }
            }
            
            // Exponential backoff
            if attempt < self.max_retries {
                let backoff = Duration::from_millis(100 * (1 << (attempt - 1)));
                tokio::time::sleep(backoff).await;
            }
        }
        
        Err(last_error.unwrap_or_else(|| LlmError::NetworkError("All retry attempts failed".to_string())))
    }
    
    /// Refresh model list
    pub async fn refresh_models(&mut self) -> LlmResult<()> {
        self.list_models().await?;
        Ok(())
    }
    
    /// Estimate model memory usage (rough approximation)
    pub fn estimate_memory_usage(&self, model_name: &str) -> Option<u64> {
        if let Some(model) = self.available_models.get(model_name) {
            // Rough estimate: model size in bytes + 2x for runtime overhead
            Some(model.size * 3)
        } else {
            None
        }
    }
    
    /// Get recommended models for trading tasks
    pub fn get_trading_models(&self) -> Vec<&OllamaModel> {
        self.available_models.values()
            .filter(|model| {
                // Filter for models suitable for trading analysis
                model.details.family.contains("llama") ||
                model.details.family.contains("codellama") ||
                model.details.family.contains("deepseek") ||
                model.details.family.contains("qwen")
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ollama_client_creation() {
        let client = OllamaClient::new("http://localhost:11434");
        assert_eq!(client.base_url, "http://localhost:11434");
    }
    
    #[tokio::test]
    async fn test_model_availability() {
        let mut client = OllamaClient::new("http://localhost:11434");
        
        // Initially no models
        assert!(!client.is_model_ready("llama2"));
        
        // Mock some models
        client.available_models.insert("llama2".to_string(), OllamaModel {
            name: "llama2".to_string(),
            size: 1000000,
            digest: "abc123".to_string(),
            details: ModelDetails {
                format: "gguf".to_string(),
                family: "llama".to_string(),
                families: Some(vec!["llama".to_string()]),
                parameter_size: "7B".to_string(),
                quantization_level: "Q4_0".to_string(),
            },
            status: ModelStatus {
                name: "llama2".to_string(),
                size: 1000000,
                digest: "abc123".to_string(),
                status: "ready".to_string(),
            },
        });
        
        assert!(client.is_model_ready("llama2"));
        assert!(!client.is_model_ready("nonexistent"));
    }
    
    #[tokio::test]
    async fn test_memory_estimation() {
        let mut client = OllamaClient::new("http://localhost:11434");
        
        client.available_models.insert("test".to_string(), OllamaModel {
            name: "test".to_string(),
            size: 1000000, // 1MB
            digest: "abc123".to_string(),
            details: ModelDetails {
                format: "gguf".to_string(),
                family: "test".to_string(),
                families: None,
                parameter_size: "1B".to_string(),
                quantization_level: "Q4_0".to_string(),
            },
            status: ModelStatus {
                name: "test".to_string(),
                size: 1000000,
                digest: "abc123".to_string(),
                status: "ready".to_string(),
            },
        });
        
        let estimated = client.estimate_memory_usage("test");
        assert_eq!(estimated, Some(3000000)); // 3x model size
        assert_eq!(client.estimate_memory_usage("nonexistent"), None);
    }
    
    #[test]
    fn test_generation_options() {
        let options = GenerationOptions {
            temperature: Some(0.7),
            top_p: Some(0.9),
            top_k: Some(40),
            num_predict: Some(100),
            repeat_penalty: Some(1.1),
            stop: None,
        };
        
        // Test serialization
        let json = serde_json::to_value(options).unwrap();
        assert_eq!(json["temperature"], 0.7);
        assert_eq!(json["num_predict"], 100);
    }
}
