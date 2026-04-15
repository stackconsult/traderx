# Branch Workflow Propagation Analysis

**Analysis Date**: 2026-04-15  
**Analyst**: Cascade  
**Purpose**: Determine if `/repository-audit` workflow needs to be included in other GitHub branches  
**Scope**: All remote branches on origin

---

## Executive Summary

### **FINDING: YES - Propagation Required**

**Status**: The newly created `/repository-audit` workflow is **NOT PRESENT** in any remote branch yet (it only exists locally in `feature/github-mcp-setup`).

**Action Required**: Commit and push to `feature/github-mcp-setup`, then merge to `main`.

---

## Branch Inventory

### **Remote Branches on GitHub (origin)**

| Branch | Commit SHA | Last Updated | Status |
|--------|-----------|--------------|--------|
| **main** | `3ff25f4` | Recent | Default branch |
| **feature/github-mcp-setup** | `438b8d9` | Recent | Active development |
| **fix/oms-engine-compilation-errors** | `913e700` | Recent | Fix branch |

**Total Branches**: 3 remote branches

### **Local Branches**

| Branch | Tracking | Status |
|--------|----------|--------|
| **feature/github-mcp-setup** | origin/feature/github-mcp-setup | ✅ Up to date |
| **fix/oms-engine-compilation-errors** | origin/fix/oms-engine-compilation-errors | ⚠️ Local out of date |
| **main** | origin/main | ✅ Fast-forwardable |

---

## Workflow Files by Branch

### **Main Branch (`3ff25f4`)**

**`.windsurf/workflows/` Directory Contents:**

```
.windsurf/workflows/
├── preflight-checklist.md              ✅ Present
├── quality-guardian.md                   ✅ Present
├── quant-enhancement.yaml              ✅ Present
├── session-start.md                      ✅ Present
├── task-refinement.md                  ✅ Present
├── repository-audit.md                 ❌ NOT PRESENT
└── spec-driven-workflow/               ✅ Present (73 items)
    ├── prompts/
    │   ├── SDD-1-generate-spec.md
    │   ├── SDD-2-generate-task-list-from-spec.md
    │   ├── SDD-3-manage-tasks.md
    │   └── SDD-4-validate-spec-implementation.md
    └── ... (other files)
```

**Summary**: 6 workflow files + 73 spec-driven items = 79 total

---

### **Feature Branch (`438b8d9`) - Current**

**`.windsurf/workflows/` Directory Contents:**

```
.windsurf/workflows/
├── preflight-checklist.md              ✅ Present
├── quality-guardian.md                   ✅ Present
├── quant-enhancement.yaml              ✅ Present
├── session-start.md                      ✅ Present
├── task-refinement.md                  ✅ Present
├── repository-audit.md                 ⚠️ LOCAL ONLY (not pushed)
└── spec-driven-workflow/               ✅ Present (73 items)
```

**Summary**: Same as main + `repository-audit.md` (local only) = 80 total

**Difference from Main**:
- `.windsurf/AGENTS.internal.md` (confidential agent governance)

---

### **Fix Branch (`913e700`)**

**`.windsurf/workflows/` Directory Contents:**

```
.windsurf/workflows/
├── preflight-checklist.md              ✅ Present
├── quality-guardian.md                   ✅ Present
├── quant-enhancement.yaml              ✅ Present
├── session-start.md                      ✅ Present
├── task-refinement.md                  ✅ Present
├── repository-audit.md                 ❌ NOT PRESENT
└── spec-driven-workflow/               ✅ Present
```

**Summary**: Same as main = 79 total

**Difference from Main**:
- Root-level test files (`standalone_test.rs`, `test_trading_flow.rs`)
- OMS Engine fixes (compilation, new binaries)
- No workflow changes

---

## Propagation Strategy

### **Option 1: Merge to Main (RECOMMENDED)**

**Flow**:
```
feature/github-mcp-setup (local) 
  ├─ Commit repository-audit.md
  ├─ Push to origin/feature/github-mcp-setup
  ├─ Create PR → main
  └─ Merge → main now has all workflows
```

**Advantages**:
- ✅ Clean propagation path
- ✅ Main gets all MCP governance + new workflow
- ✅ Fix branch will inherit on next rebase/merge
- ✅ Follows standard git workflow

**Timeline**:
1. Commit `repository-audit.md` locally (now)
2. Push to `origin/feature/github-mcp-setup` (5 min)
3. Create PR to `main` (5 min)
4. Review and merge (15 min)
5. **Total**: 25 minutes

---

### **Option 2: Cherry-Pick to Each Branch**

**Flow**:
```
Commit repository-audit.md
  ├─ Cherry-pick to main
  ├─ Cherry-pick to fix/oms-engine-compilation-errors
  └─ All branches have it independently
```

**Advantages**:
- ✅ Immediate availability on all branches
- ✅ No merge dependencies

**Disadvantages**:
- ❌ Creates duplicate commits
- ❌ History fragmentation
- ❌ Maintenance overhead

**Timeline**:
1. Commit and push (5 min)
2. Cherry-pick to main (2 min)
3. Cherry-pick to fix branch (2 min)
4. Push all branches (5 min)
5. **Total**: 14 minutes

---

### **Option 3: Separate Workflow Branch**

**Flow**:
```
Create workflow/feature-branch
  ├─ Add repository-audit.md
  ├─ Merge to main
  ├─ Merge to feature/github-mcp-setup
  └─ Merge to fix branch
```

**Advantages**:
- ✅ Isolated workflow changes
- ✅ Can be reviewed independently

**Disadvantages**:
- ❌ Overhead of managing another branch
- ❌ Unnecessary complexity

**Timeline**: 30+ minutes

---

## Recommendation

### **APPROVED: Option 1 - Merge to Main**

**Rationale**:
1. `feature/github-mcp-setup` already contains all MCP governance files
2. The branch is designed to be merged to main (feature branch purpose)
3. Fix branch should merge main eventually anyway
4. Cleanest git history
5. Aligns with MCP governance workflow (Discuss → Plan → Execute → Verify → Review → Ship)

**Execution Plan**:

| Step | Action | Command | Time |
|------|--------|---------|------|
| 1 | Stage new files | `git add .windsurf/workflows/repository-audit.md MCP_RULES_AUDIT_ANALYSIS.md` | 1 min |
| 2 | Commit | `git commit -m "feat(workflow): add repository-audit for automated repo analysis"` | 1 min |
| 3 | Push | `git push origin feature/github-mcp-setup` | 2 min |
| 4 | Create PR | GitHub UI / gh CLI | 5 min |
| 5 | Review | Ensure all checks pass | 10 min |
| 6 | Merge | Merge to main | 2 min |
| 7 | Verify | Confirm main has workflow | 2 min |

**Total Time**: ~25 minutes

---

## Impact Analysis

### **After Propagation to Main**

| Branch | Will Have repository-audit.md | Method |
|--------|------------------------------|--------|
| **main** | ✅ Yes | Direct merge |
| **feature/github-mcp-setup** | ✅ Yes | Source of truth |
| **fix/oms-engine-compilation-errors** | ✅ Yes | Via main merge/rebase |

### **Benefits of Propagation**

1. **Standardization**: All branches use same audit workflow
2. **Consistency**: No branch lacks repository analysis capability
3. **Automation**: Future repo analysis uses standardized workflow
4. **Compliance**: All branches comply with Law 4 (Workflow Absolutism)
5. **Documentation**: Audit reports generated consistently across branches

---

## Files to Commit

### **New Files (Created This Session)**

| File | Purpose | Lines | Branch |
|------|---------|-------|--------|
| `.windsurf/workflows/repository-audit.md` | Automated repository audit workflow | 450 | feature/github-mcp-setup |
| `MCP_RULES_AUDIT_ANALYSIS.md` | Analysis document | ~600 | feature/github-mcp-setup |
| `REPO_REVIEW_POST_AGENT_UPDATES.md` | Repository review | ~800 | feature/github-mcp-setup |

**Total**: ~1,850 lines of new documentation/workflow

---

## Verification Checklist

### **Before Merge**

- [ ] `repository-audit.md` committed locally
- [ ] `MCP_RULES_AUDIT_ANALYSIS.md` committed locally
- [ ] All workflow files present in `.windsurf/workflows/`
- [ ] No syntax errors in markdown files
- [ ] Local tests pass (if applicable)

### **After Merge to Main**

- [ ] `repository-audit.md` visible in main branch
- [ ] File accessible via GitHub web interface
- [ ] Can execute `/repository-audit` command (in IDE)
- [ ] All 3 branches have consistent workflow set

---

## Long-Term Strategy

### **Future Workflow Management**

**Recommendation**: Establish workflow as infrastructure

```
.windsurf/workflows/ (treat as shared infrastructure)
├── Core workflows (all branches must have)
│   ├── session-start.md              ← MANDATORY
│   ├── preflight-checklist.md        ← MANDATORY
│   ├── quality-guardian.md           ← MANDATORY
│   ├── repository-audit.md           ← MANDATORY (new)
│   └── task-refinement.md            ← Optional
├── Specialized workflows (branch-specific)
│   └── [branch-specific workflows]
└── Deprecated workflows (to be removed)
    └── [marked for deletion]
```

**Policy**: Any new workflow added to `feature/github-mcp-setup` should be:
1. Tested locally
2. Committed and pushed
3. Merged to main within 48 hours
4. Propagated to active feature/fix branches

---

## Conclusion

### **Answer**: **YES - Propagation REQUIRED**

**Current State**: `repository-audit.md` exists only locally in `feature/github-mcp-setup`

**Required State**: All branches should have consistent workflow infrastructure

**Recommended Action**:
1. ✅ Commit all new workflow files locally (NOW)
2. ✅ Push to `origin/feature/github-mcp-setup` (NOW)
3. ⏳ Create PR to `main` (NEXT)
4. ⏳ Merge after review (NEXT)
5. ⏳ Propagate to `fix/oms-engine-compilation-errors` (AFTER main merge)

**Timeline**: 25 minutes to full propagation

---

**Status**: Ready to execute propagation plan
