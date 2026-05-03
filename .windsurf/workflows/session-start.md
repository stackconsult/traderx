---
description: Mandatory session initialization - MUST run before any work begins. Validates environment, syncs skills, analyzes past work, checks repositories, confirms roadmap clarity
---

# session-start

**MANDATORY WORKFLOW** - Execute at the start of EVERY session. No exceptions.

**Purpose**: Ensure absolute guardrails are active before any work proceeds.

---

## Pre-Flight Checklist

### Phase 1: Environment Validation (30 seconds)

```powershell
# 1.1 Verify Windsurf MCP Configuration
$MCP_CONFIG = "$env:USERPROFILE\.windsurf\mcp_config.json"
if (Test-Path $MCP_CONFIG) { 
    Write-Host "✅ MCP Config: FOUND" -ForegroundColor Green
    $mcpContent = Get-Content $MCP_CONFIG -Raw | ConvertFrom-Json
    if ($mcpContent.mcpServers.github) {
        Write-Host "✅ GitHub MCP: CONFIGURED" -ForegroundColor Green
    } else {
        Write-Host "⚠️  GitHub MCP: NOT CONFIGURED" -ForegroundColor Yellow
    }
} else { 
    Write-Host "❌ MCP Config: MISSING" -ForegroundColor Red
}

# 1.2 Verify GitHub Token
if ($env:GITHUB_TOKEN) { 
    Write-Host "✅ GitHub Token: SET" -ForegroundColor Green
    # Test token validity
    try {
        $response = curl -s -H "Authorization: Bearer $env:GITHUB_TOKEN" https://api.github.com/user
        Write-Host "✅ GitHub Token: VALID" -ForegroundColor Green
    } catch {
        Write-Host "⚠️  GitHub Token: VALIDATION FAILED" -ForegroundColor Yellow
    }
} else { 
    Write-Host "❌ GitHub Token: NOT SET" -ForegroundColor Red
    Write-Host "   Set with: $env:GITHUB_TOKEN = 'your_token_here'" -ForegroundColor Cyan
}

# 1.3 Verify Project Structure
$requiredDirs = @(".windsurf/workflows", ".windsurf/skills", "proofs", "docs", "src")
foreach ($dir in $requiredDirs) {
    if (Test-Path $dir) {
        Write-Host "✅ $dir : EXISTS" -ForegroundColor Green
    } else {
        Write-Host "❌ $dir : MISSING" -ForegroundColor Red
    }
}
```

**Validation Gate**: If any ❌ above, STOP and fix before proceeding.

---

### Phase 2: Skill Sync & Validation (60 seconds)

```powershell
# 2.1 Check learnship installation
if (Test-Path "packages/learnship/bin/install.js") {
    Write-Host "✅ Learnship: INSTALLED" -ForegroundColor Green
} else {
    Write-Host "⚠️  Learnship: Checking alternative locations..." -ForegroundColor Yellow
}

# 2.2 Verify workflow files exist
$requiredWorkflows = @(
    ".windsurf/workflows/sync-upstream-skills.md",
    ".windsurf/workflows/session-start.md"
)
foreach ($workflow in $requiredWorkflows) {
    if (Test-Path $workflow) {
        Write-Host "✅ Workflow $(Split-Path $workflow -Leaf): PRESENT" -ForegroundColor Green
    } else {
        Write-Host "❌ Workflow $(Split-Path $workflow -Leaf): MISSING" -ForegroundColor Red
    }
}

# 2.3 Check skills directory structure
$skillDirs = @(
    ".windsurf/skills/agentic-learning",
    ".windsurf/skills/impeccable"
)
foreach ($skillDir in $skillDirs) {
    if (Test-Path $skillDir) {
        $count = (Get-ChildItem $skillDir -Recurse -File).Count
        Write-Host "✅ $skillDir : $count files" -ForegroundColor Green
    } else {
        Write-Host "⚠️  $skillDir : NOT FOUND - Will sync from upstream" -ForegroundColor Yellow
    }
}
```

**Action Required**: If skills missing, execute `/sync-upstream-skills` immediately.

---

### Phase 3: Previous Session Analysis (60 seconds)

```powershell
# 3.1 Read last JOURNAL.md entry
$journal = Get-Content "JOURNAL.md" -Raw
$entries = $journal -split "---" | Select-Object -Last 2
Write-Host "`n📋 LAST SESSION ACTIVITY:" -ForegroundColor Cyan
Write-Host $entries[0].Substring(0, [Math]::Min(500, $entries[0].Length))

# 3.2 Check for uncommitted changes
git status --short
$uncommitted = git status --short
if ($uncommitted) {
    Write-Host "⚠️  Uncommitted changes detected:" -ForegroundColor Yellow
    Write-Host $uncommitted
    Write-Host "   Commit or stash before proceeding" -ForegroundColor Cyan
} else {
    Write-Host "✅ Working directory: CLEAN" -ForegroundColor Green
}

# 3.3 Check last commit
git log -1 --oneline
Write-Host "`n✅ Last commit shown above" -ForegroundColor Green
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

```powershell
# 4.1 Check GitHub for updates
Write-Host "`n🔍 Checking upstream repositories..." -ForegroundColor Cyan

# Check agentic-learning
try {
    $agenticHead = curl -s https://api.github.com/repos/FavioVazquez/agentic-learning/commits/main | ConvertFrom-Json
    Write-Host "✅ agentic-learning: $($agenticHead.sha.Substring(0,7)) - $($agenticHead.commit.message.Split("`n")[0])" -ForegroundColor Green
} catch {
    Write-Host "⚠️  Could not check agentic-learning" -ForegroundColor Yellow
}

# Check impeccable
try {
    $impeccableHead = curl -s https://api.github.com/repos/pbakaus/impeccable/commits/main | ConvertFrom-Json
    Write-Host "✅ impeccable: $($impeccableHead.sha.Substring(0,7)) - $($impeccableHead.commit.message.Split("`n")[0])" -ForegroundColor Green
} catch {
    Write-Host "⚠️  Could not check impeccable" -ForegroundColor Yellow
}

# 4.2 Check local vs origin
git fetch origin --quiet
$local = git rev-parse HEAD
$remote = git rev-parse origin/feature/github-mcp-setup 2>$null
if ($remote) {
    if ($local -eq $remote) {
        Write-Host "✅ Local branch: UP TO DATE with origin" -ForegroundColor Green
    } else {
        Write-Host "⚠️  Local branch: DIVERGED from origin" -ForegroundColor Yellow
        git log --oneline --left-right --graph HEAD...origin/feature/github-mcp-setup
    }
} else {
    Write-Host "ℹ️  Branch not yet on origin" -ForegroundColor Cyan
}
```

**Action**: If upstream has new commits, evaluate if skills need update.

---

### Phase 5: Roadmap & Direction Clarity (60 seconds)

```powershell
# 5.1 Read milestones
if (Test-Path "MILESTONES.md") {
    $milestones = Get-Content "MILESTONES.md" -Raw
    Write-Host "`n📊 MILESTONES:" -ForegroundColor Cyan
    # Extract current milestone (first non-completed)
    $lines = $milestones -split "`n" | Select-Object -First 30
    Write-Host ($lines -join "`n")
}

# 5.2 Check implementation plan
if (Test-Path "IMPLEMENTATION_PLAN.md") {
    $plan = Get-Content "IMPLEMENTATION_PLAN.md" -Raw
    Write-Host "`n📝 IMPLEMENTATION PLAN (first 50 lines):" -ForegroundColor Cyan
    $planLines = $plan -split "`n" | Select-Object -First 50
    Write-Host ($planLines -join "`n")
}

# 5.3 Current status
Write-Host "`n🎯 CURRENT STATUS CHECK:" -ForegroundColor Cyan
Write-Host "Branch: $(git branch --show-current)"
Write-Host "Last 3 commits:"
git log --oneline -3

# 5.4 Mem0 Memory Retrieval
Write-Host "`n🧠 Retrieving relevant memories from mem0..." -ForegroundColor Cyan
# Query mem0 for past session context, lessons learned, and relevant patterns
# This will be implemented via mem0 MCP server integration
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

```powershell
# Store session initialization result in mem0
# This includes: environment state, skills synced, roadmap clarity, current phase
# Enables future sessions to learn from initialization patterns
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
