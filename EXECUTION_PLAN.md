# TraderX Execution Plan

**Date:** 2026-05-01
**Status:** READY FOR EXECUTION
**Capability Enhancement:** 8 MCP servers added to enable proper functioning

---

## 1. CAPABILITY ENHANCEMENT COMPLETED

### Added MCP Servers to `.windsurf/mcp_config.json`:

| Server | Purpose | Usage |
|--------|---------|-------|
| **filesystem** | File operations | Read/write project files |
| **postgres** | Database operations | Execute migrations, verify schema |
| **brave-search** | Web search | Research, validation |
| **sqlite** | Local database | Development/testing |
| **puppeteer** | Browser automation | UI testing, web scraping |
| **sequential-thinking** | Complex reasoning | Multi-step problem solving |
| **memory** | Persistent context | Remember state across sessions |
| **git** | Version control | Commit, branch, repository operations |

**Current MCP Configuration:** 10 servers total (2 existing + 8 new)

---

## 2. EXECUTION STRATEGY

### Approach: Strict Sequential Chunk Execution

Following TRADERX_AGENT_ACTION_SPEC.md exactly:
- One chunk session per session
- No phase jumping
- Every task uses Global Task Object Schema
- Reviewer verification before DONE
- Plan approval for protected paths

### Execution Order:

1. **Dependency Installation** (System-level)
2. **CS-0.0: Task Management Setup** (Create tasks.json)
3. **CS-0.1: Repo Foundation** (A-01 through A-07)
4. **CS-1.1: Schema Migration** (B-01 through B-04)
5. **CS-1.2: Audit View & Hash Chain** (B-05, B-08, B-09)
6. **CS-1.3: Seed Tables** (B-06, B-07)
7. **CS-2.1: BAM Genesis Lock** (C-01 through C-04)
8. **CS-2.2: Fabric Registry & Dual Key** (C-05 through C-08)
9. **CS-2.3: TLTT Engine & Golden Tests** (C-09, D-01 through D-08)

---

## 3. STEP 1: DEPENDENCY INSTALLATION

### Objective: Install all system-level dependencies

**Dependencies Required:**
- Rust 1.70+ with cargo
- PostgreSQL 15
- Redis 7
- Docker Desktop
- Node.js 18+
- Python 3.10+
- ruff, mypy, pytest

**Execution Commands:**

```bash
# Rust toolchain
brew install rustup
rustup install 1.70.0
rustup default 1.70.0

# PostgreSQL 15
brew install postgresql@15
brew services start postgresql@15
psql --version  # Verify 15.x

# Redis 7
brew install redis
brew services start redis
redis-cli ping  # Verify PONG

# Docker Desktop
brew install --cask docker
docker --version  # Verify 20.x+
docker compose version

# Python dependencies
pip install ruff mypy pytest pytest-asyncio

# Node.js (if needed)
brew install node@18
node --version  # Verify 18.x+
```

**Verification Commands:**
```bash
cargo --version
psql --version
redis-cli ping
docker --version
ruff --version
mypy --version
pytest --version
```

**Success Criteria:**
- All dependencies installed
- All version commands return expected outputs
- Services running (PostgreSQL, Redis, Docker)

**Estimated Time:** 15-20 minutes

---

## 4. STEP 2: CS-0.0 TASK MANAGEMENT SETUP

### Objective: Create tasks.json with Global Task Object Schema

**Task Object Schema (from Part 1.2):**
```json
{
  "id": "string",
  "phase": "string",
  "chunk_session": "string",
  "owner": "string",
  "goal": "string",
  "files": ["array"],
  "read_only_files": ["array"],
  "prerequisites": ["array"],
  "inputs": ["array"],
  "outputs": ["array"],
  "actions": ["array"],
  "tests_required": ["array"],
  "lint_required": ["array"],
  "plan_required": true,
  "plan_approver": "human | TeamLead",
  "reviewer": "string",
  "bam_signal": "string",
  "done_definition": "string"
}
```

**Tasks to Define:**

1. **CS-0.1 Tasks (7 tasks):**
   - p0-scaffold-repo (A-01)
   - p0-write-agents-md (A-02)
   - p0-write-team-lead-prompt (A-03)
   - p0-configure-precommit (A-04)
   - p0-write-docker-compose (A-05)
   - p0-write-env-example (A-06)
   - p0-verify-boot (A-07)

2. **CS-1.1 Tasks (4 tasks):**
   - p1-write-migration (B-01)
   - p1-run-migration (B-02)
   - p1-verify-schema-tables (B-03)
   - p1-verify-immutability (B-04)

3. **CS-1.2 Tasks (3 tasks):**
   - p1-verify-audit-view (B-05)
   - p1-verify-hash-chain (B-08)
   - p1-write-regulatory-field-map (B-09)

4. **CS-1.3 Tasks (2 tasks):**
   - p1-seed-tltt-definitions (B-06)
   - p1-seed-fabric-nodes (B-07)

5. **CS-2.1 Tasks (4 tasks):**
   - p2-load-bam-table (C-01)
   - p2-compute-genesis-hash (C-02)
   - p2-verify-bam-integrity (C-03)
   - p2-verify-bam-mutation-halt (C-04)

6. **CS-2.2 Tasks (4 tasks):**
   - p2-build-fabric-registry (C-07)
   - p2-assemble-dual-key (C-05)
   - p2-verify-dual-key-roundtrip (C-06)
   - p2-write-mcp-bam-tools (C-08)

7. **CS-2.3 Tasks (8 tasks):**
   - p2-test-redis-hot-path (C-09)
   - p2-write-pulse-trace-formatter (C-10)
   - p2-implement-tltt-engine (D-01)
   - p2-write-golden-scenarios (D-02)
   - p2-run-golden-tests (D-03)
   - p2-verify-vertical-gate (D-05)
   - p2-verify-horizontal-gate (D-06)
   - p2-verify-diagonal-gate (D-07)
   - p2-verify-tltt-block (D-08)

**File Location:** `/Users/kirtissiemens/CascadeProjects/traderx-repo/tasks.json`

**Success Criteria:**
- tasks.json created with all 32 tasks defined
- Each task has all required fields populated
- Human reviewer approves task plan

**Estimated Time:** 30 minutes

---

## 5. STEP 3: CS-0.1 REPO FOUNDATION

### Objective: Complete all A-01 through A-07 actions

**Task Sequence:**

#### Task 1: A-01 SCAFFOLD_REPO
**Owner:** TeamLead
**Actions:**
1. Create directory tree
2. Create pyproject.toml
3. Create .env.example
4. Update AGENTS.md with phase 0 decisions

**Directories to Create:**
```
.claude/
app/agents/
app/api/v1/
app/bam/
app/fabric/
app/feeds/adapters/
app/messaging/
app/routers/
app/schemas/
app/services/
docs/architecture/
docs/operations/
docs/onboarding/
docs/user/
migrations/
packages/oms-engine/src/bin/
packages/oms-engine/src/state_machine/
packages/oms-engine/src/oms/
scripts/
tests/integration/
tests/performance/
tests/unit/
tests/golden/
proofs/
```

#### Task 2: A-02 WRITE_AGENTS_MD
**Owner:** TeamLead
**Actions:**
1. Update AGENTS.md with phase 0 decisions
2. Add gotchas discovered during foundation
3. Document file ownership map

#### Task 3: A-03 WRITE_TEAM_LEAD_PROMPT
**Owner:** TeamLead
**Actions:**
1. Create .claude/team-lead.md
2. Define spawn rules
3. Define hook rules
4. Define BAM-domain map

#### Task 4: A-04 CONFIGURE_PRECOMMIT
**Owner:** TeamLead
**Actions:**
1. Create .pre-commit-config.yaml
2. Configure ruff
3. Configure pytest
4. Configure migration lint

#### Task 5: A-05 WRITE_DOCKER_COMPOSE
**Owner:** TeamLead
**Actions:**
1. Create docker-compose.yml
2. Define PostgreSQL 15 service
3. Define Redis 7 service
4. Define Qdrant service
5. Define networking

#### Task 6: A-06 WRITE_ENV_EXAMPLE
**Owner:** TeamLead
**Actions:**
1. Create .env.example
2. Define all required environment variables
3. Add types and descriptions
4. Add default values where safe

#### Task 7: A-07 VERIFY_BOOT
**Owner:** Reviewer
**Actions:**
1. Run uvicorn app.main:app
2. Verify /health returns status: ok
3. Verify BAM integrity check passes
4. Run all tests

**Success Criteria:**
- All directories exist
- All config files created
- App boots successfully
- /health endpoint returns ok
- CI runs successfully

**Estimated Time:** 2 hours

---

## 6. STEP 4: CS-1.1 SCHEMA MIGRATION

### Objective: Create database schema with immutability

**Prerequisites:** CS-0.1 DONE, PostgreSQL running

**Task Sequence:**

#### Task 1: B-01 WRITE_MIGRATION
**Owner:** TeamLead
**Plan Required:** Yes
**Actions:**
1. Create migrations/001_initial_schema.sql
2. Define all required tables
3. Define constraints and indexes
4. Define audit views
5. Define hash chain logic

**Tables Required:**
- market_events
- agent_queries
- trade_events
- tltt_evaluations
- tltt_definitions
- fabric_nodes
- correlation_states
- portfolios
- portfolio_positions
- portfolio_snapshots
- factor_exposures

#### Task 2: B-02 RUN_MIGRATION
**Owner:** Reviewer
**Actions:**
1. Execute migration against dev DB
2. Verify all tables created
3. Verify constraints applied
4. Verify indexes created

#### Task 3: B-03 VERIFY_SCHEMA_TABLES
**Owner:** Reviewer
**Actions:**
1. Query information_schema
2. Assert all tables exist
3. Assert correct columns and types
4. Assert constraints present

#### Task 4: B-04 VERIFY_IMMUTABILITY
**Owner:** Reviewer
**Actions:**
1. INSERT into agent_queries
2. Attempt UPDATE on agent_queries
3. Attempt DELETE on agent_queries
4. Assert both fail with exception
5. Repeat for trade_events

**Success Criteria:**
- All tables exist with correct schema
- Immutable tables reject UPDATE/DELETE
- Audit views return data

**Estimated Time:** 3 hours

---

## 7. STEP 5: CS-1.2 AUDIT VIEW & HASH CHAIN

### Objective: Implement audit trail with hash chain

**Prerequisites:** CS-1.1 DONE

**Task Sequence:**

#### Task 1: B-05 VERIFY_AUDIT_VIEW
**Owner:** AuditAgent
**Actions:**
1. Insert sample trade chain
2. Query v_audit_trade_full
3. Assert completeness of row
4. Verify all required fields present

#### Task 2: B-08 VERIFY_HASH_CHAIN
**Owner:** AuditAgent
**Actions:**
1. Insert 3 trade events
2. Query prev_hash chain
3. Assert chain unbroken
4. Verify SHA-256 computation correct

#### Task 3: B-09 WRITE_REGULATORY_FIELD_MAP
**Owner:** AuditAgent
**Actions:**
1. Create docs/REGULATORY_CONTROL_MAP.md
2. Map CIRO/OSC fields to schema
3. Map SEC fields to schema
4. Map FINRA fields to schema
5. Document UMIR 10.11 requirements

**Success Criteria:**
- Audit view returns complete row
- Hash chain verified for 3 inserts
- Regulatory field map documented

**Estimated Time:** 2 hours

---

## 8. STEP 6: CS-1.3 SEED TABLES

### Objective: Seed TLTT definitions and fabric nodes

**Prerequisites:** CS-1.1 DONE

**Task Sequence:**

#### Task 1: B-06 SEED_TLTT_DEFINITIONS
**Owner:** Router
**Actions:**
1. Insert vertical gate rows
2. Insert horizontal gate rows
3. Insert diagonal gate rows
4. Verify ≥14 rows present
5. Verify gate criteria populated

**TLTT Gates Required (14+):**
- EQ_ENTRY_VALID (vertical)
- PORTFOLIO_RISK_OK (vertical)
- LIQUIDITY_OK (vertical)
- CROSS_ASSET_EQ_BOND (diagonal)
- CROSS_ASSET_FX_COMMODITY (diagonal)
- And 9 more

#### Task 2: B-07 SEED_FABRIC_NODES
**Owner:** Router
**Actions:**
1. Insert container nodes
2. Insert splice nodes
3. Insert trace nodes
4. Verify all TraderX domains present
5. Verify node relationships

**Success Criteria:**
- tltt_definitions has ≥14 rows
- fabric_nodes has all TraderX containers
- All node types present

**Estimated Time:** 1 hour

---

## 9. STEP 7: CS-2.1 BAM GENESIS LOCK

### Objective: Establish BAM system with genesis hash

**Prerequisites:** CS-1.3 DONE

**Task Sequence:**

#### Task 1: C-01 LOAD_BAM_TABLE
**Owner:** Router
**Actions:**
1. Call load_traderx_bam_table()
2. Confirm all domain codes present
3. Verify BAM registry loaded

#### Task 2: C-02 COMPUTE_GENESIS_HASH
**Owner:** Router
**Plan Required:** Yes
**Actions:**
1. Generate canonical JSON
2. Compute SHA-256 hash
3. Write to .env
4. Verify hash format

#### Task 3: C-03 VERIFY_BAM_INTEGRITY
**Owner:** Router
**Actions:**
1. Call verify_integrity()
2. Assert VERIFIED response
3. Log integrity check result

#### Task 4: C-04 VERIFY_BAM_MUTATION_HALT
**Owner:** Reviewer
**Actions:**
1. Mutate a test code
2. Re-verify integrity
3. Assert VIOLATION response
4. Assert 503 response code
5. Restore canonical state

**Success Criteria:**
- Genesis hash locked in .env
- verify_integrity() returns VERIFIED
- Mutation causes VIOLATION + 503

**Estimated Time:** 2 hours

---

## 10. STEP 8: CS-2.2 FABRIC REGISTRY & DUAL KEY

### Objective: Build fabric registry and dual key system

**Prerequisites:** CS-2.1 DONE

**Task Sequence:**

#### Task 1: C-07 BUILD_FABRIC_REGISTRY
**Owner:** Router
**Actions:**
1. Populate all TraderX containers
2. Populate all splice nodes
3. Populate all trace nodes
4. Verify registry completeness

#### Task 2: C-05 ASSEMBLE_DUAL_KEY
**Owner:** Router
**Actions:**
1. For 3+ domain nodes, generate dual_key
2. Validate signal_hex::bam_raw format
3. Verify dual_key uniqueness

#### Task 3: C-06 VERIFY_DUAL_KEY_ROUNDTRIP
**Owner:** Reviewer
**Actions:**
1. Decode dual_key back to fabric signal
2. Decode dual_key back to BAM compound
3. Assert equality with original
4. Verify roundtrip preserves data

#### Task 4: C-08 WRITE_MCP_BAM_TOOLS
**Owner:** Router
**Actions:**
1. Author MCP server tool definitions
2. Define encode tool
3. Define decode tool
4. Define dual_signal tool
5. Define hotpaths tool
6. Define verify tool

**Success Criteria:**
- Fabric registry complete
- Dual keys validate for ≥3 domains
- Roundtrip preserves data
- MCP tools respond correctly

**Estimated Time:** 2 hours

---

## 11. STEP 9: CS-2.3 TLTT ENGINE & GOLDEN TESTS

### Objective: Implement TLTT engine with golden test scenarios

**Prerequisites:** CS-1.3 DONE

**Task Sequence:**

#### Task 1: C-09 TEST_REDIS_HOT_PATH
**Owner:** Signal
**Actions:**
1. Simulate 10K+ calls on one compound
2. Assert alias expands
3. Assert alias retrievable
4. Measure performance

#### Task 2: C-10 WRITE_PULSE_TRACE_FORMATTER
**Owner:** Router
**Actions:**
1. Author binary-Morse overlay formatter
2. Test with sample traces
3. Verify output format
4. Integrate with logging

#### Task 3: D-01 IMPLEMENT_TLTT_ENGINE
**Owner:** Router
**Actions:**
1. Create app/fabric/tltt.py
2. Implement evaluate() function
3. Implement evaluate_all() function
4. Add gate criteria logic

#### Task 4: D-02 WRITE_GOLDEN_SCENARIOS
**Owner:** TeamLead
**Actions:**
1. Create tests/golden/scenarios.json
2. Define all required gate scenarios
3. Define expected pass/fail outcomes
4. Add edge cases

#### Task 5: D-03 RUN_GOLDEN_TESTS
**Owner:** Reviewer
**Actions:**
1. Execute golden test suite
2. Assert all scenarios produce expected outcomes
3. Verify diagonal gate requires prior state
4. Document any failures

#### Task 6: D-05 VERIFY_VERTICAL_GATE
**Owner:** Reviewer
**Actions:**
1. Run vertical TLTT test
2. Assert correct outcome
3. Test with valid input
4. Test with invalid input

#### Task 7: D-06 VERIFY_HORIZONTAL_GATE
**Owner:** Reviewer
**Actions:**
1. Run horizontal TLTT test
2. Assert enrichment criteria satisfied
3. Test with sufficient enrichment
4. Test with insufficient enrichment

#### Task 8: D-07 VERIFY_DIAGONAL_GATE
**Owner:** Reviewer
**Actions:**
1. Run diagonal TLTT test
2. Assert cross-asset activation only when prior confirmed
3. Test with prior confirmed
4. Test without prior confirmed

#### Task 9: D-08 VERIFY_TLTT_BLOCK
**Owner:** Reviewer
**Actions:**
1. Trigger failed gate
2. Verify downstream action blocked
3. Verify HALT signal emitted
4. Verify full envelope logged

**Success Criteria:**
- Redis hot path works (10K+ calls)
- TLTT engine evaluates gates correctly
- All golden scenarios pass
- Diagonal gate requires prior state
- Failed gate blocks downstream action

**Estimated Time:** 3 hours

---

## 12. REVIEWER VERIFICATION PROCESS

### After Each Task:

1. **Run tests_required** from task object
2. **Run lint_required** from task object
3. **Assert done_definition** condition met
4. **If fail:** Emit TASK_REVIEW_FAIL, return to owner
5. **If pass:** Emit TASK_REVIEW_PASS, mark DONE

### After Each Chunk Session:

1. **Run full test suite** from cold state
2. **Verify all exit criteria** met
3. **Generate proof artifact** in proofs/ directory
4. **Update tasks.json** with completion status
5. **Register completion** in structured log

---

## 13. HOOK TABLE ENFORCEMENT

### Blocking Hooks:

| Hook Event | Required Action | Status |
|------------|----------------|--------|
| TaskCompleted | Run tests + lint | WILL ENFORCE |
| PlanSubmitted | Block execution, route to approver | WILL ENFORCE |
| PreCommit | ruff check && pytest tests | WILL ENFORCE |
| BAMViolation | HALT, page TeamLead, block dispatch | WILL ENFORCE |
| TLTTFail | Block downstream, log with envelope | WILL ENFORCE |
| ImmutableWriteAttempt | Raise exception, alert TeamLead | WILL ENFORCE |
| EnvelopeInvalid | Reject message, log | WILL ENFORCE |

---

## 14. TOTAL ESTIMATED TIME

| Step | Duration | Cumulative |
|------|----------|------------|
| Dependency Installation | 20 min | 20 min |
| CS-0.0 Task Management | 30 min | 50 min |
| CS-0.1 Repo Foundation | 2 hours | 2.5 hours |
| CS-1.1 Schema Migration | 3 hours | 5.5 hours |
| CS-1.2 Audit View & Hash Chain | 2 hours | 7.5 hours |
| CS-1.3 Seed Tables | 1 hour | 8.5 hours |
| CS-2.1 BAM Genesis Lock | 2 hours | 10.5 hours |
| CS-2.2 Fabric Registry & Dual Key | 2 hours | 12.5 hours |
| CS-2.3 TLTT Engine & Golden Tests | 3 hours | 15.5 hours |

**Total Estimated Time:** 15.5 hours

---

## 15. NEXT IMMEDIATE ACTION

**Current Status:** Capabilities enhanced with 8 MCP servers
**Ready to Begin:** Step 1 - Dependency Installation

**Command to Execute:**
```bash
brew install rustup && rustup install 1.70.0 && rustup default 1.70.0
```

**Awaiting User Approval to Begin Execution.**

---

**Plan prepared by:** Self-audit  
**Date:** 2026-05-01  
**Status:** READY FOR EXECUTION
