// Model Divergence Monitor
// Phase B: Advanced Model - Task B5: Model divergence monitor

use std::collections::HashMap;

/// Divergence threshold configuration
#[derive(Debug, Clone)]
pub struct DivergenceConfig {
    pub threshold: f64,
    pub window_size: usize,
}

impl Default for DivergenceConfig {
    fn default() -> Self {
        Self {
            threshold: 0.1,
            window_size: 100,
        }
    }
}

/// Divergence event
#[derive(Debug, Clone)]
pub struct DivergenceEvent {
    pub model_id: String,
    pub divergence_score: f64,
    pub timestamp: i64,
    pub triggered: bool,
}

/// Model divergence monitor - tracks model predictions divergence
pub struct DivergenceMonitor {
    config: DivergenceConfig,
    prediction_history: HashMap<String, Vec<f64>>,
}

impl DivergenceMonitor {
    /// Create a new divergence monitor
    pub fn new(config: DivergenceConfig) -> Self {
        Self {
            config,
            prediction_history: HashMap::new(),
        }
    }
    
    /// Record prediction for a model
    pub fn record_prediction(&mut self, model_id: String, prediction: f64) {
        let history = self.prediction_history.entry(model_id.clone()).or_insert_with(Vec::new);
        history.push(prediction);
        
        // Maintain window size
        if history.len() > self.config.window_size {
            history.remove(0);
        }
    }
    
    /// Calculate divergence between two models
    pub fn calculate_divergence(&self, model_a: &str, model_b: &str) -> Option<f64> {
        let history_a = self.prediction_history.get(model_a)?;
        let history_b = self.prediction_history.get(model_b)?;
        
        if history_a.len() != history_b.len() {
            return None;
        }
        
        let mut divergence = 0.0;
        for (a, b) in history_a.iter().zip(history_b.iter()) {
            divergence += (a - b).abs();
        }
        
        Some(divergence / history_a.len() as f64)
    }
    
    /// Check if divergence exceeds threshold
    pub fn check_divergence(&self, model_a: &str, model_b: &str) -> DivergenceEvent {
        let divergence = self.calculate_divergence(model_a, model_b).unwrap_or(0.0);
        let triggered = divergence > self.config.threshold;
        
        DivergenceEvent {
            model_id: format!("{} vs {}", model_a, model_b),
            divergence_score: divergence,
            timestamp: chrono::Utc::now().timestamp_millis(),
            triggered,
        }
    }
    
    /// Get all model IDs being tracked
    pub fn get_tracked_models(&self) -> Vec<String> {
        self.prediction_history.keys().cloned().collect()
    }
    
    /// Get prediction history for a model
    pub fn get_history(&self, model_id: &str) -> Option<&Vec<f64>> {
        self.prediction_history.get(model_id)
    }
}

impl Default for DivergenceMonitor {
    fn default() -> Self {
        Self::new(DivergenceConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_divergence_monitor() {
        let config = DivergenceConfig {
            threshold: 0.5,
            window_size: 10,
        };
        
        let mut monitor = DivergenceMonitor::new(config);
        
        // Record predictions
        for i in 0..10 {
            monitor.record_prediction("model_a".to_string(), i as f64);
            monitor.record_prediction("model_b".to_string(), (i as f64) + 0.1);
        }
        
        // Calculate divergence
        let divergence = monitor.calculate_divergence("model_a", "model_b").unwrap();
        assert_eq!(divergence, 0.1);
        
        // Check divergence
        let event = monitor.check_divergence("model_a", "model_b");
        assert!(!event.triggered); // 0.1 < 0.5 threshold
    }
    
    #[test]
    fn test_divergence_trigger() {
        let config = DivergenceConfig {
            threshold: 0.05,
            window_size: 10,
        };
        
        let mut monitor = DivergenceMonitor::new(config);
        
        // Record predictions with high divergence
        for i in 0..10 {
            monitor.record_prediction("model_a".to_string(), i as f64);
            monitor.record_prediction("model_b".to_string(), (i as f64) + 0.1);
        }
        
        // Check divergence
        let event = monitor.check_divergence("model_a", "model_b");
        assert!(event.triggered); // 0.1 > 0.05 threshold
    }
}
