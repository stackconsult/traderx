# Phase 2 Build Instructions: Audit.rs Mem0 Security Event Wiring

**Build ID:** MEM0-PHASE2-BUILD
**Spec Reference:** MEM0_PHASE2_AUDIT_SPEC.md
**Audit Reference:** MEM0_PHASE1_AUDIT_SPEC.md (PASSED)
**Target Branch:** feature/github-mcp-setup
**Target File:** `packages/ectoledger/crates/host/src/commands/audit.rs`
**Date:** 2026-05-02
**Status:** READY FOR IMPLEMENTATION

---

## Build Overview

Wire every security-critical path in `audit.rs` to append mem0 memory imprints to the journal. This enables cross-session pattern learning for security events, guard decisions, session outcomes, and error patterns.

**Build Scope:** 7 mem0 imprint insertion points across 6 error/exit paths + 1 success path.
**Estimated Effort:** 2-3 hours (including tests + compilation verification)
**Risk Level:** LOW (best-effort storage — never blocks audit operation)

---

## Pre-Build Verification

```bash
# 1. Confirm branch
git branch --show-current  # Expected: feature/github-mcp-setup

# 2. Confirm clean working tree
git status  # Expected: nothing to commit, working tree clean

# 3. Verify Phase 1 artifacts exist
ls -la packages/oms-engine/src/journal.rs  # Expected: exists
ls -la .planning/MEM0_PHASE1_AUDIT_SPEC.md  # Expected: exists
ls -la .planning/MEM0_PHASE2_AUDIT_SPEC.md  # Expected: exists

# 4. Verify compilation baseline
cargo check --package oms-engine --lib  # Expected: 0 errors
```

---

## Build Step 1: Import Dependencies (5 min)

### File: `packages/ectoledger/crates/host/src/commands/audit.rs`

### Add Import
```rust
use crate::journal::{EventJournal, Mem0MemoryImprint};
use std::sync::Arc;
```

**Location:** After existing imports (around line 20), before `AuditArgs` struct.

**Rationale:** `EventJournal` and `Mem0MemoryImprint` are in `crate::journal`. However, `journal.rs` is in `packages/oms-engine/src/journal.rs`, not in `ectoledger`. Need to verify module path.

**⚠️ CRITICAL:** `journal.rs` is in `packages/oms-engine/src/journal.rs`. `audit.rs` is in `packages/ectoledger/crates/host/src/commands/audit.rs`. These are **different crates**. Cannot directly import.

**Alternative:** Add a shared mem0 types crate OR duplicate minimal types in ectoledger OR use `serde_json::Value` for generic imprint structure.

**Decision:** Create a minimal `Mem0Imprint` type in `ectoledger` that mirrors the journal structure but uses `serde_json::Value` for the data payload. This avoids cross-crate dependency while maintaining compatibility.

### Revised Build Step 1

```rust
// Add to audit.rs after existing imports (around line 20)
use uuid::Uuid;
use chrono::Utc;
use serde_json::json;

/// Minimal mem0 imprint structure for audit.rs (mirrors journal.rs Mem0MemoryImprint)
/// Stored as JSON in the ledger event log for cross-session analysis
#[derive(Debug, Clone, serde::Serialize)]
struct AuditMem0Imprint {
    memory_id: Uuid,
    memory_type: String,
    content: String,
    agent_role: Option<String>,
    category: Option<String>,
    confidence: Option<f64>,
    tags: Vec<String>,
    related_files: Vec<String>,
    session_id: Uuid,
    timestamp: String,
}
```

---

## Build Step 2: Helper Function (10 min)

### Add After `AuditArgs` struct (around line 36)

```rust
/// Create a mem0 imprint for the audit session
/// Best-effort: never fails audit operation
fn create_audit_mem0_imprint(
    memory_type: &str,
    content: String,
    category: Option<&str>,
    tags: Vec<String>,
    session_id: Uuid,
) -> AuditMem0Imprint {
    AuditMem0Imprint {
        memory_id: Uuid::new_v4(),
        memory_type: memory_type.to_string(),
        content,
        agent_role: Some("audit".to_string()),
        category: category.map(|s| s.to_string()),
        confidence: Some(1.0),
        tags,
        related_files: vec![],
        session_id,
        timestamp: Utc::now().to_rfc3339(),
    }
}

/// Append mem0 imprint to ledger as a Thought event (best-effort)
/// Returns Ok(()) always — mem0 storage is non-critical to audit
async fn append_mem0_imprint(
    pool: &sqlx::PgPool,
    imprint: &AuditMem0Imprint,
    session_signing_key: Option<&crate::signing::SessionSigningKey>,
) {
    let content = match serde_json::to_string(imprint) {
        Ok(json) => format!("[MEM0_IMPRINT] {}", json),
        Err(e) => {
            tracing::warn!("Failed to serialize mem0 imprint: {}", e);
            return;
        }
    };

    if let Err(e) = crate::ledger::append_event(
        pool,
        crate::schema::EventPayload::Thought { content },
        Some(imprint.session_id),
        None,
        session_signing_key,
    ).await {
        tracing::warn!("Failed to append mem0 imprint to ledger: {}", e);
    }
}
```

**Rationale:** 
- Stores mem0 imprints as `EventPayload::Thought` events in the ledger
- This makes them event-sourced (same journal, same replay capability)
- No dependency on `oms-engine` crate — self-contained in `ectoledger`
- Best-effort: warns on failure but never returns error to caller
- `session_signing_key` used for audit trail integrity

---

## Build Step 3: Wire Session Outcome — Completed (15 min)

### Location: `audit.rs` lines 312-321 (inside `agent::run_cognitive_loop` success path)

### Current Code
```rust
result = agent::run_cognitive_loop(&agent_pool, &client, agent_config) => {
    match &result {
        Ok(()) => {
            if let Err(e) = ledger::finish_session(&pool, session_id, "completed").await {
                tracing::warn!("Failed to mark session {} as completed: {}", session_id, e);
            }
            tracing::info!("Cognitive loop finished.");
        }
```

### After `ledger::finish_session` call, add:
```rust
        Ok(()) => {
            if let Err(e) = ledger::finish_session(&pool, session_id, "completed").await {
                tracing::warn!("Failed to mark session {} as completed: {}", session_id, e);
            }
            
            // MEM0 PHASE 2: Store session outcome
            let imprint = create_audit_mem0_imprint(
                "session_outcome",
                format!("Session {} completed. Prompt: {}", session_id, prompt),
                Some("audit_session"),
                vec!["audit".into(), "completed".into()],
                session_id,
            );
            append_mem0_imprint(&pool, &imprint, session_signing_key.as_deref()).await;
            
            tracing::info!("Cognitive loop finished.");
        }
```

---

## Build Step 4: Wire Security Event — Goal Mismatch (15 min)

### Location: `audit.rs` lines 322-338 (AgentError::Append(AppendError::GoalMismatch))

### Current Code
```rust
        Err(AgentError::Append(AppendError::GoalMismatch)) => {
            if let Err(e) = ledger::append_event(
                &pool,
                EventPayload::Thought {
                    content: "Security: session goal mismatch (possible redirect); aborting.".to_string(),
                },
                Some(session_id),
                None,
                None,
            ).await {
                tracing::warn!("Failed to append goal-mismatch thought: {}", e);
            }
            if let Err(e) = ledger::finish_session(&pool, session_id, "aborted").await {
                tracing::warn!("Failed to mark session {} as aborted: {}", session_id, e);
            }
            cancel.cancel();
            return Err("Session aborted: goal mismatch.".into());
        }
```

### After `ledger::finish_session`, before `cancel.cancel()`, add:
```rust
        Err(AgentError::Append(AppendError::GoalMismatch)) => {
            // ... existing ledger::append_event and ledger::finish_session ...
            
            // MEM0 PHASE 2: Store security event — goal mismatch
            let imprint = create_audit_mem0_imprint(
                "security_event",
                format!(
                    "Goal mismatch detected for session {}. Prompt: {}. Policy hash: {:?}. Session aborted.",
                    session_id, prompt, policy_hash
                ),
                Some("goal_mismatch"),
                vec!["audit".into(), "security".into(), "goal_mismatch".into(), "aborted".into()],
                session_id,
            );
            append_mem0_imprint(&pool, &imprint, session_signing_key.as_deref()).await;
            
            cancel.cancel();
            return Err("Session aborted: goal mismatch.".into());
        }
```

---

## Build Step 5: Wire Security Event — Unverified Evidence (15 min)

### Location: `audit.rs` lines 340-357 (AgentError::Append(AppendError::UnverifiedEvidence(msg)))

### After `ledger::finish_session`, before `cancel.cancel()`, add:
```rust
        Err(AgentError::Append(AppendError::UnverifiedEvidence(msg))) => {
            // ... existing ledger::append_event and ledger::finish_session ...
            
            // MEM0 PHASE 2: Store security event — unverified evidence
            let imprint = create_audit_mem0_imprint(
                "security_event",
                format!(
                    "Unverified evidence for session {}. Message: {}. Prompt: {}. Policy hash: {:?}. Session failed.",
                    session_id, msg, prompt, policy_hash
                ),
                Some("unverified_evidence"),
                vec!["audit".into(), "security".into(), "unverified_evidence".into(), "failed".into()],
                session_id,
            );
            append_mem0_imprint(&pool, &imprint, session_signing_key.as_deref()).await;
            
            cancel.cancel();
            return Err(format!("Session failed: {}", msg).into());
        }
```

---

## Build Step 6: Wire Security Event — Tripwire Abort (15 min)

### Location: `audit.rs` lines 358-364 (AgentError::TripwireAbort(reason))

### After `ledger::finish_session`, before `cancel.cancel()`, add:
```rust
        Err(AgentError::TripwireAbort(reason)) => {
            // ... existing ledger::finish_session ...
            
            // MEM0 PHASE 2: Store security event — tripwire abort
            let imprint = create_audit_mem0_imprint(
                "security_event",
                format!(
                    "Tripwire triggered for session {}. Reason: {}. Prompt: {}. Session aborted.",
                    session_id, reason, prompt
                ),
                Some("tripwire_abort"),
                vec!["audit".into(), "security".into(), "tripwire".into(), "aborted".into()],
                session_id,
            );
            append_mem0_imprint(&pool, &imprint, session_signing_key.as_deref()).await;
            
            cancel.cancel();
            return Err(format!("Session aborted (tripwire): {}", reason).into());
        }
```

---

## Build Step 7: Wire Security Event — Cancelled (15 min)

### Location: `audit.rs` lines 365-370 (AgentError::Cancelled)

### After `ledger::finish_session`, before `cancel.cancel()`, add:
```rust
        Err(AgentError::Cancelled) => {
            // ... existing ledger::finish_session ...
            
            // MEM0 PHASE 2: Store security event — cancelled
            let imprint = create_audit_mem0_imprint(
                "security_event",
                format!(
                    "Session {} cancelled by user or system. Prompt: {}. Session aborted.",
                    session_id, prompt
                ),
                Some("cancelled"),
                vec!["audit".into(), "security".into(), "cancelled".into(), "aborted".into()],
                session_id,
            );
            append_mem0_imprint(&pool, &imprint, session_signing_key.as_deref()).await;
            
            cancel.cancel();
        }
```

---

## Build Step 8: Wire Session Outcome — Server Panic (15 min)

### Location: `audit.rs` lines 383-395 (server_handle result)

### After `ledger::finish_session`, before `cancel.cancel()`, add:
```rust
        result = &mut server_handle => {
            match result {
                Ok(()) => tracing::error!("Observer server exited unexpectedly during audit."),
                Err(e) => tracing::error!("Observer server panicked during audit: {}", e),
            }
            if let Err(e) = ledger::finish_session(&pool, session_id, "failed").await {
                tracing::warn!("Failed to mark session {} as failed: {}", session_id, e);
            }
            
            // MEM0 PHASE 2: Store session outcome — server failure
            let imprint = create_audit_mem0_imprint(
                "session_outcome",
                format!(
                    "Observer server failed for session {}. Prompt: {}. Server result: {:?}.",
                    session_id, prompt, result
                ),
                Some("server_failure"),
                vec!["audit".into(), "server".into(), "failed".into()],
                session_id,
            );
            append_mem0_imprint(&pool, &imprint, session_signing_key.as_deref()).await;
            
            cancel.cancel();
            (true, true)
        }
```

---

## Build Step 9: Wire Session Outcome — Ctrl-C Abort (15 min)

### Location: `audit.rs` lines 396-400 (Ctrl-C signal)

### After `cancel.cancel()`, add:
```rust
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Shutdown signal received; cancelling tasks…");
            cancel.cancel();
            
            // MEM0 PHASE 2: Store session outcome — signal abort
            let imprint = create_audit_mem0_imprint(
                "session_outcome",
                format!(
                    "Session {} aborted by shutdown signal. Prompt: {}.",
                    session_id, prompt
                ),
                Some("signal_abort"),
                vec!["audit".into(), "signal".into(), "aborted".into()],
                session_id,
            );
            append_mem0_imprint(&pool, &imprint, session_signing_key.as_deref()).await;
            
            (true, false)
        }
```

---

## Build Step 10: Wire Recovery Patterns (Optional - Phase 2 Extended) (20 min)

### Location: `audit.rs` lines 110-115 (startup recovery)

### After `recover_incomplete_actions`, add:
```rust
    // MEM0 PHASE 2: Store recovery patterns
    let zombie_count = match crate::wakeup::recover_zombie_sessions(&pool).await {
        Ok(n) => n.len(),
        Err(_) => 0,
    };
    let incomplete_count = match crate::wakeup::recover_incomplete_actions(&pool).await {
        Ok(n) => n.len(),
        Err(_) => 0,
    };
    
    if zombie_count > 0 || incomplete_count > 0 {
        let imprint = create_audit_mem0_imprint(
            "recovery_pattern",
            format!(
                "Startup recovery: {} zombie sessions, {} incomplete actions recovered.",
                zombie_count, incomplete_count
            ),
            Some("startup_recovery"),
            vec!["audit".into(), "recovery".into(), "startup".into()],
            Uuid::nil(), // No session yet — use nil UUID for pre-session events
        );
        // Note: session_signing_key not available yet — pass None
        append_mem0_imprint(&pool, &imprint, None).await;
    }
```

**⚠️ Note:** This requires `recover_zombie_sessions()` and `recover_incomplete_actions()` to return counts. Currently they return `Result<(), _>`. May need wrapper or skip this step.

**Decision:** SKIP for Phase 2 core. Document as Phase 2 Extended in MEM0_EXECUTION_PLAN.md.

---

## Build Step 11: Compilation Verification (10 min)

```bash
# Verify ectoledger compiles
cd packages/ectoledger/crates/host
cargo check 2>&1 | grep "^error" | wc -l  # Expected: 0

# If errors, fix type mismatches, missing imports, or lifetime issues
```

**Common Issues:**
- `session_signing_key` type mismatch — may need `as_deref()` or `as_ref()`
- `pool` borrow conflict — may need `pool.clone()` in async block
- `prompt` not available in all match arms — ensure moved before select! or cloned

---

## Build Step 12: Integration Test (30 min)

### File: `packages/ectoledger/crates/host/src/commands/audit_test.rs` (new)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    
    #[test]
    fn test_create_audit_mem0_imprint() {
        let imprint = create_audit_mem0_imprint(
            "session_outcome",
            "Test content".to_string(),
            Some("test"),
            vec!["test".into()],
            Uuid::new_v4(),
        );
        
        assert_eq!(imprint.memory_type, "session_outcome");
        assert_eq!(imprint.content, "Test content");
        assert_eq!(imprint.category, Some("test".to_string()));
        assert_eq!(imprint.tags, vec!["test"]);
        assert_eq!(imprint.agent_role, Some("audit".to_string()));
        assert_eq!(imprint.confidence, Some(1.0));
        assert!(imprint.memory_id != Uuid::nil());
        assert!(!imprint.timestamp.is_empty());
    }
    
    #[test]
    fn test_audit_mem0_imprint_serialization() {
        let imprint = create_audit_mem0_imprint(
            "security_event",
            "Goal mismatch detected".to_string(),
            Some("goal_mismatch"),
            vec!["audit".into(), "security".into()],
            Uuid::new_v4(),
        );
        
        let json = serde_json::to_string(&imprint).unwrap();
        assert!(json.contains("security_event"));
        assert!(json.contains("Goal mismatch detected"));
        assert!(json.contains("goal_mismatch"));
    }
}
```

---

## Build Step 13: Commit (5 min)

```bash
cd /Users/kirtissiemens/CascadeProjects/traderx-repo
git add packages/ectoledger/crates/host/src/commands/audit.rs
git add packages/ectoledger/crates/host/src/commands/audit_test.rs
git commit -m "feat(audit): Wire mem0 security event storage to all audit.rs exit paths

- Add AuditMem0Imprint struct mirroring journal.rs Mem0MemoryImprint
- Add create_audit_mem0_imprint() helper for consistent imprint creation
- Add append_mem0_imprint() best-effort ledger storage (never blocks audit)
- Wire 7 mem0 insertion points:
  1. Session completed (cognitive loop success)
  2. Goal mismatch (security event — abort)
  3. Unverified evidence (security event — fail)
  4. Tripwire abort (security event — abort)
  5. Cancelled (security event — abort)
  6. Server panic (session outcome — fail)
  7. Ctrl-C signal (session outcome — abort)
- Tags: [audit, security, event_subtype, outcome]
- Content includes: session_id, prompt, policy_hash, error details
- Serialization: JSON via serde, stored as EventPayload::Thought
- No oms-engine dependency — self-contained in ectoledger
- Tests: AuditMem0Imprint creation + serialization

Refs: MEM0_PHASE2_AUDIT_SPEC.md, MEM0_PHASE1_AUDIT_SPEC.md"
```

---

## Build Step 14: Final Verification (10 min)

```bash
# Full compilation check
cargo check --package ectoledger 2>&1 | grep "^error" | wc -l  # Expected: 0

# Full test suite
cargo test --package ectoledger 2>&1 | tail -5  # Expected: tests pass

# Verify git status clean
git status  # Expected: nothing to commit

# Verify commit log
git log --oneline -3
# Expected: feat(audit): Wire mem0 security event storage...
```

---

## Post-Build: Update MEM0_EXECUTION_PLAN.md

Mark Phase 2 as COMPLETED in `.planning/MEM0_EXECUTION_PLAN.md`:
```markdown
- Phase 2: Audit.rs security event memory (COMPLETED)
  - 7 mem0 insertion points wired across all exit paths
  - Best-effort storage (never blocks audit)
  - Self-contained in ectoledger (no cross-crate dependency)
  - AuditMem0Imprint struct + 2 helper functions
  - Integration tests: creation + serialization
```

---

## Risk Mitigation

| Risk | Probability | Mitigation |
|------|-------------|------------|
| Cross-crate import failure | HIGH | Use self-contained AuditMem0Imprint instead of importing from oms-engine |
| `prompt` moved before select! | MEDIUM | Clone prompt before match arms: `let prompt_for_mem0 = prompt.clone();` |
| `session_signing_key` lifetime | LOW | Use `as_deref()` or clone Arc before async block |
| Compilation errors in ectoledger | MEDIUM | Run `cargo check` after each build step |
| Test environment (no Postgres) | HIGH | Tests use unit tests only (no DB required) |

---

## SDD Compliance Checklist (Build Phase)

| SDD Requirement | Status | Evidence |
|-----------------|--------|----------|
| Spec written before code | ✅ | MEM0_PHASE2_AUDIT_SPEC.md exists |
| One-sentence description | ✅ | "Wire security-critical audit operations to mem0 journal" |
| Exact input format | ✅ | 7 runtime inputs + AuditMem0Imprint struct |
| Exact output format | ✅ | 7 ledger EventPayload::Thought events |
| Edge cases identified | ✅ | 7 edge cases with handling |
| Success criteria defined | ✅ | 9 criteria including tests + compilation |
| Test examples | ✅ | 2 test functions with assertions |
| Components communicate | ✅ | audit.rs → ledger::append_event → Postgres |
| Message format | ✅ | JSON via serde_json::to_string |
| Failure modes | ✅ | 5 failure modes with probability/impact/recovery |

---

**Build Instructions Author:** Engineering Agent Orchestra (per SDD workflow)
**Build Instructions Reviewer:** Autonomous Upskilling Engine
**Approval Status:** READY FOR BUILD EXECUTION
**Next Action:** Execute Build Steps 1-14 in sequence
