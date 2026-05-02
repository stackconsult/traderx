use std::collections::HashMap;
use super::{FeatureVector, Prediction, PredictionType, MlResult, MlError};
use uuid::Uuid;

pub struct InferenceEngine {
    model_version: String,
    cache: std::sync::Arc<tokio::sync::RwLock<HashMap<String, Prediction>>>,
}

impl InferenceEngine {
    pub fn new(model_version: &str) -> Self {
        Self {
            model_version: model_version.to_string(),
            cache: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    pub async fn predict_batch(&self, features: &[FeatureVector]) -> MlResult<Vec<Prediction>> {
        let mut predictions = Vec::with_capacity(features.len());
        
        for feature_vec in features {
            let prediction = self.predict_single(feature_vec).await?;
            predictions.push(prediction);
        }
        
        Ok(predictions)
    }

    pub async fn predict_single(&self, features: &FeatureVector) -> MlResult<Prediction> {
        let cache_key = format!("{}:{}", features.symbol, features.timestamp.timestamp());
        
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }
        
        let prediction = self.mock_inference(features).await?;
        
        {
            let mut cache = self.cache.write().await;
            cache.insert(cache_key, prediction.clone());
        }
        
        Ok(prediction)
    }

    async fn mock_inference(&self, features: &FeatureVector) -> MlResult<Prediction> {
        let close_idx = features.feature_names.iter().position(|n| n == "close")
            .ok_or_else(|| MlError::FeatureExtraction("Missing close feature".to_string()))?;
        let close = features.features[close_idx];
        
        let ma5_idx = features.feature_names.iter().position(|n| n == "close_ma5_diff");
        let ma20_idx = features.feature_names.iter().position(|n| n == "close_ma20_diff");
        
        let mut probabilities = HashMap::new();
        
        let ma5_signal = ma5_idx.map(|i| features.features[i]).unwrap_or(0.0);
        let ma20_signal = ma20_idx.map(|i| features.features[i]).unwrap_or(0.0);
        
        let mut buy_prob: f64 = 0.33;
        let mut sell_prob: f64 = 0.33;
        let mut hold_prob: f64 = 0.34;
        
        if ma5_signal > 0.0 {
            buy_prob += 0.15;
            sell_prob -= 0.075;
            hold_prob -= 0.075;
        } else if ma5_signal < 0.0 {
            sell_prob += 0.15;
            buy_prob -= 0.075;
            hold_prob -= 0.075;
        }
        
        if ma20_signal > 0.0 {
            buy_prob += 0.1;
            sell_prob -= 0.05;
            hold_prob -= 0.05;
        } else if ma20_signal < 0.0 {
            sell_prob += 0.1;
            buy_prob -= 0.05;
            hold_prob -= 0.05;
        }
        
        let total = buy_prob + sell_prob + hold_prob;
        buy_prob /= total;
        sell_prob /= total;
        hold_prob /= total;
        
        probabilities.insert("buy".to_string(), buy_prob);
        probabilities.insert("sell".to_string(), sell_prob);
        probabilities.insert("hold".to_string(), hold_prob);
        
        let confidence = (buy_prob.max(sell_prob).max(hold_prob) * 100.0).min(99.0);
        
        Ok(crate::ml::Prediction {
            prediction_id: Uuid::new_v4(),
            symbol: features.symbol.clone(),
            prediction_type: PredictionType::PriceDirection,
            confidence,
            probabilities,
            timestamp: chrono::Utc::now(),
            model_version: self.model_version.clone(),
            latency_ms: 1,
        })
    }

    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    pub async fn get_cache_size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }
}
