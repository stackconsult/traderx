use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, mpsc};
use tracing::{info, warn, error, debug};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::llm::{LlmRequest, AgentResponse, LlmResult, LlmError};
use crate::observability::{AgentMetrics, StructuredLogger};

/// Function execution step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub step_id: Uuid,
    pub step_name: String,
    pub function_name: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub dependencies: Vec<Uuid>,
    pub timeout_ms: u64,
    pub retry_count: u32,
    pub status: StepStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
    Timeout,
}

/// Orchestration workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub workflow_id: Uuid,
    pub workflow_name: String,
    pub steps: Vec<ExecutionStep>,
    pub context: HashMap<String, serde_json::Value>,
    pub status: WorkflowStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub total_duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkflowStatus {
    Created,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Parameter validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub parameter_name: String,
    pub rule_type: ValidationType,
    pub required: bool,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub allowed_values: Option<Vec<serde_json::Value>>,
    pub regex_pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    Enum,
    Regex,
    Custom,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Function execution context
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub workflow_id: Uuid,
    pub step_id: Uuid,
    pub context: HashMap<String, serde_json::Value>,
    pub metadata: HashMap<String, String>,
}

/// Function handler trait
#[async_trait::async_trait]
pub trait FunctionHandler: Send + Sync {
    async fn execute(
        &self,
        context: &ExecutionContext,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, String>;
    
    fn validate_parameters(
        &self,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> ValidationResult;
    
    fn function_name(&self) -> &str;
}

/// Multi-step function orchestrator
pub struct FunctionOrchestrator {
    workflows: Arc<RwLock<HashMap<Uuid, Workflow>>>,
    function_handlers: Arc<RwLock<HashMap<String, Arc<dyn FunctionHandler>>>>,
    validation_rules: HashMap<String, Vec<ValidationRule>>,
    metrics: Option<Arc<AgentMetrics>>,
    logger: Option<Arc<StructuredLogger>>,
    max_concurrent_steps: usize,
    step_timeout_ms: u64,
}

impl FunctionOrchestrator {
    pub fn new() -> Self {
        Self {
            workflows: Arc::new(RwLock::new(HashMap::new())),
            function_handlers: Arc::new(RwLock::new(HashMap::new())),
            validation_rules: HashMap::new(),
            metrics: None,
            logger: None,
            max_concurrent_steps: 10,
            step_timeout_ms: 30000, // 30 seconds default
        }
    }

    pub fn with_metrics(mut self, metrics: Arc<AgentMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    pub fn with_logger(mut self, logger: Arc<StructuredLogger>) -> Self {
        self.logger = Some(logger);
        self
    }

    pub fn with_max_concurrent_steps(mut self, max: usize) -> Self {
        self.max_concurrent_steps = max;
        self
    }

    pub fn with_step_timeout_ms(mut self, timeout: u64) -> Self {
        self.step_timeout_ms = timeout;
        self
    }

    /// Register a function handler
    pub async fn register_handler(&self, handler: Arc<dyn FunctionHandler>) {
        let handler_name = handler.function_name().to_string();
        let mut handlers = self.function_handlers.write().await;
        handlers.insert(handler_name.clone(), handler);
        
        info!("Registered function handler: {}", handler_name);
    }

    /// Register validation rules for a function
    pub fn register_validation_rules(&mut self, function_name: String, rules: Vec<ValidationRule>) {
        self.validation_rules.insert(function_name, rules);
    }

    /// Create a new workflow
    pub async fn create_workflow(
        &self,
        workflow_name: String,
        steps: Vec<ExecutionStep>,
        context: HashMap<String, serde_json::Value>,
    ) -> Uuid {
        let workflow_id = Uuid::new_v4();
        let workflow = Workflow {
            workflow_id,
            workflow_name: workflow_name.clone(),
            steps,
            context,
            status: WorkflowStatus::Created,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            total_duration_ms: None,
        };

        let mut workflows = self.workflows.write().await;
        workflows.insert(workflow_id, workflow.clone());

        info!("Created workflow: {} (ID: {})", workflow_name, workflow_id);

        workflow_id
    }

    /// Execute a workflow
    pub async fn execute_workflow(&self, workflow_id: Uuid) -> LlmResult<Workflow> {
        let start_time = Instant::now();
        
        // Update workflow status
        {
            let mut workflows = self.workflows.write().await;
            if let Some(workflow) = workflows.get_mut(&workflow_id) {
                workflow.status = WorkflowStatus::Running;
                workflow.started_at = Some(Utc::now());
            }
        }

        // Execute steps
        let result = self.execute_workflow_steps(workflow_id).await;

        // Update workflow status
        let duration = start_time.elapsed();
        {
            let mut workflows = self.workflows.write().await;
            if let Some(workflow) = workflows.get_mut(&workflow_id) {
                workflow.completed_at = Some(Utc::now());
                workflow.total_duration_ms = Some(duration.as_millis() as u64);
                
                match &result {
                    Ok(_) => {
                        workflow.status = WorkflowStatus::Completed;
                    },
                    Err(e) => {
                        workflow.status = WorkflowStatus::Failed;
                        error!("Workflow {} failed: {}", workflow_id, e);
                    }
                }
            }
        }

        result
    }

    /// Execute workflow steps
    async fn execute_workflow_steps(&self, workflow_id: Uuid) -> LlmResult<Workflow> {
        let mut workflows = self.workflows.write().await;
        let workflow = workflows.get_mut(&workflow_id)
            .ok_or_else(|| LlmError::RequestFailed(format!("Workflow {} not found", workflow_id)))?;

        let mut completed_steps = Vec::new();
        let mut failed_steps = Vec::new();

        // Execute steps in dependency order
        let steps_copy = workflow.steps.clone();
        let mut execution_order = self.calculate_execution_order(&steps_copy);

        while let Some(step_id) = execution_order.pop() {
            let step_index = workflow.steps.iter().position(|s| s.step_id == step_id);
            if let Some(idx) = step_index {
                // Check if dependencies are satisfied
                let steps_snapshot = workflow.steps.clone();
                if !self.are_dependencies_satisfied(&steps_snapshot[idx], &steps_snapshot) {
                    workflow.steps[idx].status = StepStatus::Skipped;
                    continue;
                }

                // Execute step
                let execution_context = ExecutionContext {
                    workflow_id,
                    step_id: workflow.steps[idx].step_id,
                    context: workflow.context.clone(),
                    metadata: HashMap::from([
                        ("step_name".to_string(), workflow.steps[idx].step_name.clone()),
                        ("function_name".to_string(), workflow.steps[idx].function_name.clone()),
                    ]),
                };

                workflow.steps[idx].status = StepStatus::Running;
                workflow.steps[idx].start_time = Some(Utc::now());

                let result = self.execute_step(&execution_context, &workflow.steps[idx]).await;

                workflow.steps[idx].end_time = Some(Utc::now());
                if let Some(start) = workflow.steps[idx].start_time {
                    workflow.steps[idx].duration_ms = Some((Utc::now() - start).num_milliseconds() as u64);
                }

                match result {
                    Ok(value) => {
                        workflow.steps[idx].status = StepStatus::Completed;
                        workflow.steps[idx].result = Some(value);
                        completed_steps.push(workflow.steps[idx].step_id);
                        
                        // Update context with result
                        if let Some(ref result_val) = workflow.steps[idx].result {
                            workflow.context.insert(workflow.steps[idx].step_name.clone(), result_val.clone());
                        }
                    },
                    Err(e) => {
                        workflow.steps[idx].status = StepStatus::Failed;
                        workflow.steps[idx].error = Some(e.clone());
                        failed_steps.push(workflow.steps[idx].step_id);
                        
                        error!("Step {} failed: {}", workflow.steps[idx].step_name, e);
                    }
                }
            }
        }

        // Determine workflow result
        if failed_steps.is_empty() {
            Ok(workflow.clone())
        } else {
            Err(LlmError::RequestFailed(format!("Workflow failed: {} steps failed", failed_steps.len())))
        }
    }

    /// Calculate execution order based on dependencies
    fn calculate_execution_order(&self, steps: &[ExecutionStep]) -> Vec<Uuid> {
        let mut order = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut visiting = std::collections::HashSet::new();

        for step in steps {
            if !visited.contains(&step.step_id) {
                self.visit_step(&step.step_id, steps, &mut order, &mut visited, &mut visiting);
            }
        }

        order
    }

    fn visit_step(
        &self,
        step_id: &Uuid,
        steps: &[ExecutionStep],
        order: &mut Vec<Uuid>,
        visited: &mut std::collections::HashSet<Uuid>,
        visiting: &mut std::collections::HashSet<Uuid>,
    ) {
        if visited.contains(step_id) {
            return;
        }

        if visiting.contains(step_id) {
            panic!("Cycle detected in step dependencies");
        }

        visiting.insert(*step_id);

        if let Some(step) = steps.iter().find(|s| s.step_id == *step_id) {
            for dep_id in &step.dependencies {
                self.visit_step(dep_id, steps, order, visited, visiting);
            }
        }

        visiting.remove(step_id);
        visited.insert(*step_id);
        order.push(*step_id);
    }

    /// Check if step dependencies are satisfied
    fn are_dependencies_satisfied(&self, step: &ExecutionStep, all_steps: &[ExecutionStep]) -> bool {
        step.dependencies.iter().all(|dep_id| {
            all_steps.iter().any(|s| s.step_id == *dep_id && s.status == StepStatus::Completed)
        })
    }

    /// Execute a single step
    async fn execute_step(
        &self,
        context: &ExecutionContext,
        step: &ExecutionStep,
    ) -> Result<serde_json::Value, String> {
        // Validate parameters
        let validation = self.validate_parameters(&step.function_name, &step.parameters).await;
        if !validation.is_valid {
            return Err(format!("Parameter validation failed: {:?}", validation.errors));
        }

        // Get handler
        let handlers = self.function_handlers.read().await;
        let handler = handlers.get(&step.function_name)
            .ok_or_else(|| format!("No handler registered for function: {}", step.function_name))?;

        // Execute with timeout
        let timeout = Duration::from_millis(step.timeout_ms);
        let result = tokio::time::timeout(
            timeout,
            handler.execute(context, &step.parameters)
        ).await;

        match result {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(format!("Step execution timed out after {}ms", step.timeout_ms)),
        }
    }

    /// Validate parameters
    async fn validate_parameters(
        &self,
        function_name: &str,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        if let Some(rules) = self.validation_rules.get(function_name) {
            for rule in rules {
                if rule.required && !parameters.contains_key(&rule.parameter_name) {
                    errors.push(format!("Required parameter '{}' is missing", rule.parameter_name));
                    continue;
                }

                if let Some(value) = parameters.get(&rule.parameter_name) {
                    match &rule.rule_type {
                        ValidationType::String => {
                            if !value.is_string() {
                                errors.push(format!("Parameter '{}' must be a string", rule.parameter_name));
                            }
                        },
                        ValidationType::Number => {
                            if !value.is_number() {
                                errors.push(format!("Parameter '{}' must be a number", rule.parameter_name));
                            } else if let Some(num) = value.as_f64() {
                                if let Some(min) = rule.min_value {
                                    if num < min {
                                        errors.push(format!("Parameter '{}' must be >= {}", rule.parameter_name, min));
                                    }
                                }
                                if let Some(max) = rule.max_value {
                                    if num > max {
                                        errors.push(format!("Parameter '{}' must be <= {}", rule.parameter_name, max));
                                    }
                                }
                            }
                        },
                        ValidationType::Boolean => {
                            if !value.is_boolean() {
                                errors.push(format!("Parameter '{}' must be a boolean", rule.parameter_name));
                            }
                        },
                        ValidationType::Array => {
                            if !value.is_array() {
                                errors.push(format!("Parameter '{}' must be an array", rule.parameter_name));
                            }
                        },
                        ValidationType::Object => {
                            if !value.is_object() {
                                errors.push(format!("Parameter '{}' must be an object", rule.parameter_name));
                            }
                        },
                        ValidationType::Enum => {
                            if let Some(allowed) = &rule.allowed_values {
                                if !allowed.contains(value) {
                                    errors.push(format!("Parameter '{}' must be one of {:?}", rule.parameter_name, allowed));
                                }
                            }
                        },
                        ValidationType::Regex => {
                            // Regex validation requires regex crate - skip for now
                            warnings.push(format!("Regex validation for '{}' not implemented", rule.parameter_name));
                        },
                        ValidationType::Custom => {
                            // Custom validation would be implemented by the handler
                            warnings.push(format!("Custom validation for '{}' not implemented", rule.parameter_name));
                        }
                    }
                }
            }
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    /// Get workflow status
    pub async fn get_workflow(&self, workflow_id: Uuid) -> Option<Workflow> {
        let workflows = self.workflows.read().await;
        workflows.get(&workflow_id).cloned()
    }

    /// Cancel a workflow
    pub async fn cancel_workflow(&self, workflow_id: Uuid) -> LlmResult<()> {
        let mut workflows = self.workflows.write().await;
        if let Some(workflow) = workflows.get_mut(&workflow_id) {
            workflow.status = WorkflowStatus::Cancelled;
            info!("Workflow {} cancelled", workflow_id);
            Ok(())
        } else {
            Err(LlmError::RequestFailed(format!("Workflow {} not found", workflow_id)))
        }
    }

    /// Get all workflows
    pub async fn get_all_workflows(&self) -> Vec<Workflow> {
        let workflows = self.workflows.read().await;
        workflows.values().cloned().collect()
    }
}

impl Default for FunctionOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use chrono::Utc;

    #[test]
    fn test_step_status_variants() {
        assert_eq!(StepStatus::Pending, StepStatus::Pending);
        assert_eq!(StepStatus::Running, StepStatus::Running);
    }

    #[test]
    fn test_workflow_status_variants() {
        assert_eq!(WorkflowStatus::Created, WorkflowStatus::Created);
        assert_eq!(WorkflowStatus::Running, WorkflowStatus::Running);
    }

    #[tokio::test]
    async fn test_function_orchestrator_creation() {
        let orchestrator = FunctionOrchestrator::new();
        assert_eq!(orchestrator.max_concurrent_steps, 10);
        assert_eq!(orchestrator.step_timeout_ms, 30000);
    }

    #[test]
    fn test_validation_result_default() {
        let result = ValidationResult {
            is_valid: true,
            errors: vec![],
            warnings: vec![],
        };
        assert!(result.is_valid);
    }
}
