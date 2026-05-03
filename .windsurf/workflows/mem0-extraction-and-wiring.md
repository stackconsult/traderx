# Mem0 Extraction and Wiring Workflow

## Description

Unified mem0 type extraction and cross-crate wiring workflow. Extracts mem0 types from local crate definitions into a shared crate (`traderx-mem0-types`), wires all downstream consumers (oms-engine, ectoledger, and future crates) to use the shared types, and validates compilation across workspaces.

**Core Principle**: *"A single source of truth for mem0 schema prevents data collisions across storage backends and enables unified retrieval."*

---

## Prerequisites

| Tool | Purpose |
|------|---------|
| `cargo` | Rust compilation and crate management |
| `git` | Version control for extraction tracking |
| `rg` | Fast search across crate boundaries |

---

## Phase 1: Type Extraction — Create Shared Crate

### Step 1.1: Initialize `traderx-mem0-types`

```bash
mkdir -p packages/traderx-mem0-types/src
```

### Step 1.2: Write `Cargo.toml`

```toml
[package]
name = "traderx-mem0-types"
version = "0.1.0"
edition = "2021"
description = "Shared mem0 memory types for cross-crate/ws event sourcing and pattern learning"
license = "Apache-2.0"

[dependencies]
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
uuid = { workspace = true, features = ["v4", "serde"] }
chrono = { workspace = true, features = ["serde"] }
```

### Step 1.3: Extract Types from All Crates

Search all crates for mem0 type definitions:
```bash
rg "pub struct Mem0" packages/ --type rust
rg "struct.*Mem0.*Imprint" packages/ --type rust
rg "Mem0MemoryImprint|Mem0RetrievalEvent|Mem0Telemetry" packages/ --type rust
```

**Canonical types** (unified in `src/lib.rs`):
- `Mem0MemoryImprint` — atomic memory unit with `memory_id: Uuid`
- `Mem0RetrievalEvent` — query/result tracking
- `Mem0Telemetry` — aggregate metrics snapshot

**Design rules**:
- No backend-specific fields (no Redis key, no Postgres row ID)
- All types derive `Serialize`, `Deserialize`, `PartialEq`
- `timestamp: DateTime<Utc>` — unified across ALL crates
- Builder pattern (`new()`, `with_*()`) for ergonomic construction

### Step 1.4: Register in Workspaces

**Root workspace** (`Cargo.toml`):
```toml
[workspace]
members = [
    "packages/traderx-mem0-types",
    # ... existing members
]

[workspace.dependencies]
traderx-mem0-types = { path = "packages/traderx-mem0-types" }
```

**Ectoledger workspace** (`packages/ectoledger/Cargo.toml`):
```toml
[workspace.dependencies]
# Add traderx-mem0-types as path dependency (escapes ectoledger ws)
traderx-mem0-types = { path = "../../../../traderx-mem0-types" }
```

---

## Phase 2: Consumer Wiring — OMS Engine

### Step 2.1: Add Dependency

```toml
# packages/oms-engine/Cargo.toml
traderx-mem0-types = { workspace = true }
```

### Step 2.2: Replace Local Types with Import

```rust
// packages/oms-engine/src/journal.rs
// BEFORE: local struct definitions (~60 lines)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mem0MemoryImprint { ... }

// AFTER: single re-export line
pub use traderx_mem0_types::{Mem0MemoryImprint, Mem0RetrievalEvent, Mem0Telemetry};
```

### Step 2.3: Update Call Sites to Builder Pattern

```rust
// BEFORE: struct literal (8 fields)
let imprint = Mem0MemoryImprint {
    memory_id: Uuid::new_v4(),
    memory_type: "agent_qa".to_string(),
    content: "...".to_string(),
    agent_role: Some("...".to_string()),
    category: Some("...".to_string()),
    confidence: Some(0.95),
    tags: vec!["test".to_string()],
    related_files: vec!["test.rs".to_string()],
    session_id,
};

// AFTER: builder chain
let imprint = Mem0MemoryImprint::new("agent_qa", "...", session_id)
    .with_agent_role("SystemsArchitect")
    .with_category("Architecture")
    .with_confidence(0.95)
    .with_tags(vec!["test".to_string()])
    .with_related_files(vec!["test.rs".to_string()]);
```

### Step 2.4: Validate Compilation

```bash
cd packages/oms-engine && cargo check --lib 2>&1 | grep "^error" | wc -l
# Expected: 0
```

---

## Phase 3: Consumer Wiring — Ectoledger

### Step 3.1: Add Dependency

```toml
# packages/ectoledger/crates/host/Cargo.toml
traderx-mem0-types = { path = "../../../../traderx-mem0-types" }
```

### Step 3.2: Replace Local `AuditMem0Imprint`

```rust
// packages/ectoledger/crates/host/src/commands/audit.rs
// BEFORE: local struct + manual field construction
#[derive(Debug, Clone, serde::Serialize)]
struct AuditMem0Imprint { ... }

fn create_audit_mem0_imprint(...) -> AuditMem0Imprint { ... }

// AFTER: shared type with builder
use traderx_mem0_types::Mem0MemoryImprint;

fn create_audit_mem0_imprint(
    memory_type: &str,
    content: String,
    category: Option<&str>,
    tags: Vec<String>,
    session_id: Uuid,
) -> Mem0MemoryImprint {
    Mem0MemoryImprint::new(memory_type, content, session_id)
        .with_agent_role("audit")
        .with_category(category.unwrap_or("audit_session"))
        .with_confidence(1.0)
        .with_tags(tags)
}
```

### Step 3.3: Update Tests

```rust
// BEFORE
let imprint = create_audit_mem0_imprint(...);
assert_eq!(imprint.memory_type, "session_outcome");
assert_eq!(imprint.timestamp.is_empty(), false);  // String timestamp

// AFTER
let imprint = create_audit_mem0_imprint(...);
assert_eq!(imprint.memory_type, "session_outcome");
assert!(imprint.timestamp <= Utc::now());  // DateTime<Utc>
```

### Step 3.4: Validate Compilation

```bash
cd packages/ectoledger/crates/host && cargo check --lib 2>&1 | grep "^error" | wc -l
# Expected: 0 (or match pre-existing baseline from ectoledger_core)
```

---

## Phase 4: Backend Collision Resolution

### Problem Statement

Even with unified types, mem0 data lives in **two backends**:
- `journal.rs` → **Redis** (event-sourced journal for replay)
- `audit.rs` → **Postgres** (ledger table for cryptographic audit trail)

Retrieval must query **both** backends to get complete session history.

### Step 4.1: Unified Query Interface (Journal Layer)

```rust
// In journal.rs — extend get_mem0_events to accept external sources
pub async fn get_all_mem0_events(
    &self,
    session_id: Uuid,
    // Optional: ectoledger Postgres bridge
    pg_bridge: Option<&dyn Mem0PostgresBridge>,
) -> Result<Vec<Mem0MemoryImprint>, JournalError> {
    let mut results = self.get_mem0_imprints(session_id).await?;
    
    if let Some(bridge) = pg_bridge {
        let pg_imprints = bridge.query_mem0_by_session(session_id).await?;
        results.extend(pg_imprints);
    }
    
    // Deduplicate by memory_id (UUID collision is astronomically unlikely but safe)
    let mut seen = HashSet::new();
    results.retain(|i| seen.insert(i.memory_id));
    
    Ok(results)
}
```

### Step 4.2: Postgres Bridge Trait (Ectoledger Side)

```rust
// In ectoledger — implement bridge for journal.rs to call
#[async_trait]
pub trait Mem0PostgresBridge: Send + Sync {
    async fn query_mem0_by_session(
        &self,
        session_id: Uuid,
    ) -> Result<Vec<Mem0MemoryImprint>, LedgerError>;
}

// Implementation: parse [MEM0_IMPRINT] JSON from Thought events
#[async_trait]
impl Mem0PostgresBridge for EctoLedger {
    async fn query_mem0_by_session(&self, session_id: Uuid) -> Result<Vec<Mem0MemoryImprint>, LedgerError> {
        let events = query_thought_events_by_session(&self.pool, session_id).await?;
        let mut imprints = Vec::new();
        for event in events {
            if let Some(json) = event.content.strip_prefix("[MEM0_IMPRINT] ") {
                match serde_json::from_str::<Mem0MemoryImprint>(json) {
                    Ok(imprint) => imprints.push(imprint),
                    Err(e) => tracing::warn!("Failed to parse mem0 imprint: {}", e),
                }
            }
        }
        Ok(imprints)
    }
}
```

### Step 4.3: Cross-Backend Sync Task (Optional)

For eventual consistency (not required for Phase 2):
```rust
// Background task: periodically sync Postgres mem0 events to Redis
async fn sync_postgres_mem0_to_redis(
    ledger: &EctoLedger,
    journal: &EventJournal,
) {
    let imprints = ledger.query_mem0_by_session(Uuid::nil()).await.unwrap();
    for imprint in imprints {
        journal.append_mem0_imprint(&imprint).await.ok();
    }
}
```

---

## Phase 5: Validation Checklist

### Compilation
- [ ] `traderx-mem0-types` compiles standalone: `cargo check --package traderx-mem0-types`
- [ ] `oms-engine` compiles: `cargo check --package oms-engine --lib`
- [ ] `ectoledger` compiles: `cargo check --package ectoledger --lib`

### Tests
- [ ] `traderx-mem0-types` unit tests pass: `cargo test --package traderx-mem0-types`
- [ ] `oms-engine` journal mem0 tests pass: `cargo test --package oms-engine test_mem0`
- [ ] `ectoledger` audit mem0 tests pass: `cargo test --package ectoledger audit::tests`

### Integration
- [ ] `Mem0MemoryImprint` serde roundtrip: JSON serialize → deserialize → assert_eq
- [ ] Cross-crate type identity: `oms-engine::Mem0MemoryImprint` == `ectoledger::Mem0MemoryImprint`
- [ ] Builder produces valid `timestamp: DateTime<Utc>` in both crates

### Collision Detection
- [ ] No duplicate type definitions remaining: `rg "pub struct Mem0MemoryImprint" packages/ --type rust` returns ONLY `traderx-mem0-types/src/lib.rs`
- [ ] No `AuditMem0Imprint` or similar local aliases: `rg "struct.*Mem0.*Imprint" packages/ --type rust` returns ONLY `traderx-mem0-types`

---

## Phase 6: Commit and Knowledge Persistence

### Commit Template

```bash
git add packages/traderx-mem0-types/
git add packages/oms-engine/src/journal.rs packages/oms-engine/Cargo.toml
git add packages/ectoledger/crates/host/src/commands/audit.rs packages/ectoledger/crates/host/Cargo.toml
git add Cargo.toml

git commit -m "feat(mem0): Extract shared mem0 types and wire cross-crate consumers

Extract Mem0MemoryImprint, Mem0RetrievalEvent, Mem0Telemetry into
traderx-mem0-types shared crate for unified schema across workspaces.

- New crate: packages/traderx-mem0-types/
  - Backend-agnostic types (no Redis/Postgres fields)
  - Builder pattern: new(), with_*(), tag()
  - Unified timestamp: DateTime<Utc> across ALL consumers
  - serde Serialize/Deserialize for JSON/JSONB/Protobuf

- oms-engine: Re-export from traderx_mem0_types
  - Replace local struct definitions (~60 lines → 1 line)
  - Update tests to use builder pattern

- ectoledger: Replace AuditMem0Imprint with shared type
  - Remove local struct definition
  - create_audit_mem0_imprint() uses Mem0MemoryImprint::new() builder
  - timestamp now DateTime<Utc> (was String) for unified querying

Collision resolution:
- Both Redis (journal.rs) and Postgres (audit.rs) store SAME schema
- [MEM0_IMPRINT] prefix enables downstream bridge/parse
- Unified query layer planned for Phase 4 (ledger mirror)

Refs: MEM0_PHASE2_AUDIT_SPEC.md, MEM0_PHASE2_BUILD_INSTRUCTIONS.md"
```

### Knowledge Persistence

Store in mem0:
```
Type: mem0_type_extraction
Content: "Unified Mem0MemoryImprint schema in traderx-mem0-types crate. 
          All consumers: oms-engine (journal.rs), ectoledger (audit.rs). 
          Timestamp: DateTime<Utc>. Builder pattern."
Tags: ["mem0", "types", "cross-crate", "schema", "unification"]
Related: ["packages/traderx-mem0-types/src/lib.rs", 
          "packages/oms-engine/src/journal.rs",
          "packages/ectoledger/crates/host/src/commands/audit.rs"]
```

---

## Emergency Rollback

If compilation fails catastrophically:
```bash
git reset --hard HEAD~1  # undo extraction commit
# OR selective revert:
git checkout HEAD -- packages/oms-engine/src/journal.rs  # restore local types
git rm -rf packages/traderx-mem0-types/
```

---

## Future Extensions (Phase 4+)

1. **Federated query layer** — `get_all_mem0_events()` queries Redis + Postgres
2. **Sync bridge task** — background process moves Postgres mem0 to Redis
3. **Protobuf schema** — for external vector DB integration (mem0.ai)
4. **Type evolution** — `Mem0MemoryImprintV2` with migration path from V1
