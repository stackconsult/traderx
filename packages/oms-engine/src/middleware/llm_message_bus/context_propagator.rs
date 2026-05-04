use crate::llm::LlmResult;
use crate::middleware::llm_message_bus::types::{
    LlmMessage, LlmMessageType, LlmProvider, MessageContext,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Context propagator for maintaining context across message flow
pub struct ContextPropagator {
    pub(crate) context_store: Arc<RwLock<HashMap<Uuid, MessageContext>>>,
    propagation_rules: Vec<PropagationRule>,
    cleanup_interval: Duration,
    context_ttl: Duration,
}

#[derive(Debug, Clone)]
pub struct PropagationRule {
    pub source_key: String,
    pub target_keys: Vec<String>,
    pub condition: PropagationCondition,
}

#[derive(Debug, Clone)]
pub enum PropagationCondition {
    Always,
    IfPresent(String),
    IfValue(String),
    IfProvider(LlmProvider),
}

impl ContextPropagator {
    pub fn new() -> Self {
        Self {
            context_store: Arc::new(RwLock::new(HashMap::new())),
            propagation_rules: vec![
                PropagationRule {
                    source_key: "symbol".to_string(),
                    target_keys: vec!["trading_symbol".to_string(), "market_symbol".to_string()],
                    condition: PropagationCondition::Always,
                },
                PropagationRule {
                    source_key: "user_id".to_string(),
                    target_keys: vec!["requester_id".to_string(), "session_user".to_string()],
                    condition: PropagationCondition::Always,
                },
            ],
            cleanup_interval: Duration::from_secs(300),
            context_ttl: Duration::from_secs(3600),
        }
    }

    pub async fn propagate_context(&self, message: LlmMessage) -> LlmResult<LlmMessage> {
        let mut enriched_message = message;

        for rule in &self.propagation_rules {
            if self.should_apply_rule(&rule, &enriched_message) {
                self.apply_propagation_rule(&mut enriched_message, &rule)
                    .await?;
            }
        }

        Ok(enriched_message)
    }

    pub async fn update_context(
        &self,
        correlation_id: Uuid,
        key: String,
        value: serde_json::Value,
    ) -> LlmResult<()> {
        let mut store = self.context_store.write().await;
        let context = store
            .entry(correlation_id)
            .or_insert_with(|| MessageContext {
                trading_context: None,
                user_context: None,
                system_context: None,
                propagation_chain: Vec::new(),
            });

        match key.as_str() {
            "symbol" | "direction" | "conviction" | "max_notional" => {
                // Update trading context
            }
            "user_id" | "session_id" => {
                // Update user context
            }
            _ => {
                context.propagation_chain.push(format!("{}:{}", key, value));
            }
        }

        Ok(())
    }

    pub async fn update_context_from_response(&self, message: &LlmMessage) -> LlmResult<()> {
        if let crate::middleware::llm_message_bus::types::LlmMessagePayload::Response(response) =
            &message.payload
        {
            for (key, value) in &response.metadata {
                self.update_context(
                    message.correlation_id,
                    key.clone(),
                    serde_json::Value::String(value.clone()),
                )
                .await?;
            }
        }

        Ok(())
    }

    fn should_apply_rule(&self, rule: &PropagationRule, message: &LlmMessage) -> bool {
        match &rule.condition {
            PropagationCondition::Always => true,
            PropagationCondition::IfPresent(_key) => false,
            PropagationCondition::IfValue(_value) => false,
            PropagationCondition::IfProvider(provider) => {
                matches!(&message.message_type, LlmMessageType::LlmRequest { provider: p, .. } if p == provider)
            }
        }
    }

    async fn apply_propagation_rule(
        &self,
        _message: &mut LlmMessage,
        _rule: &PropagationRule,
    ) -> LlmResult<()> {
        Ok(())
    }
}
