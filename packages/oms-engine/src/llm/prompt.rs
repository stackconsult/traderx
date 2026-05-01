use std::collections::HashMap;
use super::{LlmRequest, LlmResult, LlmError};

pub struct PromptEngine {
    templates: HashMap<String, String>,
}

impl PromptEngine {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        
        templates.insert(
            "signal_analysis".to_string(),
            r#"Analyze the following trading signal and provide a detailed assessment:

Symbol: {{symbol}}
Direction: {{direction}}
Conviction: {{conviction}}
Max Notional: ${{max_notional}}

Please assess:
1. Risk level (Low/Medium/High)
2. Recommended position size as % of max notional
3. Key factors to monitor
4. Suggested stop-loss and take-profit levels (if applicable)

Respond in a structured format."#.to_string()
        );
        
        templates.insert(
            "risk_assessment".to_string(),
            r#"Evaluate the risk profile for the following scenario:

Agent: {{agent_id}}
Signal: {{direction}} on {{symbol}}
Conviction: {{conviction}}
Portfolio context: {{portfolio_context}}

Assess:
1. Concentration risk
2. Correlation risk with existing positions
3. Liquidity risk for {{symbol}}
4. Recommended risk mitigation actions

Provide a risk score (0-100) and reasoning."#.to_string()
        );
        
        templates.insert(
            "market_summary".to_string(),
            r#"Provide a concise market summary for {{symbol}} based on the following:

Recent signals: {{signal_count}} in last 24h
Average conviction: {{avg_conviction}}
Directional bias: {{directional_bias}}

Key observations and actionable insights:"#.to_string()
        );
        
        templates.insert(
            "default".to_string(),
            r#"Process the following trading signal and provide analysis:

{{signal_data}}

Your response should be clear, structured, and actionable."#.to_string()
        );
        
        Self { templates }
    }

    pub async fn render(&self, request: &LlmRequest) -> LlmResult<String> {
        let template_name = request.metadata
            .get("template")
            .cloned()
            .unwrap_or_else(|| "default".to_string());
        
        let template = self.templates
            .get(&template_name)
            .or_else(|| self.templates.get("default"))
            .ok_or_else(|| LlmError::InvalidResponse("No template found".to_string()))?;
        
        let mut rendered = template.clone();
        
        for (key, value) in &request.metadata {
            let placeholder = format!("{{{{{}}}}}", key);
            rendered = rendered.replace(&placeholder, value);
        }
        
        if request.metadata.get("template").is_none() {
            rendered = rendered.replace("{{signal_data}}", &request.prompt);
        }
        
        Ok(rendered)
    }

    pub fn register_template(&mut self, name: &str, template: &str) {
        self.templates.insert(name.to_string(), template.to_string());
    }

    pub fn get_templates(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }
}
