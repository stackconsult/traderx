# TraderX MCP Branch - Agentic Manufacturing Governance

**Branch:** `feature/github-mcp-setup`  
**Repository:** https://github.com/stackconsult/traderx  
**Mode:** ABSOLUTE GUARDRAILED PRODUCTION ENGINEERING  
**Agent:** Cascade (Windsurf IDE)  
**Classification:** NO PSEUDO-CODE / NO MIMICS / ONLY PRODUCTION-GRADE ARTIFACTS

---

## ABSOLUTE LAWS (Zero Exceptions)

### Law 1: SESSION START MANDATE
**EVERY session MUST begin with `/session-start` workflow execution.**

**FAILURE TO EXECUTE = SESSION INVALID. NO WORK MAY PROCEED.**

Pre-flight checklist (ALL must pass):
- [ ] Workflows synced from upstream (FavioVazquez/agentic-learning, pbakaus/impeccable)
- [ ] Skills downloaded and validated (21 impeccable + agentic-learning)
- [ ] Agent skills loaded and functional
- [ ] Previous session work analyzed and graded
- [ ] Full repository sync check (GitHub + Hugging Face)
- [ ] Roadmap status reviewed and confirmed
- [ ] No ambiguity about next build phase

### Law 2: PRODUCTION-ONLY CODE MANDATE
**PSEUDO-CODE IS FORBIDDEN. MIMICS ARE FORBIDDEN. ONLY PRODUCTION CODE.**

- Every line must be masterfully engineered
- Every function must be tested during construction
- Every module must pass recursive up-engineering validation
- Self-learning mechanisms must be embedded
- Self-healing error recovery must be implemented

### Law 3: RECURSIVE UP-ENGINEERING MANDATE
**All code MUST undergo continuous improvement cycles:**

1. Build → Test → Validate → Grade
2. Analyze weaknesses → Research solutions → Implement upgrades
3. Verify improvements → Document learnings → Compound knowledge
4. Iterate until benchmark quality achieved

### Law 4: WORKFLOW ABSOLUTISM
**NO task may begin without active workflows, skills, and agent skills.**

Guardrails must be engaged at all times:
- Phase Loop: Discuss → Plan → Execute → Verify → Review → Ship → Compound
- Wave-ordered execution with binary milestone verification
- UAT-driven validation before any merge
- Multi-persona code review before any commit

### Law 5: PRE-TASK INTELLIGENCE MANDATE
**Before ANY coding task:**

1. **Past Work Analysis**: Grade previous session work (A-F scale)
   - Code quality assessment
   - Test coverage validation
   - Documentation completeness
   - Architectural alignment

2. **Repository Sync Check**:
   - Query GitHub API for new workflow releases
   - Check Hugging Face for updated agent skills
   - Validate local `.windsurf/` matches upstream
   - Flag any divergence for immediate sync

3. **Skill Integration Assessment**:
   - Identify if new building/coding skills available
   - Evaluate if current skills need updates
   - Plan integration points for new capabilities
   - Execute `/sync-upstream-skills` if needed

### Law 6: ROADMAP CLARITY MANDATE
**ZERO confusion about status, direction, or next steps.**

Required at session start:
- Full scope roadmap review (`ROADMAP.md` or `MILESTONES.md`)
- Current phase status confirmation
- Next 3 deliverables clearly defined
- Blockers identified with mitigation plans
- Dependencies mapped and validated

### Law 7: VALIDATION BENCHMARK MANDATE
**ALL work must achieve top-tier benchmarks:**

| Category | Minimum Standard | Validation Method |
|----------|-----------------|-------------------|
| Code Quality | A-grade (no lint errors, 100% type coverage) | `proofs/test-[component].json` |
| Test Coverage | 90%+ unit, 80%+ integration | `proofs/coverage-[component].json` |
| Security | Zero critical/high vulnerabilities | `proofs/security-[component].json` |
| Performance | Within 10% of baseline or better | `proofs/perf-[component].json` |
| Documentation | Complete API docs + usage examples | `proofs/docs-[component].json` |

---

## Git Workflow for MCP Branch

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
1. **Every commit** requires:
   - JOURNAL.md entry explaining changes
   - Proof artifacts in `proofs/`
   - Test results attached
   - Performance metrics recorded

2. **Commit message format** (Conventional Commits):
   ```
   type(scope): subject
   
   [body explaining what and why]
   
   Proof: proofs/test-[component].json
   Proof: proofs/perf-[component].json
   ```

3. **Pre-commit checks** (MUST pass):
   - All tests passing
   - Linting clean
   - Type checking passed
   - Security scan clean
   - Documentation updated

### PR Requirements
Before creating PR to `main`:
- [ ] All proof artifacts generated
- [ ] Multi-persona review completed (`/review`)
- [ ] UAT validation passed (`/verify-work`)
- [ ] Security audit passed
- [ ] Performance regression test passed
- [ ] JOURNAL.md updated with complete entry
- [ ] DECISIONS.md updated if architectural changes

---

## Session Start Protocol (MANDATORY)

### Step 1: Environment Validation
```bash
# Verify Windsurf MCP config
Test-Path "$env:USERPROFILE\.windsurf\mcp_config.json"

# Verify GitHub token available
$env:GITHUB_TOKEN -ne $null

# Verify learnship installed
Test-Path ".windsurf/workflows/sync-upstream-skills.md"
```

### Step 2: Skill Sync & Validation
```
EXECUTE: /sync-upstream-skills
- Pull latest from FavioVazquez/agentic-learning
- Pull latest from pbakaus/impeccable  
- Verify all 21 impeccable sub-skills present
- Re-run installer for all platforms
- Validate skill integrity
```

### Step 3: Previous Session Analysis
```
EXECUTE: /audit-milestone (if milestone completed)
OR
EXECUTE: Manual grading of last session work:
- Review last JOURNAL.md entry
- Check git diff of changes
- Validate all proof artifacts present
- Grade: A (excellent), B (good), C (acceptable), D (needs work), F (failed)
- Document lessons learned
```

### Step 4: Repository Sync Check
```
QUERY GitHub API:
- Check for new releases in workflow repos
- Check for new commits in skills repos
- Verify local branch status vs origin
- Check for any unmerged changes

QUERY Hugging Face:
- Check for new agent skill uploads
- Validate current skills are latest versions
```

### Step 5: Roadmap Review
```
READ: MILESTONES.md
READ: IMPLEMENTATION_PLAN.md (relevant phase)
CONFIRM:
- Current phase status
- Next 3 deliverables
- Blockers and mitigations
- Dependencies ready
```

### Step 6: Session Initialization Complete
Only when ALL steps pass:
```
STATUS: ✅ SESSION GUARDRAILS ACTIVE
WORKFLOWS: ✅ LOADED
SKILLS: ✅ VALIDATED
AGENT SKILLS: ✅ ACTIVE
ROADMAP: ✅ CONFIRMED
READY TO BUILD: ✅ PRODUCTION MODE
```

---

## Coding Standards (ABSOLUTE)

### Code Quality Requirements
1. **No TODOs in production code** - Either implement or remove
2. **No commented-out code** - Use git history if needed
3. **No magic numbers** - Use named constants
4. **No silent failures** - All errors logged and handled
5. **No unhandled exceptions** - Graceful degradation required
6. **No blocking I/O in hot paths** - Async/await mandatory
7. **No hardcoded secrets** - Environment variables only
8. **No direct DB queries in business logic** - Repository pattern required

### Testing Requirements (During Construction)
- Write tests BEFORE or WITH implementation
- Test at function level as you build
- Integration tests for every module
- Performance tests for critical paths
- Security tests for all inputs

### Documentation Requirements
- Every public function documented
- Complex logic explained with comments
- Architecture decisions in DECISIONS.md
- API changes in CHANGELOG.md
- Usage examples for all public APIs

### Self-Learning & Self-Healing Requirements
1. **Self-Learning**:
   - Log all decisions with rationale
   - Capture patterns in `/compound` solutions
   - Build knowledge base from each session
   - Cross-reference with similar past problems

2. **Self-Healing**:
   - Implement circuit breakers for external calls
   - Add retry logic with exponential backoff
   - Monitor health and auto-recover
   - Alert on anomalies with context

---

## Agent Hoard Protocol

### Multi-Agent Coordination
When parallel agents engaged:
1. **Claude 4.6 (Thinking)**: Planning, architecture, complex decisions
2. **Gemma 4 (Execution)**: Implementation, testing, validation
3. **Cascade (Windsurf)**: Local orchestration, IDE integration
4. **Meta-Coordinator**: Handoff management, state synchronization

### Handoff Protocol
```
HandoffPackage:
  - Task definition with clear boundaries
  - Compressed context (TurboQuant)
  - Deterministic plan with rollback
  - Expected outcomes and validation criteria
  - Timeout and error handling
```

### State Synchronization
- All agents read from same JOURNAL.md
- Proof artifacts shared via git
- DECISIONS.md as single source of truth
- Milestone tracking via binary verification

---

## Build Experience Mandate

### Developer Experience Requirements
- Clear error messages with solutions
- Fast feedback loops (tests run quickly)
- Intelligent defaults with override options
- Progress visibility on long operations
- Rollback capability for all changes

### End User Experience (when applicable)
- Responsive UI with loading states
- Clear feedback on actions
- Graceful error handling
- Accessibility compliance
- Performance optimized

---

## Validation & Verification Protocol

### Continuous Validation
```
BUILD → TEST → VALIDATE → GRADE → COMPOUND
   ↑_________________________________|
```

### Before Any Commit
1. Run full test suite
2. Check code coverage
3. Run security scan
4. Run performance benchmark
5. Update proof artifacts
6. Grade the work (A-F)

### Before Any PR
1. Multi-persona review (`/review`)
2. UAT validation (`/verify-work`)
3. Documentation sync check (`/sync-docs`)
4. Milestone audit (`/audit-milestone`)
5. Health check (`/health`)

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
1. Read MILESTONES.md
2. Read current phase in IMPLEMENTATION_PLAN.md
3. Execute `/ls` for status
4. Ask for clarification if still unclear
5. Document the confusion source for improvement

### If Quality Degrades
1. Halt current work
2. Run `/review` for assessment
3. Identify root cause
4. Apply `/compound` to capture fix
5. Resume only after quality restored

---

**ACKNOWLEDGMENT REQUIRED**: Every session MUST begin by confirming these laws are understood and the `/session-start` workflow has been executed. No exceptions.
