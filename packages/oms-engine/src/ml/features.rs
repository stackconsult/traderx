use super::{MarketData, FeatureVector, MlResult, MlError};

pub struct FeatureExtractor;

impl FeatureExtractor {
    pub fn new() -> Self {
        Self
    }

    pub async fn extract_batch(&self, data: &[MarketData]) -> MlResult<Vec<FeatureVector>> {
        if data.is_empty() {
            return Err(MlError::InvalidInput("Empty market data".to_string()));
        }
        
        let mut features = Vec::with_capacity(data.len());
        
        for (i, candle) in data.iter().enumerate() {
            let feature_vec = self.extract_single(candle, data, i).await?;
            features.push(feature_vec);
        }
        
        Ok(features)
    }

    pub async fn extract_single(
        &self,
        candle: &MarketData,
        data: &[MarketData],
        index: usize,
    ) -> MlResult<FeatureVector> {
        let mut features = Vec::new();
        let mut names = Vec::new();
        
        // Basic price features
        features.push(candle.open);
        names.push("open".to_string());
        features.push(candle.high);
        names.push("high".to_string());
        features.push(candle.low);
        names.push("low".to_string());
        features.push(candle.close);
        names.push("close".to_string());
        features.push(candle.volume as f64);
        names.push("volume".to_string());
        
        // Derived features
        let range = candle.high - candle.low;
        let body = (candle.close - candle.open).abs();
        let upper_shadow = candle.high - candle.close.max(candle.open);
        let lower_shadow = candle.close.min(candle.open) - candle.low;
        
        features.push(range);
        names.push("range".to_string());
        features.push(body);
        names.push("body_size".to_string());
        features.push(upper_shadow);
        names.push("upper_shadow".to_string());
        features.push(lower_shadow);
        names.push("lower_shadow".to_string());
        
        // Candlestick pattern features
        features.push(body / range.max(0.0001));
        names.push("body_ratio".to_string());
        features.push(if candle.close > candle.open { 1.0 } else { -1.0 });
        names.push("direction".to_string());
        
        // Moving averages (if enough data)
        if index >= 5 {
            let ma5 = data[index - 5..=index].iter().map(|d| d.close).sum::<f64>() / 6.0;
            features.push(candle.close - ma5);
            names.push("close_ma5_diff".to_string());
        } else {
            features.push(0.0);
            names.push("close_ma5_diff".to_string());
        }
        
        if index >= 19 {
            let ma20 = data[index - 19..=index].iter().map(|d| d.close).sum::<f64>() / 20.0;
            features.push(candle.close - ma20);
            names.push("close_ma20_diff".to_string());
        } else {
            features.push(0.0);
            names.push("close_ma20_diff".to_string());
        }
        
        // Volume features
        if index > 0 {
            let vol_change = (candle.volume as f64 - data[index - 1].volume as f64) 
                / data[index - 1].volume.max(1) as f64;
            features.push(vol_change);
            names.push("volume_change".to_string());
        } else {
            features.push(0.0);
            names.push("volume_change".to_string());
        }
        
        // Volatility (if enough data)
        if index >= 1 {
            let returns: Vec<f64> = data[1..=index]
                .iter()
                .zip(data[0..index].iter())
                .map(|(curr, prev)| (curr.close - prev.close) / prev.close.max(0.0001))
                .collect();
            
            if !returns.is_empty() {
                let mean = returns.iter().sum::<f64>() / returns.len() as f64;
                let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64;
                let std_dev = variance.sqrt();
                features.push(std_dev);
                names.push("volatility".to_string());
            } else {
                features.push(0.0);
                names.push("volatility".to_string());
            }
        } else {
            features.push(0.0);
            names.push("volatility".to_string());
        }
        
        Ok(FeatureVector {
            symbol: candle.symbol.clone(),
            timestamp: candle.timestamp,
            features,
            feature_names: names,
        })
    }
}
