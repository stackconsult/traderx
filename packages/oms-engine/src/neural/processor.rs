use super::{NeuralInput, NeuralOutput, NeuralResult, NeuralError, TensorDtype};
use crate::llm::AgentSignal;

pub struct SignalProcessor;

impl SignalProcessor {
    pub fn new() -> Self {
        Self
    }

    pub async fn signal_to_tensor(&self, signal: &AgentSignal) -> NeuralResult<NeuralInput> {
        let tensor_data = self.encode_signal(signal)?;
        let shape = vec![1, tensor_data.len()];
        
        Ok(NeuralInput {
            input_id: uuid::Uuid::new_v4(),
            tensor_data,
            shape,
            dtype: TensorDtype::F32,
            metadata: signal.meta.clone(),
        })
    }

    pub async fn tensor_to_decision(&self, output: &NeuralOutput) -> NeuralResult<NeuralOutput> {
        if output.tensor_data.is_empty() {
            return Err(NeuralError::Inference("Empty output tensor".to_string()));
        }
        
        let mut enhanced = output.clone();
        let probs = self.extract_probabilities(&output.tensor_data);
        
        if let Some((_, max_prob)) = probs.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal)) {
            enhanced.confidence = *max_prob as f64;
        }
        
        Ok(enhanced)
    }

    fn encode_signal(&self, signal: &AgentSignal) -> NeuralResult<Vec<f32>> {
        let mut features = Vec::with_capacity(10);
        
        features.push(signal.conviction as f32);
        features.push((signal.max_notional_usd / 100000.0).min(1.0) as f32);
        features.push((signal.ttl_ms as f32 / 10000.0).min(1.0));
        
        let direction_encoding = match signal.direction.as_str() {
            "long" => 1.0f32,
            "buy" => 1.0f32,
            "short" => -1.0f32,
            "sell" => -1.0f32,
            "hold" => 0.0f32,
            _ => 0.0f32,
        };
        features.push(direction_encoding);
        
        let symbol_hash = signal.symbol.bytes().fold(0u64, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(b as u64)
        }) % 1000;
        features.push((symbol_hash as f32 / 1000.0).min(1.0));
        
        let agent_hash = signal.agent_id.bytes().fold(0u64, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(b as u64)
        }) % 1000;
        features.push((agent_hash as f32 / 1000.0).min(1.0));
        
        if let Some(meta) = signal.meta.as_object() {
            let meta_count = meta.len() as f32 / 10.0;
            features.push(meta_count.min(1.0));
        } else {
            features.push(0.0f32);
        }
        
        features.push(signal.conviction.powi(2) as f32);
        
        features.push(1.0f32 - (signal.conviction as f32 - 0.5f32).abs() * 2.0f32);
        
        features.push(signal.conviction as f32 * direction_encoding);
        
        while features.len() < 10 {
            features.push(0.0f32);
        }
        
        features.truncate(10);
        
        Ok(features)
    }

    fn extract_probabilities(&self, tensor_data: &[f32]) -> Vec<f32> {
        if tensor_data.is_empty() {
            return vec![0.33f32, 0.33f32, 0.34f32];
        }
        
        let chunks: Vec<&[f32]> = tensor_data.chunks(3).collect();
        if let Some(last_chunk) = chunks.last() {
            if last_chunk.len() == 3 {
                let sum: f32 = last_chunk.iter().sum();
                if sum > 0.0 {
                    return last_chunk.iter().map(|&v| v / sum).collect();
                }
            }
        }
        
        vec![0.33f32, 0.33f32, 0.34f32]
    }

    pub async fn batch_signals(&self, signals: &[AgentSignal]) -> NeuralResult<Vec<NeuralInput>> {
        let mut results = Vec::with_capacity(signals.len());
        for signal in signals {
            results.push(self.signal_to_tensor(signal).await?);
        }
        Ok(results)
    }
}
