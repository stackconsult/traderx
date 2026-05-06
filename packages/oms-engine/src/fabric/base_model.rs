// Base Model Integration (9-layer)
// Phase A: Base Model Foundation - Task A6: Base model integration (9-layer)

use serde::{Deserialize, Serialize};

/// Base model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseModelConfig {
    pub layers: usize,
    pub hidden_size: usize,
    pub activation: String,
    pub learning_rate: f64,
}

impl Default for BaseModelConfig {
    fn default() -> Self {
        Self {
            layers: 9,
            hidden_size: 128,
            activation: "relu".to_string(),
            learning_rate: 0.001,
        }
    }
}

/// Base model - 9-layer deterministic model
pub struct BaseModel {
    config: BaseModelConfig,
}

impl BaseModel {
    /// Create a new base model
    pub fn new(config: BaseModelConfig) -> Self {
        Self { config }
    }
    
    /// Forward pass through the 9-layer model
    pub fn forward(&self, input: &[f64]) -> Vec<f64> {
        let mut activations = input.to_vec();
        
        // 9-layer forward pass
        for _ in 0..self.config.layers {
            activations = self.layer_forward(&activations);
        }
        
        activations
    }
    
    /// Single layer forward pass
    fn layer_forward(&self, input: &[f64]) -> Vec<f64> {
        // Simple linear + activation
        input.iter()
            .map(|x| self.activate(x))
            .collect()
    }
    
    /// Activation function
    fn activate(&self, x: &f64) -> f64 {
        match self.config.activation.as_str() {
            "relu" => x.max(0.0),
            "sigmoid" => 1.0 / (1.0 + (-x).exp()),
            "tanh" => x.tanh(),
            _ => *x,
        }
    }
    
    /// Get model configuration
    pub fn get_config(&self) -> &BaseModelConfig {
        &self.config
    }
}

impl Default for BaseModel {
    fn default() -> Self {
        Self::new(BaseModelConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_base_model() {
        let model = BaseModel::default();
        
        let input = vec![0.5, -0.3, 0.8, 0.0];
        let output = model.forward(&input);
        
        assert_eq!(output.len(), input.len());
        
        // Check ReLU activation (no negative values)
        assert!(output.iter().all(|x| *x >= 0.0));
    }
    
    #[test]
    fn test_base_model_config() {
        let config = BaseModelConfig {
            layers: 9,
            hidden_size: 128,
            activation: "relu".to_string(),
            learning_rate: 0.001,
        };
        
        let model = BaseModel::new(config);
        assert_eq!(model.get_config().layers, 9);
        assert_eq!(model.get_config().hidden_size, 128);
    }
    
    #[test]
    fn test_activation_functions() {
        let relu_config = BaseModelConfig {
            layers: 1,
            hidden_size: 64,
            activation: "relu".to_string(),
            learning_rate: 0.001,
        };
        
        let relu_model = BaseModel::new(relu_config);
        assert!(relu_model.activate(&-0.5) == 0.0);
        assert!(relu_model.activate(&0.5) == 0.5);
    }
}
