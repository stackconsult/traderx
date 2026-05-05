---
description: Contextual handoff between Genesis sessions and between Genesis ↔ Windsurf/Cascade — ensures zero context loss across medium switches
---

# /handoff-protocol — Cross-Session & Cross-Medium Context Handoff

Governs how context is preserved when:
- Switching from Windsurf/Cascade → Genesis (or back)
- Ending a session mid-task
- Handing off between swarm agents
- Resuming after a machine restart

---

## Rule: The Handoff Invariant

> At any point in time, a fresh agent starting with GENESIS_AGENT.md + AGENTS.md + JOURNAL.md
> (last 3 entries) + `git log --oneline -5` must be able to resume work without asking any questions.

If that's not possible, the handoff is incomplete.

---

## Medium Handoff Map

```
Windsurf/Cascade session ends
         ↓
  Write JOURNAL.md entry (Phase 7 of /session-start)
         ↓
  git commit "chore(journal): session snapshot — [date]"
         ↓
Genesis sidebar picks up via GENESIS_AGENT.md startup checklist
         ↓
Genesis session ends / machine restarts
         ↓
  Write JOURNAL.md entry + git push
         ↓
Next Cascade session reads: AGENTS.md + GENESIS_AGENT.md + JOURNAL.md last 3
```

---

## Handoff Snapshot Format

Append to `JOURNAL.md` at end of every working session:

```markdown
---
## Handoff Snapshot — [YYYY-MM-DD HH:MM] — [medium: Windsurf|Genesis|Terminal]

### State
- Branch: feature/github-mcp-setup
- Last commit: [hash] [message]
- Cargo errors: [N] (was [N] at session start)
- BAM Gate: G[N] — [SIGNED|PENDING]

### What was completed this session
- [bullet list of concrete actions taken]

### What is in progress (do not start something new — finish this first)
- [file being edited / test being written / workflow being created]

### Exact next action
- [single most important next step, phrased as a command]

### Open questions / blockers
- [anything that needs a decision before proceeding]

### Files modified this session
- [file1]: [what changed]
- [file2]: [what changed]
```

---

## Swarm Agent Handoff

When a swarm run ends and a worker agent has partial output:

1. Worker writes result to `.windsurf/handoff/[agent-role]-[timestamp].md`
2. Integrator agent reads all handoff files before synthesizing
3. Lead session collects from integrator before closing

```bash
mkdir -p .windsurf/handoff
# Worker writes:
echo "## scout result — $(date)\n$OUTPUT" >> .windsurf/handoff/scout-$(date +%s).md
# Integrator reads all:
cat .windsurf/handoff/*.md
```

---

## Cascade ↔ Genesis Bidirectional Handoff

### When switching from Cascade to Genesis sidebar:

Genesis reads on startup (in order):
1. `GENESIS_AGENT.md` — identity + skill map + guarded lines
2. `AGENTS.md` — project rules + non-obvious patterns
3. `JOURNAL.md` — last 3 entries
4. `git log --oneline -5` — recent commits
5. `cargo check --package oms-engine 2>&1 | grep "^error" | wc -l` — error baseline

### When switching from Genesis to Cascade:

Cascade reads on startup (automatic via checkpoint system):
- Previous conversation checkpoint
- Any new JOURNAL.md entries written since last Cascade session
- Git diff since last Cascade commit

### Never assume the other medium saw your context.
Always write it to JOURNAL.md + git commit before switching.

---

## Quick Commands

```bash
# Write handoff snapshot now
cat >> JOURNAL.md << 'EOF'
---
## Handoff Snapshot — $(date '+%Y-%m-%d %H:%M') — Windsurf

### State
- Branch: $(git branch --show-current)
- Last commit: $(git log --oneline -1)
- Cargo errors: $(cargo check --package oms-engine 2>&1 | grep -c '^error' || echo 0)

### Exact next action
- [FILL IN]
EOF

git add JOURNAL.md
git commit -m "chore(journal): handoff snapshot"
git push origin feature/github-mcp-setup
```

---

## Failure Mode: "I don't know where we left off"

If an agent genuinely cannot orient from the above, it must:
1. Read `JOURNAL.md` last 5 entries
2. Run `git log --oneline -10`
3. Run `cargo check` and note current error count
4. Read `.planning/MASTER_THREE_MODEL_BUILD_PLAN.md` open checkboxes
5. Report: "Based on the above, we were working on X. Next action is Y. Confirm?"

Never guess silently. Always surface the reconstruction.
