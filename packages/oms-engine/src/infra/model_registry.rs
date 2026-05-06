// Model Registry
// Phase 0: Foundation - Model versioning and tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

/// Model type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModelType {
    Base,
    Advanced,
    HyperStatic,
}

/// Model version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl ModelVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn as_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub model_type: ModelType,
    pub version: ModelVersion,
    pub created_at: i64,
    pub parameters: HashMap<String, String>,
    pub performance_metrics: HashMap<String, f64>,
}

/// Model registry
pub struct ModelRegistry {
    models: RwLock<HashMap<String, ModelMetadata>>,
}

impl ModelRegistry {
    /// Create a new model registry
    pub fn new() -> Self {
        Self {
            models: RwLock::new(HashMap::new()),
        }
    }

    /// Register a model
    pub fn register_model(
        &self,
        model_id: String,
        metadata: ModelMetadata,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut models = self.models.write().unwrap();
        models.insert(model_id, metadata);
        Ok(())
    }

    /// Get model metadata
    pub fn get_model(&self, model_id: &str) -> Option<ModelMetadata> {
        let models = self.models.read().unwrap();
        models.get(model_id).cloned()
    }

    /// Get all models of a type
    pub fn get_models_by_type(&self, model_type: ModelType) -> Vec<(String, ModelMetadata)> {
        let models = self.models.read().unwrap();
        models
            .iter()
            .filter(|(_, metadata)| metadata.model_type == model_type)
            .map(|(id, metadata)| (id.clone(), metadata.clone()))
            .collect()
    }

    /// Get latest model of a type
    pub fn get_latest_model(&self, model_type: ModelType) -> Option<(String, ModelMetadata)> {
        let models = self.models.read().unwrap();
        models
            .iter()
            .filter(|(_, metadata)| metadata.model_type == model_type)
            .max_by_key(|(_, metadata)| metadata.created_at)
            .map(|(id, metadata)| (id.clone(), metadata.clone()))
    }

    /// List all models
    pub fn list_models(&self) -> Vec<String> {
        let models = self.models.read().unwrap();
        models.keys().cloned().collect()
    }

    /// Remove a model
    pub fn remove_model(&self, model_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut models = self.models.write().unwrap();
        models
            .remove(model_id)
            .map(|_| ())
            .ok_or_else(|| format!("Model {} not found", model_id).into())
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_registry() {
        let registry = ModelRegistry::new();

        let metadata = ModelMetadata {
            model_type: ModelType::Base,
            version: ModelVersion::new(1, 0, 0),
            created_at: 1234567890,
            parameters: HashMap::new(),
            performance_metrics: HashMap::new(),
        };

        registry
            .register_model("base_v1".to_string(), metadata)
            .unwrap();

        let retrieved = registry.get_model("base_v1").unwrap();
        assert_eq!(retrieved.model_type, ModelType::Base);

        let base_models = registry.get_models_by_type(ModelType::Base);
        assert_eq!(base_models.len(), 1);

        let latest = registry.get_latest_model(ModelType::Base);
        assert!(latest.is_some());
    }
}
