# MCP Branch Agentic Manufacturing Governance

**Branch**: `feature/github-mcp-setup`  
**Created**: 2026-04-15  
**Purpose**: Absolute guardrails for production-grade agentic code manufacturing

---

## Executive Summary

This document describes the comprehensive governance system established for the TraderX MCP branch. It ensures:

- **Zero ambiguity** about status or direction
- **Zero missing guardrails** - workflows/skills always active
- **Production-only code** - no pseudo-code, no mimics
- **A-grade quality** through recursive up-engineering
- **Self-learning & self-healing** systems embedded
- **Multi-agent coordination** for maximum efficiency

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    AGENTS.branch.mcp.md                     │
│              (7 Absolute Laws - Zero Exceptions)              │
└─────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        ▼                     ▼                     ▼
┌───────────────┐   ┌───────────────────┐   ┌───────────────┐
│  /session-    │   │  /preflight-      │   │  /quality-    │
│   start       │   │   checklist       │   │   guardian    │
│               │   │                   │   │               │
│ • Environment │   │ • Binary checks   │   │ • 5 Gates     │
│ • Skill sync  │   │ • 32 validations  │   │ • A-F Grading │
│ • Past work   │   │ • Go/No-Go        │   │ • Self-heal   │
│ • Repo sync   │   │ • PS1 script      │   │ • Proof gen   │
│ • Roadmap     │   │                   │   │               │
└───────────────┘   └───────────────────┘   └───────────────┘
        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    PRODUCTION WORK                          │
│          (A-Grade, Tested, Documented, Secured)             │
└─────────────────────────────────────────────────────────────┘
```

---

## The 7 Absolute Laws

### Law 1: Session Start Mandate
**Every session MUST begin with `/session-start` workflow execution.**

Failure = Session Invalid. No work may proceed.

**Phases**:
1. Environment Validation (MCP config, GitHub token, structure)
2. Skill Sync & Validation (21 impeccable + agentic-learning)
3. Previous Session Analysis (JOURNAL review, grade A-F)
4. Repository Sync Check (GitHub API, Hugging Face)
5. Roadmap & Direction Clarity (MILESTONES.md confirmation)
6. Session Initialization Summary (binary GO/NO-GO)

### Law 2: Production-Only Code Mandate
**PSEUDO-CODE IS FORBIDDEN. MIMICS ARE FORBIDDEN. Only production code.**

Requirements:
- Every line masterfully engineered
- Every function tested during construction
- Every module passes recursive up-engineering
- Self-learning mechanisms embedded
- Self-healing error recovery implemented

### Law 3: Recursive Up-Engineering Mandate
**All code undergoes continuous improvement cycles:**

```
Build → Test → Validate → Grade
   ↑________↓
Analyze → Research → Upgrade
   ↑________↓
Verify → Document → Compound
```

### Law 4: Workflow Absolutism
**NO task begins without active workflows, skills, and agent skills.**

Guardrails engaged at all times:
- Phase Loop: Discuss → Plan → Execute → Verify → Review → Ship → Compound
- Wave-ordered execution with binary milestones
- UAT-driven validation before merge
- Multi-persona code review before commit

### Law 5: Pre-Task Intelligence Mandate
**Before ANY coding task:**

1. **Past Work Analysis**: Grade previous session (A-F)
   - Code quality assessment
   - Test coverage validation
   - Documentation completeness
   - Architectural alignment

2. **Repository Sync Check**:
   - Query GitHub API for new workflow releases
   - Check Hugging Face for updated agent skills
   - Validate local `.windsurf/` matches upstream
   - Flag divergence for immediate sync

3. **Skill Integration Assessment**:
   - Identify new building/coding skills
   - Evaluate skill update needs
   - Plan integration points
   - Execute `/sync-upstream-skills` if needed

### Law 6: Roadmap Clarity Mandate
**ZERO confusion about status, direction, or next steps.**

Required at session start:
- Full scope roadmap review (`MILESTONES.md`)
- Current phase status confirmation
- Next 3 deliverables clearly defined
- Blockers identified with mitigation plans
- Dependencies mapped and validated

### Law 7: Validation Benchmark Mandate
**ALL work must achieve top-tier benchmarks:**

| Category | Minimum Standard | Proof Artifact |
|----------|-----------------|----------------|
| Code Quality | A-grade (no lint, 100% type) | `proofs/test-[component].json` |
| Test Coverage | 90% unit, 80% integration | `proofs/coverage-[component].json` |
| Security | Zero critical/high vulns | `proofs/security-[component].json` |
| Performance | Within 10% of baseline | `proofs/perf-[component].json` |
| Documentation | Complete API + examples | `proofs/docs-[component].json` |

---

## Git Workflow

### Branch Strategy
```
local machine (Windsurf/Cascade)
    ↓
feature/github-mcp-setup (origin)
    ↓
Pull Request (review required)
    ↓
main (merged)
```

### Commit Protocol
Every commit requires:
- JOURNAL.md entry
- Proof artifacts in `proofs/`
- Test results attached
- Performance metrics recorded
- Conventional commit format

### PR Requirements
- `/review` multi-persona code review
- `/verify-work` UAT validation
- `/audit-milestone` validation
- All proof artifacts present

---

## Workflow Commands

### Session Start (MANDATORY)
```
/session-start
```
6-phase initialization that MUST complete before any work.

### Preflight Checklist
```
/preflight-checklist
```
32-point binary validation with PowerShell automation.

### Quality Guardian
```
/quality-guardian
```
5-gate quality enforcement with A-F grading.

### Skill Sync
```
/sync-upstream-skills
```
Pulls latest from:
- `FavioVazquez/agentic-learning`
- `pbakaus/impeccable` (21 sub-skills)

### Additional Commands
- `/ls` - Show current status
- `/health` - Project health check
- `/review` - Multi-persona code review
- `/compound` - Capture solution
- `/audit-milestone` - Validate milestone

---

## Multi-Agent Coordination

### Agent Roles
1. **Claude 4.6 (Thinking)**: Planning, architecture, decisions
2. **Gemma 4 (Execution)**: Implementation, testing, validation
3. **Cascade (Windsurf)**: Local orchestration, IDE integration
4. **Meta-Coordinator**: Handoff management, state sync

### Handoff Protocol
```typescript
interface HandoffPackage {
  id: string;
  task: TaskDefinition;
  context: CompressedContext;  // TurboQuant compression
  plan: DeterministicPlan;
  expectedOutcome: Outcome;
  timeout: number;
  rollbackPlan: RollbackPlan;
}
```

---

## Self-Learning & Self-Healing

### Self-Learning
- Log all decisions with rationale
- Capture patterns in `/compound` solutions
- Build knowledge base from each session
- Cross-reference with similar past problems

### Self-Healing
- Circuit breakers for external calls
- Retry logic with exponential backoff
- Health monitoring and auto-recovery
- Anomaly alerts with full context

---

## Quality Gates

### Gate 1: Code Quality
**Zero tolerance for**:
- TODOs in production code
- Commented-out code
- Magic numbers
- Silent failures
- Unhandled exceptions
- Blocking I/O in hot paths
- Hardcoded secrets
- Direct DB queries in logic

### Gate 2: Test Coverage
- 90%+ unit coverage
- 80%+ integration coverage
- Property-based tests
- Edge case documentation

### Gate 3: Security
- Zero critical/high vulnerabilities
- No secrets in code
- Input validation on all entry points
- Dependency vulnerability scanning

### Gate 4: Performance
- Benchmarks for critical paths
- No regression > 10% from baseline
- Memory usage profiled
- Latency measured for trading ops

### Gate 5: Documentation
- 100% public function coverage
- README updated for API changes
- Architecture decisions in DECISIONS.md
- CHANGELOG updated
- Usage examples provided

---

## Grading System

### A-Grade (Excellent)
- All gates pass
- Zero violations
- Coverage >= 90%
- No security issues
- Performance neutral or improved
- Documentation 100%

### B-Grade (Good)
- All gates pass
- Minor violations (< 5)
- Coverage >= 85%
- No critical/high security issues
- Performance < 5% regression
- Documentation >= 95%

### C-Grade (Acceptable)
- All gates pass
- Some violations (< 10)
- Coverage >= 80%
- Only low security issues
- Performance < 10% regression
- Documentation >= 90%

### D-Grade (Needs Work)
- Some gates fail
- Multiple violations
- Coverage < 80%
- Medium security issues
- Performance regression > 10%
- Documentation incomplete

### F-Grade (Failed)
- Multiple gates fail
- Many violations
- Coverage < 70%
- High/critical security issues
- Severe performance regression
- Documentation missing

---

## Files Reference

### Governance
- `AGENTS.branch.mcp.md` - 7 absolute laws

### Workflows
- `.windsurf/workflows/session-start.md` - Mandatory initialization
- `.windsurf/workflows/preflight-checklist.md` - Binary validation
- `.windsurf/workflows/quality-guardian.md` - Quality enforcement
- `.windsurf/workflows/sync-upstream-skills.md` - Skill sync (from learnship)

### Configuration
- `.windsurf/mcp_config.json` - MCP server config
- `.env.example` - Environment template (GITHUB_TOKEN)

### Documentation
- `docs/GITHUB_MCP_SETUP.md` - GitHub MCP setup guide
- `docs/MCP_BRANCH_GOVERNANCE.md` - This document

### Proof Artifacts
- `proofs/MCP_BRANCH_SETUP.json` - Completion proof

---

## Emergency Protocols

### If Workflows Missing
1. STOP all work immediately
2. Execute `/sync-upstream-skills`
3. Validate all workflows present
4. Only resume when guardrails restored

### If Skills Outdated
1. Check upstream repositories
2. Pull latest versions
3. Re-run installer
4. Validate functionality
5. Document version changes

### If Confusion About Direction
1. Read `MILESTONES.md`
2. Read `IMPLEMENTATION_PLAN.md`
3. Execute `/ls` for status
4. Ask for clarification
5. Document confusion source

### If Quality Degrades
1. Halt current work
2. Run `/review` for assessment
3. Identify root cause
4. Apply `/compound` to capture fix
5. Resume only after quality restored

---

## Success Criteria

This branch is successful when:
- ✅ GitHub MCP fully operational
- ✅ Session start workflow automated
- ✅ All skills synced and validated
- ✅ Zero pseudo-code in repository
- ✅ All code A-grade quality
- ✅ Self-learning mechanisms active
- ✅ Self-healing systems operational
- ✅ Build experience optimized
- ✅ All proof artifacts current
- ✅ Documentation comprehensive
- ✅ Ready for PR to main

---

## Quick Start

### At Beginning of Every Session
```bash
# 1. Start Windsurf IDE
# 2. Execute:
/session-start

# 3. Confirm all checks pass
# 4. Begin production work
```

### Before Every Commit
```bash
# 1. Run quality guardian
/quality-guardian

# 2. Confirm grade is A or B
# 3. Generate proof artifacts
# 4. Update JOURNAL.md
# 5. Commit with conventional format
```

### Before Every PR
```bash
# 1. Full quality check
/quality-guardian --full

# 2. Multi-persona review
/review

# 3. UAT validation
/verify-work

# 4. Create PR
```

---

**ACKNOWLEDGMENT REQUIRED**: Every session MUST begin by confirming these laws are understood and the `/session-start` workflow has been executed.
