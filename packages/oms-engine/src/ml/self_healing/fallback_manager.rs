use tracing::info;
use chrono::{DateTime, Utc};

/// Fallback manager for model failover
pub struct FallbackManager {
    fallback_chain: Vec<String>,
    current_model: String,
    fallback_history: Vec<(DateTime<Utc>, String, String, bool)>,
}

impl FallbackManager {
    pub fn new(primary_model: String, fallback_models: Vec<String>) -> Self {
        let mut fallback_chain = vec![primary_model.clone()];
        fallback_chain.extend(fallback_models);
        
        Self {
            fallback_chain,
            current_model: primary_model,
            fallback_history: Vec::new(),
        }
    }
    
    pub fn get_next_fallback(&self) -> Option<String> {
        let current_index = self.fallback_chain.iter()
            .position(|m| m == &self.current_model)?;
        
        self.fallback_chain.get(current_index + 1).cloned()
    }
    
    pub async fn execute_fallback(&mut self) -> Result<String, String> {
        let from_model = self.current_model.clone();
        
        if let Some(to_model) = self.get_next_fallback() {
            info!("Executing fallback from {} to {}", from_model, to_model);
            
            let success = true;
            
            self.fallback_history.push((Utc::now(), from_model.clone(), to_model.clone(), success));
            self.current_model = to_model.clone();
            
            if success {
                Ok(to_model)
            } else {
                Err("Fallback failed".to_string())
            }
        } else {
            Err("No more fallback models available".to_string())
        }
    }
    
    pub fn reset_to_primary(&mut self) {
        if let Some(primary) = self.fallback_chain.first() {
            self.current_model = primary.clone();
            info!("Reset to primary model: {}", primary);
        }
    }
    
    pub fn current_model(&self) -> &str {
        &self.current_model
    }
}
