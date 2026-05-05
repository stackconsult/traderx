# AGENTS.md — TraderX

## Project Overview

Multi-crate Rust HFT trading system. Workspace root at `packages/`. Main crate: `packages/oms-engine` (Rust, tokio async, rust_decimal, uuid 1.x). Python strategies in `src/`. Do not mix Python and Rust module paths.

## Key Commands

```bash
# Check compilation (run after every file change)
cargo check --package oms-engine 2>&1 | grep "^error" | wc -l

# Run specific test
cargo test --package oms-engine [test_name] -- --nocapture

# Check error count trend
cargo check --package oms-engine 2>&1 | grep "^error" | head -5

# Git sync check
git fetch && git status && git log --oneline -3

# Journal sync across branches
bash scripts/journal_sync.sh 2>/dev/null || echo 'no journal sync script'
```

## Non-Obvious Patterns (Highest Signal)

These cannot be inferred from code — read before touching any of these areas:

- `Order`, `Side`, `OrderType` live in `crate::state_machine`, **not** `crate::oms`
- `OrderId` does not exist as a type — use `uuid::Uuid` directly or `use uuid::Uuid as OrderId`
- `WorkflowNode` contains `Box<dyn Fn>` → cannot derive `Clone`, `Debug`, `Serialize`, `Deserialize`
- `HumanApprovalSystem` has `mpsc::Receiver` → cannot derive `Clone`
- Recursive async fns require `Box::pin(async move { ... })` return type explicitly
- `TimeInForce` exists in BOTH `protocol` and `orders::advanced` — use `AdvancedTimeInForce` alias for the orders version
- `cargo check` errors are split: ~47 in `aeron_journal`, ~44 in `oms` (pre-existing), ~18 in `backtest/mod` (new)
- All trading orders MUST pass through `RiskBus` before execution — never bypass
- Paper trading mode required before live deployment

## Code Style

- No `unwrap()` in production paths — use `?` or explicit `match`
- No `// TODO` in committed code — fix or remove
- No blocking I/O inside tokio tasks — use `tokio::fs`, `tokio::net`
- Error types: use `thiserror::Error` derive, never `Box<dyn Error>` in domain logic
- Commit format: `type(scope): description` — e.g. `fix(oms-engine): correct OrderId import`

## Testing Rules

- Write test before implementation for any new trading logic
- Run `cargo check` before every commit — error count must not increase
- Integration tests use REAL components, not mocks, for core trading flow
- Performance tests: risk check must complete in <100ns

## Permissions

### ✅ Always allowed

- Read files, grep, list directories
- `cargo check`, `cargo test [specific_test]`
- `git fetch`, `git status`, `git log`
- Edit source files within `packages/oms-engine/`

### ⚠️ Ask first

- `cargo add` / modify `Cargo.toml` dependencies
- Delete or rename files
- `git push` to any branch
- Modify `packages/learnship/` or `src/` Python code

### 🚫 Never

- `git push --force` or `git rebase` on shared branches
- Commit secrets, API keys, or `.env` files
- Modify `backup/*` branches
- Skip `cargo check` before committing
- Write pseudo-code or stub implementations and commit as real

## Context Compaction Preservation

When context compacts, always preserve:

- Current error count (before/after)
- Current branch name
- Last commit hash
- Which files were last modified
- The "Non-Obvious Patterns" section above

## Reference Docs (load only when needed)

- Full governance: `AGENT_MASTER_SYSTEM.md`
- Session history: `JOURNAL.md` (last 5 entries)
- Telemetry journal: `GENESIS_ROADMAP.md` (task log + session snapshots)
- Branch status: `MASTER_OPERATIONAL_CHECKLIST.md`
- Module rules: `packages/oms-engine/AGENTS.md`
- Skills index: `.windsurf/skills/skills_index.json`

## Agent Swarm Orchestration

### Installed skill system

- Source: `addyosmani/agent-skills` v0.6.0
- Active rules: `.windsurfrules` (always loaded) + `.windsurf/skills/skills_index.json` (always_active list)
- All skills: `.windsurf/skills/` (63 skills — load on demand by phase)
- **Always-active skills** (load every session): `reasoning-logic`, `neural-context`, `llm-modelling`, `language-selection`, `full-scope-search`, `self-upskill`
- Specialist personas: `.windsurf/agents/` (code-reviewer, security-auditor, test-engineer)
- Reference checklists: `.windsurf/references/`
- Project swarm: `/Users/kirtissiemens/CascadeProjects/.ai/` (35 agents, all skill-bound + genesis_model wired)
- Telemetry journal: `GENESIS_ROADMAP.md` (session snapshots + task log)
- Web search: TinyFish MCP (`tinyfish/search` + `tinyfish/fetch` — free, no credits)

### Workflow commands (Windsurf slash commands)

| Command | When to use |
|---------|-------------|
| `/genesis-hoard-agent` | Full autonomous audit + self-upskill + wiring fix loop |
| `/orchestrate` | Any multi-agent task — routes intent to right agent(s), no terminal needed |
| `/ship` | Before any merge — parallel fan-out: code-review + security + test coverage |
| `/security-gate` | Before any commit touching external APIs, auth, or dependencies |
| `/gate-check` | At each BAM stream transition (G0→G7) |
| `/debug-team` | Hard bugs with multiple competing root causes |
| `/context-decipher` | Always active — plain language → intent mapping |
| `/handoff-protocol` | End of session or medium switch |
| `/genesis-settings` | Provider config, model routing, keyboard shortcuts |

Workflow files: `.windsurf/workflows/`

### Orchestration rules (enforced)

- **User is the orchestrator** — agents do not spawn agents
- **Max orchestration depth = 1** — slash command → persona (no persona-calls-persona)
- **Parallel fan-out** only when sub-tasks are independent (no shared mutable state)
- **Agent Teams** (`/debug-team`) for adversarial debugging only

### Agent Teams setup (experimental)

Enable once per shell session for competing-hypothesis debugging:

```bash
export CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1
```

Or persist in `~/.claude/settings.json`:

```json
{ "env": { "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1" } }
```

### BAM build gate sequence

G0 (contracts) → G1 (infra) → G2 (Model 1) → G3 (all models) → G4 (testing) → G5 (measurement) → G6 (winner) → G7 (live)
Run `/gate-check` at each transition. No stream starts before its gate passes.
