//! Engineering Agent Orchestra Q&A System
//!
//! Deterministic multi-agent system for engineering Q&A resolution
//! with role specialization and intelligent routing
//!
//! ## Mem0 Memory Integration
//! This system integrates mem0 for persistent knowledge building:
//! - Stores agent Q&A interactions with context
//! - Retrieves relevant past experiences for similar questions
//! - Builds knowledge base from agent interactions over time
//! - Enables guided knowledge building across sessions

mod memory;
mod types;
mod agents;
mod routing;
mod quality;
mod error;

pub use memory::{MemoryRecord, MemoryLayer, MemoryMetadata, MemoryType};
pub use types::{
    AgentRole, QuestionCategory, ComplexityLevel, Priority,
    QuestionRequest, QuestionMetadata, AgentResponse,
    CompiledResponse, QualityScore, QualityValidatedResponse,
    Agent
};
pub use agents::{
    ConductorAgent, SystemsArchitectAgent, ImplementationEngineerAgent,
    QualityAssuranceAgent, DevOpsAgent
};
pub use routing::RoutingMatrix;
pub use quality::{QualityFramework, AccuracyValidator, CompletenessChecker,
                ConsistencyValidator, RelevanceScorer};
pub use error::OrchestraError;

use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// ORCHESTRA ENGINE
// ============================================================================

#[derive(Debug)]
pub struct EngineeringOrchestra {
    conductor: ConductorAgent,
    agents: HashMap<AgentRole, Box<dyn Agent>>,
    quality_framework: QualityFramework,
    memory_layer: MemoryLayer,
}

impl EngineeringOrchestra {
    pub fn new() -> Self {
        let mut agents: HashMap<AgentRole, Box<dyn Agent>> = HashMap::new();
        
        agents.insert(AgentRole::Conductor, Box::new(ConductorAgent::new()));
        agents.insert(AgentRole::SystemsArchitect, Box::new(SystemsArchitectAgent::new()));
        agents.insert(AgentRole::ImplementationEngineer, Box::new(ImplementationEngineerAgent::new()));
        agents.insert(AgentRole::QualityAssurance, Box::new(QualityAssuranceAgent::new()));
        agents.insert(AgentRole::DevOps, Box::new(DevOpsAgent::new()));
        
        Self {
            conductor: ConductorAgent::new(),
            agents,
            quality_framework: QualityFramework::new(),
            memory_layer: MemoryLayer::new("traderx".to_string()),
        }
    }

    pub async fn process_question(&mut self, question: String) -> Result<QualityValidatedResponse, OrchestraError> {
        // Step 0: Retrieve relevant memories from past sessions
        let relevant_memories = self.memory_layer.retrieve_relevant_qa(&question, 3);
        if !relevant_memories.is_empty() {
            println!("\n🧠 Retrieved {} relevant memories from past sessions:", relevant_memories.len());
            for (i, mem) in relevant_memories.iter().enumerate() {
                println!("   {}. {} (tags: {:?})", i + 1, 
                    mem.content.lines().next().unwrap_or(&mem.content[..50]),
                    mem.metadata.tags);
            }
        }

        // Step 1: Create question request
        let request = QuestionRequest {
            id: Uuid::new_v4(),
            question: question.clone(),
            timestamp: chrono::Utc::now(),
            metadata: QuestionMetadata {
                category: QuestionCategory::Integration, // Will be updated by conductor
                complexity: ComplexityLevel::Medium,
                domain_specificity: 0.5,
                multi_agent_required: false,
                priority: Priority::Medium,
                keywords: Vec::new(),
            },
        };

        // Step 2: Conductor analysis
        let _conductor_response = self.conductor.process_question(&request)?;
        let metadata = self.conductor.classify_question(&request.question);
        let routing = self.conductor.route_to_agents(&metadata);

        // Step 3: Execute agents
        let mut agent_responses = Vec::new();
        
        for agent_role in &routing {
            if let Some(agent) = self.agents.get(agent_role) {
                let response = agent.process_question(&request)?;
                
                // Store agent Q&A in memory
                self.memory_layer.store_agent_qa(&request.question, &response);
                
                agent_responses.push(response);
            }
        }

        // Step 4: Integrate responses
        let compiled_response = self.integrate_responses(request.id, agent_responses)?;

        // Step 5: Quality validation
        let validated_response = self.quality_framework.validate_response(&compiled_response)?;

        // Step 6: Store lesson learned from this interaction
        let lesson = format!(
            "Question category: {:?}, Contributing agents: {:?}, Quality score: {:.2}",
            metadata.category,
            validated_response.response.contributing_agents,
            validated_response.quality_score.overall
        );
        let tags = vec![
            format!("{:?}", metadata.category),
            "orchestra_qa".to_string()
        ];
        self.memory_layer.store_lesson(&lesson, tags);

        Ok(validated_response)
    }

    fn integrate_responses(&self, question_id: Uuid, responses: Vec<AgentResponse>) -> Result<CompiledResponse, OrchestraError> {
        if responses.is_empty() {
            return Err(OrchestraError::IntegrationFailed("No agent responses to integrate".to_string()));
        }

        let primary_response = &responses[0];
        let mut supporting_insights = Vec::new();
        let mut contributing_agents = Vec::new();

        for response in &responses {
            if response.agent_role != primary_response.agent_role {
                supporting_insights.push(response.response.clone());
            }
            contributing_agents.push(response.agent_role.clone());
        }

        let confidence_score = responses.iter()
            .map(|r| r.confidence)
            .sum::<f64>() / responses.len() as f64;

        Ok(CompiledResponse {
            question_id,
            primary_answer: primary_response.response.clone(),
            supporting_insights,
            confidence_score,
            contributing_agents,
            integration_timestamp: chrono::Utc::now(),
        })
    }
}

// ============================================================================
// DEMONSTRATION FUNCTION
// ============================================================================

pub async fn demonstrate_orchestra() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎭 Engineering Agent Orchestra Q&A Demonstration");
    println!("{}", "=".repeat(50));
    println!("🧠 Mem0 Memory Layer: ENABLED");
    println!("   - Stores agent Q&A interactions");
    println!("   - Retrieves relevant past experiences");
    println!("   - Builds knowledge base across sessions");
    println!("{}", "=".repeat(50));
    
    let mut orchestra = EngineeringOrchestra::new();
    
    let test_questions = vec![
        "How should I design a microservices architecture for a trading system?".to_string(),
        "What's the best way to implement a REST API in Rust?".to_string(),
        "How do I set up comprehensive testing for my application?".to_string(),
        "What deployment strategy should I use for a high-frequency trading system?".to_string(),
        "How can I optimize the performance of my database queries?".to_string(),
    ];

    for (i, question) in test_questions.iter().enumerate() {
        println!("\n📝 Question {}: {}", i + 1, question);
        println!("{}", "-".repeat(40));
        
        match orchestra.process_question(question.clone()).await {
            Ok(validated_response) => {
                println!("✅ Primary Answer:");
                println!("   {}", validated_response.response.primary_answer);
                
                if !validated_response.response.supporting_insights.is_empty() {
                    println!("\n🔍 Supporting Insights:");
                    for (j, insight) in validated_response.response.supporting_insights.iter().enumerate() {
                        println!("   {}. {}", j + 1, insight);
                    }
                }
                
                println!("\n📊 Quality Metrics:");
                println!("   Overall Score: {:.2}", validated_response.quality_score.overall);
                println!("   Accuracy: {:.2}", validated_response.quality_score.accuracy);
                println!("   Completeness: {:.2}", validated_response.quality_score.completeness);
                println!("   Consistency: {:.2}", validated_response.quality_score.consistency);
                println!("   Relevance: {:.2}", validated_response.quality_score.relevance);
                
                println!("\n👥 Contributing Agents: {:?}", validated_response.response.contributing_agents);
                println!("⏱️  Processing Time: {:?}", validated_response.response.integration_timestamp);
                println!("💾 Stored in memory: {}", orchestra.memory_layer.memories.len() > 0);
            }
            Err(e) => {
                println!("❌ Error processing question: {}", e);
            }
        }
        
        println!("\n{}", "=".repeat(50));
    }

    println!("\n🎉 Orchestra demonstration completed!");
    println!("🧠 Total memories stored: {}", orchestra.memory_layer.memories.len());
    Ok(())
}

// ============================================================================
// MAIN FUNCTION FOR EXECUTION
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    demonstrate_orchestra().await
}
