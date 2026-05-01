# TraderX Corrective Action Plan

**Date:** 2026-05-01
**Status:** IMMEDIATE EXECUTION REQUIRED
**Authority:** TRADERX_AGENT_ACTION_SPEC.md
**Audit Reference:** AUDIT_REPORT_20260501.md

---

## 1. CORRECTIVE PRINCIPLES

1. **No phase jumping** — Complete Phase N before Phase N+1
2. **No action invention** — Use only Action IDs from Master Library
3. **Every task uses Global Task Object Schema** (Part 1.2)
4. **One chunk session per Cascade session**
5. **HALT on blockers**, then resume
6. **Fail closed** — ambiguity → HALT
7. **Reviewer gate** — task not DONE until confirmed
8. **Plan approval** — protected paths require approval

---

## 2. IMMEDIATE ACTIONS (Complete Before Any Feature Work)

### Action 0.1: HALT ALL NON-FOUNDATIONAL WORK
**Priority:** CRITICAL  
**Status:** IN PROGRESS

Stop Phase 7, 18, and any non-foundational work. Mark all as BLOCKED in tasks.json.

### Action 0.2: CREATE tasks.json
**Priority:** CRITICAL  
**Action ID:** Custom (task management system)  
**Owner:** TeamLead  
**Plan Required:** Yes — Human approval

Create tasks.json using Global Task Object Schema:
- id, phase, chunk_session, owner, goal, files, read_only_files
- prerequisites, inputs, outputs, actions
- tests_required, lint_required
- plan_required, plan_approver, reviewer
- bam_signal, done_definition

**Done Definition:** tasks.json exists with all Phase 0-2 tasks defined, human approved.

### Action 0.3: INSTALL DEPENDENCIES
**Priority:** CRITICAL  
**Action ID:** Infrastructure (requires human approval)  
**Owner:** TeamLead

```bash
brew install rustup && rustup install 1.70.0
brew install postgresql@15 && brew services start postgresql@15
brew install redis && brew services start redis
brew install --cask docker
pip install ruff mypy pytest
```

**Done Definition:** cargo ≥1.70, psql ≥15, redis-cli ping = PONG, docker ≥20.x

### Action 0.4: SCAFFOLD REPOSITORY
**Priority:** CRITICAL  
**Action ID:** A-01 (SCAFFOLD_REPO)  
**Owner:** TeamLead

Create directory tree + config files:
```
.claude/
app/agents, app/api/v1, app/bam, app/fabric, app/feeds/adapters
app/messaging, app/routers, app/schemas, app/services
docs/architecture, docs/operations, docs/onboarding, docs/user
migrations, packages/oms-engine/src/{bin,state_machine,oms}
scripts, tests/{integration,performance,unit,golden}
```

Create: pyproject.toml, .env.example, AGENTS.md (updated), .claude/team-lead.md, docker-compose.yml, .pre-commit-config.yaml

**Done Definition:** All dirs exist, all config files created.

---

## 3. PHASE 0: FOUNDATION LOCK

### CS-0.0 — Halt & Task Management
**Duration:** 30 min  
**Owner:** TeamLead  
**Reviewer:** Human

Tasks:
1. Create tasks.json with ≥20 tasks (Phases 0-2)
2. Human approval of task plan
3. Mark non-foundational work BLOCKED

**Exit Criteria:** tasks.json exists, human approved.

### CS-0.1 — Repo Foundation
**Duration:** 1-2 hours  
**Owner:** TeamLead  
**Reviewer:** Human  
**Prerequisites:** CS-0.0 DONE

Tasks:
1. A-01 SCAFFOLD_REPO — all dirs, config files
2. A-02 WRITE_AGENTS_MD — update with phase decisions, gotchas, ownership
3. A-03 WRITE_TEAM_LEAD_PROMPT — .claude/team-lead.md with spawn rules
4. A-04 CONFIGURE_PRECOMMIT — ruff, pytest, migration lint
5. A-05 WRITE_DOCKER_COMPOSE — Postgres 15, Redis 7, Qdrant
6. A-06 WRITE_ENV_EXAMPLE — all required env vars
7. A-07 VERIFY_BOOT — uvicorn app.main:app, /health returns ok

**Exit Criteria:** App boots, CI runs, AGENTS.md has phase 0 decisions.

---

## 4. PHASE 1: SCHEMA & DATA LAYER

### CS-1.1 — Schema Migration
**Duration:** 2-3 hours  
**Owner:** TeamLead (plan) + Reviewer (verify)  
**Plan Required:** Yes  
**Prerequisites:** CS-0.1 DONE

Tasks:
1. B-01 WRITE_MIGRATION — migrations/NNN_*.sql for all tables
2. B-02 RUN_MIGRATION — execute against dev DB, verify tables exist
3. B-03 VERIFY_SCHEMA_TABLES — assert columns and types correct
4. B-04 VERIFY_IMMUTABILITY — INSERT succeeds, UPDATE/DELETE fails

**Exit Criteria:** All tables exist. Immutable tables reject mutation.

### CS-1.2 — Audit View & Hash Chain
**Duration:** 1-2 hours  
**Owner:** AuditAgent + Reviewer  
**Prerequisites:** CS-1.1 DONE

Tasks:
1. B-05 VERIFY_AUDIT_VIEW — insert sample, query v_audit_trade_full, assert complete
2. B-08 VERIFY_HASH_CHAIN — insert 3 events, assert prev_hash unbroken
3. B-09 WRITE_REGULATORY_FIELD_MAP — CIRO/OSC/SEC field mappings

**Exit Criteria:** Audit view complete. Hash chain verifies.

### CS-1.3 — Seed Tables
**Duration:** 1 hour  
**Owner:** Router  
**Prerequisites:** CS-1.1 DONE

Tasks:
1. B-06 SEED_TLTT_DEFINITIONS — ≥14 rows (vertical, horizontal, diagonal)
2. B-07 SEED_FABRIC_NODES — all TraderX containers, splices, traces

**Exit Criteria:** tltt_definitions ≥14 rows. fabric_nodes complete.

---

## 5. PHASE 2: BAM & FABRIC

### CS-2.1 — BAM Genesis Lock
**Duration:** 2 hours  
**Owner:** Router  
**Plan Required:** Yes  
**Prerequisites:** CS-1.3 DONE

Tasks:
1. C-01 LOAD_BAM_TABLE — all domain codes present
2. C-02 COMPUTE_GENESIS_HASH — SHA-256, write to .env
3. C-03 VERIFY_BAM_INTEGRITY — assert VERIFIED
4. C-04 VERIFY_BAM_MUTATION_HALT — mutation → VIOLATION + 503

**Exit Criteria:** Genesis hash locked. Mutation halts. Integrity passes.

### CS-2.2 — Fabric Registry & Dual Key
**Duration:** 2 hours  
**Owner:** Router  
**Prerequisites:** CS-2.1 DONE

Tasks:
1. C-07 BUILD_FABRIC_REGISTRY — all containers, splices, traces
2. C-05 ASSEMBLE_DUAL_KEY — 3+ domain nodes validated
3. C-06 VERIFY_DUAL_KEY_ROUNDTRIP — decode equals original
4. C-08 WRITE_MCP_BAM_TOOLS — MCP server tool definitions

**Exit Criteria:** Registry complete. Dual key roundtrips. MCP tools defined.

---

## 6. PHASE 3: TLTT ENGINE

### CS-3.1 — TLTT Implementation
**Duration:** 2-3 hours  
**Owner:** Router  
**Prerequisites:** CS-2.2 DONE

Tasks:
1. D-01 IMPLEMENT_TLTT_ENGINE — app/fabric/tltt.py with evaluate(), evaluate_all()
2. D-02 WRITE_GOLDEN_SCENARIOS — tests/golden/scenarios.json
3. D-03 RUN_GOLDEN_TESTS — all scenarios produce expected pass/fail

**Exit Criteria:** TLTT engine evaluates. Golden tests pass.

---

## 7. EXECUTION CHECKLIST

| Step | Action | Owner | Status |
|------|--------|-------|--------|
| 0.1 | Halt feature work | TeamLead | IN PROGRESS |
| 0.2 | Create tasks.json | TeamLead | NOT DONE |
| 0.3 | Install dependencies | TeamLead | NOT DONE |
| 0.4 | Scaffold repo | TeamLead | NOT DONE |
| 0.5 | CS-0.0: Halt & tasks | TeamLead | NOT DONE |
| 0.6 | CS-0.1: Repo foundation | TeamLead | NOT DONE |
| 0.7 | CS-1.1: Schema migration | TeamLead + Reviewer | NOT DONE |
| 0.8 | CS-1.2: Audit & hash chain | AuditAgent + Reviewer | NOT DONE |
| 0.9 | CS-1.3: Seed tables | Router | NOT DONE |
| 0.10 | CS-2.1: BAM genesis | Router | NOT DONE |
| 0.11 | CS-2.2: Fabric registry | Router | NOT DONE |
| 0.12 | CS-3.1: TLTT engine | Router | NOT DONE |

---

## 8. BLOCKING DEPENDENCIES

| Dependency | Required For | Resolution |
|-----------|-------------|------------|
| Rust 1.70+ | OMS engine, cargo check | brew install rustup |
| PostgreSQL 15 | Schema, migrations, tables | brew install postgresql@15 |
| Redis 7 | Caching, hot paths | brew install redis |
| Docker | Container orchestration | brew install --cask docker |
| ruff, mypy, pytest | Lint, type check, test | pip install |

---

## 9. HUMAN APPROVAL REQUIRED

Before proceeding with any work:

1. **Approve tasks.json schema** — Review Global Task Object Schema
2. **Approve Phase 0 plan** — CS-0.0, CS-0.1 task definitions
3. **Approve dependency installation** — System package changes
4. **Approve Phase 1 schema plan** — Database migration design
5. **Approve BAM genesis** — Genesis hash computation
6. **Approve TLTT engine** — Gate evaluation logic

**Do not proceed past any approval gate without explicit human confirmation.**

---

**Plan prepared by:** Self-audit  
**Date:** 2026-05-01  
**Awaiting human approval for Step 0.2 (tasks.json creation)**
