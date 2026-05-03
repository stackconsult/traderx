use std::collections::HashMap;
use tokio::sync::RwLock;
use super::LlmResult;

#[derive(Clone)]
pub struct ContextEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub role: String,
    pub content: String,
    pub token_count: usize,
}

pub struct ContextManager {
    conversations: RwLock<HashMap<String, Vec<ContextEntry>>>,
    max_context_length: usize,
    max_conversation_age_hours: u64,
}

impl ContextManager {
    pub fn new(max_context_length: usize, max_conversation_age_hours: u64) -> Self {
        Self {
            conversations: RwLock::new(HashMap::new()),
            max_context_length,
            max_conversation_age_hours,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(4096, 24)
    }

    pub async fn enrich(&self, prompt: &str, agent_id: &str) -> LlmResult<String> {
        let history = self.get_recent_history(agent_id).await;
        
        if history.is_empty() {
            return Ok(prompt.to_string());
        }
        
        let mut context_parts = Vec::new();
        context_parts.push("--- PREVIOUS CONTEXT ---".to_string());
        
        let _total_tokens: usize = history.iter().map(|e| e.token_count).sum();
        let mut included_tokens = 0;
        
        for entry in history.iter().rev() {
            if included_tokens + entry.token_count > self.max_context_length {
                break;
            }
            included_tokens += entry.token_count;
            context_parts.push(format!("[{}] {}: {}", 
                entry.timestamp.format("%H:%M:%S"),
                entry.role,
                entry.content.chars().take(200).collect::<String>()
            ));
        }
        
        context_parts.push("--- CURRENT REQUEST ---".to_string());
        context_parts.push(prompt.to_string());
        
        Ok(context_parts.join("\n\n"))
    }

    pub async fn store_interaction(
        &self,
        agent_id: &str,
        prompt: &str,
        response: &str,
    ) -> LlmResult<()> {
        let mut conversations = self.conversations.write().await;
        
        let prompt_entry = ContextEntry {
            timestamp: chrono::Utc::now(),
            role: "user".to_string(),
            content: prompt.to_string(),
            token_count: self.estimate_tokens(prompt),
        };
        
        let response_entry = ContextEntry {
            timestamp: chrono::Utc::now(),
            role: "assistant".to_string(),
            content: response.to_string(),
            token_count: self.estimate_tokens(response),
        };
        
        conversations
            .entry(agent_id.to_string())
            .or_insert_with(Vec::new)
            .push(prompt_entry);
        
        conversations
            .entry(agent_id.to_string())
            .or_insert_with(Vec::new)
            .push(response_entry);
        
        self.prune_old_entries(agent_id, &mut conversations);
        
        Ok(())
    }

    pub async fn get_conversation(&self, agent_id: &str) -> Vec<ContextEntry> {
        let conversations = self.conversations.read().await;
        conversations
            .get(agent_id)
            .cloned()
            .unwrap_or_default()
    }

    pub async fn clear_conversation(&self, agent_id: &str) {
        let mut conversations = self.conversations.write().await;
        conversations.remove(agent_id);
    }

    pub async fn get_all_agent_ids(&self) -> Vec<String> {
        let conversations = self.conversations.read().await;
        conversations.keys().cloned().collect()
    }

    async fn get_recent_history(&self, agent_id: &str) -> Vec<ContextEntry> {
        let conversations = self.conversations.read().await;
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(self.max_conversation_age_hours as i64);
        
        conversations
            .get(agent_id)
            .map(|entries| {
                entries
                    .iter()
                    .filter(|e| e.timestamp > cutoff)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    fn estimate_tokens(&self, text: &str) -> usize {
        text.split_whitespace().count() + text.chars().count() / 6
    }

    fn prune_old_entries(&self, agent_id: &str, conversations: &mut HashMap<String, Vec<ContextEntry>>) {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(self.max_conversation_age_hours as i64);
        
        if let Some(entries) = conversations.get_mut(agent_id) {
            entries.retain(|e| e.timestamp > cutoff);
        }
        
        if let Some(entries) = conversations.get_mut(agent_id) {
            let mut total_tokens: usize = entries.iter().map(|e| e.token_count).sum();
            while total_tokens > self.max_context_length && entries.len() > 2 {
                let removed_count = entries.drain(0..2).next().map(|e| e.token_count).unwrap_or(0);
                total_tokens -= removed_count;
            }
        }
    }
}
