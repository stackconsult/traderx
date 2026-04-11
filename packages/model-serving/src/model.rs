//! Model registry with MLflow integration and dynamic loading.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Model metadata from MLflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub version: String,
    pub stage: String, // "Production", "Staging", "Archived"
    pub run_id: String,
    pub model_uri: String,
    pub input_schema: Vec<String>,
    pub output_schema: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub tags: HashMap<String, String>,
}

/// Model registry that tracks loaded models and their metadata.
pub struct ModelRegistry {
    models: Arc<RwLock<HashMap<String, Arc<LoadedModel>>>>,
    mlflow_tracking_uri: String,
}

/// A loaded ONNX model with inference session.
pub struct LoadedModel {
    pub info: ModelInfo,
    pub session: ort::Session,
    pub input_names: Vec<String>,
    pub output_names: Vec<String>,
}

impl ModelRegistry {
    pub fn new(mlflow_tracking_uri: String) -> Self {
        Self {
            models: Arc::new(RwLock::new(HashMap::new())),
            mlflow_tracking_uri,
        }
    }

    /// Load a model from MLflow by name and version.
    pub async fn load_model(&self, name: &str, version: &str) -> Result<Arc<LoadedModel>> {
        let key = format!("{}:{}", name, version);
        
        // Check if already loaded
        {
            let models = self.models.read().await;
            if let Some(model) = models.get(&key) {
                return Ok(Arc::clone(model));
            }
        }

        // Fetch model info from MLflow
        let info = self.fetch_model_info(name, version).await?;
        
        // Download model artifact
        let model_path = self.download_model(&info.model_uri).await?;
        
        // Create ONNX session
        let session = ort::Session::builder()?
            .with_intra_threads(4)?
            .with_inter_threads(2)?
            .with_optimization_level(ort::GraphOptimizationLevel::Level3)?
            .commit_from_file(&model_path)?;

        // Get input/output names
        let input_names = session.inputs.iter().map(|i| i.name.clone()).collect();
        let output_names = session.outputs.iter().map(|o| o.name.clone()).collect();

        let loaded_model = Arc::new(LoadedModel {
            info: info.clone(),
            session,
            input_names,
            output_names,
        });

        // Store in registry
        {
            let mut models = self.models.write().await;
            models.insert(key, Arc::clone(&loaded_model));
        }

        info!("Loaded model {}:{}", name, version);
        Ok(loaded_model)
    }

    /// Get the production version of a model.
    pub async fn get_production_model(&self, name: &str) -> Result<Arc<LoadedModel>> {
        let info = self.fetch_production_model_info(name).await?;
        self.load_model(name, &info.version).await
    }

    /// List all registered models.
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let url = format!("{}/api/2.0/mlflow/registered-models/list", self.mlflow_tracking_uri);
        let response = reqwest::get(&url).await?;
        let json: serde_json::Value = response.json().await?;
        
        let models = json["registered_models"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid response"))?
            .iter()
            .filter_map(|m| {
                let name = m["name"].as_str()?;
                Some(self.fetch_latest_model_info(name))
            })
            .collect::<Vec<_>>();

        // Execute all fetches in parallel
        let results = futures::future::join_all(models).await;
        let mut model_infos = Vec::new();
        for result in results {
            match result {
                Ok(info) => model_infos.push(info),
                Err(e) => warn!("Failed to fetch model info: {}", e),
            }
        }

        Ok(model_infos)
    }

    async fn fetch_model_info(&self, name: &str, version: &str) -> Result<ModelInfo> {
        let url = format!(
            "{}/api/2.0/mlflow/model-versions/get",
            self.mlflow_tracking_uri
        );
        
        let response = reqwest::Client::new()
            .post(&url)
            .json(&serde_json::json!({
                "name": name,
                "version": version
            }))
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        let mv = &json["model_version"];

        let tags = mv["tags"]
            .as_object()
            .map(|o| o.iter()
                .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string())))
                .collect())
            .unwrap_or_default();

        Ok(ModelInfo {
            name: mv["name"].as_str().unwrap().to_string(),
            version: mv["version"].as_str().unwrap().to_string(),
            stage: mv["current_stage"].as_str().unwrap().to_string(),
            run_id: mv["run_id"].as_str().unwrap().to_string(),
            model_uri: mv["source"].as_str().unwrap().to_string(),
            input_schema: vec![], // Would fetch from MLflow model signature
            output_schema: vec![], // Would fetch from MLflow model signature
            created_at: chrono::DateTime::parse_from_rfc3339(
                mv["creation_timestamp"].as_str().unwrap()
            )?.with_timezone(&chrono::Utc),
            tags,
        })
    }

    async fn fetch_production_model_info(&self, name: &str) -> Result<ModelInfo> {
        let url = format!(
            "{}/api/2.0/mlflow/model-versions/get-latest",
            self.mlflow_tracking_uri
        );
        
        let response = reqwest::Client::new()
            .post(&url)
            .json(&serde_json::json!({
                "name": name,
                "stages": ["Production"]
            }))
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        let mv = &json["model_version"];
        
        self.fetch_model_info(name, mv["version"].as_str().unwrap()).await
    }

    async fn fetch_latest_model_info(&self, name: &str) -> Result<ModelInfo> {
        let url = format!(
            "{}/api/2.0/mlflow/model-versions/get-latest",
            self.mlflow_tracking_uri
        );
        
        let response = reqwest::Client::new()
            .post(&url)
            .json(&serde_json::json!({
                "name": name
            }))
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        let mv = &json["model_version"];
        
        self.fetch_model_info(name, mv["version"].as_str().unwrap()).await
    }

    async fn download_model(&self, model_uri: &str) -> Result<String> {
        // In production, this would download from MLflow artifact store
        // For now, assume local path
        Ok(model_uri.replace("file://", ""))
    }

    /// Unload a model to free memory.
    pub async fn unload_model(&self, name: &str, version: &str) -> Result<()> {
        let key = format!("{}:{}", name, version);
        let mut models = self.models.write().await;
        if models.remove(&key).is_some() {
            info!("Unloaded model {}:{}", name, version);
        }
        Ok(())
    }

    /// Get a loaded model by name and version.
    pub async fn get_model(&self, name: &str, version: &str) -> Option<Arc<LoadedModel>> {
        let key = format!("{}:{}", name, version);
        let models = self.models.read().await;
        models.get(&key).cloned()
    }
}
