# Product Manager Skill

**Unified Workflow Team:** Sprint Director  
**Follows:** `.windsurf/workflows/unified-team-execution.md` — DIAGNOSE → PLAN → EXECUTE → VERIFY → COMMIT → HANDOFF

## Trigger

Need to prioritize tasks, define success metrics, coordinate team efforts, track progress

## Action

### 1. Prioritize Build Optimization Tasks

```bash
# Assess current state
cargo check --package oms-engine 2>&1 | grep "^error" | wc -l
time cargo check --package oms-engine
cargo check --package oms-engine 2>&1 | grep "^warning" | wc -l
```

### 2. Define Success Metrics

- **P0 (Must Have):** 0 compilation errors, build completes successfully
- **P1 (Should Have):** Build time <10s, warnings <25, feature flags working
- **P2 (Nice to Have):** All modules <10KB, workspace separation, tooling installed

### 3. Coordinate Team Efforts

- Assign tasks based on priority
- Ensure dependencies between tasks are respected
- Track progress against timeline
- Resolve blockers

### 4. Timeline Management

- **Immediate (Today):** Fix P0 compilation errors
- **Short-term (This Week):** Apply workspace reorganization
- **Medium-term (Next Week):** Split monolithic modules
- **Ongoing:** Install and configure tooling

### 5. Risk Management

- Identify high-risk changes
- Define rollback procedures
- Ensure testing before deployment
- Monitor for regressions

### 6. Communication

- Update stakeholders on progress
- Document decisions and rationale
- Create status reports
- Coordinate with other teams

## Verification

- Tasks prioritized by severity ✅
- Success metrics defined ✅
- Team coordination established ✅
- Timeline tracked ✅
- Risks mitigated ✅

## Prevention Skills

- task-prioritizer.md
- metric-tracker.md
- risk-assessor.md
