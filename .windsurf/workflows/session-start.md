---
description: Mandatory session initialization - MUST run before any work begins. Validates environment, syncs skills, analyzes past work, checks repositories, confirms roadmap clarity
---

# /session-start

**MANDATORY WORKFLOW** — Execute at the start of EVERY session. No exceptions.
**OS**: macOS (zsh). All commands use `bash`/`zsh` syntax, never PowerShell.
**Plain-language trigger**: Saying "let's start", "pick up where we left off", "what's the status", or "continue" runs this automatically.

**Purpose**: Ensure absolute guardrails are active before any work proceeds.

---

## Pre-Flight Checklist

### Phase 1: Environment Validation (30 seconds)

```bash
# 1.1 Verify Genesis server is available
if command -v genesis &>/dev/null; then
  echo "✅ Genesis binary: $(genesis --version 2>/dev/null || echo 'found')"
else
  echo "❌ Genesis: NOT ON PATH — run: source ~/.zshrc"
fi

# Check Genesis server is running (for VS Code extension)
curl -s --connect-timeout 2 http://127.0.0.1:7700 &>/dev/null \
  && echo "✅ Genesis server: RUNNING at 127.0.0.1:7700" \
  || echo "⚠️  Genesis server: NOT RUNNING — start with: genesis --server --transport ws --listen 127.0.0.1:7700 --cwd $(pwd) --mode normal"

# 1.2 Verify GitHub Token
if [ -n "$GITHUB_TOKEN" ]; then
  echo "✅ GitHub Token: SET"
  curl -sf -H "Authorization: Bearer $GITHUB_TOKEN" https://api.github.com/user | python3 -c "import json,sys; u=json.load(sys.stdin); print('✅ GitHub Token: VALID —', u['login'])" 2>/dev/null || echo "⚠️  GitHub Token: INVALID"
else
  echo "⚠️  GitHub Token: NOT SET — export GITHUB_TOKEN=your_token"
fi

# 1.3 Verify project structure
for dir in .windsurf/workflows .windsurf/skills packages/oms-engine src; do
  [ -d "$dir" ] && echo "✅ $dir: EXISTS" || echo "❌ $dir: MISSING"
done

# 1.4 Disk space check (must have >500MB free)
avail=$(df -m / | tail -1 | awk '{print $4}')
[ "$avail" -gt 500 ] && echo "✅ Disk space: ${avail}MB free" || echo "❌ Disk space: CRITICAL — ${avail}MB free, run cargo clean"
```

**Validation Gate**: If any ❌ above, STOP and fix before proceeding.

---

### Phase 2: Skill Sync & Validation (60 seconds)

```bash
# 2.1 Count installed skills
skill_count=$(ls .windsurf/skills/*.md 2>/dev/null | wc -l | tr -d ' ')
[ "$skill_count" -ge 20 ] \
  && echo "✅ Skills: $skill_count installed" \
  || echo "⚠️  Skills: only $skill_count found — run /sync-upstream-skills"

# 2.2 Verify core workflows exist
for wf in ship security-gate gate-check debug-team self-audit sync-upstream-skills context-decipher handoff-protocol; do
  [ -f ".windsurf/workflows/${wf}.md" ] \
    && echo "✅ Workflow /${wf}: PRESENT" \
    || echo "❌ Workflow /${wf}: MISSING"
done

# 2.3 Verify GENESIS_AGENT.md is present (master context)
[ -f "GENESIS_AGENT.md" ] && echo "✅ GENESIS_AGENT.md: PRESENT" || echo "❌ GENESIS_AGENT.md: MISSING"
```

**Action Required**: If skills missing, execute `/sync-upstream-skills` immediately.

---

### Phase 3: Previous Session Analysis (60 seconds)

```bash
# 3.1 Show last journal entry (if exists)
if [ -f JOURNAL.md ]; then
  echo "\n📋 LAST SESSION ENTRY:"
  awk '/^---/{c++} c>=2{print}' JOURNAL.md | head -20
else
  echo "⚠️  JOURNAL.md not found"
fi

# 3.2 Uncommitted changes check
uncommitted=$(git status --short 2>/dev/null)
if [ -n "$uncommitted" ]; then
  echo "⚠️  Uncommitted changes:"
  git status --short
  echo "   Commit or stash before starting new work"
else
  echo "✅ Working directory: CLEAN"
fi

# 3.3 Current branch + last 3 commits
echo "\nBranch: $(git branch --show-current 2>/dev/null)"
git log --oneline -3 2>/dev/null

# 3.4 Cargo error baseline
err_count=$(cargo check --package oms-engine 2>&1 | grep -c "^error" || echo 0)
echo "\nCargo errors (baseline): $err_count"
```

**Grading Task**: Grade previous session work (A-F scale):

- **A**: Excellent - All requirements met, tests passing, documentation complete
- **B**: Good - Minor improvements possible, solid foundation
- **C**: Acceptable - Works but needs refinement
- **D**: Needs Work - Incomplete or has issues
- **F**: Failed - Must redo

Document grade in new JOURNAL entry.

---

### Phase 4: Repository Sync Check (60 seconds)

```bash
# 4.1 Check agent-skills for upstream updates
latest=$(curl -sf https://api.github.com/repos/addyosmani/agent-skills/releases/latest \
  | python3 -c "import json,sys; r=json.load(sys.stdin); print(r['tag_name'])" 2>/dev/null || echo "unknown")
installed=$(cat .windsurf/skills/.version 2>/dev/null || echo "unknown")
echo "agent-skills upstream: $latest | installed: $installed"
[ "$latest" != "$installed" ] && echo "⚠️  Skills outdated — run /sync-upstream-skills" || echo "✅ Skills: up to date"

# 4.2 Local vs origin
git fetch origin --quiet 2>/dev/null
branch=$(git branch --show-current)
local_sha=$(git rev-parse HEAD 2>/dev/null)
remote_sha=$(git rev-parse origin/$branch 2>/dev/null || echo "no-remote")
[ "$local_sha" = "$remote_sha" ] \
  && echo "✅ Branch $branch: UP TO DATE with origin" \
  || echo "⚠️  Branch $branch: DIVERGED — local=$local_sha remote=$remote_sha"
```

**Action**: If upstream has new commits, evaluate if skills need update.

---

### Phase 5: Roadmap & Direction Clarity (60 seconds)

```bash
# 5.1 Current BAM gate
current_gate=$(grep -m1 "SIGNED\|PENDING\|G[0-9]" .planning/01_INTEGRATION_CONTRACTS.md 2>/dev/null | head -1 || echo "unknown")
echo "BAM Gate status: $current_gate"

# 5.2 Show master build plan summary
if [ -f ".planning/MASTER_THREE_MODEL_BUILD_PLAN.md" ]; then
  echo "\n� BUILD PLAN (current streams):"
  grep -E "^##|\[ \]|\[x\]" .planning/MASTER_THREE_MODEL_BUILD_PLAN.md | head -20
fi

# 5.3 Load context-decipher for this session
echo "\n📖 Context decipher layer: .windsurf/workflows/context-decipher.md"
echo "   Plain-language → intent mapping active for this session"
```

**Confirmation Required**: Answer these questions:

1. What is the current phase of work?
2. What are the next 3 deliverables?
3. Are there any blockers?
4. What is the immediate next task?
5. What memories from past sessions are relevant to current work?

---

## Phase 6: Session Initialization Summary & Memory Storage

```
╔══════════════════════════════════════════════════════════════════╗
║              SESSION INITIALIZATION SUMMARY                       ║
╠══════════════════════════════════════════════════════════════════╣
║ Environment:        [ ] PASS   [ ] FAIL                          ║
║ Skills Synced:      [ ] PASS   [ ] FAIL   [ ] SYNC NEEDED         ║
║ Past Work Graded:   [ ] A   [ ] B   [ ] C   [ ] D   [ ] F        ║
║ Repositories:       [ ] UP TO DATE   [ ] UPDATES AVAILABLE      ║
║ Roadmap Clear:     [ ] YES   [ ] UNCLEAR                        ║
║ Mem0 Connected:    [ ] ACTIVE   [ ] NOT CONFIGURED               ║
╠══════════════════════════════════════════════════════════════════╣
║ GUARDRAILS STATUS:                                               ║
║ Workflows:         [ ] LOADED                                    ║
║ Skills:            [ ] VALIDATED                                 ║
║ Agent Skills:      [ ] ACTIVE                                    ║
║ Memory Layer:      [ ] MEM0 ACTIVE                               ║
║ Ready to Build:    [ ] PRODUCTION MODE                           ║
╚══════════════════════════════════════════════════════════════════╝
```

**ONLY PROCEED IF ALL CHECKS PASS**

If any check fails:

- Execute `/sync-upstream-skills` for skill issues
- Review `AGENTS.branch.mcp.md` for clarity on direction
- Configure mem0 MCP server if not connected
- Ask for help if blockers cannot be resolved

---

## Phase 7: Store Session Context in Mem0

After session initialization completes successfully, store the session context in mem0:

```bash
# Write session init snapshot to JOURNAL.md
cat >> JOURNAL.md << EOF

---
## Session Start — $(date '+%Y-%m-%d %H:%M')
- Branch: $(git branch --show-current)
- Last commit: $(git log --oneline -1)
- Cargo errors: $(cargo check --package oms-engine 2>&1 | grep -c '^error' || echo 0)
- Genesis: $(genesis --version 2>/dev/null || echo 'check PATH')
- Skills: $(ls .windsurf/skills/*.md 2>/dev/null | wc -l | tr -d ' ') installed
EOF
echo "✅ Session snapshot written to JOURNAL.md"
```

Memory schema:

```json
{
  "type": "session_init",
  "workflow": "session-start",
  "environment_pass": true,
  "skills_synced": true,
  "roadmap_clear": true,
  "current_phase": "...",
  "branch": "feature/github-mcp-setup",
  "timestamp": "2026-05-02T19:00:00Z"
}
```

---

## Quick Commands Reference

| Command | Purpose | When to Use |
|---------|---------|-------------|
| `/sync-upstream-skills` | Update all skills from upstream | Skills outdated or missing |
| `/ls` | Show current status | Need orientation |
| `/health` | Project health check | Before major work |
| `/review` | Multi-persona code review | Before commits |
| `/compound` | Capture solution | After solving problem |
| `/audit-milestone` | Validate milestone | Phase completion |

---

## Success Criteria

Session start is successful when:

- ✅ All environment checks pass
- ✅ Skills are present and current
- ✅ Previous work is graded and understood
- ✅ Repositories are checked for updates
- ✅ Roadmap is clear and next steps defined
- ✅ All guardrails are confirmed active
- ✅ Ready to build in production mode

---

**REMEMBER**: No work may proceed until this workflow completes successfully.
