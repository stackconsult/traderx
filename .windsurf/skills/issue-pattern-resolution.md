# Issue Pattern Resolution Skill

## Trigger
Recurring issues that block progress and require systematic resolution

## Action

### 1. Pattern Detection
```bash
# Identify recurring pattern
git log --oneline -50 | grep -E "(fix|error|fail)" | head -10
cargo check --package oms-engine 2>&1 | grep "^error" | wc -l
```

### 2. Root Cause Analysis
- Check if issue is compilation, dependency, or workflow related
- Verify if similar errors occurred in past sessions
- Identify if issue is environmental or code-related

### 3. Resolution Strategy Selection

#### Pattern A: Compilation Errors
- Fix borrow checker errors
- Resolve dependency conflicts
- Update Cargo.toml if needed

#### Pattern B: Git/Workflow Issues
- Verify git status and remote sync
- Check if commit was pushed
- Validate GitHub Actions status

#### Pattern C: Build Performance
- Apply profile optimizations
- Clean target directory
- Use incremental compilation

#### Pattern D: Disk Space
- Run cargo clean
- Remove unused dependencies
- Clear build artifacts

### 4. Execute Resolution
- Apply the fix specific to detected pattern
- Verify the fix resolves the issue
- Document the pattern and solution

### 5. Prevention
- Add skill to prevent recurrence
- Update workflows with checks
- Configure automated monitoring

## Verification
- Issue resolved ✅
- System builds successfully ✅
- No regression introduced ✅
- Pattern documented ✅

## Prevention Skills
- production-guard.md (validation)
- commit-effectiveness.md (git workflow)
- build-optimization.md (performance)
- disk-space-management.md (storage)
