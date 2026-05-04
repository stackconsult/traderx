use std::collections::HashMap;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

// ============================================================================
// CORE TYPES AND ENUMS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentRole {
    Conductor,
    SystemsArchitect,
    ImplementationEngineer,
    QualityAssurance,
    DevOps,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuestionCategory {
    Architecture,
    Implementation,
    Quality,
    Operations,
    Integration,
    Performance,
    Security,
    Testing,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Simple,
    Medium,
    Complex,
    Expert,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

// ============================================================================
// QUESTION AND RESPONSE STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionRequest {
    pub id: Uuid,
    pub question: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: QuestionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionMetadata {
    pub category: QuestionCategory,
    pub complexity: ComplexityLevel,
    pub domain_specificity: f64,
    pub multi_agent_required: bool,
    pub priority: Priority,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub agent_role: AgentRole,
    pub question_id: Uuid,
    pub response: String,
    pub confidence: f64,
    pub reasoning: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledResponse {
    pub question_id: Uuid,
    pub primary_answer: String,
    pub supporting_insights: Vec<String>,
    pub confidence_score: f64,
    pub contributing_agents: Vec<AgentRole>,
    pub integration_timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScore {
    pub overall: f64,
    pub accuracy: f64,
    pub completeness: f64,
    pub consistency: f64,
    pub relevance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityValidatedResponse {
    pub response: CompiledResponse,
    pub quality_score: QualityScore,
    pub validation_timestamp: DateTime<Utc>,
}

// ============================================================================
// AGENT TRAIT
// ============================================================================

use crate::engineering_orchestra::error::OrchestraError;

pub trait Agent: Send + Sync + std::fmt::Debug {
    fn role(&self) -> AgentRole;
    fn capabilities(&self) -> Vec<String>;
    fn domain_expertise(&self) -> HashMap<String, f64>;
    fn process_question(&self, question: &QuestionRequest) -> Result<AgentResponse, OrchestraError>;
    fn can_handle(&self, category: &QuestionCategory) -> bool;
}
