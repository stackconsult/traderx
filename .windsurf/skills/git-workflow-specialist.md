# Git Workflow Specialist Skill

**Unified Workflow Team:** Version Control  
**Follows:** `.windsurf/workflows/unified-team-execution.md` — DIAGNOSE → PLAN → EXECUTE → VERIFY → COMMIT → HANDOFF

## Trigger

Git workflow issues, commit problems, sync failures

## Action

### 1. Diagnose Git Issues

```bash
# Check current state
git status --short
git log --oneline -3
git remote -v

# Verify sync
git fetch origin
git log --oneline origin/feature/github-mcp-setup -3
```

### 2. Resolve Common Issues

#### Commit Not Pushed

```bash
# Check if committed locally but not pushed
git rev-parse HEAD
git rev-parse origin/feature/github-mcp-setup

# Push if needed
git push origin feature/github-mcp-setup
```

#### Staging Issues

```bash
# Clear and re-stage
git reset
git add -A
git status --short
```

#### Merge Conflicts

```bash
# Fetch and rebase
git fetch origin
git rebase origin/feature/github-mcp-setup

# Resolve conflicts
git status
git add .
git rebase --continue
```

### 3. Verify Workflow

- Local commits match remote
- Branch is up to date
- No uncommitted changes
- GitHub Actions triggered

### 4. Prevent Recurrence

- Add pre-commit hooks
- Configure status checks
- Set up branch protection
- Document workflow steps

## Verification

- Local SHA == Remote SHA ✅
- Branch status: clean ✅
- Actions running on GitHub ✅
- No pending changes ✅

## Prevention Skills

- commit-validation.md
- branch-sync.md
- remote-verification.md
- workflow-automation.md
