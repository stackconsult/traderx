use std::time::Duration;
use tracing::info;
use chrono::{DateTime, Utc};
use super::types::{AnomalyType, OptimizationStrategy};

/// Auto-tuning engine for model optimization
pub struct AutoTuner {
    optimization_strategies: Vec<OptimizationStrategy>,
    tuning_history: Vec<(DateTime<Utc>, OptimizationStrategy, bool)>,
    max_history_size: usize,
}

impl AutoTuner {
    pub fn new() -> Self {
        Self {
            optimization_strategies: Vec::new(),
            tuning_history: Vec::new(),
            max_history_size: 100,
        }
    }
    
    pub fn select_strategy(&self, anomaly: &AnomalyType) -> Option<OptimizationStrategy> {
        match anomaly {
            AnomalyType::PerformanceDegradation { metric, .. } => {
                match metric.as_str() {
                    "latency" => Some(OptimizationStrategy::ReduceBatchSize { current: 32, new: 16 }),
                    "memory" => Some(OptimizationStrategy::AdjustMemoryLimit { current: 8192.0, new: 4096.0 }),
                    _ => None,
                }
            },
            AnomalyType::MemoryLeak { .. } => Some(OptimizationStrategy::RestartModel),
            AnomalyType::AccuracyDrift { .. } => Some(OptimizationStrategy::RetrainModel),
            AnomalyType::HighErrorRate { .. } => Some(OptimizationStrategy::IncreaseTimeout { 
                current: Duration::from_secs(30), 
                new: Duration::from_secs(60) 
            }),
            AnomalyType::ResourceExhaustion { resource, .. } => {
                match resource.as_str() {
                    "memory" => Some(OptimizationStrategy::SwitchModel { 
                        from: "deepseek-coder:6.7b".to_string(), 
                        to: "deepseek-coder:1.3b".to_string() 
                    }),
                    _ => None,
                }
            },
        }
    }
    
    pub async fn apply_strategy(&mut self, strategy: OptimizationStrategy) -> Result<(), String> {
        let timestamp = Utc::now();
        
        let success = match &strategy {
            OptimizationStrategy::ReduceBatchSize { new, .. } => {
                info!("Applying batch size reduction to {}", new);
                true
            },
            OptimizationStrategy::IncreaseTimeout { new, .. } => {
                info!("Applying timeout increase to {:?}", new);
                true
            },
            OptimizationStrategy::AdjustMemoryLimit { new, .. } => {
                info!("Applying memory limit adjustment to {}MB", new);
                true
            },
            OptimizationStrategy::SwitchModel { from, to } => {
                info!("Switching model from {} to {}", from, to);
                true
            },
            OptimizationStrategy::RestartModel => {
                info!("Restarting model");
                true
            },
            OptimizationStrategy::RetrainModel => {
                info!("Initiating model retraining");
                true
            },
        };
        
        self.tuning_history.push((timestamp, strategy, success));
        
        if self.tuning_history.len() > self.max_history_size {
            self.tuning_history.remove(0);
        }
        
        if success {
            Ok(())
        } else {
            Err("Failed to apply optimization strategy".to_string())
        }
    }
    
    pub fn get_tuning_stats(&self) -> (usize, f64) {
        let total_attempts = self.tuning_history.len();
        let successful_attempts = self.tuning_history.iter().filter(|(_, _, success)| *success).count();
        let success_rate = if total_attempts > 0 {
            successful_attempts as f64 / total_attempts as f64
        } else {
            0.0
        };
        
        (total_attempts, success_rate)
    }
}
