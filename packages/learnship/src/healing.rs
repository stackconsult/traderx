//! Self-healing engine for automatic component recovery.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Health check definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub component: String,
    pub check_type: CheckType,
    pub endpoint: String,
    pub timeout_secs: u64,
    pub expected_status: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckType {
    Http,
    Tcp,
    Grpc,
    Custom,
}

/// Healing action to take when a component fails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealingAction {
    Restart,
    ScaleUp,
    ScaleDown,
    Failover,
    Disable,
    Custom(String),
}

/// Healing rule mapping failures to actions.
#[derive(Debug, Clone)]
pub struct HealingRule {
    pub component_pattern: String,
    pub error_patterns: Vec<String>,
    pub severity_threshold: crate::agents::HealthSeverity,
    pub action: HealingAction,
    pub cooldown_mins: u64,
    pub max_attempts: u32,
}

/// Self-healing engine.
pub struct HealingEngine {
    rules: HashMap<String, HealingRule>,
    last_actions: HashMap<String, chrono::DateTime<chrono::Utc>>,
    attempt_counts: HashMap<String, u32>,
}

impl HealingEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            rules: HashMap::new(),
            last_actions: HashMap::new(),
            attempt_counts: HashMap::new(),
        };
        
        // Initialize default healing rules
        engine.setup_default_rules();
        engine
    }

    /// Determine healing action for a component failure.
    pub async fn determine_action(
        &self,
        component: &str,
        error: &str,
        severity: &crate::agents::HealthSeverity,
    ) -> HealingAction {
        info!("Determining healing action for {}: {}", component, error);

        // Find matching rule
        for rule in self.rules.values() {
            if self.component_matches(component, &rule.component_pattern) {
                if self.error_matches(error, &rule.error_patterns) {
                    if self.severity_meets_threshold(severity, &rule.severity_threshold) {
                        // Check cooldown
                        if self.is_in_cooldown(component, &rule) {
                            debug!("Component {} in cooldown, skipping action", component);
                            return HealingAction::Custom("in_cooldown".to_string());
                        }

                        // Check max attempts
                        if self.exceeded_max_attempts(component, &rule) {
                            warn!("Component {} exceeded max attempts, escalating", component);
                            return HealingAction::Failover;
                        }

                        info!("Applying healing rule: {:?}", rule.action);
                        return rule.action.clone();
                    }
                }
            }
        }

        // Default action based on severity
        match severity {
            crate::agents::HealthSeverity::Critical => HealingAction::Restart,
            crate::agents::HealthSeverity::High => HealingAction::Restart,
            crate::agents::HealthSeverity::Medium => HealingAction::ScaleUp,
            crate::agents::HealthSeverity::Low => HealingAction::Custom("log_only".to_string()),
        }
    }

    /// Execute a healing action.
    pub async fn execute_action(
        &mut self,
        component: &str,
        action: &HealingAction,
    ) -> anyhow::Result<()> {
        info!("Executing healing action {:?} for component {}", action, component);

        match action {
            HealingAction::Restart => self.restart_component(component).await?,
            HealingAction::ScaleUp => self.scale_component(component, 2).await?,
            HealingAction::ScaleDown => self.scale_component(component, 0.5).await?,
            HealingAction::Failover => self.failover_component(component).await?,
            HealingAction::Disable => self.disable_component(component).await?,
            HealingAction::Custom(cmd) => self.execute_custom_command(component, cmd).await?,
        }

        // Update tracking
        self.last_actions.insert(
            component.to_string(),
            chrono::Utc::now(),
        );
        
        let count = self.attempt_counts.entry(component.to_string()).or_insert(0);
        *count += 1;

        Ok(())
    }

    /// Add a custom healing rule.
    pub fn add_rule(&mut self, name: String, rule: HealingRule) {
        self.rules.insert(name, rule);
    }

    /// Get healing statistics.
    pub fn get_stats(&self) -> HealingStats {
        HealingStats {
            total_rules: self.rules.len(),
            active_cooldowns: self.count_active_cooldowns(),
            components_with_attempts: self.attempt_counts.len(),
        }
    }

    // Private methods

    fn setup_default_rules(&mut self) {
        // Model server rules
        self.rules.insert(
            "model_server_restart".to_string(),
            HealingRule {
                component_pattern: "model-server.*".to_string(),
                error_patterns: vec![
                    "connection refused".to_string(),
                    "timeout".to_string(),
                    "500".to_string(),
                ],
                severity_threshold: crate::agents::HealthSeverity::Medium,
                action: HealingAction::Restart,
                cooldown_mins: 5,
                max_attempts: 3,
            },
        );

        // QuestDB rules
        self.rules.insert(
            "questdb_restart".to_string(),
            HealingRule {
                component_pattern: "questdb.*".to_string(),
                error_patterns: vec![
                    "connection refused".to_string(),
                    "disk full".to_string(),
                ],
                severity_threshold: crate::agents::HealthSeverity::High,
                action: HealingAction::Restart,
                cooldown_mins: 10,
                max_attempts: 2,
            },
        );

        // Redis/Feature store rules
        self.rules.insert(
            "redis_scale".to_string(),
            HealingRule {
                component_pattern: "redis.*".to_string(),
                error_patterns: vec![
                    "memory limit".to_string(),
                    "slow response".to_string(),
                ],
                severity_threshold: crate::agents::HealthSeverity::Medium,
                action: HealingAction::ScaleUp,
                cooldown_mins: 15,
                max_attempts: 3,
            },
        );

        // OMS rules
        self.rules.insert(
            "oms_failover".to_string(),
            HealingRule {
                component_pattern: "oms.*".to_string(),
                error_patterns: vec![
                    "order rejected".to_string(),
                    "position mismatch".to_string(),
                ],
                severity_threshold: crate::agents::HealthSeverity::Critical,
                action: HealingAction::Failover,
                cooldown_mins: 5,
                max_attempts: 1,
            },
        );
    }

    fn component_matches(&self, component: &str, pattern: &str) -> bool {
        // Simple pattern matching - in production use regex
        if pattern.contains('*') {
            let base = pattern.replace('*', "");
            component.contains(&base)
        } else {
            component == pattern
        }
    }

    fn error_matches(&self, error: &str, patterns: &[String]) -> bool {
        patterns.iter().any(|pattern| error.to_lowercase().contains(&pattern.to_lowercase()))
    }

    fn severity_meets_threshold(
        &self,
        severity: &crate::agents::HealthSeverity,
        threshold: &crate::agents::HealthSeverity,
    ) -> bool {
        use crate::agents::HealthSeverity;
        match (severity, threshold) {
            (HealthSeverity::Critical, _) => true,
            (HealthSeverity::High, HealthSeverity::Critical) => false,
            (HealthSeverity::High, _) => true,
            (HealthSeverity::Medium, HealthSeverity::Critical | HealthSeverity::High) => false,
            (HealthSeverity::Medium, _) => true,
            (HealthSeverity::Low, HealthSeverity::Low) => true,
            (HealthSeverity::Low, _) => false,
        }
    }

    fn is_in_cooldown(&self, component: &str, rule: &HealingRule) -> bool {
        if let Some(&last_action) = self.last_actions.get(component) {
            let cooldown_end = last_action + chrono::Duration::minutes(rule.cooldown_mins as i64);
            chrono::Utc::now() < cooldown_end
        } else {
            false
        }
    }

    fn exceeded_max_attempts(&self, component: &str, rule: &HealingRule) -> bool {
        self.attempt_counts
            .get(component)
            .copied()
            .unwrap_or(0) >= rule.max_attempts
    }

    async fn restart_component(&self, component: &str) -> anyhow::Result<()> {
        warn!("Restarting component: {}", component);
        
        // In production, this would call the appropriate service manager
        // e.g., Kubernetes API, Docker API, systemd, etc.
        match component {
            c if c.starts_with("model-server") => {
                // Restart model server deployment
                self.kubectl_restart("deployment", component).await?;
            }
            c if c.starts_with("questdb") => {
                // Restart QuestDB pod
                self.kubectl_restart("pod", component).await?;
            }
            _ => {
                // Generic restart
                self.systemctl_restart(component).await?;
            }
        }
        
        Ok(())
    }

    async fn scale_component(&self, component: &str, factor: f64) -> anyhow::Result<()> {
        info!("Scaling component {} by factor {}", component, factor);
        
        // Scale Kubernetes deployment
        let current_replicas = self.get_current_replicas(component).await?;
        let new_replicas = ((current_replicas as f64) * factor).max(1.0) as i32;
        
        self.kubectl_scale(component, new_replicas).await?;
        
        Ok(())
    }

    async fn failover_component(&self, component: &str) -> anyhow::Result<()> {
        error!("Initiating failover for component: {}", component);
        
        // Switch to backup/standby instance
        match component {
            c if c.starts_with("oms") => {
                // Switch to backup OMS
                self.switch_oms_primary().await?;
            }
            _ => {
                // Generic failover
                self.activate_backup(component).await?;
            }
        }
        
        Ok(())
    }

    async fn disable_component(&self, component: &str) -> anyhow::Result<()> {
        warn!("Disabling component: {}", component);
        
        // Scale to zero or mark as disabled
        self.kubectl_scale(component, 0).await?;
        
        Ok(())
    }

    async fn execute_custom_command(&self, component: &str, command: &str) -> anyhow::Result<()> {
        debug!("Executing custom command '{}' for component {}", command, component);
        
        match command {
            "log_only" => {
                // Just log, no action
                info!("Custom action: log_only for {}", component);
            }
            "clear_cache" => {
                self.clear_component_cache(component).await?;
            }
            _ => {
                warn!("Unknown custom command: {}", command);
            }
        }
        
        Ok(())
    }

    // Implementation helpers (placeholders)

    async fn kubectl_restart(&self, resource_type: &str, name: &str) -> anyhow::Result<()> {
        let output = tokio::process::Command::new("kubectl")
            .args(["rollout", "restart", resource_type, name])
            .output()
            .await?;
        
        if !output.status.success() {
            anyhow::bail!("kubectl restart failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }

    async fn systemctl_restart(&self, service: &str) -> anyhow::Result<()> {
        let output = tokio::process::Command::new("sudo")
            .args(["systemctl", "restart", service])
            .output()
            .await?;
        
        if !output.status.success() {
            anyhow::bail!("systemctl restart failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }

    async fn get_current_replicas(&self, deployment: &str) -> anyhow::Result<i32> {
        let output = tokio::process::Command::new("kubectl")
            .args(["get", "deployment", deployment, "-o", "jsonpath='{.spec.replicas}'"])
            .output()
            .await?;
        
        if !output.status.success() {
            anyhow::bail!("kubectl get replicas failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        let replicas_str = String::from_utf8_lossy(&output.stdout).trim_matches('\'').to_string();
        Ok(replicas_str.parse::<i32>().unwrap_or(1))
    }

    async fn kubectl_scale(&self, deployment: &str, replicas: i32) -> anyhow::Result<()> {
        let output = tokio::process::Command::new("kubectl")
            .args(["scale", "deployment", deployment, &format!("--replicas={}", replicas)])
            .output()
            .await?;
        
        if !output.status.success() {
            anyhow::bail!("kubectl scale failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }

    async fn switch_oms_primary(&self) -> anyhow::Result<()> {
        // Update service selector to point to backup
        info!("Switching OMS primary to backup instance");
        Ok(())
    }

    async fn activate_backup(&self, component: &str) -> anyhow::Result<()> {
        info!("Activating backup for component: {}", component);
        Ok(())
    }

    async fn clear_component_cache(&self, component: &str) -> anyhow::Result<()> {
        info!("Clearing cache for component: {}", component);
        Ok(())
    }

    fn count_active_cooldowns(&self) -> usize {
        let now = chrono::Utc::now();
        self.last_actions
            .values()
            .filter(|&&ts| {
                // Check if any rule would still be in cooldown
                self.rules.values().any(|rule| {
                    ts + chrono::Duration::minutes(rule.cooldown_mins as i64) > now
                })
            })
            .count()
    }
}

/// Healing engine statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingStats {
    pub total_rules: usize,
    pub active_cooldowns: usize,
    pub components_with_attempts: usize,
}
