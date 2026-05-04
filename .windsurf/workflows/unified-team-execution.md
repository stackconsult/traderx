# Unified Team Execution Workflow

## Description
Master workflow that unifies all specialist teams under a single systematic protocol. Every team executes through the same action function set, with team-specific implementations within each phase. Guarantees safe, predictable, non-overloaded execution across all work types.

## Unified Team Registry

| Team | Skill File | Specialty | Unified Role |
|------|-----------|-----------|-------------|
| **Agent Safety** | `agent-execution-safety.md` | Prevents overload, manages cycles | Cycle Controller |
| **QA Team** | `qa-team.md` | Multi-lens review (6 lenses) | Quality Gatekeeper |
| **Product Manager** | `product-manager.md` | Prioritization, coordination | Sprint Director |
| **Issue Resolver** | `issue-pattern-resolution.md` | Recurring issue detection/fix | Pattern Analyst |
| **Build Engineer** | `build-system-engineer.md` | Compilation, optimization | Build Specialist |
| **Git Specialist** | `git-workflow-specialist.md` | Sync, commit, push | Version Control |
| **Workspace Architect** | `workspace-architect.md` | Workspace reorganization | Structure Designer |
| **Feature Flag Engineer** | `feature-flag-engineer.md` | Conditional compilation | Scope Manager |
| **Module Splitter** | `module-splitter.md` | Monolithic breakdown | Refactoring Specialist |
| **Tooling Specialist** | `tooling-specialist.md` | Dev tools, benchmarking | Tool Manager |

---

## Unified Action Function Set

All 10 teams execute through the same 6 action functions. Each team implements their specialty within these functions.

### Action Function 1: `DIAGNOSE()`
**Purpose:** Assess current state before any work begins
**Mandatory for all teams**

```
DIAGNOSE() {
  1. What is the ONE thing I am trying to do?
  2. What is my confidence level (0-10)?
  3. How many attempts have I already made?
  4. Is the context clean or bloated?
  5. Which team specialty applies? (see registry above)
  6. What is the current project state? (run state check)
}
```

**Team-Specific State Checks:**
- **Agent Safety:** Context attempt count, bloat detection
- **QA Team:** Run `cargo check`, count errors/warnings, identify files to review
- **PM:** Run `git status`, check todo list status, identify blockers
- **Issue Resolver:** `git log --oneline -20`, `cargo check` error patterns
- **Build Engineer:** `time cargo check`, `cargo check 2>&1 | grep "^error" | wc -l`
- **Git Specialist:** `git status --short`, `git log --oneline -3`, `git fetch origin`
- **Workspace Architect:** `cargo metadata`, `find src -name "*.rs" -exec wc -l {} +`
- **Feature Flag:** `grep -r "cfg(feature" src/`, check Cargo.toml [features]
- **Module Splitter:** `find src -name "*.rs" -exec wc -l {} + | sort -n | tail -5`
- **Tooling:** `which cargo-nextest`, `which cargo-watch`, `which sccache`

**Abort Criteria (All Teams):**
- Confidence < 3 after 2+ attempts → STOP, research or escalate
- Context > 10 failed attempts → Commit current state, start fresh cycle
- Multiple unrelated tasks mixed → Split into separate DIAGNOSE cycles

---

### Action Function 2: `PLAN()`
**Purpose:** Define the single task, approach, and success criteria
**Mandatory for all teams**

```
PLAN() {
  1. State the ONE specific task (one sentence)
  2. Define approach (research / code / config / docs)
  3. Set time limit (15-30 min max)
  4. Define verification method (cargo check / test / git status / etc.)
  5. Identify which team will execute next (if handoff needed)
  6. State expected outcome in one sentence
}
```

**Team-Specific Planning:**
- **Agent Safety:** Define cycle parameters, set abort criteria
- **QA Team:** Select lens (correctness/testing/security/performance/maintainability/adversarial)
- **PM:** Prioritize tasks, assign to teams, set sprint structure
- **Issue Resolver:** Identify pattern type (A=compilation, B=git, C=build perf, D=disk)
- **Build Engineer:** Select optimization (profile/dependency/cache)
- **Git Specialist:** Select action (commit/push/rebase/resolve conflict)
- **Workspace Architect:** Define workspace split (core/hft/ai)
- **Feature Flag:** Select feature scope and conditional compilation targets
- **Module Splitter:** Identify split point and API compatibility strategy
- **Tooling:** Select tool to install/configure and benchmark method

---

### Action Function 3: `EXECUTE()`
**Purpose:** Perform the single planned task
**Mandatory for all teams**

```
EXECUTE() {
  1. Work on ONE thing only
  2. One file or one logical change at a time
  3. Max 30 minutes
  4. No batching (never >2 new files, never >1 error fix without check)
  5. If stuck after 2 attempts → switch approach
  6. If stuck after 3 attempts → ABORT cycle, do not guess
}
```

**Team-Specific Execution Patterns:**
- **Agent Safety:** Enforce cycle boundaries, monitor for overload symptoms
- **QA Team:** Apply selected lens, read files, produce structured findings (P0-P3)
- **PM:** Update todo list, assign next team, track metrics
- **Issue Resolver:** Apply pattern-specific fix, document solution
- **Build Engineer:** Edit Cargo.toml profiles, run cargo fix, configure cache
- **Git Specialist:** Stage, commit with proper format, push, verify remote
- **Workspace Architect:** Edit Cargo.toml workspaces, move modules, update imports
- **Feature Flag:** Add `#[cfg(feature = "...")]` gates, update Cargo.toml features
- **Module Splitter:** Extract functions to new file, re-export for API compat
- **Tooling:** Run cargo install, configure environment, set up hooks

---

### Action Function 4: `VERIFY()`
**Purpose:** Confirm the task completed successfully
**Mandatory for all teams**

```
VERIFY() {
  1. Did the change work? (run verification command)
  2. Is the state clean? (git status, no unverified changes)
  3. Can I explain what I did in one sentence?
  4. Were there any regressions?
  5. Should I commit now before proceeding?
}
```

**Team-Specific Verification:**
- **Agent Safety:** Check cycle time <30 min, attempt count, context state
- **QA Team:** Verify findings have severity (P0-P3) and confidence (0.0-1.0)
- **PM:** Verify todo list updated, metrics tracked, blockers resolved
- **Issue Resolver:** Verify pattern resolved, no recurrence, documented
- **Build Engineer:** `cargo check` passes, build time measured, warnings counted
- **Git Specialist:** `git status` clean, `git log` shows commit, remote synced
- **Workspace Architect:** `cargo check` passes, workspace builds, no circular deps
- **Feature Flag:** `cargo check --no-default-features --features X` passes for each feature
- **Module Splitter:** `cargo check` passes, all modules <10KB, API compatible
- **Tooling:** Tool installed and functional, benchmark shows improvement

**Verification Failure Protocol (All Teams):**
- 1st failure: Try ONE different approach
- 2nd failure: STOP and research the API/library/docs
- 3rd failure: ABORT cycle. Report to user. Do not guess.

---

### Action Function 5: `COMMIT()`
**Purpose:** Save verified changes with traceability
**Mandatory for all teams after every successful cycle**

```
COMMIT() {
  1. Stage only verified changes: git add [specific files]
  2. Write commit message: "type(scope): description"
  3. Verify commit: git log --oneline -1
  4. Push to remote: git push origin [branch]
  5. Verify remote: git log --oneline origin/[branch] -1
}
```

**Team-Specific Commit Types:**
- **Agent Safety:** `docs(safety):` or `feat(safety):`
- **QA Team:** `docs(qa):` or `fix(qa):`
- **PM:** `docs(pm):` or `chore(pm):`
- **Issue Resolver:** `fix(issue):` or `docs(issue):`
- **Build Engineer:** `fix(build):` or `perf(build):`
- **Git Specialist:** `chore(git):` or `fix(git):`
- **Workspace Architect:** `refactor(workspace):` or `feat(workspace):`
- **Feature Flag:** `feat(flags):` or `config(flags):`
- **Module Splitter:** `refactor(module):` or `chore(module):`
- **Tooling:** `chore(tools):` or `feat(tools):`

---

### Action Function 6: `HANDOFF()`
**Purpose:** Pass execution to next team or declare cycle complete
**Mandatory for all teams**

```
HANDOFF() {
  1. State what was accomplished in one sentence
  2. Identify next team needed (or "cycle complete")
  3. Pass relevant context (files modified, findings, metrics)
  4. Update todo list: mark current task done, set next task
  5. If cycle complete → Start new DIAGNOSE for next priority
}
```

**Team Handoff Matrix:**

| From Team | Typical Next Team | Handoff Context |
|-----------|------------------|-----------------|
| **Agent Safety** | Any team needing cycle enforcement | Cycle parameters, abort criteria |
| **QA Team** | Issue Resolver (for P0 findings) | P0 findings with file:line references |
| **PM** | Any team assigned to next task | Task priority, success metrics, timeline |
| **Issue Resolver** | Build Engineer (for compilation) | Pattern type, root cause, fix applied |
| **Build Engineer** | QA Team (for regression check) | Build metrics before/after |
| **Git Specialist** | PM (for status update) | Commit SHA, branch state |
| **Workspace Architect** | Feature Flag Engineer | Workspace structure, split points |
| **Feature Flag** | Module Splitter | Feature scopes, conditional targets |
| **Module Splitter** | Build Engineer | Module sizes, API changes, import updates |
| **Tooling** | QA Team (for benchmark validation) | Tool versions, performance metrics |

---

## Unified Execution Cycle Template

Every team uses this exact template for every cycle:

```
=== CYCLE START ===
Team: [Team Name]
Task: [ONE specific thing]
Confidence: [X/10]
Attempt: [# of this cycle]

[DIAGNOSE] → [PLAN] → [EXECUTE] → [VERIFY] → [COMMIT] → [HANDOFF]

Verification: [PASS / FAIL]
If FAIL: [1st retry / 2nd research / 3rd abort]

Commit: "type(scope): [description]"
Push: git push origin [branch]

Next: [Team Name] → [Task Description]
=== CYCLE END ===
```

---

## Sprint Structure (PM Team Rule)

```
SPRING = 1-3 Cycles (max 90 minutes total)

Cycle 1: Team A executes (30 min max)
  → DIAGNOSE → PLAN → EXECUTE → VERIFY → COMMIT → HANDOFF

Cycle 2: Team B executes (30 min max)
  → DIAGNOSE → PLAN → EXECUTE → VERIFY → COMMIT → HANDOFF

Cycle 3: Team C executes (30 min max)
  → DIAGNOSE → PLAN → EXECUTE → VERIFY → COMMIT → HANDOFF

Between Sprints: Full status report, retrospective, next sprint plan
```

---

## QA Gate Between All Cycles

After every COMMIT and before every HANDOFF, QA asks:

```
QA GATE:
1. Did you verify before committing? (Y/N)
2. Are you working on ONE task? (One/Many)
3. How many failed attempts this cycle? (0/1/2/3+)
4. Is context clean or bloated? (Clean/Bloated)
5. Does commit message follow type(scope): format? (Y/N)
6. Are there uncommitted changes left? (Y/N)

If ANY answer is wrong → STOP cycle, reassess, run DIAGNOSE again
```

---

## Escalation Protocol (All Teams)

```
ESCALATE when:
- Cycle fails 3 times on same task
- Context bloat detected (>10 failed attempts accumulated)
- Multiple unrelated tasks mixed in one cycle
- Confidence < 3 on critical (P0) task
- User says "hung" / "stuck" / "overload"
- Build time exceeds 5 minutes on cargo check
- Git push fails 2 times

ESCALATION ACTION:
1. STOP all current work
2. Commit any verified changes
3. Report to user: "Team [X] stuck on [task] after [N] attempts. Current state: [summary]. Options: [A] research [B] clarify [C] reassign to different team."
4. Wait for user direction before proceeding
```

---

## Team Activation Rules

```
WHEN to activate each team:

Agent Safety    → ALWAYS active (monitors every cycle)
QA Team         → Before merge, after any code change, when user requests review
PM              → When multiple tasks exist, when prioritization needed, sprint planning
Issue Resolver  → When recurring pattern detected, when same error appears 2+ times
Build Engineer  → When cargo check fails, when build time >10s, when warnings >25
Git Specialist  → When git status shows uncommitted changes, before/after push
Workspace Arch  → When workspace needs reorganization, when modules >10KB
Feature Flag    → When conditional compilation needed, when build scope too large
Module Splitter → When monolithic module >10KB, when SRP violated
Tooling         → When dev tools missing, when build benchmarking needed
```

---

## Success Metrics (All Teams)

```
Per Cycle:
- [ ] Single task completed
- [ ] Verification passed
- [ ] Changes committed with proper format
- [ ] Can explain in one sentence
- [ ] QA gate passed

Per Sprint:
- [ ] 1-3 cycles completed
- [ ] All commits pushed to remote
- [ ] No context bloat
- [ ] Metrics tracked (build time, errors, warnings)
- [ ] Status reported to user

Per Project:
- [ ] Build time < 10 seconds
- [ ] Compilation errors = 0
- [ ] Warnings < 25
- [ ] All modules < 10KB
- [ ] Feature flags working
- [ ] Tooling installed and functional
- [ ] Git sync verified
```

---

## Reference Links

- Agent Safety: `.windsurf/skills/agent-execution-safety.md`
- QA Team: `.windsurf/skills/qa-team.md`
- Product Manager: `.windsurf/skills/product-manager.md`
- Issue Resolver: `.windsurf/skills/issue-pattern-resolution.md`
- Build Engineer: `.windsurf/skills/build-system-engineer.md`
- Git Specialist: `.windsurf/skills/git-workflow-specialist.md`
- Workspace Architect: `.windsurf/skills/workspace-architect.md`
- Feature Flag Engineer: `.windsurf/skills/feature-flag-engineer.md`
- Module Splitter: `.windsurf/skills/module-splitter.md`
- Tooling Specialist: `.windsurf/skills/tooling-specialist.md`

---

## Unified Workflow Execution Order Example

```
# Example: Fix Build Performance Issue

Cycle 1: PM Team
  DIAGNOSE: Build time 40s, errors 0, warnings 150, disk 5GB
  PLAN: Prioritize build optimization, assign teams
  EXECUTE: Update todo list, set metrics
  VERIFY: Todo list updated, priorities clear
  COMMIT: "docs(pm): Prioritize build optimization tasks"
  HANDOFF → Build Engineer

Cycle 2: Build Engineer
  DIAGNOSE: Profile config missing, no incremental flags
  PLAN: Add dev-optimized profile, configure incremental
  EXECUTE: Edit Cargo.toml profiles
  VERIFY: cargo check passes, build time measured
  COMMIT: "perf(build): Add dev-optimized profile configuration"
  HANDOFF → Tooling Specialist

Cycle 3: Tooling Specialist
  DIAGNOSE: cargo-nextest missing, cargo-watch missing
  PLAN: Install nextest, configure watch
  EXECUTE: cargo install cargo-nextest
  VERIFY: nextest works, benchmark baseline recorded
  COMMIT: "feat(tools): Install cargo-nextest for faster testing"
  HANDOFF → QA Team

Cycle 4: QA Team
  DIAGNOSE: Build time now 15s, warnings 25, modules need check
  PLAN: Review build performance, check module sizes
  EXECUTE: Apply performance lens
  VERIFY: Findings structured (P0-P3)
  COMMIT: "docs(qa): Build performance assessment report"
  HANDOFF → PM

SPRINT COMPLETE
```
