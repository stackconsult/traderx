//! # traderx-mem0-types
//!
//! Shared types for mem0 memory imprint telemetry across all TraderX crates
//! and workspaces. Provides a unified schema for event-sourced mem0 operations
//! regardless of backend (Redis journal, Postgres ledger, or external vector DB).
//!
//! ## Design Principles
//!
//! - **Backend-agnostic**: Types contain no backend-specific fields (no Redis keys, no Postgres row IDs).
//! - **Serde-ready**: All types derive `Serialize`/`Deserialize` for JSON/JSONB/Protobuf use.
//! - **Deterministic IDs**: All mem0 events carry a stable `memory_id: Uuid` for deduplication across sync bridges.
//! - **Session-scoped**: `session_id` ties mem0 events to audit/orchestrate sessions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single mem0 memory imprint — the atomic unit of persistent agent knowledge.
///
/// Stored by journal backends (Redis, Postgres, SQLite) as JSONB/JSON.
/// Retrieved by session_id for cross-session pattern analysis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mem0MemoryImprint {
    /// Unique identifier for this mem0 record. Used for deduplication across sync bridges.
    pub memory_id: Uuid,
    /// Semantic type: `session_outcome`, `security_event`, `guard_decision`, `error_pattern`, `skill_evolution`, etc.
    pub memory_type: String,
    /// Human- or LLM-readable content. Max 100KB (truncated by writers if exceeded).
    pub content: String,
    /// Agent role that produced this memory (e.g. `audit`, `conductor`, `risk_validator`).
    pub agent_role: Option<String>,
    /// Sub-category for fine-grained filtering (e.g. `goal_mismatch`, `tripwire_abort`).
    pub category: Option<String>,
    /// Confidence score [0.0, 1.0]. Optional — not all memories have probabilistic scores.
    pub confidence: Option<f64>,
    /// Taxonomy tags for retrieval. Convention: `[domain, severity, outcome]`.
    pub tags: Vec<String>,
    /// Source files related to this memory. Used for code-context retrieval.
    pub related_files: Vec<String>,
    /// Session that produced this memory. Nil UUID (`Uuid::nil()`) for pre-session events.
    pub session_id: Uuid,
    /// Event timestamp. Normalized to `DateTime<Utc>` across all crates.
    pub timestamp: DateTime<Utc>,
}

impl Mem0MemoryImprint {
    /// Create a new mem0 imprint with fresh UUID and current timestamp.
    pub fn new(
        memory_type: impl Into<String>,
        content: impl Into<String>,
        session_id: Uuid,
    ) -> Self {
        Self {
            memory_id: Uuid::new_v4(),
            memory_type: memory_type.into(),
            content: content.into(),
            agent_role: None,
            category: None,
            confidence: None,
            tags: Vec::new(),
            related_files: Vec::new(),
            session_id,
            timestamp: Utc::now(),
        }
    }

    /// Fluent builder: set agent_role.
    pub fn with_agent_role(mut self, role: impl Into<String>) -> Self {
        self.agent_role = Some(role.into());
        self
    }

    /// Fluent builder: set category.
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Fluent builder: set confidence.
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = Some(confidence);
        self
    }

    /// Fluent builder: set tags (replaces existing).
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Fluent builder: add a single tag.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Fluent builder: set related_files (replaces existing).
    pub fn with_related_files(mut self, files: Vec<String>) -> Self {
        self.related_files = files;
        self
    }
}

/// A mem0 retrieval event — records what the agent looked up and what it found.
///
/// Stored to analyze retrieval hit rates, latency trends, and query patterns.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mem0RetrievalEvent {
    /// The query string used for retrieval.
    pub query: String,
    /// Number of results returned.
    pub results_count: usize,
    /// memory_id values of the returned results.
    pub result_ids: Vec<Uuid>,
    /// End-to-end retrieval latency in milliseconds.
    pub retrieval_latency_ms: u64,
    /// Session that performed the retrieval.
    pub session_id: Uuid,
    /// Event timestamp.
    pub timestamp: DateTime<Utc>,
}

impl Mem0RetrievalEvent {
    pub fn new(query: impl Into<String>, results_count: usize, session_id: Uuid) -> Self {
        Self {
            query: query.into(),
            results_count,
            result_ids: Vec::new(),
            retrieval_latency_ms: 0,
            session_id,
            timestamp: Utc::now(),
        }
    }

    pub fn with_result_ids(mut self, ids: Vec<Uuid>) -> Self {
        self.result_ids = ids;
        self
    }

    pub fn with_latency_ms(mut self, ms: u64) -> Self {
        self.retrieval_latency_ms = ms;
        self
    }
}

/// Periodic mem0 telemetry snapshot — aggregates counters and distributions.
///
/// Emitted every N minutes or every N events for trend analysis and anomaly detection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mem0Telemetry {
    /// Total mem0 imprints stored since session start.
    pub total_memories_stored: u64,
    /// Total retrieval operations performed.
    pub total_retrievals: u64,
    /// Fraction of retrievals that returned >=1 results [0.0, 1.0].
    pub retrieval_hit_rate: f64,
    /// Average retrieval latency in milliseconds.
    pub average_retrieval_latency_ms: f64,
    /// Distribution of memory_type counts. Stored as `serde_json::Value` for schema flexibility.
    pub memory_types_distribution: serde_json::Value,
    /// Session being reported on.
    pub session_id: Uuid,
    /// Snapshot timestamp.
    pub timestamp: DateTime<Utc>,
}

impl Mem0Telemetry {
    pub fn new(session_id: Uuid) -> Self {
        Self {
            total_memories_stored: 0,
            total_retrievals: 0,
            retrieval_hit_rate: 0.0,
            average_retrieval_latency_ms: 0.0,
            memory_types_distribution: serde_json::json!({}),
            session_id,
            timestamp: Utc::now(),
        }
    }

    pub fn with_totals(mut self, stored: u64, retrievals: u64) -> Self {
        self.total_memories_stored = stored;
        self.total_retrievals = retrievals;
        self
    }

    pub fn with_hit_rate(mut self, hit_rate: f64) -> Self {
        self.retrieval_hit_rate = hit_rate;
        self
    }

    pub fn with_latency_ms(mut self, avg_ms: f64) -> Self {
        self.average_retrieval_latency_ms = avg_ms;
        self
    }

    pub fn with_distribution(mut self, dist: serde_json::Value) -> Self {
        self.memory_types_distribution = dist;
        self
    }
}

/// Convenience re-export of all mem0 event types.
pub mod prelude {
    pub use super::{Mem0MemoryImprint, Mem0RetrievalEvent, Mem0Telemetry};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mem0_imprint_builder() {
        let imprint = Mem0MemoryImprint::new(
            "security_event",
            "Goal mismatch detected",
            Uuid::new_v4(),
        )
        .with_agent_role("audit")
        .with_category("goal_mismatch")
        .with_confidence(1.0)
        .tag("audit")
        .tag("security")
        .tag("aborted");

        assert_eq!(imprint.memory_type, "security_event");
        assert_eq!(imprint.agent_role, Some("audit".to_string()));
        assert_eq!(imprint.category, Some("goal_mismatch".to_string()));
        assert_eq!(imprint.confidence, Some(1.0));
        assert_eq!(imprint.tags, vec!["audit", "security", "aborted"]);
        assert!(imprint.memory_id != Uuid::nil());
    }

    #[test]
    fn test_mem0_retrieval_builder() {
        let session_id = Uuid::new_v4();
        let event = Mem0RetrievalEvent::new("audit security events", 3, session_id)
            .with_result_ids(vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()])
            .with_latency_ms(42);

        assert_eq!(event.results_count, 3);
        assert_eq!(event.result_ids.len(), 3);
        assert_eq!(event.retrieval_latency_ms, 42);
    }

    #[test]
    fn test_mem0_telemetry_builder() {
        let telemetry = Mem0Telemetry::new(Uuid::new_v4())
            .with_totals(100, 50)
            .with_hit_rate(0.85)
            .with_latency_ms(12.5)
            .with_distribution(serde_json::json!({"security_event": 30, "session_outcome": 20}));

        assert_eq!(telemetry.total_memories_stored, 100);
        assert_eq!(telemetry.total_retrievals, 50);
        assert!((telemetry.retrieval_hit_rate - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn test_serde_roundtrip() {
        let original = Mem0MemoryImprint::new("session_outcome", "Completed", Uuid::new_v4())
            .with_agent_role("conductor")
            .with_confidence(0.95);

        let json = serde_json::to_string(&original).unwrap();
        let restored: Mem0MemoryImprint = serde_json::from_str(&json).unwrap();

        assert_eq!(original.memory_type, restored.memory_type);
        assert_eq!(original.content, restored.content);
        assert_eq!(original.agent_role, restored.agent_role);
        assert_eq!(original.confidence, restored.confidence);
        assert_eq!(original.session_id, restored.session_id);
        // Timestamp may differ by sub-ms in serde, compare approximately
        assert_eq!(
            original.timestamp.timestamp(),
            restored.timestamp.timestamp()
        );
    }
}
