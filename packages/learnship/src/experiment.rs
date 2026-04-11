//! Experiment tracking for Learnship retraining and validation runs.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info};

/// Experiment configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    pub model_name: String,
    pub experiment_type: ExperimentType,
    pub parameters: HashMap<String, serde_json::Value>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperimentType {
    Retraining,
    HyperparameterTuning,
    FeatureEngineering,
    AblationStudy,
}

/// Experiment tracking metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentMetadata {
    pub experiment_id: String,
    pub model_name: String,
    pub status: ExperimentStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub config: ExperimentConfig,
    pub metrics: HashMap<String, f64>,
    pub artifacts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperimentStatus {
    Created,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Experiment tracker integrated with MLflow.
pub struct ExperimentTracker {
    mlflow_client: reqwest::Client,
    mlflow_uri: String,
}

impl ExperimentTracker {
    pub fn new(mlflow_uri: String) -> Result<Self> {
        Ok(Self {
            mlflow_client: reqwest::Client::new(),
            mlflow_uri,
        })
    }

    /// Create a new experiment.
    pub async fn create_experiment(
        &self,
        model_name: &str,
        reason: &str,
    ) -> Result<String> {
        let experiment_id = format!("exp_{}_{}", model_name, chrono::Utc::now().timestamp());
        
        let config = ExperimentConfig {
            model_name: model_name.to_string(),
            experiment_type: ExperimentType::Retraining,
            parameters: HashMap::from([
                ("reason".to_string(), serde_json::Value::String(reason.to_string())),
                ("trigger".to_string(), serde_json::Value::String("learnship".to_string())),
            ]),
            tags: vec!["learnship".to_string(), "automated".to_string()],
        };

        let metadata = ExperimentMetadata {
            experiment_id: experiment_id.clone(),
            model_name: model_name.to_string(),
            status: ExperimentStatus::Created,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            config,
            metrics: HashMap::new(),
            artifacts: Vec::new(),
        };

        // Register with MLflow
        self.register_experiment(&metadata).await?;

        info!("Created experiment {} for model {}", experiment_id, model_name);
        Ok(experiment_id)
    }

    /// Log metrics to an experiment.
    pub async fn log_metric(
        &self,
        experiment_id: &str,
        metric_name: &str,
        value: f64,
        step: Option<i64>,
    ) -> Result<()> {
        let url = format!("{}/api/2.0/mlflow/runs/log-metric", self.mlflow_uri);
        
        let mut payload = serde_json::json!({
            "run_id": experiment_id,
            "key": metric_name,
            "value": value,
        });
        
        if let Some(s) = step {
            payload["step"] = serde_json::Value::Number(s.into());
        }

        let response = self.mlflow_client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to log metric to MLflow")?;

        if !response.status().is_success() {
            anyhow::bail!("MLflow returned error: {}", response.status());
        }

        debug!("Logged metric {}={} to experiment {}", metric_name, value, experiment_id);
        Ok(())
    }

    /// Log parameters to an experiment.
    pub async fn log_parameter(
        &self,
        experiment_id: &str,
        param_name: &str,
        value: &str,
    ) -> Result<()> {
        let url = format!("{}/api/2.0/mlflow/runs/log-parameter", self.mlflow_uri);
        
        let payload = serde_json::json!({
            "run_id": experiment_id,
            "key": param_name,
            "value": value,
        });

        let response = self.mlflow_client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to log parameter to MLflow")?;

        if !response.status().is_success() {
            anyhow::bail!("MLflow returned error: {}", response.status());
        }

        debug!("Logged parameter {}={} to experiment {}", param_name, value, experiment_id);
        Ok(())
    }

    /// Log an artifact to an experiment.
    pub async fn log_artifact(
        &self,
        experiment_id: &str,
        file_path: &str,
        artifact_path: Option<&str>,
    ) -> Result<()> {
        // In production, this would upload the file to MLflow's artifact store
        info!("Logging artifact {} to experiment {}", file_path, experiment_id);
        Ok(())
    }

    /// Update experiment status.
    pub async fn update_status(
        &self,
        experiment_id: &str,
        status: ExperimentStatus,
    ) -> Result<()> {
        let url = format!("{}/api/2.0/mlflow/runs/update", self.mlflow_uri);
        
        let payload = serde_json::json!({
            "run_id": experiment_id,
            "status": format!("{:?}", status),
        });

        let response = self.mlflow_client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to update experiment status")?;

        if !response.status().is_success() {
            anyhow::bail!("MLflow returned error: {}", response.status());
        }

        info!("Updated experiment {} status to {:?}", experiment_id, status);
        Ok(())
    }

    /// Log validation result to experiment.
    pub async fn log_validation_result(
        &self,
        model_name: &str,
        stage: crate::agents::ValidationStage,
        passed: bool,
        metrics: &HashMap<String, f64>,
    ) -> Result<()> {
        // Find the latest experiment for this model
        let experiment_id = self.find_latest_experiment(model_name).await?;
        
        // Log stage result
        self.log_metric(
            &experiment_id,
            &format!("validation_{:?}_passed", stage),
            if passed { 1.0 } else { 0.0 },
            None,
        ).await?;
        
        // Log stage metrics
        for (name, value) in metrics {
            self.log_metric(
                &experiment_id,
                &format!("validation_{:?}_{}", stage, name),
                *value,
                None,
            ).await?;
        }
        
        Ok(())
    }

    /// Get experiment history for a model.
    pub async fn get_experiment_history(
        &self,
        model_name: &str,
        limit: Option<usize>,
    ) -> Result<Vec<ExperimentMetadata>> {
        let url = format!("{}/api/2.0/mlflow/search/runs", self.mlflow_uri);
        
        let filter = format!("tags.model_name = '{}'", model_name);
        let payload = serde_json::json!({
            "filter": filter,
            "max_results": limit.unwrap_or(50),
        });

        let response = self.mlflow_client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to search experiments")?;

        if !response.status().is_success() {
            anyhow::bail!("MLflow returned error: {}", response.status());
        }

        let json: serde_json::Value = response.json().await?;
        let runs = json["runs"].as_array().ok_or_else(|| anyhow::anyhow!("Invalid response"))?;
        
        let mut experiments = Vec::new();
        for run in runs {
            // Parse MLflow run into ExperimentMetadata
            // This is simplified - in production would map all fields properly
            let experiment_id = run["info"]["run_id"].as_str().unwrap_or("").to_string();
            let status = match run["info"]["status"].as_str().unwrap_or("") {
                "FINISHED" => ExperimentStatus::Completed,
                "FAILED" => ExperimentStatus::Failed,
                "RUNNING" => ExperimentStatus::Running,
                _ => ExperimentStatus::Created,
            };
            
            experiments.push(ExperimentMetadata {
                experiment_id,
                model_name: model_name.to_string(),
                status,
                created_at: chrono::Utc::now(), // Would parse from MLflow
                updated_at: chrono::Utc::now(),
                config: ExperimentConfig {
                    model_name: model_name.to_string(),
                    experiment_type: ExperimentType::Retraining,
                    parameters: HashMap::new(),
                    tags: Vec::new(),
                },
                metrics: HashMap::new(),
                artifacts: Vec::new(),
            });
        }
        
        Ok(experiments)
    }

    // Helper methods

    async fn register_experiment(&self, metadata: &ExperimentMetadata) -> Result<()> {
        let url = format!("{}/api/2.0/mlflow/runs/create", self.mlflow_uri);
        
        let payload = serde_json::json!({
            "run_id": metadata.experiment_id,
            "experiment_id": metadata.experiment_id,
            "tags": [
                {"key": "model_name", "value": metadata.model_name},
                {"key": "experiment_type", "value": format!("{:?}", metadata.config.experiment_type)},
            ]
        });

        let response = self.mlflow_client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to register experiment")?;

        if !response.status().is_success() {
            anyhow::bail!("MLflow returned error: {}", response.status());
        }

        Ok(())
    }

    async fn find_latest_experiment(&self, model_name: &str) -> Result<String> {
        let history = self.get_experiment_history(model_name, Some(1)).await?;
        history
            .into_iter()
            .next()
            .map(|e| e.experiment_id)
            .ok_or_else(|| anyhow::anyhow!("No experiments found for model {}", model_name))
    }
}
