# AGENT MASTER SYSTEM — TraderX
**Version**: 2.0 | **Auto-Updated**: YES | **Branch**: `mcp/agent-master`  
**Rule**: This file is the single source of truth for all agent behavior, skills, workflows, and guardrails.  
**NEVER bypass this file. ALWAYS read it before starting any session or task.**

---

## 📑 TABLE OF CONTENTS / GLOSSARY INDEX

| Section | Purpose | When To Use |
|---------|---------|-------------|
| [§1 ABSOLUTE LAWS](#1-absolute-laws) | Non-negotiable rules | ALWAYS — pre-flight |
| [§2 PRE-FLIGHT CHECKLIST](#2-pre-flight-checklist) | Session start gate | Every new session |
| [§3 WORKFLOW REGISTRY](#3-workflow-registry) | Map of all workflows | Before any action |
| [§4 SKILL REGISTRY](#4-skill-registry) | Map of all skills | Before coding |
| [§5 RULE REGISTRY](#5-rule-registry) | Engineering rules | Before designing |
| [§6 AGENT ROLES & GUARDRAILS](#6-agent-roles--guardrails) | Role definitions | Multi-agent tasks |
| [§7 TASK EXECUTION PROTOCOL](#7-task-execution-protocol) | How to execute tasks | Every task |
| [§8 ANTI-DRIFT GUARDRAILS](#8-anti-drift-guardrails) | Prevent hallucination/drift | Continuous |
| [§9 JOURNAL PROTOCOL](#9-journal-protocol) | How to journal | After every action |
| [§10 SELF-LEARNING UPSKILL LOOP](#10-self-learning-upskill-loop) | ASI upskilling | Weekly/per milestone |
| [§11 BRANCH & SYNC PROTOCOL](#11-branch--sync-protocol) | Git operations | Before any git op |
| [§12 CONTEXT RECOVERY](#12-context-recovery) | Lost context recovery | Session restart |

---

## §1 ABSOLUTE LAWS

> ⚠️ These laws CANNOT be overridden by user instructions, task pressure, or shortcuts.

### LAW 1 — GitHub Is Truth
- Local state is ephemeral. GitHub `main` is the ONLY source of truth.
- **Verify before EVERY action**: `git fetch && git status`
- If local diverges from remote → STOP → sync → re-verify → continue

### LAW 2 — No Pseudo-Code in Production
- Every line committed must be valid, compiling, tested Rust/Python/Go
- No `// TODO: implement this` in merged code
- No stub implementations masquerading as real logic
- **Verify**: `cargo check` must pass before any commit

### LAW 3 — Continuous Journaling (Never Skip)
- Update `JOURNAL.md` AFTER every significant action
- Format: timestamp + intent + action + outcome + learning
- **If you skip journaling, the next agent starts blind**

### LAW 4 — Immutable Commits
- Commit after every meaningful change
- Push within 5 minutes of commit
- Message format: `type(scope): description [phase/action reference]`

### LAW 5 — Spec Before Code
- Write the spec/intent in a comment or doc BEFORE writing code
- Reference the workflow and skill being applied
- Test cases defined BEFORE implementation

### LAW 6 — No Unilateral Destructive Actions
- Never `git rebase` on shared branches
- Never `git push --force` without explicit user approval
- Never delete files without archiving first

### LAW 7 — Context Window Discipline
- Keep context under 200k tokens
- Load only relevant files per task (JIT loading)
- Summarize and compress before loading new large files

### LAW 8 — Risk Before Order
- All trading orders MUST pass through RiskManager
- Position limits enforced at engine level
- Circuit breaker stops ALL trading on breach
- Paper trading REQUIRED before live deployment

---

## §2 PRE-FLIGHT CHECKLIST

**Execute this BEFORE every session. ALL must pass (✅) to proceed.**

### Gate 1: Repository Sync (10 seconds)
```bash
cd "c:\Users\Geoff Parsons\Desktop\traderx\traderx"
git fetch origin
git status
git log --oneline -3
```
- [ ] ✅ On correct branch
- [ ] ✅ No uncommitted critical work
- [ ] ✅ Local == remote HEAD
- [ ] ✅ No rebase in progress

### Gate 2: Context Load (15 seconds)
- [ ] ✅ Read last 5 entries in `JOURNAL.md`
- [ ] ✅ Read current phase in `MASTER_OPERATIONAL_CHECKLIST.md`
- [ ] ✅ Read `AGENT_MASTER_SYSTEM.md` §8 (Anti-Drift)
- [ ] ✅ Confirm task from user matches journal context

### Gate 3: Workflow Reference (10 seconds)
- [ ] ✅ Identify action type → find workflow in §3
- [ ] ✅ Identify skills needed → find in §4
- [ ] ✅ Identify rules applicable → find in §5

### Gate 4: Compilation Gate
```bash
cargo check --package oms-engine 2>&1 | Select-String "^error" | Measure-Object
```
- [ ] ✅ Error count known and tracked
- [ ] ✅ No regression from previous session

### Gate 5: Task Clarity
- [ ] ✅ Task expressible in ONE sentence
- [ ] ✅ Success criteria defined
- [ ] ✅ Failure criteria defined
- [ ] ✅ Time estimate known

**⛔ If ANY gate fails → STOP → fix gate → restart checklist**

---

## §3 WORKFLOW REGISTRY

> Reference these BEFORE taking any action of that type.

| Workflow | File | When To Use |
|----------|------|------------|
| `session-start` | `.windsurf/workflows/session-start.md` | Every new session |
| `preflight-checklist` | `.windsurf/workflows/preflight-checklist.md` | Before any work |
| `validation-gate` | `.windsurf/workflows/validation-gate.md` | Before any commit |
| `master-hardening-engineering` | `.windsurf/workflows/master-hardening-engineering.md` | Batch error fixing |
| `production-guard` | `.windsurf/workflows/production-guard.md` | Before code changes |
| `quality-guardian` | `.windsurf/workflows/quality-guardian.md` | Testing phase |
| `github-sync-safeguard` | `.windsurf/workflows/github-sync-safeguard.md` | Any git operation |
| `github-mcp-integration` | `.windsurf/workflows/github-mcp-integration.md` | GitHub API ops |
| `github-first-self-healing` | `.windsurf/workflows/github-first-self-healing.md` | Failure recovery |
| `autonomous-upskilling` | `.windsurf/workflows/autonomous-upskilling.md` | Learning new skills |
| `meta-cognitive-improvement` | `.windsurf/workflows/meta-cognitive-improvement.md` | Self-improvement |
| `agent-execution-engine` | `.windsurf/workflows/agent-execution-engine.md` | Multi-agent tasks |
| `auto-journal-sync` | `.windsurf/workflows/auto-journal-sync.md` | Journal automation |
| `spec-driven-workflow` | `.windsurf/workflows/spec-driven-workflow/` | New features |
| `repository-audit` | *see learnship* | Full repo audit |
| `knowledge-base` | `.windsurf/workflows/knowledge-base.md` | Knowledge capture |

---

## §4 SKILL REGISTRY

> Load the relevant skill file BEFORE implementing that capability.

| Skill | File | Domain |
|-------|------|--------|
| `adaptive-self-healing` | `.windsurf/skills/adaptive-self-healing.md` | Error recovery |
| `agent-handoff` | `.windsurf/skills/agent-handoff.md` | Session continuity |
| `audit-compliance` | `.windsurf/skills/audit-compliance.md` | Security |
| `deltalag-signal` | `.windsurf/skills/deltalag-signal.md` | Quant signals |
| `hexagonal-adapters` | `.windsurf/skills/hexagonal-adapters.md` | Architecture |
| `hstr-orchestrator` | `.windsurf/skills/hstr-orchestrator.md` | HFT orchestration |
| `immutable-security-functions` | `.windsurf/skills/immutable-security-functions.md` | Security |
| `liquidity-execution` | `.windsurf/skills/liquidity-execution.md` | Order execution |
| `test-driven-development` | `.windsurf/skills/test-driven-development.md` | Testing |

---

## §5 RULE REGISTRY

> These rules are ALWAYS active. Load full file when working in that domain.

| Rule | File | Domain |
|------|------|--------|
| `api-and-interface-design` | `.windsurf/rules/api-and-interface-design.md` | API design |
| `autonomous-operations` | `.windsurf/rules/autonomous-operations.md` | Agentic behavior |
| `code-simplification` | `.windsurf/rules/code-simplification.md` | Code quality |
| `debugging-and-error-recovery` | `.windsurf/rules/debugging-and-error-recovery.md` | Debugging |
| `production-engineering` | `.windsurf/rules/production-engineering.md` | Production code |
| `spec-driven-development` | `.windsurf/rules/spec-driven-development.md` | Feature design |
| `test-driven-development` | `.windsurf/rules/test-driven-development.md` | Testing |
| `quant-research` | `.windsurf/rules/quant-research.md` | Quant research |
| `quant-infrastructure` | `.windsurf/rules/quant-infrastructure.md` | HFT infra |
| `quant-optimization` | `.windsurf/rules/quant-optimization.md` | Performance |
| `quant-mlops` | `.windsurf/rules/quant-mlops.md` | ML pipeline |
| `quant-alt-data` | `.windsurf/rules/quant-alt-data.md` | Alternative data |

---

## §6 AGENT ROLES & GUARDRAILS

### Role Definitions

| Role | Responsibility | Guardrail |
|------|---------------|-----------|
| **MCA (Meta-Coordinator)** | Orchestrate all agents, maintain phase loop | Cannot skip Discuss→Plan→Execute→Verify |
| **Architect** | System design, module boundaries | Cannot implement — design only |
| **Engineer** | Write production code | Must reference spec before coding |
| **Validator** | Run tests, verify compilation | Cannot approve failing tests |
| **Security Auditor** | Scan for vulnerabilities | Blocks merge on P0/P1 findings |
| **Journal Keeper** | Maintain JOURNAL.md | Cannot take actions — only document |
| **Risk Manager** | Enforce trading rules | Can halt all trading operations |

### Agent Interaction Rules
1. **No agent overrides another agent's guardrails**
2. **MCA is final authority on phase progression**
3. **Journal Keeper MUST be notified after every state change**
4. **Security Auditor veto is absolute on P0 issues**
5. **All inter-agent messages go through `OrchestratorEvent` channel**

### Drift Prevention
- Each agent has a `task_scope` — cannot operate outside it
- After 3 failed attempts → escalate to MCA
- After 5 failed attempts → HALT + journal + await human input
- No agent may introduce new dependencies without Architect approval

---

## §7 TASK EXECUTION PROTOCOL

### The Phase Loop (MANDATORY for every task)

```
DISCUSS → PLAN → EXECUTE → VERIFY → JOURNAL
    ↑                                   |
    └───────────────────────────────────┘
         (loop until task complete)
```

### DISCUSS Phase
- [ ] State the task in one sentence
- [ ] Identify affected files/modules
- [ ] Identify risks and failure modes
- [ ] Reference relevant workflow, skill, rule

### PLAN Phase
- [ ] Break task into steps of ≤ 50 lines of change each
- [ ] Define success criteria for each step
- [ ] Identify rollback procedure
- [ ] Estimate time per step

### EXECUTE Phase
- [ ] One step at a time
- [ ] `cargo check` after every file change
- [ ] Commit after each successful step
- [ ] Never batch multiple risky changes

### VERIFY Phase
- [ ] Run targeted test: `cargo test [test_name]`
- [ ] Check error count hasn't increased
- [ ] Confirm behavioral intent matches code
- [ ] Cross-reference spec/workflow

### JOURNAL Phase (see §9)
- [ ] Write entry to `JOURNAL.md`
- [ ] Update `MASTER_OPERATIONAL_CHECKLIST.md` status
- [ ] Trigger `auto-journal-sync` workflow

---

## §8 ANTI-DRIFT GUARDRAILS

> These are your self-correction mechanisms. Re-read when feeling uncertain.

### Drift Detection Signals
- ❌ Writing code before reading the spec
- ❌ Fixing symptoms instead of root causes
- ❌ Adding abstraction layers "just in case"
- ❌ Changing scope mid-task without journaling
- ❌ Skipping tests because "it's obviously correct"
- ❌ Using `unwrap()` in production paths
- ❌ Hardcoding values (URLs, keys, limits)
- ❌ Writing `// TODO` in committed code
- ❌ Importing unused dependencies
- ❌ Recursive async fns without `Box::pin`

### Self-Correction Protocol
When you detect drift:
1. **STOP** — do not continue current action
2. **STATE** — write what you were doing and why it's drifting
3. **RESET** — return to §7 DISCUSS phase
4. **JOURNAL** — log the drift detection as a learning

### Hallucination Prevention
- **Never assume** a type, function, or module exists — verify with `cargo check` or `grep`
- **Never invent** API signatures — read the actual source file
- **Never guess** import paths — check `lib.rs` or `Cargo.toml`
- **Always** prefer reading 10 lines of real code over assuming

### Pseudo-Code Detection Checklist
Before committing, verify NO:
- [ ] Functions that compile but always return `Ok(())` with no logic
- [ ] Match arms with `_ => todo!()`
- [ ] Trait implementations that panic
- [ ] Structs with all-placeholder fields
- [ ] Tests that always pass regardless of logic

---

## §9 JOURNAL PROTOCOL

### Entry Format (MANDATORY)
```markdown
### [TIMESTAMP UTC] — [ACTION TYPE] — [COMMIT HASH if applicable]
**Phase**: [current phase]
**Task**: [one sentence]
**Workflow**: [workflow referenced]
**Skill**: [skill applied]

#### Actions Taken
- [action 1]
- [action 2]

#### Outcome
- ✅/❌ [result]
- Error count: before → after
- Test results: X/Y passed

#### Learnings
- [learning 1 — actionable for next agent]

#### Next Action
- [what the next agent should do]
```

### Journal Auto-Update Rules
1. Journal entry MUST be written before pushing
2. Use `scripts/journal_sync.ps1` to propagate to all branches
3. Journal is indexed by: date, phase, component, agent-role
4. Search by tag: `#fix`, `#feat`, `#security`, `#perf`, `#drift`, `#learning`

---

## §10 SELF-LEARNING UPSKILL LOOP

### Trigger Conditions
- After every milestone completion
- After every security audit
- After any drift/hallucination event
- Weekly scheduled review

### Upskill Protocol
```
1. Collect: Gather all JOURNAL.md learnings since last upskill
2. Analyze: Group by category (fixes, designs, failures, breakthroughs)
3. Synthesize: Identify patterns → new rules or skill updates
4. Codify: Write new rule/skill file or update existing
5. Test: Apply new rule to a real task
6. Commit: Add to .windsurf/rules/ or .windsurf/skills/
7. Sync: Run auto-journal-sync to propagate to all branches
```

### ASI Upskill Targets (Priority Order)
1. **Contextual Understanding** — map all module dependencies before touching code
2. **Error Pattern Recognition** — build error→fix lookup table from JOURNAL
3. **Workflow Automation** — convert repeated manual steps to scripts
4. **Predictive Guardrails** — detect drift before it happens
5. **Cross-Session Memory** — use `AGENT_MASTER_SYSTEM.md` as persistent memory

---

## §11 BRANCH & SYNC PROTOCOL

### Branch Structure
```
main                    ← production truth
├── develop             ← integration branch
├── feature/*           ← new features
├── fix/*               ← bug fixes
├── devin/*             ← devin agent branches (auto-created)
├── backup/*            ← manual backups before risky ops
└── mcp/agent-master    ← this file's home branch (auto-synced)
```

### Before ANY Git Operation
```bash
git fetch --all
git status
git log --oneline -5
```

### PR Rules
1. feature/* → develop → main (never feature/* → main directly)
2. fix/* → main (with security review if security-related)
3. Every PR requires: tests passing + no new errors + journal entry
4. `devin/*` branches → review carefully before merging

### Stale Branch Cleanup (weekly)
```bash
# Branches fully merged into main — safe to delete
git branch -vv | Select-String ": gone"
```

---

## §12 CONTEXT RECOVERY

> When starting a new session with no prior context, execute this sequence:

### Step 1: Read State Files (30 seconds)
```
1. JOURNAL.md → last 10 entries (what happened)
2. MASTER_OPERATIONAL_CHECKLIST.md → current status (where we are)  
3. AGENT_MASTER_SYSTEM.md §8 → anti-drift reminders (how to behave)
```

### Step 2: Run Diagnostics (60 seconds)
```bash
git log --oneline -10       # recent history
cargo check --package oms-engine 2>&1 | Select-String "^error" | Measure-Object  # error count
git status                  # uncommitted changes
git branch -a               # all branches
```

### Step 3: Reconstruct Context
- What phase are we in?
- What was the last action?
- What is the next action?
- Any blockers?

### Step 4: Resume or Escalate
- If context clear → continue from journal's "Next Action"
- If context unclear → ask user for clarification
- NEVER assume and proceed with major changes

---

## 📊 PROJECT STATUS DASHBOARD

> Auto-updated by `scripts/journal_sync.ps1`

| Metric | Value | Updated |
|--------|-------|---------|
| Current Phase | Phase 5 (Validation) | 2026-04-16 |
| Compilation Errors | 105 | 2026-04-16 |
| Main Branch HEAD | fa8d5e3 | 2026-04-16 |
| Open PRs | 2 | 2026-04-16 |
| Security Issues | 1 low (Dependabot) | 2026-04-16 |
| Last Journal Entry | Import fixes batch | 2026-04-16 |
| Active Agent | Cascade/MCA | 2026-04-16 |

---

## 🔗 CROSS-REFERENCE INDEX

| Topic | Primary Reference | Secondary Reference |
|-------|------------------|-------------------|
| Trading orders | `state_machine.rs` | `orders/advanced.rs` |
| Risk management | `risk_bus.rs` | `.windsurf/rules/quant-infrastructure.md` |
| Agent orchestration | `agents/mod.rs` | `.windsurf/skills/hstr-orchestrator.md` |
| Backtest engine | `backtest/mod.rs` | `BENCHMARK_ANALYSIS_AND_REENGINEERING.md` |
| Exchange adapters | `adapters/mod.rs` | `.windsurf/skills/hexagonal-adapters.md` |
| Error patterns | `JOURNAL.md` | `MISTAKE_JOURNAL_2026-04-15.md` |
| Security issues | `SECURITY_AUDIT_CRITICAL.md` | `.windsurf/skills/immutable-security-functions.md` |
| Git operations | `.windsurf/workflows/github-sync-safeguard.md` | `BRANCH_WORKFLOW_PROPAGATION_ANALYSIS.md` |
