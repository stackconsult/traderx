# Mem0 Execution Plan: Full System Integration

**Plan ID:** MEM0-EXEC-001
**Status:** Phase 1 Complete (Journal Telemetry) / Phase 2 Ready (Audit Integration)
**Date:** 2026-05-02
**Owner:** Engineering Agent Orchestra + Autonomous Upskilling Engine

---

## Objective

Execute mem0 usecase upgrades and wiring across all system layers:
- ✅ Skills: debug-team, qa-team
- ✅ Workflows: session-start, autonomous-upskilling
- ✅ Orchestration: engineering_orchestra.rs
- ✅ Journal: mem0 telemetry flash imprints
- 🔲 Audit: audit.rs security event memory
- 🔲 Infrastructure: K8s deployment annotations
- 🔲 Cross-module: ledger mem0 mirror, metrics baselines

---

## Phase 1: COMPLETED — Journal Telemetry Integration

### Deliverables
- `Mem0MemoryImprint` struct: Maps mem0 memory records to journal events
- `Mem0RetrievalEvent` struct: Tracks query operations and latency
- `Mem0Telemetry` struct: Performance metrics (hit rate, latency, distribution)
- `append_mem0_imprint()`: Stores memory records as journal events with session correlation
- `append_mem0_retrieval()`: Stores query events as journal events
- `append_mem0_telemetry()`: Stores performance snapshots as journal events
- `get_mem0_imprints()`: Retrieves memory imprints by session ID
- `get_mem0_telemetry()`: Retrieves telemetry events by session ID
- `get_mem0_events()`: Retrieves all mem0-related events (imprints + retrievals + telemetry)

### Test Coverage
- `test_mem0_imprint_append_and_retrieve()`: Validates imprint round-trip (memory_id, type, content, tags)
- Validates telemetry round-trip (total_memories_stored, hit_rate, latency)
- Validates composite mem0 event retrieval (2 events per session)

### Architecture Pattern
```
Mem0 Memory Layer → Mem0MemoryImprint → journal.append_mem0_imprint()
                                              ↓
                                    Redis-backed EventJournal
                                              ↓
                              journal.get_mem0_imprints(session_id)
```

**Best-effort design**: All mem0 operations wrapped in `.ok()` or explicit error handling that never fails the primary operation. Mem0 is a read model, not a write-model dependency.

---

## Phase 2: Audit.rs Security Event Memory (NEXT)

### Task: MEM0-AUDIT-001
**Priority:** P0
**Effort:** Medium (1-2 hours)
**Blocked By:** None

### Implementation

After each `ledger::append_event()` call in `audit.rs`, add mem0 storage:

```rust
// After: ledger::finish_session(&pool, session_id, "completed")
// Add:
let mem0_imprint = Mem0MemoryImprint {
    memory_id: Uuid::new_v4(),
    memory_type: "session_outcome".to_string(),
    content: format!("Session {} completed. Prompt: {}", session_id, prompt),
    agent_role: None,
    category: Some("audit_session".to_string()),
    confidence: Some(1.0),
    tags: vec!["audit".into(), "completed".into()],
    related_files: vec![],
    session_id,
};
if let Some(journal) = &journal {
    let _ = journal.append_mem0_imprint(&mem0_imprint).await;
}
```

### Event Types to Store

| audit.rs Location | Event Type | Trigger | Content |
|-------------------|------------|---------|---------|
| Line 317 | session_outcome | completed | Session completion with prompt |
| Line 322 | security_event | goal_mismatch | Goal redirect detection |
| Line 340 | security_event | unverified_evidence | Evidence verification failure |
| Line 358 | security_event | tripwire_abort | Banned command trigger |
| Line 365 | security_event | cancelled | User or system cancellation |
| Line 389 | session_outcome | server_panic | Observer server crash |
| Line 412 | session_outcome | aborted_signal | Ctrl-C shutdown |

---

## Phase 3: Orchestrate.rs Three-Phase Memory (NEXT)

### Task: MEM0-ORCH-001
**Priority:** P1
**Effort:** Medium (1 hour)

### Implementation

After `run_orchestration()` completes:

```rust
// After: Ok(result) in run_orchestrate()
// Add:
let mem0_imprint = Mem0MemoryImprint {
    memory_id: Uuid::new_v4(),
    memory_type: "orchestration_result".to_string(),
    content: format!(
        "Recon: {}\nAnalysis: {}\nVerify: {}\nSeal: {}",
        result.recon_session_id,
        result.analysis_session_id,
        result.verify_session_id,
        result.seal_hash
    ),
    agent_role: Some("Orchestrator".to_string()),
    category: Some("multi_agent".to_string()),
    confidence: Some(1.0),
    tags: vec!["orchestrate".into(), "recon".into(), "analysis".into(), "verify".into()],
    related_files: vec![],
    session_id: result.recon_session_id,
};
```

After red-team `report.passed_all > 0`:
```rust
let mem0_imprint = Mem0MemoryImprint {
    memory_type: "red_team_finding".to_string(),
    content: format!("{} injections passed all defense layers", report.passed_all),
    agent_role: Some("RedTeam".to_string()),
    category: Some("security_finding".to_string()),
    // ...
};
```

---

## Phase 4: Cross-Module Mirror (FUTURE)

### Task: MEM0-MIRROR-001
**Priority:** P2
**Effort:** Large (4+ hours)

### Ledger Mirror
Mirror high-value `ledger::append_event()` calls to mem0:
- Session creation events
- Policy hash changes
- Guard verdict events
- Thought events with security implications

### Metrics Baseline
Store `metrics.inc_sessions_created()` and `metrics.inc_events_appended()` as `Mem0Telemetry` entries to detect anomaly trends:
- Unusual session creation rate (DDoS, loop)
- Unusual event append failures (Redis saturation)
- Guard rejection rate trends

---

## Verification Checklist

### Phase 1 Verification
- [x] `cargo check --package oms-engine --lib` passes
- [x] `cargo test --package oms-engine test_mem0_imprint_append_and_retrieve` passes
- [x] Mem0MemoryImprint struct serialized/deserialized correctly
- [x] Mem0Telemetry round-trip through Redis validated
- [x] All 3 event types (imprint, retrieval, telemetry) retrievable

### Phase 2 Verification (Pending)
- [ ] audit.rs session outcomes stored in journal mem0 events
- [ ] audit.rs security events (goal mismatch, tripwire, etc.) stored
- [ ] audit.rs guard decisions stored
- [ ] audit.rs error patterns retrievable by session

### Phase 3 Verification (Pending)
- [ ] orchestrate.rs recon/analysis/verify results stored
- [ ] orchestrate.rs red-team findings stored
- [ ] orchestrate.rs diff audit comparisons stored

### Phase 4 Verification (Pending)
- [ ] Ledger mirror: high-value events in mem0
- [ ] Metrics baseline: anomaly detection from telemetry
- [ ] Cross-session pattern retrieval working

---

## Commit History

| Commit | Description | Status |
|--------|-------------|--------|
| `a57a9b4` | Mem0 skill/workflow/orchestra integration | ✅ Merged |
| `6f67862` | Fabric orchestrator API drift fixes + audit report | ✅ Merged |
| `c5a0fd9` | Dependabot + K8s mem0 annotations + deploy spec | ✅ Merged |
| `cb24084` | Journal mem0 telemetry flash imprints | ✅ Ready to push |

---

## Next Actions

1. **Execute Phase 2**: Add mem0 storage calls after all `ledger::append_event()` and `ledger::finish_session()` calls in `audit.rs`
2. **Execute Phase 3**: Add mem0 storage calls in `orchestrate.rs` after orchestration completes and red-team reports
3. **Execute Phase 4**: Add ledger mirror and metrics baseline telemetry
4. **Continuous**: Run `cargo check` and `cargo test` after each phase
5. **Commit**: Use conventional commits: `feat(audit): mem0 security event memory`, `feat(orch): mem0 orchestration memory`

---

**The mem0 telemetry flash imprint system is now live in the journal. Every memory operation leaves an event-sourced trace.**
