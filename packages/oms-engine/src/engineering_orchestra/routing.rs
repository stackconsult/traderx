use std::collections::HashMap;
use crate::engineering_orchestra::types::{AgentRole, QuestionCategory};

#[derive(Debug, Clone)]
pub struct RoutingMatrix {
    routing_rules: HashMap<QuestionCategory, Vec<AgentRole>>,
}

impl RoutingMatrix {
    pub fn new() -> Self {
        let mut routing_rules = HashMap::new();
        
        routing_rules.insert(QuestionCategory::Architecture, vec![AgentRole::SystemsArchitect]);
        routing_rules.insert(QuestionCategory::Implementation, vec![AgentRole::ImplementationEngineer]);
        routing_rules.insert(QuestionCategory::Quality, vec![AgentRole::QualityAssurance]);
        routing_rules.insert(QuestionCategory::Operations, vec![AgentRole::DevOps]);
        routing_rules.insert(QuestionCategory::Integration, vec![AgentRole::SystemsArchitect, AgentRole::ImplementationEngineer]);
        routing_rules.insert(QuestionCategory::Performance, vec![AgentRole::ImplementationEngineer, AgentRole::QualityAssurance]);
        routing_rules.insert(QuestionCategory::Security, vec![AgentRole::QualityAssurance, AgentRole::ImplementationEngineer]);
        routing_rules.insert(QuestionCategory::Testing, vec![AgentRole::QualityAssurance, AgentRole::ImplementationEngineer]);
        
        Self { routing_rules }
    }

    pub fn route_question(&self, category: &QuestionCategory) -> Vec<AgentRole> {
        self.routing_rules.get(category).cloned().unwrap_or_default()
    }
}
