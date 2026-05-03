use super::{AgentSignal, LlmRequest, LlmResult};
use uuid::Uuid;
use std::collections::HashMap;

pub struct AgentRouter {
    agent_capabilities: HashMap<String, Vec<String>>,
}

impl AgentRouter {
    pub fn new() -> Self {
        let mut capabilities = HashMap::new();
        capabilities.insert("llm_analyst".to_string(), vec![
            "signal_analysis".to_string(),
            "risk_assessment".to_string(),
            "market_summary".to_string(),
        ]);
        capabilities.insert("ml_predictor".to_string(), vec![
            "price_prediction".to_string(),
            "volatility_forecast".to_string(),
            "trend_detection".to_string(),
        ]);
        capabilities.insert("neural_executor".to_string(), vec![
            "pattern_recognition".to_string(),
            "signal_classification".to_string(),
            "anomaly_detection".to_string(),
        ]);
        
        Self { agent_capabilities: capabilities }
    }

    pub async fn route_signal(&self, signal: &AgentSignal) -> LlmResult<LlmRequest> {
        let (agent_type, template, max_tokens) = self.determine_routing(signal);
        
        let mut metadata = HashMap::new();
        metadata.insert("template".to_string(), template.clone());
        metadata.insert("symbol".to_string(), signal.symbol.clone());
        metadata.insert("direction".to_string(), signal.direction.clone());
        metadata.insert("conviction".to_string(), format!("{:.2}", signal.conviction));
        metadata.insert("max_notional".to_string(), format!("{:.2}", signal.max_notional_usd));
        metadata.insert("agent_id".to_string(), signal.agent_id.clone());
        metadata.insert("agent_type".to_string(), agent_type);
        metadata.insert("ttl_ms".to_string(), signal.ttl_ms.to_string());
        
        let prompt = self.build_prompt(signal, &template);
        
        Ok(LlmRequest {
            request_id: Uuid::new_v4(),
            prompt,
            model: self.select_model(signal.conviction),
            max_tokens,
            temperature: self.select_temperature(signal.conviction),
            metadata,
        })
    }

    fn determine_routing(&self, signal: &AgentSignal) -> (String, String, u32) {
        if signal.conviction >= 0.9 {
            ("neural_executor".to_string(), "signal_analysis".to_string(), 1024)
        } else if signal.conviction >= 0.6 {
            ("ml_predictor".to_string(), "signal_analysis".to_string(), 2048)
        } else {
            ("llm_analyst".to_string(), "risk_assessment".to_string(), 4096)
        }
    }

    fn select_model(&self, conviction: f64) -> String {
        if conviction >= 0.9 {
            "gpt-4".to_string()
        } else if conviction >= 0.7 {
            "gpt-4".to_string()
        } else {
            "gpt-3.5-turbo".to_string()
        }
    }

    fn select_temperature(&self, conviction: f64) -> f32 {
        if conviction >= 0.9 {
            0.2
        } else if conviction >= 0.7 {
            0.5
        } else {
            0.8
        }
    }

    fn build_prompt(&self, signal: &AgentSignal, template: &str) -> String {
        format!(
            "Trading Signal Analysis\n\nSymbol: {}\nDirection: {}\nConviction: {:.2}\nMax Notional: ${:.2}\nTTL: {}ms\n\nMeta: {}\n\nTemplate: {}",
            signal.symbol,
            signal.direction,
            signal.conviction,
            signal.max_notional_usd,
            signal.ttl_ms,
            signal.meta.to_string(),
            template
        )
    }

    pub fn get_agent_capabilities(&self) -> &HashMap<String, Vec<String>> {
        &self.agent_capabilities
    }

    pub fn get_agents_for_capability(&self, capability: &str) -> Vec<String> {
        self.agent_capabilities
            .iter()
            .filter(|(_, caps)| caps.contains(&capability.to_string()))
            .map(|(name, _)| name.clone())
            .collect()
    }
}
