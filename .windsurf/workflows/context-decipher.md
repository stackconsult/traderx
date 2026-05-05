---
description: Plain-language intent decoder — agent reads this to accurately interpret casual, shorthand, or ambiguous user messages and map them to exact actions with zero clarification needed
---

# /context-decipher — Plain Language → Precise Intent Mapping

This is the **always-active linguistic layer**. Before responding to ANY message, the agent
checks this file to determine what the user actually means, not just what they literally said.

**Rule**: Never ask for clarification when intent can be inferred from context + this map.
**Rule**: When in doubt about scope, choose the SMALLEST correct interpretation and verify.
**Rule**: Typos, abbreviations, and run-on sentences are normal — parse the intent, not the grammar.

---

## Core Intent Categories

### 1. "Let's go" / "continue" / "pick up where we left off" / "resume"
→ Run `/session-start` immediately. Orient around last JOURNAL.md entry + current git status.

### 2. "What's the status" / "where are we" / "what's next" / "what do we have"
→ Run `/ls` (show position in plan) + print last 3 commits + current BAM gate.

### 3. "Fix it" / "sort that out" / "clean that up" / "handle that"
→ Refers to the MOST RECENT error, failing test, or open issue visible in context.
→ Run `/debug` on the nearest identifiable problem. Do not ask which one.

### 4. "Ship it" / "push it" / "send it" / "commit this"
→ Run `/ship` (parallel fan-out review). If all gates pass, commit with conventional format and push.
→ Never push without passing `/ship` first.

### 5. "Review it" / "check this" / "look at this" / "audit this"
→ Invoke `code-reviewer` persona on the current diff (`git diff --staged` or last changed files).

### 6. "Is it secure" / "scan it" / "check for holes" / "security check"
→ Run `/security-gate` against the current staged diff or last modified files.

### 7. "Run the tests" / "does it pass" / "are tests green"
→ `cargo test --package oms-engine -- --nocapture` + report pass/fail count.

### 8. "Build" / "compile" / "check if it compiles" / "does it build"
→ `cargo check --package oms-engine 2>&1 | grep "^error"` — report error delta vs baseline.

### 9. "Update the skills" / "sync skills" / "get latest skills"
→ Run `/sync-upstream-skills`.

### 10. "Upskill" / "improve yourself" / "learn from this" / "add that to your skills"
→ Run `/self-audit` to identify the gap + draft SKILL.md for the new capability.

### 11. "What can you do" / "show me the workflows" / "what commands do you have"
→ Print the workflow command table from `AGENTS.md` + `GENESIS_AGENT.md`.

### 12. "Start the server" / "launch genesis" / "fire it up" / "spin it up"
→ Execute:
```bash
genesis --server --transport ws --listen 127.0.0.1:7700 \
  --cwd /Users/kirtissiemens/CascadeProjects/traderx-repo \
  --mode normal --mcp-config genesis.mcp.json
```

### 13. "Swarm it" / "run the swarm" / "use the whole team" / "throw everything at it"
→ `genesis --task "[last stated task]" --agents 5 --cycles 8`
→ If no task was stated, ask ONE question: "What task should the swarm tackle?"

### 14. "What's broken" / "what are the errors" / "how many errors"
→ `cargo check --package oms-engine 2>&1 | grep "^error" | wc -l` + show top 5 errors.

### 15. "Gate check" / "are we good to proceed" / "can we move to the next phase"
→ Run `/gate-check` for the current BAM gate number.

### 16. "Plan it out" / "break it down" / "make a plan for"
→ Load `planning-and-task-breakdown` skill + produce task breakdown with acceptance criteria.

### 17. "Research" / "look into" / "find out about" / "check the docs on"
→ Use web_fetch/web_search tools. Cite sources. Flag unverified claims.
→ Activate `source-driven-development` skill.

### 18. "Write the spec" / "spec it out" / "define it properly"
→ Load `spec-driven-development` skill. Produce PRD covering: objective, inputs, outputs, edge cases, success criteria.

### 19. "Add a test" / "test that" / "prove it works"
→ Load `test-driven-development` skill. Write the FAILING test first, then implementation.

### 20. "Simplify this" / "clean this up" / "this is too complex"
→ Load `code-simplification` skill. Apply Chesterton's Fence. Preserve exact behaviour.

---

## Abbreviation + Shorthand Dictionary

| User says | Agent reads as |
|-----------|---------------|
| `OMS` | `packages/oms-engine/` |
| `RiskBus` | `packages/oms-engine/src/risk_bus.rs` |
| `SHM` | shared memory bridge — `src/shm_bridge.py` or Rust equivalent |
| `BAM` | BAM grid trading model — the three-model build system |
| `M1` / `Model 1` | Base BAM model — `03_STREAM_MODEL1.md` |
| `M2` / `Model 2` | Physics-ML model — `04_STREAM_MODEL2.md` |
| `M3` / `Model 3` | Binary assembly + FPGA model — `05_STREAM_MODEL3.md` |
| `G0`–`G7` | BAM stream gates — run `/gate-check G[N]` |
| `the dashboard` | `dashboard/` — Next.js scoring UI |
| `the contracts` | `.planning/01_INTEGRATION_CONTRACTS.md` |
| `the master plan` | `.planning/MASTER_THREE_MODEL_BUILD_PLAN.md` |
| `the journal` | `JOURNAL.md` |
| `the swarm` | `.ai/` agent directory + genesis `--task` swarm mode |
| `the extension` | Genesis VS Code extension (sidebar) |
| `wired in` | configured, connected, and tested — not just installed |
| `dial it in` | review settings/config for optimal values and apply them |
| `upskill` | run `/self-audit` + add skill gap to queue |
| `manufacture a VD` | create a virtual disk (macOS `hdiutil`) to free space |
| `agnostic` | works across all mediums: TUI, VS Code, web, channels |
| `self-healing` | error detected → auto-fix attempted → verify → journal |

---

## Compound Intent Patterns

These are multi-word phrases that map to multi-step workflows:

| Pattern | Workflow |
|---------|----------|
| "review and ship" | `/ship` → if all pass → commit + push |
| "fix and test" | identify error → fix → `cargo test` → verify |
| "plan and build" | `planning-and-task-breakdown` → `incremental-implementation` |
| "scan and commit" | `/security-gate` → if clean → commit |
| "upskill and document" | `/self-audit` → draft SKILL.md → commit |
| "check and sync" | `cargo check` → `git fetch` → report delta |
| "start fresh" | `/session-start` → `/ls` → confirm next task |
| "run everything" | `/ship` (includes code review + security + tests) |

---

## Tone & Register Rules

The user communicates casually, with typos and shorthand. Respond in kind:
- **Concise** — no preamble, no recap of what was just said
- **Direct** — state what you're doing, then do it
- **Confident** — do not ask permission to run standard checks
- **No hedging** — never "it seems like" or "perhaps we could" — just do it
- **Escalate only for** — destructive operations (delete, force push), ambiguous scope where both interpretations have very different consequences

---

## Escalation Triggers (ask ONE question before acting)

Only pause and ask when:
1. "Delete", "remove", "wipe", "nuke", "drop" — confirm target before executing
2. "Push to main/master" — confirm branch target
3. The request could mean either file A or file B and they have opposite effects
4. A swarm task is stated but no goal is clear (ask: "What outcome do you want?")

In all other cases: infer, act, report.
