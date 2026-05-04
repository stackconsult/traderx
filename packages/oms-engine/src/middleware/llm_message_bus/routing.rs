use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::llm::LlmError;
use crate::middleware::llm_message_bus::types::{LlmProvider, LlmRoute, RoutingCondition, RouteHealth};

/// LLM routing table for provider selection
#[derive(Debug, Clone)]
pub struct LlmRoutingTable {
    routes: HashMap<String, Vec<LlmRoute>>,
    default_provider: LlmProvider,
    load_balancer: LlmLoadBalancer,
}

#[derive(Debug, Clone)]
pub struct LlmRoute {
    pub provider: LlmProvider,
    pub model: String,
    pub weight: f64,
    pub conditions: Vec<RoutingCondition>,
    pub health_status: RouteHealth,
}

#[derive(Debug, Clone)]
pub enum RoutingCondition {
    ConvictionRange { min: f64, max: f64 },
    SymbolPattern(String),
    TimeWindow { start: u32, end: u32 },
    ModelSize { min: u64, max: u64 },
    LatencyRequirement { max_ms: u64 },
}

#[derive(Debug, Clone)]
pub enum RouteHealth {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
}

/// Load balancer for LLM providers
#[derive(Debug, Clone)]
pub struct LlmLoadBalancer {
    provider_stats: HashMap<LlmProvider, ProviderStats>,
    balancing_strategy: BalancingStrategy,
}

#[derive(Debug, Clone)]
pub struct ProviderStats {
    pub request_count: u64,
    pub success_count: u64,
    pub avg_latency_ms: f64,
    pub error_rate: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum BalancingStrategy {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    LatencyBased,
}

impl LlmRoutingTable {
    pub fn new() -> Self {
        let mut routes = HashMap::new();
        
        routes.insert("default".to_string(), vec![
            LlmRoute {
                provider: LlmProvider::Ollama,
                model: "deepseek-coder:1.3b".to_string(),
                weight: 0.6,
                conditions: vec![
                    RoutingCondition::ConvictionRange { min: 0.5, max: 1.0 },
                    RoutingCondition::LatencyRequirement { max_ms: 500 },
                ],
                health_status: RouteHealth::Healthy,
            },
            LlmRoute {
                provider: LlmProvider::OpenAI,
                model: "gpt-4".to_string(),
                weight: 0.3,
                conditions: vec![
                    RoutingCondition::ConvictionRange { min: 0.8, max: 1.0 },
                ],
                health_status: RouteHealth::Healthy,
            },
            LlmRoute {
                provider: LlmProvider::Anthropic,
                model: "claude-3".to_string(),
                weight: 0.1,
                conditions: vec![
                    RoutingCondition::ConvictionRange { min: 0.9, max: 1.0 },
                ],
                health_status: RouteHealth::Healthy,
            },
        ]);
        
        Self {
            routes,
            default_provider: LlmProvider::Ollama,
            load_balancer: LlmLoadBalancer::new(),
        }
    }
    
    pub fn select_route(&self, model: &str, provider: &LlmProvider) -> LlmResult<&LlmRoute> {
        let key = format!("{}:{:?}", model, provider);
        
        if let Some(route_list) = self.routes.get(&key) {
            Ok(self.load_balancer.select_route(route_list))
        } else {
            let default_routes = self.routes.get("default")
                .ok_or_else(|| LlmError::RouterError("No default routes available".to_string()))?;
            
            Ok(self.load_balancer.select_route(default_routes))
        }
    }
    
    pub fn update_model_status(&mut self, model: &str, status: &str) {
        for routes in self.routes.values_mut() {
            for route in routes {
                if route.model == model {
                    route.health_status = match status {
                        "ready" => RouteHealth::Healthy,
                        "loading" => RouteHealth::Degraded { reason: "Loading".to_string() },
                        _ => RouteHealth::Unhealthy { reason: status.to_string() },
                    };
                }
            }
        }
    }
}

impl LlmLoadBalancer {
    pub fn new() -> Self {
        Self {
            provider_stats: HashMap::new(),
            balancing_strategy: BalancingStrategy::WeightedRoundRobin,
        }
    }
    
    pub fn select_route<'a>(&self, routes: &'a [LlmRoute]) -> &'a LlmRoute {
        if routes.is_empty() {
            panic!("No routes available for selection");
        }
        routes.iter()
            .max_by(|a, b| a.weight.partial_cmp(&b.weight).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(&routes[0])
    }
}
