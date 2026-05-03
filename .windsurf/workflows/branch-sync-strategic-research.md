# Branch Sync Strategic Research Workflow

## Description

Comprehensive branch scanning, merge conflict detection, and autonomous sync workflow. Operates entirely offline using local git state — no dependency on GitHub automation which may fail or flag issues.

**Core Principle**: *"Branch divergence is debt. Every unsynced commit is a merge conflict waiting to happen."*

---

## Required CLIs

| CLI | Purpose | Installation |
|-----|---------|-------------|
| `git` | Core SCM | System default |
| `gh` | GitHub API fallback | `brew install gh` |
| `jq` | JSON parsing | `brew install jq` |
| `fd` | Fast file search | `brew install fd` |
| `rg` (ripgrep) | Fast text search | `brew install ripgrep` |

---

## Phase 1: Branch State Scan

### Step 1.1: Enumerate All Branches
```bash
git branch -a --format='%(refname:short) %(upstream:short) %(upstream:track)'
```

**Output Format**:
```
branch_name origin/branch_name [ahead N, behind M]
branch_name origin/branch_name [gone]
branch_name (no upstream)
```

### Step 1.2: Calculate Divergence Matrix
```bash
for branch in $(git branch -a | grep origin/ | sed 's/remotes\///'); do
    behind=$(git log --oneline HEAD..$branch | wc -l)
    ahead=$(git log --oneline $branch..HEAD | wc -l)
    echo "$branch behind:$behind ahead:$ahead"
done
```

### Step 1.3: Detect Stale Branches
```bash
git branch -a --merged origin/main --format='%(refname:short) %(committerdate:short)'
```

---

## Phase 2: Merge Conflict Prediction

### Step 2.1: Identify Touching Files
```bash
git log --oneline --left-right --name-only HEAD...origin/TARGET_BRANCH
```

### Step 2.2: Detect File-Level Conflicts
```bash
git merge-tree $(git merge-base HEAD origin/TARGET_BRANCH) HEAD origin/TARGET_BRANCH
```

**Conflict Severity**: HIGH if:
- Same file modified on both sides (non-ancestor changes)
- Binary files differ (cannot auto-merge)
- `.md` strategy files modified on both sides

### Step 2.3: Commit-Level Conflict Analysis
```bash
git log --oneline --graph --left-right --decorate HEAD...origin/TARGET_BRANCH
```

**Conflict Patterns**:
- Diamond merge: Both branches diverged from same ancestor, then merged back
- Fast-forward block: Target branch has commits that HEAD doesn't, prevents FF
- Divergent spec files: `.planning/*.md` modified on both sides

---

## Phase 3: Autonomous Merge Strategy

### Strategy Decision Matrix

| Scenario | Behind | Ahead | Strategy | Risk |
|----------|--------|-------|----------|------|
| Fast-forward | 0 | N | `git merge --ff-only` | None |
| Rebase | M | N (M small) | `git rebase origin/TARGET` | Low |
| Squash merge | M | N | `git merge --squash` | Medium |
| Cherry-pick | M | N (selective) | `git cherry-pick SHA` | Medium |
| Manual merge | M > 10 | N > 10 | `git merge` + resolve | High |
| Reset + replay | M > 50 | N > 50 | Reset to base, replay commits | High |

### Step 3.1: Attempt Fast-Forward
```bash
git merge --ff-only origin/TARGET_BRANCH 2>&1
# If fails: "Not possible to fast-forward, aborting."
# → Move to Step 3.2
```

### Step 3.2: Attempt Rebase (if behind < 20)
```bash
git rebase origin/TARGET_BRANCH
# If conflicts:
#   1. git status → identify conflicted files
#   2. For each file: resolve, git add, git rebase --continue
#   3. If unresolvable: git rebase --abort, move to manual merge
```

### Step 3.3: Manual Merge (if rebase fails)
```bash
git merge origin/TARGET_BRANCH --no-edit
# Resolve conflicts per AGENTS.md rules:
#   - Prefer upstream changes for `.md` governance files
#   - Prefer HEAD changes for implementation code
#   - For specs: merge both, deduplicate
# git commit -m "merge: Sync with origin/TARGET_BRANCH resolving conflicts"
```

### Step 3.4: Strategy File Conflict Resolution (Critical)

For `.windsurf/workflows/*.md`, `.planning/*.md`, `AGENTS.md`:
```bash
# 1. Check both versions
git show HEAD:file.md > file_HEAD.md
git show origin/TARGET_BRANCH:file.md > file_TARGET.md

# 2. If both added new sections → concatenate
diff -u file_TARGET.md file_HEAD.md | patch -R -o file_merged.md

# 3. If both modified same section → human review required
#    Flag as BLOCKING, request user decision
```

---

## Phase 4: Validation & Safety Checks

### Step 4.1: Post-Merge Compilation
```bash
cargo check --package oms-engine --lib 2>&1 | grep "^error" | wc -l
# Must be 0 (or match pre-merge baseline)
```

### Step 4.2: Critical File Integrity
```bash
git diff --name-only HEAD~1 | grep -E "(Cargo\.toml|\.github/|k8s/)"
# If any: run additional validation
```

### Step 4.3: Commit Message Audit
```bash
git log --oneline -5
# Verify all commits follow conventional format: type(scope): description
# If any violate: flag for rebase --interactive rewrite
```

---

## Phase 5: Sync Execution (No GitHub Dependency)

### Step 5.1: Local Sync
```bash
# Push to origin (if network available)
git push origin HEAD:branch_name

# If network unavailable or GitHub fails:
#   - Keep local branch updated
#   - Log sync attempt with timestamp
#   - Retry on next session start
```

### Step 5.2: Multi-Branch Sync Order
```
1. main (baseline)
2. develop (if exists)
3. feature/* branches (newest first)
4. fix/* branches (critical first)
5. Dependabot branches (auto-generated, low priority)
```

---

## Phase 6: Knowledge Persistence

### Step 6.1: Sync Report Generation
```bash
cat > .planning/BRANCH_SYNC_REPORT_$(date +%Y%m%d_%H%M%S).md << 'EOF'
# Branch Sync Report: TIMESTAMP

## Branch States
| Branch | Behind | Ahead | Status | Action |
|--------|--------|-------|--------|--------|
...auto-generated...

## Conflicts Detected
...auto-generated...

## Merge Actions Taken
...auto-generated...

## Blockers
...auto-generated...
EOF
```

### Step 6.2: Store in mem0
```rust
// Store sync report as memory
mem0.add(
    content=report_content,
    user_id="traderx",
    agent_id="branch-sync",
    metadata={"type": "sync_report", "timestamp": "...", "branches_synced": N}
)
```

---

## Emergency Procedures

### GitHub Completely Unavailable
```bash
# 1. Work entirely in local branches
# 2. Use git bundle for offline transfer:
git bundle create backup.bundle --all
# 3. Transfer via filesystem, email, or alternative remote
# 4. On target machine: git clone backup.bundle
```

### Force Push Required (AGENTS.md prohibits on shared branches)
```bash
# ONLY for feature/* branches with NO collaborators
git push origin HEAD:feature/branch --force-with-lease
# --force-with-lease: fails if remote has commits we don't have
```

### Detached HEAD Recovery
```bash
git reflog | head -20
git checkout -b recovery_branch HEAD@{N}
```

---

## Automation Trigger

Run this workflow:
1. **Before every session start** (per session-start.md)
2. **After every commit** (post-commit hook)
3. **On user request**: `/branch-sync`
4. **On divergence detection**: > 10 commits behind any tracked branch
