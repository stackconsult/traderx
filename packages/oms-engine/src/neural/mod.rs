use std::sync::Arc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use thiserror::Error;
use chrono::{DateTime, Utc};

pub mod runtime;
pub mod processor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralInput {
    pub input_id: Uuid,
    pub tensor_data: Vec<f32>,
    pub shape: Vec<usize>,
    pub dtype: TensorDtype,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralOutput {
    pub output_id: Uuid,
    pub tensor_data: Vec<f32>,
    pub shape: Vec<usize>,
    pub confidence: f64,
    pub inference_time_us: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TensorDtype {
    F32,
    F16,
    I32,
    I64,
}

#[derive(Debug, Error)]
pub enum NeuralError {
    #[error("Model load failed: {0}")]
    ModelLoad(String),
    #[error("Inference failed: {0}")]
    Inference(String),
    #[error("Invalid tensor shape: expected {expected:?}, got {got:?}")]
    InvalidShape { expected: Vec<usize>, got: Vec<usize> },
    #[error("Quantization not supported: {0}")]
    Quantization(String),
    #[error("Runtime not initialized")]
    RuntimeNotInitialized,
}

pub type NeuralResult<T> = Result<T, NeuralError>;

pub struct NeuralEngine {
    runtime: Arc<runtime::OnnxRuntime>,
    processor: Arc<processor::SignalProcessor>,
}

impl NeuralEngine {
    pub fn new(
        runtime: Arc<runtime::OnnxRuntime>,
        processor: Arc<processor::SignalProcessor>,
    ) -> Self {
        Self { runtime, processor }
    }

    pub async fn process_signal(
        &self,
        signal: &crate::llm::AgentSignal,
    ) -> NeuralResult<NeuralOutput> {
        let input = self.processor.signal_to_tensor(signal).await?;
        let output = self.runtime.infer(input).await?;
        let processed = self.processor.tensor_to_decision(&output).await?;
        Ok(processed)
    }

    pub async fn batch_process(
        &self,
        signals: &[crate::llm::AgentSignal],
    ) -> NeuralResult<Vec<NeuralOutput>> {
        let mut results = Vec::with_capacity(signals.len());
        for signal in signals {
            results.push(self.process_signal(signal).await?);
        }
        Ok(results)
    }
}
