use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use thiserror::Error;
use chrono::{DateTime, Utc};

pub mod self_healing;
pub mod inference;
pub mod features;
pub mod dynamic_model_selection;

pub use features::FeatureExtractor;
pub use inference::InferenceEngine;
pub use self_healing::{
    SelfHealingModel, HealthMonitor, AutoTuner, FallbackManager,
    PerformanceMetrics, PerformanceThresholds, HealthStatus, AnomalyType, OptimizationStrategy
};
pub use dynamic_model_selection::{
    DynamicModelSelector, ModelPerformanceMetrics, SelectionCriteria, ConvictionConfig,
    ModelSelectionDecision
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureVector {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub features: Vec<f64>,
    pub feature_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    pub prediction_id: Uuid,
    pub symbol: String,
    pub prediction_type: PredictionType,
    pub confidence: f64,
    pub probabilities: HashMap<String, f64>,
    pub timestamp: DateTime<Utc>,
    pub model_version: String,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PredictionType {
    PriceDirection,
    Volatility,
    TrendStrength,
    RiskScore,
    Custom(String),
}

#[derive(Debug, Error)]
pub enum MlError {
    #[error("Feature extraction failed: {0}")]
    FeatureExtraction(String),
    #[error("Model inference failed: {0}")]
    Inference(String),
    #[error("Invalid input data: {0}")]
    InvalidInput(String),
    #[error("Model not loaded: {0}")]
    ModelNotLoaded(String),
    #[error("Cache miss: {0}")]
    CacheMiss(String),
}

pub type MlResult<T> = Result<T, MlError>;

pub struct MlEngine {
    feature_extractor: Arc<features::FeatureExtractor>,
    inference_engine: Arc<inference::InferenceEngine>,
    predictions: Arc<RwLock<Vec<Prediction>>>,
}

impl MlEngine {
    pub fn new(
        feature_extractor: Arc<features::FeatureExtractor>,
        inference_engine: Arc<inference::InferenceEngine>,
    ) -> Self {
        Self {
            feature_extractor,
            inference_engine,
            predictions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn predict(&self, data: &[MarketData]) -> MlResult<Vec<Prediction>> {
        let start = std::time::Instant::now();
        
        let features = self.feature_extractor.extract_batch(data).await?;
        let predictions = self.inference_engine.predict_batch(&features).await?;
        
        let latency_ms = start.elapsed().as_millis() as u64;
        
        let mut result = Vec::new();
        for mut pred in predictions {
            pred.latency_ms = latency_ms / data.len().max(1) as u64;
            result.push(pred);
        }
        
        {
            let mut cache = self.predictions.write().await;
            cache.extend(result.clone());
            let excess = cache.len().saturating_sub(10000);
            if excess > 0 {
                cache.drain(0..excess);
            }
        }
        
        Ok(result)
    }

    pub async fn get_cached_predictions(&self, symbol: &str, limit: usize) -> Vec<Prediction> {
        let cache = self.predictions.read().await;
        cache
            .iter()
            .filter(|p| p.symbol == symbol)
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }
}
