# Skills Wiring Report
**Generated:** 2026-05-03
**Purpose:** Systematic wiring of 48 skills for repeatable full stack engineering

## Executive Summary

This report provides a comprehensive wiring strategy for 48 skills installed from three sources:
- **12 skills** from autonomous-upskilling workflow
- **8 skills** from learnship marketplace
- **19 skills** from addyosmani/agent-skills
- **9 existing skills** from original TraderX codebase

**Total: 48 skills organized into systematic execution pipelines**

## MCP Server Integration

**Available MCP Server:** `io.windsurf/exa-code`
- `web_search_exa` - Real-time web search (enabled)
- `web_fetch_exa` - Extract content from specific URLs (enabled)

**Integration Strategy:**
- Skills use MCP for external data retrieval
- MCP provides tool access; skills provide workflow logic
- Skills orchestrate multiple MCP tools for complex workflows

## Skill Architecture Principles (Based on 2026 Research)

1. **Skills as workflow nodes** - Skills are complete capabilities, not API calls
2. **Progressive disclosure** - Overview in main file, details in separate files
3. **Skill composition** - Skills reference other skills, don't embed them
4. **MCP/skill separation** - MCP = tools, Skills = workflow logic
5. **80/20 rule** - Focus on 20% of workflows consuming 80% of time
6. **Outcome-based evaluation** - Success criteria in skill itself

## Skill Taxonomy

### Phase 1: Define & Plan (8 skills)

**Clarification & Ideation:**
- `idea-refine` - Clarify what to build, refine ideas
- `agentic-learning` - Neuroscience-backed learning partner

**Specification & Planning:**
- `spec-driven-development` - Spec-driven workflow
- `planning-and-task-breakdown` - Break down tasks
- `learnship-planner` - Create PLAN.md with wave-ordered tasks

**Context Engineering:**
- `context-engineering` - Context hierarchy management
- `source-driven-development` - Follow official docs

**Decision Tracking:**
- `decision-log` - Capture architectural decisions

### Phase 2: Build (12 skills)

**Implementation:**
- `incremental-implementation` - Incremental builds
- `learnship-executor` - Execute PLAN.md atomically
- `test-driven-development` - TDD workflow

**Backend Engineering:**
- `api-and-interface-design` - Stable API design
- `code-simplification` - Simplification principles
- `adaptive-self-healing` - Immutable security functions

**Frontend Engineering:**
- `frontend-ui-engineering` - Production UIs
- `impeccable` - 21 UI design quality actions
- `hexagonal-adapters` - Hexagonal architecture

**AI/LLM Engineering:**
- `pattern-layer-architecture` - Pattern layer design
- `ml-physics-modeling` - ML physics modeling

**Database Engineering:**
- `rls-multi-tenancy` - Row-level security patterns

### Phase 3: Verify (7 skills)

**Testing & Validation:**
- `browser-testing-with-devtools` - Browser testing
- `production-guard` - 4-layer validation system

**Debugging:**
- `debugging-and-error-recovery` - Systematic debugging
- `debug-team` - Hypothesis testing with mem0
- `learnship-debugger` - Debugging agent persona

**Quality Assurance:**
- `qa-team` - Multi-lens code review (7 lenses)

**Verification:**
- `learnship-verifier` - Phase verification with must-haves

### Phase 4: Review (6 skills)

**Code Review:**
- `code-review-and-quality` - Quality gates
- `review` - Multi-persona code review

**Security:**
- `security-and-hardening` - OWASP Top 10 prevention
- `security-audit-gate` - Security audit gate

**Performance:**
- `performance-optimization` - Core Web Vitals optimization

**Audit:**
- `repository-audit` - Comprehensive repository audit
- `audit-compliance` - Audit compliance

### Phase 5: Ship (5 skills)

**Git & Versioning:**
- `git-workflow-and-versioning` - Git workflows
- `commit-effectiveness` - 6-step commit verification

**CI/CD:**
- `ci-cd-and-automation` - CI/CD pipeline setup

**Deployment:**
- `shipping-and-launch` - Deployment strategies
- `ship` - Ship pipeline

**Migration:**
- `deprecation-and-migration` - Migration strategies

### Phase 6: Operations (10 skills)

**Documentation:**
- `documentation-and-adrs` - ADR documentation
- `sync-docs` - Detect stale documentation

**Monitoring:**
- `github-first-self-healing` - Continuous monitoring
- `quality-guardian` - Continuous quality enforcement
- `production-guard` - Production guard

**Knowledge Management:**
- `knowledge-base` - Aggregate key learnings
- `auto-journal-sync` - Sync journal and agent system

**Project Management:**
- `new-milestone` - Start new milestone
- `complete-milestone` - Archive completed milestone
- `milestone-retrospective` - Structured learning retrospective
- `new-project` - Initialize new project

**Branch Management:**
- `branch-sync-strategic-research` - Branch synchronization
- `github-sync-safeguard` - GitHub sync safeguard

**Issue Resolution:**
- `diagnose-issues` - Batch-diagnose multiple UAT issues
- `verify-work` - Manual user acceptance testing

**Continuous Improvement:**
- `autonomous-upskilling` - Self-improving system
- `deep-dive-analysis` - Systematic analysis when certainty < 0.80
- `autonomous-audit-loop` - Continuous validation with auto-execution

### Domain-Specific Skills (4 skills)

**Trading:**
- `full-stack-execution` - Full stack execution architecture
- `liquidity-execution` - Liquidity execution
- `deltalag-signal` - Deltalag signal handling
- `quant-fabric-predictability` - Quant fabric predictability
- `quant-hedge-advisory` - Quant hedge advisory
- `hstr-orchestrator` - HSTR orchestrator

**Other:**
- `nextjs-action-cards` - Next.js action cards
- `agent-handoff` - Agent handoff

## Systematic Wiring Strategy

### 1. Single-Agent Loop Pattern (Default)

**Use for:** Most tasks, simple workflows, predictable execution

**Flow:**
```
Task → Skill Selection → Tool Execution → Validation → Result
```

**Skills to use:**
- Start with `idea-refine` for clarification
- Use `context-engineering` to load relevant context
- Select appropriate skill based on task type
- Execute with MCP tools if needed
- Validate with `production-guard`
- Log with `decision-log`

### 2. Multi-Agent Delegation Pattern

**Use for:** Complex tasks, multiple domains, >15 tools

**Flow:**
```
Supervisor → Task Decomposition → Specialized Agents → Result Aggregation
```

**Skills to use:**
- `learnship-planner` for decomposition
- `learnship-executor` for each subtask
- `learnship-verifier` for aggregation
- `agentic-learning` for coordination

### 3. Supervisor Architecture Pattern

**Use for:** Independent subtasks, parallel execution

**Flow:**
```
Supervisor → Parallel Worker Agents → Collect Results → Validate
```

**Skills to use:**
- `learnship-planner` for parallel planning
- `learnship-executor` for parallel execution
- `production-guard` for validation
- `qa-team` for quality gates

## Skill Composition Patterns

### Example: Full Feature Development

```
idea-refine
  ↓
spec-driven-development
  ↓
planning-and-task-breakdown
  ↓
learnship-planner (creates PLAN.md)
  ↓
learnship-executor (executes plan)
  ↓
test-driven-development
  ↓
code-review-and-quality
  ↓
qa-team (multi-lens review)
  ↓
security-and-hardening
  ↓
performance-optimization
  ↓
production-guard (4-layer validation)
  ↓
git-workflow-and-versioning
  ↓
commit-effectiveness (6-step verification)
  ↓
ci-cd-and-automation
  ↓
shipping-and-launch
```

### Example: Bug Fix Workflow

```
debug-team (triage)
  ↓
debugging-and-error-recovery (investigate)
  ↓
deep-dive-analysis (if uncertain)
  ↓
code-simplification (minimal fix)
  ↓
test-driven-development (regression test)
  ↓
qa-team (verify fix)
  ↓
production-guard (validate)
  ↓
commit-effectiveness (verify commit)
```

### Example: Audit Workflow

```
repository-audit (comprehensive audit)
  ↓
diagnose-issues (batch-diagnose UAT issues)
  ↓
qa-team (multi-lens review)
  ↓
security-audit-gate (security review)
  ↓
performance-optimization (performance check)
  ↓
autonomous-audit-loop (continuous validation)
```

## Tool-Use Optimization

### Tool RAG for Dynamic Skill Selection

**Implementation:**
- Skill registry with capability descriptions
- Semantic retrieval based on task requirements
- Surface only relevant skills to reduce context bloat

**Skills to index:**
- All 48 skills with descriptions
- Tag by domain, phase, complexity
- Enable dynamic selection

### Parallel Execution

**Identify parallelizable operations:**
- Independent reads (config, memory, scheduler)
- Independent skill invocations
- Parallel test execution

**Skills that benefit:**
- `learnship-executor` (parallel plan execution)
- `qa-team` (parallel lens execution)
- `repository-audit` (parallel file analysis)

### Trajectory Pruning

**Implementation:**
- Lightweight result summarization after tool calls
- Pin critical context across trajectory
- Trim large JSON responses

**Skills to optimize:**
- `debugging-and-error-recovery` (large trace outputs)
- `repository-audit` (large file inventories)
- `qa-team` (large review results)

## Skill Registry

### By Phase

| Phase | Skills | Count |
|-------|-------|-------|
| Define & Plan | idea-refine, agentic-learning, spec-driven-development, planning-and-task-breakdown, learnship-planner, context-engineering, source-driven-development, decision-log | 8 |
| Build | incremental-implementation, learnship-executor, test-driven-development, api-and-interface-design, code-simplification, adaptive-self-healing, frontend-ui-engineering, impeccable, hexagonal-adapters, pattern-layer-architecture, ml-physics-modeling, rls-multi-tenancy | 12 |
| Verify | browser-testing-with-devtools, production-guard, debugging-and-error-recovery, debug-team, learnship-debugger, qa-team, learnship-verifier | 7 |
| Review | code-review-and-quality, review, security-and-hardening, security-audit-gate, performance-optimization, repository-audit, audit-compliance | 7 |
| Ship | git-workflow-and-versioning, commit-effectiveness, ci-cd-and-automation, shipping-and-launch, ship, deprecation-and-migration | 6 |
| Operations | documentation-and-adrs, sync-docs, github-first-self-healing, quality-guardian, production-guard, knowledge-base, auto-journal-sync, new-milestone, complete-milestone, milestone-retrospective, new-project, branch-sync-strategic-research, github-sync-safeguard, diagnose-issues, verify-work, autonomous-upskilling, deep-dive-analysis, autonomous-audit-loop | 18 |

### By Domain

| Domain | Skills | Count |
|-------|-------|-------|
| Backend | api-and-interface-design, code-simplification, adaptive-self-healing | 3 |
| Frontend | frontend-ui-engineering, impeccable, hexagonal-adapters, browser-testing-with-devtools | 4 |
| DevOps | ci-cd-and-automation, git-workflow-and-versioning, shipping-and-launch | 3 |
| LLM/AI | pattern-layer-architecture, ml-physics-modeling, agentic-learning | 3 |
| Database | rls-multi-tenancy | 1 |
| Security | security-and-hardening, security-audit-gate, production-guard | 3 |
| Performance | performance-optimization | 1 |
| Testing | test-driven-development, qa-team, learnship-verifier | 3 |
| Debugging | debugging-and-error-recovery, debug-team, learnship-debugger | 3 |
| Planning | idea-refine, spec-driven-development, planning-and-task-breakdown, learnship-planner | 4 |
| Execution | incremental-implementation, learnship-executor | 2 |
| Documentation | documentation-and-adrs, sync-docs, decision-log | 3 |
| Monitoring | github-first-self-healing, quality-guardian, autonomous-audit-loop | 3 |
| Knowledge | knowledge-base, auto-journal-sync | 2 |
| Project Management | new-milestone, complete-milestone, milestone-retrospective, new-project | 4 |
| Branch Management | branch-sync-strategic-research, github-sync-safeguard | 2 |
| Issue Resolution | diagnose-issues, verify-work | 2 |
| Continuous Improvement | autonomous-upskilling, deep-dive-analysis | 2 |
| Trading | full-stack-execution, liquidity-execution, deltalag-signal, quant-fabric-predictability, quant-hedge-advisory, hstr-orchestrator | 6 |
| Other | nextjs-action-cards, agent-handoff, review, ship, deprecation-and-migration, audit-compliance | 6 |

## Recommended Skill Loading Strategy

### On-Demand Loading (Default)

**Load skills based on task phase:**
- Define/Plan phase: Load 8 planning skills
- Build phase: Load 12 build skills
- Verify phase: Load 7 verify skills
- Review phase: Load 7 review skills
- Ship phase: Load 6 ship skills
- Operations: Load 18 operations skills as needed

**Benefits:**
- Reduces context window pollution
- Faster skill selection
- Lower per-task inference costs

### Progressive Disclosure

**For complex skills:**
1. Load skill overview first
2. Load detailed reference on-demand
3. Load scripts for deterministic operations

**Skills requiring progressive disclosure:**
- `impeccable` (21 actions)
- `qa-team` (7 lenses)
- `repository-audit` (multiple phases)
- `learnship-planner` (complex planning logic)

## Success Criteria

### Skill-Level Success

Each skill must define:
- Clear success criteria
- Observable must-haves
- Verification steps
- Error handling

### Workflow-Level Success

Each workflow must define:
- Phase completion criteria
- Integration verification
- Performance metrics
- Quality gates

## Next Steps

1. **Commit all skills** - Stage and commit the 19 addyosmani skills
2. **Create skill registry** - Build JSON registry with metadata
3. **Implement Tool RAG** - Enable dynamic skill selection
4. **Wire workflows** - Create workflow definitions for common patterns
5. **Test systematically** - Validate each skill and workflow
6. **Monitor usage** - Track which skills are used most frequently
7. **Iterate** - Refine based on actual usage patterns

## References

- Skills as Workflow Nodes (Forest Notes, 2026-02-17)
- AI Agent Skills Complete Guide 2026 (Calmops, 2026-03-02)
- AI Agent Orchestration Patterns (Stochastic Sandbox, 2026-04-21)
- AI Agent Tool-Use Optimization (Zylos Research, 2026-03-03)
- AI Agents and Tool Chaining (regolo.ai, 2026-03-30)
