# Issue Resolution Team Workflow

## Description
Specialized team workflow for resolving recurring issues that block development progress. Combines pattern detection, systematic resolution, and prevention.

## Team Composition

### 1. Issue Pattern Analyst
**Role**: Identify recurring issue patterns
- Analyze git history for recurring failures
- Detect compilation error patterns
- Identify workflow bottlenecks
- Map issue frequency and impact

### 2. Build System Engineer
**Role**: Resolve build and compilation issues
- Fix Rust compilation errors
- Optimize build performance
- Manage dependencies
- Configure build profiles

### 3. Git Workflow Specialist
**Role**: Resolve git and workflow issues
- Fix commit/push problems
- Resolve merge conflicts
- Ensure proper synchronization
- Validate remote state

### 4. System Architect
**Role**: Design prevention strategies
- Architect solutions to prevent recurrence
- Design monitoring systems
- Create automation workflows
- Document best practices

## Execution Flow

### Phase 1: Pattern Detection
```bash
# Issue Pattern Analyst
git log --oneline -50 | grep -E "(fix|error|fail)" | sort | uniq -c
cargo check --package oms-engine 2>&1 | grep "^error" | sort | uniq -c
```

### Phase 2: Triage and Assignment
- Build issues → Build System Engineer
- Git issues → Git Workflow Specialist
- Performance issues → Build System Engineer
- Workflow issues → Git Workflow Specialist
- Architecture issues → System Architect

### Phase 3: Resolution Execution
Each specialist applies their domain-specific skills:
- issue-pattern-resolution.md (analyst)
- build-system-engineer.md (build engineer)
- git-workflow-specialist.md (git specialist)
- system-architect.md (architect)

### Phase 4: Verification
- Test the fix
- Verify no regression
- Update documentation
- Commit and push

### Phase 5: Prevention
- Add monitoring
- Create automation
- Update workflows
- Document patterns

## Current Issue Pattern Analysis

### Recurring Issues Identified:
1. **Compilation Errors** (borrow checker, missing fields)
2. **Git Sync Issues** (commit not pushed, staging problems)
3. **Build Performance** (slow builds, disk space)
4. **Dependency Issues** (outdated deps, conflicts)

### Resolution Teams Assigned:
- Compilation Errors → Build System Engineer
- Git Sync Issues → Git Workflow Specialist
- Build Performance → Build System Engineer
- Dependency Issues → Build System Engineer

## Success Metrics
- Issue resolution time < 15 minutes
- Recurrence rate < 10%
- Zero compilation errors
- Build time < 20 seconds
- Git sync always successful

## Prevention Strategies
- Automated issue detection
- Pre-commit hooks
- Build monitoring
- Git workflow automation
- Dependency update automation
