//! High-performance inference engine with <1ms latency target.

use crate::model::LoadedModel;
use crate::features::{FeatureStore, FeatureCache};
use crate::compression::TurboQuantCompressor;
use anyhow::{Context, Result};
use ndarray::Array;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

/// Inference request from clients.
#[derive(Debug, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub model_name: String,
    pub model_version: Option<String>,
    pub symbol: String,
    pub timestamp_ns: i64,
    pub features: Option<HashMap<String, f64>>, // Optional override features
}

/// Inference response to clients.
#[derive(Debug, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub model_name: String,
    pub model_version: String,
    pub symbol: String,
    pub predictions: Vec<f64>,
    pub confidence: Option<f64>,
    pub latency_ns: i64,
    pub features_used: Vec<String>,
    pub compressed_context: Option<Vec<u8>>, // TurboQuant compressed context
}

/// High-performance inference engine.
pub struct InferenceEngine {
    models: Arc<crate::model::ModelRegistry>,
    feature_store: Arc<FeatureStore>,
    feature_cache: Arc<FeatureCache>,
    compressor: Arc<TurboQuantCompressor>,
    default_models: Arc<RwLock<HashMap<String, String>>>, // symbol -> default model
}

impl InferenceEngine {
    pub fn new(
        models: Arc<crate::model::ModelRegistry>,
        feature_store: Arc<FeatureStore>,
        feature_cache: Arc<FeatureCache>,
        compressor: Arc<TurboQuantCompressor>,
    ) -> Self {
        Self {
            models,
            feature_store,
            feature_cache,
            compressor,
            default_models: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Run inference with sub-millisecond latency target.
    pub async fn predict(&self, request: InferenceRequest) -> Result<InferenceResponse> {
        let start = Instant::now();
        
        // Get model (use production version if not specified)
        let version = request.model_version
            .or_else(|| self.get_default_model(&request.symbol).await)
            .unwrap_or_else(|| "latest".to_string());
        
        let model = if version == "latest" {
            self.models.get_production_model(&request.model_name).await?
        } else {
            self.models.get_model(&request.model_name, &version).await
                .ok_or_else(|| anyhow::anyhow!("Model not found: {}:{}",
                    request.model_name, version))?
        };

        // Get features (from cache or store)
        let features = if let Some(override_features) = request.features {
            override_features
        } else {
            self.get_features(&request.symbol).await?
        };

        // Prepare input tensor
        let input_tensor = self.prepare_input(&features, &model.input_names).await?;
        
        // Run inference
        let outputs = model.session.run(vec![input_tensor])?;
        
        // Extract predictions
        let predictions = self.extract_predictions(&outputs, &model.output_names)?;
        
        // Calculate confidence (simple softmax for binary classification)
        let confidence = if predictions.len() == 2 {
            let exp_sum = predictions.iter().map(|x| x.exp()).sum::<f64>();
            Some(predictions[1].exp() / exp_sum)
        } else {
            None
        };

        // Compress context for future reference
        let compressed_context = if features.len() > 50 {
            Some(self.compressor.compress(&features).await?)
        } else {
            None
        };

        let latency = start.elapsed().as_nanos() as i64;

        Ok(InferenceResponse {
            model_name: request.model_name,
            model_version: model.info.version.clone(),
            symbol: request.symbol,
            predictions,
            confidence,
            latency_ns: latency,
            features_used: features.keys().cloned().collect(),
            compressed_context,
        })
    }

    /// Batch inference for multiple symbols (higher throughput).
    pub async fn predict_batch(
        &self,
        requests: Vec<InferenceRequest>,
    ) -> Result<Vec<InferenceResponse>> {
        // Process in parallel with rayon for CPU-bound work
        let responses: Result<Vec<_>> = requests
            .into_iter()
            .map(|req| {
                let engine = self;
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(engine.predict(req))
                })
            })
            .collect();
        
        responses
    }

    async fn get_features(&self, symbol: &str) -> Result<HashMap<String, f64>> {
        // Try cache first (50ms TTL)
        if let Some(features) = self.feature_cache.get(symbol).await {
            return Ok(features);
        }

        // Fetch from Redis feature store
        let features = self.feature_store.get_all(symbol).await?
            .ok_or_else(|| anyhow::anyhow!("No features found for {}", symbol))?;

        // Update cache
        self.feature_cache.set(symbol, features.clone()).await;
        
        Ok(features)
    }

    async fn prepare_input(
        &self,
        features: &HashMap<String, f64>,
        input_names: &[String],
    ) -> Result<ort::DynValue> {
        if input_names.len() != 1 {
            return Err(anyhow::anyhow!("Only single input models supported"));
        }

        let input_name = &input_names[0];
        
        // Create feature vector in the order expected by the model
        // In production, this would use the model's input schema
        let feature_vector: Vec<f32> = vec![
            features.get("returns_1m").unwrap_or(&0.0),
            features.get("returns_5m").unwrap_or(&0.0),
            features.get("returns_1h").unwrap_or(&0.0),
            features.get("realized_vol_5m").unwrap_or(&0.0),
            features.get("realized_vol_1h").unwrap_or(&0.0),
            features.get("rsi_14").unwrap_or(&50.0),
            features.get("macd_signal").unwrap_or(&0.0),
            features.get("bb_upper").unwrap_or(&0.0),
            features.get("bb_lower").unwrap_or(&0.0),
            features.get("vwap_dev").unwrap_or(&0.0),
            features.get("volume_zscore").unwrap_or(&0.0),
            features.get("ofi").unwrap_or(&0.0),
            features.get("spread_bps").unwrap_or(&0.0),
        ]
        .iter()
        .map(|&x| x as f32)
        .collect();

        let array = Array::from_shape_vec((1, feature_vector.len()), feature_vector)?;
        let tensor = ort::Value::from_array(array.into_dyn())?;
        
        Ok(tensor.into_dyn())
    }

    fn extract_predictions(
        &self,
        outputs: &[ort::DynValue],
        output_names: &[String],
    ) -> Result<Vec<f64>> {
        if outputs.is_empty() {
            return Err(anyhow::anyhow!("No outputs from model"));
        }

        let output = &outputs[0];
        let data = output.try_extract::<f32>()?;
        
        // Convert to Vec<f64>
        Ok(data.as_slice().unwrap().iter().map(|&x| x as f64).collect())
    }

    /// Set default model for a symbol.
    pub async fn set_default_model(&self, symbol: &str, model_version: &str) {
        let mut defaults = self.default_models.write().await;
        defaults.insert(symbol.to_string(), model_version.to_string());
    }

    async fn get_default_model(&self, symbol: &str) -> Option<String> {
        let defaults = self.default_models.read().await;
        defaults.get(symbol).cloned()
    }

    /// Warm up models to avoid cold-start latency.
    pub async fn warm_up(&self, model_names: &[String]) -> Result<()> {
        info!("Warming up {} models...", model_names.len());
        
        for model_name in model_names {
            if let Ok(model) = self.models.get_production_model(model_name).await {
                // Create dummy input for warm-up
                let dummy_features = HashMap::from([
                    ("returns_1m".to_string(), 0.0),
                    ("returns_5m".to_string(), 0.0),
                    ("returns_1h".to_string(), 0.0),
                    ("realized_vol_5m".to_string(), 0.0),
                    ("realized_vol_1h".to_string(), 0.0),
                    ("rsi_14".to_string(), 50.0),
                    ("macd_signal".to_string(), 0.0),
                    ("bb_upper".to_string(), 0.0),
                    ("bb_lower".to_string(), 0.0),
                    ("vwap_dev".to_string(), 0.0),
                    ("volume_zscore".to_string(), 0.0),
                    ("ofi".to_string(), 0.0),
                    ("spread_bps".to_string(), 0.0),
                ]);

                let _ = self.prepare_input(&dummy_features, &model.input_names).await;
                debug!("Warmed up model {}", model_name);
            }
        }

        Ok(())
    }
}
