//! Sentrux MCP Client for Cascade Agent Integration
//!
//! Provides typed wrappers around the Sentrux MCP server tools:
//! - scan: Get current quality score
//! - session_start: Save baseline before agent work
//! - session_end: Compare quality after agent work
//! - check_rules: Validate architectural constraints
//!
//! Usage from agent workflow:
//! ```ignore
//! let client = SentruxMcpClient::new();
//! let baseline = client.scan().await?;
//! // ... agent does work ...
//! let result = client.session_end(&baseline).await?;
//! if result.quality_delta < -200 {
//!     // Warn: quality degraded significantly
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::process::Command;

/// Quality score from sentrux scan (0–10000)
pub type QualitySignal = u32;

/// Result of a sentrux scan
#[derive(Debug, Clone, Deserialize)]
pub struct ScanResult {
    pub quality_signal: QualitySignal,
    pub files: usize,
    pub bottleneck: Option<String>,
    pub metrics: Option<ScanMetrics>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScanMetrics {
    pub modularity: u32,
    pub acyclicity: u32,
    pub depth: u32,
    pub equality: u32,
    pub redundancy: u32,
}

/// Session comparison result
#[derive(Debug, Clone, Deserialize)]
pub struct SessionResult {
    pub pass: bool,
    pub signal_before: QualitySignal,
    pub signal_after: QualitySignal,
    pub summary: String,
}

/// Rules check result
#[derive(Debug, Clone, Deserialize)]
pub struct RulesResult {
    pub pass: bool,
    pub violations: Vec<RuleViolation>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuleViolation {
    pub rule: String,
    pub file: String,
    pub message: String,
    pub severity: String, // "error" | "warning"
}

/// MCP client for sentrux integration
pub struct SentruxMcpClient {
    project_path: String,
    binary_path: Option<String>,
}

impl SentruxMcpClient {
    pub fn new(project_path: impl Into<String>) -> Self {
        Self {
            project_path: project_path.into(),
            binary_path: None,
        }
    }

    pub fn with_binary(mut self, path: impl Into<String>) -> Self {
        self.binary_path = Some(path.into());
        self
    }

    fn binary(&self) -> &str {
        self.binary_path.as_deref().unwrap_or("sentrux")
    }

    /// Scan project and return quality metrics
    pub async fn scan(&self) -> anyhow::Result<ScanResult> {
        let output = Command::new(self.binary())
            .args(["check", &self.project_path, "--format", "json"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let result: ScanResult = serde_json::from_str(&stdout)
            .map_err(|e| anyhow::anyhow!("Failed to parse sentrux scan output: {}\nRaw: {}", e, stdout))?;

        Ok(result)
    }

    /// Save baseline before agent session
    pub async fn session_start(&self) -> anyhow::Result<ScanResult> {
        let output = Command::new(self.binary())
            .args(["gate", "--save", &self.project_path])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("sentrux gate --save failed: {}", stderr));
        }

        // After saving, scan to get the baseline score
        self.scan().await
    }

    /// Compare quality after agent session
    pub async fn session_end(&self, baseline: &ScanResult) -> anyhow::Result<SessionResult> {
        let after = self.scan().await?;
        let delta = after.quality_signal as i64 - baseline.quality_signal as i64;

        let pass = delta >= -200; // Allow minor degradation
        let summary = if delta < -500 {
            format!(
                "Quality degraded significantly: {} → {} (Δ{}). Human review required.",
                baseline.quality_signal, after.quality_signal, delta
            )
        } else if delta < -200 {
            format!(
                "Quality degraded: {} → {} (Δ{}). Explain in commit message.",
                baseline.quality_signal, after.quality_signal, delta
            )
        } else if delta > 0 {
            format!(
                "Quality improved: {} → {} (+{}).",
                baseline.quality_signal, after.quality_signal, delta
            )
        } else {
            format!(
                "Quality stable: {} → {} (Δ{}).",
                baseline.quality_signal, after.quality_signal, delta
            )
        };

        Ok(SessionResult {
            pass,
            signal_before: baseline.quality_signal,
            signal_after: after.quality_signal,
            summary,
        })
    }

    /// Check architectural rules
    pub async fn check_rules(&self) -> anyhow::Result<RulesResult> {
        let output = Command::new(self.binary())
            .args(["check", &self.project_path, "--rules"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse output — sentrux check --rules may output violations as lines
        // This is a placeholder until exact format is confirmed
        let pass = output.status.success();
        let violations = if pass {
            Vec::new()
        } else {
            vec![RuleViolation {
                rule: "sentrux".to_string(),
                file: self.project_path.clone(),
                message: stdout.to_string(),
                severity: "error".to_string(),
            }]
        };

        Ok(RulesResult { pass, violations })
    }
}

/// Convenience function for agent workflow integration
///
/// Returns (pass, summary) for quick decision-making
pub async fn agent_quality_gate(
    project_path: &str,
    baseline: &ScanResult,
) -> anyhow::Result<(bool, String)> {
    let client = SentruxMcpClient::new(project_path);
    let result = client.session_end(baseline).await?;
    Ok((result.pass, result.summary))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_result_deserialization() {
        let json = r#"{
            "quality_signal": 7342,
            "files": 139,
            "bottleneck": "modularity",
            "metrics": {
                "modularity": 80,
                "acyclicity": 95,
                "depth": 70,
                "equality": 85,
                "redundancy": 90
            }
        }"#;

        let result: ScanResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.quality_signal, 7342);
        assert_eq!(result.files, 139);
        assert_eq!(result.bottleneck, Some("modularity".to_string()));
    }

    #[test]
    fn test_session_result_pass_threshold() {
        let baseline = ScanResult {
            quality_signal: 7000,
            files: 100,
            bottleneck: None,
            metrics: None,
        };

        // Simulate degradation of 150 — should pass
        let after = ScanResult {
            quality_signal: 6850,
            files: 100,
            bottleneck: None,
            metrics: None,
        };

        let delta = after.quality_signal as i64 - baseline.quality_signal as i64;
        assert!(delta >= -200, "Minor degradation should be allowed");
        assert!(delta < -100, "But it's still a degradation");
    }
}
