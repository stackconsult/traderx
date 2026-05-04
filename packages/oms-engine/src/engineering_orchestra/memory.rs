use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::engineering_orchestra::types::{AgentRole, AgentResponse};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: Uuid,
    pub memory_type: MemoryType,
    pub content: String,
    pub metadata: MemoryMetadata,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryType {
    AgentQA,
    SkillExecution,
    WorkflowPhase,
    LessonLearned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetadata {
    pub agent_role: Option<AgentRole>,
    pub question_category: Option<String>,
    pub confidence: Option<f64>,
    pub tags: Vec<String>,
    pub related_files: Vec<String>,
}

#[derive(Debug)]
pub struct MemoryLayer {
    pub memories: Vec<MemoryRecord>,
    pub user_id: String,
}

impl MemoryLayer {
    pub fn new(user_id: String) -> Self {
        Self {
            memories: Vec::new(),
            user_id,
        }
    }

    pub fn store_agent_qa(&mut self, question: &str, response: &AgentResponse) -> Uuid {
        let record = MemoryRecord {
            id: Uuid::new_v4(),
            memory_type: MemoryType::AgentQA,
            content: format!("Q: {}\n\nA: {}", question, response.response),
            metadata: MemoryMetadata {
                agent_role: Some(response.agent_role.clone()),
                question_category: None,
                confidence: Some(response.confidence),
                tags: vec!["qa".to_string(), format!("{:?}", response.agent_role)],
                related_files: Vec::new(),
            },
            timestamp: Utc::now(),
        };
        let id = record.id;
        self.memories.push(record);
        id
    }

    pub fn retrieve_relevant_qa(&self, query: &str, limit: usize) -> Vec<&MemoryRecord> {
        // Simple keyword-based retrieval (in production, use vector similarity)
        let query_lower = query.to_lowercase();
        let keywords: Vec<&str> = query_lower.split_whitespace().collect();
        
        self.memories
            .iter()
            .filter(|m| {
                if matches!(m.memory_type, MemoryType::AgentQA) {
                    let content_lower = m.content.to_lowercase();
                    keywords.iter().any(|k| content_lower.contains(k))
                } else {
                    false
                }
            })
            .take(limit)
            .collect()
    }

    pub fn store_lesson(&mut self, lesson: &str, tags: Vec<String>) -> Uuid {
        let record = MemoryRecord {
            id: Uuid::new_v4(),
            memory_type: MemoryType::LessonLearned,
            content: lesson.to_string(),
            metadata: MemoryMetadata {
                agent_role: None,
                question_category: None,
                confidence: None,
                tags,
                related_files: Vec::new(),
            },
            timestamp: Utc::now(),
        };
        let id = record.id;
        self.memories.push(record);
        id
    }
}
