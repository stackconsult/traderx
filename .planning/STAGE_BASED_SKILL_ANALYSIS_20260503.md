# Stage-Based Skill Wiring Analysis
**Generated:** 2026-05-03
**Purpose:** Analyze skill reconfiguration requirements per CI/CD stage for production deployment

## Executive Summary

Based on research into 2026 production deployment practices, **skills require stage-specific reconfiguration** to follow best practices. This analysis identifies which skills need reconfiguration, how they should be configured per stage, and provides a comprehensive wiring strategy.

**Key Finding:** 100% of skills require some form of stage-specific configuration. Skills are not "one-size-fits-all" - they must be tailored to the environment (local, staging, production) and CI/CD stage (source, build, test, security, artifact, staging, production).

## Research Findings Summary

### CI/CD Pipeline Stages (7-Stage Standard)

1. **Source** - Checkout code, validate branch rules (10-30 sec)
2. **Build** - Install deps, lint, compile, package artifact (1-5 min)
3. **Test** - Unit + integration tests, coverage gate (2-8 min)
4. **Security** - SAST, SCA, container scan, secret detection (1-4 min)
5. **Artifact** - Push versioned image to registry (1-3 min)
6. **Staging** - Auto-deploy to staging + smoke/acceptance tests (3-8 min)
7. **Production** - Manual approval or auto-deploy with health checks (5-15 min)

### AI Agent Safety Primitives (Non-Negotiable)

1. **Sandboxed Execution** - Ephemeral containers, no persistent state
2. **Read-Only Repository Access by Default** - `permissions: contents: read`
3. **Structured Outputs for Human Review** - Draft artifacts, not direct actions
4. **Scoped Secrets and Permissions** - Minimum permissions, dedicated service accounts

### Environment-Specific Configuration Requirements

| Environment | Purpose | API Keys | Model Selection | Caching | Rate Limits | Debug |
|-------------|---------|----------|-----------------|---------|-------------|-------|
| Development | Local dev | Test keys | Fast models | Disabled | Low | Enabled |
| Staging | Pre-prod testing | Staging keys | Same as prod | Enabled | Medium | Disabled |
| Production | Live traffic | Production keys | Optimized | Enabled | High | Disabled |

## Skill Reconfiguration Analysis by Category

### Category 1: Planning & Specification Skills (8 skills)

**Skills Requiring Reconfiguration:**

1. **idea-refine**
   - **Local:** Use brainstorming mode, no constraints
   - **Staging:** Add business rule validation, compliance checks
   - **Production:** Strict validation, cost estimation, ROI analysis

2. **spec-driven-development**
   - **Local:** Draft specs, flexible requirements
   - **Staging:** Validate against staging infrastructure constraints
   - **Production:** Validate against production SLAs, performance requirements

3. **planning-and-task-breakdown**
   - **Local:** Include exploration tasks, research phases
   - **Staging:** Add staging deployment tasks, integration test planning
   - **Production:** Add production deployment tasks, rollback planning

4. **learnship-planner**
   - **Local:** Wave-based planning with exploration
   - **Staging:** Add staging verification waves
   - **Production:** Add production verification waves, approval gates

5. **context-engineering**
   - **Local:** Load local config files, dev environment variables
   - **Staging:** Load staging config, preview deployment URLs
   - **Production:** Load production config, production secrets (via secure channels)

6. **source-driven-development**
   - **Local:** Fetch local documentation, dev guides
   - **Staging:** Fetch staging-specific docs, preview deployment guides
   - **Production:** Fetch production docs, deployment runbooks

7. **decision-log**
   - **Local:** Log all decisions, including experimental ones
   - **Staging:** Log staging-specific decisions, rollback decisions
   - **Production:** Log production decisions, incident responses, post-mortems

8. **agentic-learning**
   - **Local:** Enable all learning modes, detailed feedback
   - **Staging:** Focus on staging-specific patterns, integration issues
   - **Production:** Focus on production patterns, incident analysis

### Category 2: Build Skills (12 skills)

**Skills Requiring Reconfiguration:**

1. **incremental-implementation**
   - **Local:** Fast iteration, skip some validation for speed
   - **Staging:** Full validation, staging-specific tests
   - **Production:** Full validation, production-specific tests, performance checks

2. **learnship-executor**
   - **Local:** Execute with dev environment, skip some gates
   - **Staging:** Execute with staging environment, staging gates
   - **Production:** Execute with production environment, production gates, approval required

3. **test-driven-development**
   - **Local:** Unit tests only, fast feedback
   - **Staging:** Unit + integration tests, coverage gates
   - **Production:** Unit + integration + E2E tests, performance tests, security tests

4. **api-and-interface-design**
   - **Local:** Draft interfaces, flexible versioning
   - **Staging:** Validate against staging infrastructure, rate limits
   - **Production:** Validate against production infrastructure, SLAs, rate limits

5. **code-simplification**
   - **Local:** Simplify aggressively for learning
   - **Staging:** Simplify with staging performance constraints
   - **Production:** Simplify with production performance constraints, maintainability

6. **adaptive-self-healing**
   - **Local:** Enable all healing modes, verbose logging
   - **Staging:** Enable healing with staging-specific thresholds
   - **Production:** Enable healing with production-specific thresholds, approval for major fixes

7. **frontend-ui-engineering**
   - **Local:** Use dev build, hot reload, debug tools
   - **Staging:** Use production build, staging CDN, staging analytics
   - **Production:** Use production build, production CDN, production analytics

8. **impeccable**
   - **Local:** Use all 21 actions, experimental features
   - **Staging:** Use staging-specific actions, staging design tokens
   - **Production:** Use production-specific actions, production design tokens, accessibility compliance

9. **hexagonal-adapters**
   - **Local:** Use mock adapters, local services
   - **Staging:** Use staging adapters, staging services
   - **Production:** Use production adapters, production services, circuit breakers

10. **pattern-layer-architecture**
    - **Local:** Draft patterns, experimental implementations
    - **Staging:** Validate patterns against staging infrastructure
    - **Production:** Validate patterns against production infrastructure, performance testing

11. **ml-physics-modeling**
    - **Local:** Use small models, fast training, local data
    - **Staging:** Use staging models, staging data, validation
    - **Production:** Use production models, production data, monitoring

12. **rls-multi-tenancy**
    - **Local:** Use local database, simple policies
    - **Staging:** Use staging database, staging policies, test tenants
    - **Production:** Use production database, production policies, all tenants

### Category 3: Verification Skills (7 skills)

**Skills Requiring Reconfiguration:**

1. **browser-testing-with-devtools**
   - **Local:** Test with Chrome DevTools, local URLs
   - **Staging:** Test with staging URLs, staging auth
   - **Production:** Test with production URLs (read-only), production auth (test account)

2. **production-guard**
   - **Local:** 4-layer validation with relaxed thresholds
   - **Staging:** 4-layer validation with staging thresholds
   - **Production:** 4-layer validation with strict thresholds, approval required

3. **debugging-and-error-recovery**
   - **Local:** Full debugging, detailed traces
   - **Staging:** Debugging with staging-specific error patterns
   - **Production:** Debugging with production-specific error patterns, read-only access

4. **debug-team**
   - **Local:** Full hypothesis testing, experimental fixes
   - **Staging:** Hypothesis testing with staging data
   - **Production:** Hypothesis testing with production data (read-only), approval for fixes

5. **learnship-debugger**
   - **Local:** Debug with full permissions
   - **Staging:** Debug with staging permissions
   - **Production:** Debug with read-only permissions, approval for actions

6. **qa-team**
   - **Local:** All 7 lenses, experimental checks
   - **Staging:** 7 lenses with staging-specific checks
   - **Production:** 7 lenses with production-specific checks, stricter severity thresholds

7. **learnship-verifier**
   - **Local:** Verify with relaxed must-haves
   - **Staging:** Verify with staging must-haves
   - **Production:** Verify with production must-haves, approval required

### Category 4: Review Skills (7 skills)

**Skills Requiring Reconfiguration:**

1. **code-review-and-quality**
   - **Local:** Review with relaxed quality gates
   - **Staging:** Review with staging quality gates
   - **Production:** Review with strict quality gates, approval required

2. **review**
   - **Local:** Multi-persona review with experimental personas
   - **Staging:** Multi-persona review with staging-specific personas
   - **Production:** Multi-persona review with production-specific personas, security persona required

3. **security-and-hardening**
   - **Local:** Basic security checks, allow some false positives
   - **Staging:** Full security checks, staging-specific rules
   - **Production:** Full security checks, production-specific rules, zero tolerance for vulnerabilities

4. **security-audit-gate**
   - **Local:** Skip audit gate for speed
   - **Staging:** Run audit gate with staging rules
   - **Production:** Run audit gate with production rules, approval required

5. **performance-optimization**
   - **Local:** Optimize for development speed
   - **Staging:** Optimize for staging performance
   - **Production:** Optimize for production performance, Core Web Vitals compliance

6. **repository-audit**
   - **Local:** Audit with relaxed thresholds
   - **Staging:** Audit with staging thresholds
   - **Production:** Audit with strict thresholds, approval required

7. **audit-compliance**
   - **Local:** Skip compliance checks for speed
   - **Staging:** Run compliance checks with staging rules
   - **Production:** Run compliance checks with production rules, approval required

### Category 5: Ship Skills (6 skills)

**Skills Requiring Reconfiguration:**

1. **git-workflow-and-versioning**
   - **Local:** Allow force push on feature branches
   - **Staging:** Enforce branch protection rules, require review
   - **Production:** Enforce strict branch protection, require approval

2. **commit-effectiveness**
   - **Local:** Skip SHA matching for speed
   - **Staging:** Verify local SHA matches staging
   - **Production:** Verify local SHA matches production, approval required

3. **ci-cd-and-automation**
   - **Local:** Use local CI, skip some gates
   - **Staging:** Use staging CI, staging gates
   - **Production:** Use production CI, production gates, manual approval required

4. **shipping-and-launch**
   - **Local:** Skip launch checks
   - **Staging:** Run launch checks with staging parameters
   - **Production:** Run launch checks with production parameters, approval required

5. **ship**
   - **Local:** Skip ship pipeline
   - **Staging:** Run ship pipeline to staging
   - **Production:** Run ship pipeline to production, approval required

6. **deprecation-and-migration**
   - **Local:** Skip deprecation checks
   - **Staging:** Run deprecation checks with staging data
   - **Production:** Run deprecation checks with production data, approval required

### Category 6: Operations Skills (18 skills)

**Skills Requiring Reconfiguration:**

1. **documentation-and-adrs**
   - **Local:** Draft ADRs, experimental documentation
   - **Staging:** Validate ADRs against staging architecture
   - **Production:** Validate ADRs against production architecture, approval required

2. **sync-docs**
   - **Local:** Sync local docs
   - **Staging:** Sync staging docs
   - **Production:** Sync production docs, approval required

3. **github-first-self-healing**
   - **Local:** Enable all healing modes
   - **Staging:** Enable healing with staging-specific thresholds
   - **Production:** Enable healing with production-specific thresholds, approval for major fixes

4. **quality-guardian**
   - **Local:** Relaxed quality gates
   - **Staging:** Staging quality gates
   - **Production:** Strict quality gates, approval required

5. **production-guard**
   - **Local:** 4-layer validation with relaxed thresholds
   - **Staging:** 4-layer validation with staging thresholds
   - **Production:** 4-layer validation with strict thresholds, approval required

6. **knowledge-base**
   - **Local:** Store all learnings, including experimental
   - **Staging:** Store staging-specific learnings
   - **Production:** Store production-specific learnings, approval required

7. **auto-journal-sync**
   - **Local:** Sync local journal
   - **Staging:** Sync staging journal
   - **Production:** Sync production journal, approval required

8. **new-milestone**
   - **Local:** Create milestone with relaxed constraints
   - **Staging:** Create milestone with staging constraints
   - **Production:** Create milestone with production constraints, approval required

9. **complete-milestone**
   - **Local:** Complete milestone without verification
   - **Staging:** Complete milestone with staging verification
   - **Production:** Complete milestone with production verification, approval required

10. **milestone-retrospective**
    - **Local:** Detailed retrospective, experimental insights
    - **Staging:** Staging-specific retrospective
    - **Production:** Production-specific retrospective, approval required

11. **new-project**
    - **Local:** Create project with relaxed constraints
    - **Staging:** Create project with staging constraints
    - **Production:** Create project with production constraints, approval required

12. **branch-sync-strategic-research**
    - **Local:** Sync local branches
    - **Staging:** Sync staging branches
    - **Production:** Sync production branches, approval required

13. **github-sync-safeguard**
    - **Local:** Allow force push on feature branches
    - **Staging:** Enforce staging sync rules
    - **Production:** Enforce production sync rules, approval required

14. **diagnose-issues**
    - **Local:** Full diagnosis, experimental fixes
    - **Staging:** Diagnosis with staging data
    - **Production:** Diagnosis with production data (read-only), approval for fixes

15. **verify-work**
    - **Local:** Verify with relaxed criteria
    - **Staging:** Verify with staging criteria
    - **Production:** Verify with production criteria, approval required

16. **autonomous-upskilling**
    - **Local:** Enable all upskilling modes
    - **Staging:** Enable upskilling with staging-specific patterns
    - **Production:** Enable upskilling with production-specific patterns, approval required

17. **deep-dive-analysis**
    - **Local:** Full analysis, experimental approaches
    - **Staging:** Analysis with staging data
    - **Production:** Analysis with production data (read-only), approval required

18. **autonomous-audit-loop**
    - **Local:** Enable all audit modes
    - **Staging:** Enable audit with staging-specific thresholds
    - **Production:** Enable audit with production-specific thresholds, approval required

### Category 7: Domain-Specific Skills (6 skills)

**Skills Requiring Reconfiguration:**

1. **full-stack-execution**
   - **Local:** Execute with dev infrastructure
   - **Staging:** Execute with staging infrastructure
   - **Production:** Execute with production infrastructure, approval required

2. **liquidity-execution**
   - **Local:** Execute with paper trading
   - **Staging:** Execute with staging trading
   - **Production:** Execute with production trading, approval required

3. **deltalag-signal**
   - **Local:** Process signals with dev data
   - **Staging:** Process signals with staging data
   - **Production:** Process signals with production data, approval required

4. **quant-fabric-predictability**
   - **Local:** Predict with dev models
   - **Staging:** Predict with staging models
   - **Production:** Predict with production models, approval required

5. **quant-hedge-advisory**
   - **Local:** Advise with dev parameters
   - **Staging:** Advise with staging parameters
   - **Production:** Advise with production parameters, approval required

6. **hstr-orchestrator**
   - **Local:** Orchestrate with dev infrastructure
   - **Staging:** Orchestrate with staging infrastructure
   - **Production:** Orchestrate with production infrastructure, approval required

### Category 8: Other Skills (4 skills)

**Skills Requiring Reconfiguration:**

1. **nextjs-action-cards**
   - **Local:** Use dev build, local environment
   - **Staging:** Use staging build, staging environment
   - **Production:** Use production build, production environment

2. **agent-handoff**
   - **Local:** Handoff with full permissions
   - **Staging:** Handoff with staging permissions
   - **Production:** Handoff with production permissions, approval required

3. **audit-compliance**
   - **Local:** Skip compliance checks
   - **Staging:** Run compliance checks with staging rules
   - **Production:** Run compliance checks with production rules, approval required

4. **nextjs-action-cards** (duplicate in analysis)
   - See above

## Stage-Based Skill Wiring Strategy

### Local Development Stage

**Skills to Load:** All 48 skills (full capability)
**Configuration:** Relaxed thresholds, experimental features enabled, debug mode on
**Permissions:** Full write access, no approval gates
**Environment:** Local database, mock services, test API keys
**Safety:** Sandbox not required (local environment)

**Skill Loading Pattern:**
```yaml
local_development:
  load_strategy: all
  config:
    thresholds: relaxed
    debug: true
    experimental_features: true
    cache: disabled
  permissions:
    repository: write
    deployment: local_only
  secrets:
    api_keys: test_keys
    database: local
```

### Staging Stage

**Skills to Load:** Phase-specific skills (on-demand)
**Configuration:** Staging thresholds, staging-specific rules, debug mode off
**Permissions:** Read-only repository, write to staging deployment
**Environment:** Staging database, staging services, staging API keys
**Safety:** Sandboxed execution, read-only by default

**Skill Loading Pattern:**
```yaml
staging:
  load_strategy: on_demand
  phases:
    - define_plan: [idea-refine, spec-driven-development, planning-and-task-breakdown]
    - build: [incremental-implementation, learnship-executor, test-driven-development]
    - verify: [browser-testing-with-devtools, production-guard, qa-team]
    - review: [code-review-and-quality, security-and-hardening, performance-optimization]
    - ship: [git-workflow-and-versioning, ci-cd-and-automation, shipping-and-launch]
  config:
    thresholds: staging
    debug: false
    experimental_features: false
    cache: enabled
  permissions:
    repository: read
    deployment: staging_only
  secrets:
    api_keys: staging_keys
    database: staging
  safety:
    sandboxed: true
    read_only_default: true
```

### Production Stage

**Skills to Load:** Phase-specific skills (on-demand, minimum set)
**Configuration:** Strict thresholds, production-specific rules, debug mode off
**Permissions:** Read-only repository, write to production deployment with approval
**Environment:** Production database, production services, production API keys (via secure channels)
**Safety:** Sandboxed execution, read-only by default, human approval gates

**Skill Loading Pattern:**
```yaml
production:
  load_strategy: on_demand_minimal
  phases:
    - define_plan: [idea-refine, spec-driven-development, planning-and-task-breakdown]
    - build: [incremental-implementation, learnship-executor, test-driven-development]
    - verify: [browser-testing-with-devtools, production-guard, qa-team]
    - review: [code-review-and-quality, security-and-hardening, performance-optimization]
    - ship: [git-workflow-and-versioning, ci-cd-and-automation, shipping-and-launch]
  config:
    thresholds: strict
    debug: false
    experimental_features: false
    cache: enabled
  permissions:
    repository: read
    deployment: production_with_approval
  secrets:
    api_keys: production_keys
    database: production
  safety:
    sandboxed: true
    read_only_default: true
    human_approval: required
```

## CI/CD Stage-Specific Skill Configuration

### Stage 1: Source (Checkout + Trigger)

**Skills:** `git-workflow-and-versioning`, `context-engineering`
**Configuration:**
- Validate branch rules
- Load environment-specific config
- Checkout code with appropriate permissions

```yaml
source_stage:
  skills:
    - git-workflow-and-versioning:
        config:
          branch_validation: strict
          permissions: contents: read
    - context-engineering:
        config:
          load_env_config: true
          env: ${ENVIRONMENT}
```

### Stage 2: Build (Compile + Lint + Package)

**Skills:** `test-driven-development` (unit), `code-simplification`, `frontend-ui-engineering`
**Configuration:**
- Run linting and static analysis
- Compile code
- Package artifacts
- Unit tests

```yaml
build_stage:
  skills:
    - test-driven-development:
        config:
          test_type: unit
          coverage_gate: 80%
    - code-simplification:
        config:
          lint: true
          simplify: false
    - frontend-ui-engineering:
        config:
          build: true
          dev_build: ${ENVIRONMENT == local}
```

### Stage 3: Test (Unit + Integration + Coverage)

**Skills:** `test-driven-development` (integration), `browser-testing-with-devtools`, `qa-team`
**Configuration:**
- Unit tests
- Integration tests
- Coverage gate
- Browser testing

```yaml
test_stage:
  skills:
    - test-driven-development:
        config:
          test_type: integration
          coverage_gate: 80%
    - browser-testing-with-devtools:
        config:
          environment: ${ENVIRONMENT}
          url: ${DEPLOY_URL}
    - qa-team:
        config:
          lenses: [correctness, testing, security]
          severity_threshold: P2
```

### Stage 4: Security (SAST + SCA + Container Scan)

**Skills:** `security-and-hardening`, `security-audit-gate`, `repository-audit`
**Configuration:**
- SAST scanning
- SCA scanning
- Container scanning
- Secret detection

```yaml
security_stage:
  skills:
    - security-and-hardening:
        config:
          sast: true
          sca: true
          container_scan: true
          secret_detection: true
          block_on: HIGH, CRITICAL
    - security-audit-gate:
        config:
          owasp_top_10: true
          cve_check: true
    - repository-audit:
        config:
          security_audit: true
          dependency_audit: true
```

### Stage 5: Artifact (Versioned Image to Registry)

**Skills:** `git-workflow-and-versioning`, `commit-effectiveness`, `ci-cd-and-automation`
**Configuration:**
- Version artifact
- Push to registry
- Verify commit effectiveness

```yaml
artifact_stage:
  skills:
    - git-workflow-and-versioning:
        config:
          version: semantic
          tag: ${GIT_SHA}
    - commit-effectiveness:
        config:
          verify_local_sha: true
          verify_remote_sha: true
    - ci-cd-and-automation:
        config:
          push_to_registry: true
          registry: ghcr.io
```

### Stage 6: Staging (Auto-Deploy + Tests)

**Skills:** `shipping-and-launch`, `production-guard`, `verify-work`
**Configuration:**
- Deploy to staging
- Run smoke tests
- Run acceptance tests

```yaml
staging_stage:
  skills:
    - shipping-and-launch:
        config:
          environment: staging
          strategy: rolling
    - production-guard:
        config:
          validation_layers: 4
          thresholds: staging
    - verify-work:
        config:
          smoke_tests: true
          acceptance_tests: true
```

### Stage 7: Production (Manual Approval + Deploy)

**Skills:** `shipping-and-launch`, `production-guard`, `verify-work`, `deprecation-and-migration`
**Configuration:**
- Manual approval gate
- Deploy to production (blue-green or canary)
- Health checks
- Automated rollback on failure

```yaml
production_stage:
  skills:
    - shipping-and-launch:
        config:
          environment: production
          strategy: blue_green
          approval: required
    - production-guard:
        config:
          validation_layers: 4
          thresholds: strict
    - verify-work:
        config:
          health_checks: true
          rollback_on_failure: true
          rollback_timeout: 10min
    - deprecation-and-migration:
        config:
          run_migrations: true
          migration_order: dev -> staging -> production
```

## Implementation Strategy

### Phase 1: Create Skill Configuration Templates

Create environment-specific configuration files for each skill:

```
.windsurf/skills/config/
├── local/
│   ├── idea-refine.yaml
│   ├── spec-driven-development.yaml
│   └── ... (all 48 skills)
├── staging/
│   ├── idea-refine.yaml
│   ├── spec-driven-development.yaml
│   └── ... (all 48 skills)
└── production/
    ├── idea-refine.yaml
    ├── spec-driven-development.yaml
    └── ... (all 48 skills)
```

### Phase 2: Implement Skill Loading Logic

Create a skill loader that:
1. Detects current environment
2. Loads appropriate configuration
3. Applies safety primitives
4. Enforces permissions

### Phase 3: Implement CI/CD Stage Integration

Integrate skills into CI/CD pipeline:
1. Map CI/CD stages to skill phases
2. Configure skill loading per stage
3. Implement approval gates
4. Add safety primitives

### Phase 4: Test and Validate

Test the configuration:
1. Local development testing
2. Staging deployment testing
3. Production deployment testing (with approval)

## Verification Against Industry Best Practices

### ✅ Compliant Practices

1. **Sandboxed Execution** - Skills will run in ephemeral containers
2. **Read-Only by Default** - Skills configured with read-only repository access
3. **Structured Outputs** - Skills produce draft artifacts, not direct actions
4. **Scoped Secrets** - Skills use environment-specific secrets
5. **Environment Parity** - Staging mirrors production configuration
6. **Progressive Disclosure** - Skills use layered configuration
7. **Skill Composition** - Skills reference other skills, don't embed them
8. **MCP/Skill Separation** - MCP provides tools, skills provide workflow logic
9. **80/20 Rule** - Skills loaded on-demand based on phase
10. **Outcome-Based Evaluation** - Skills define success criteria

### ⚠️ Gaps Identified

1. **No Tool RAG Implementation** - Skills are not dynamically selected based on task
2. **No Parallel Execution** - Skills run sequentially, not in parallel
3. **No Trajectory Pruning** - Large outputs not summarized
4. **No Skill Registry** - No centralized metadata for skill discovery

### 🔧 Recommended Improvements

1. **Implement Tool RAG** - Create skill registry with capability descriptions
2. **Enable Parallel Execution** - Identify and parallelize independent skill invocations
3. **Add Trajectory Pruning** - Summarize large skill outputs
4. **Create Skill Registry** - Build centralized metadata system

## Next Steps

1. **Create skill configuration templates** for local, staging, production
2. **Implement skill loading logic** with environment detection
3. **Integrate skills into CI/CD pipeline** with stage-specific configuration
4. **Test configuration** in each environment
5. **Implement Tool RAG** for dynamic skill selection
6. **Enable parallel execution** for independent skills
7. **Add trajectory pruning** for large outputs
8. **Create skill registry** for centralized metadata

## Conclusion

**100% of skills require stage-specific reconfiguration** to follow 2026 production deployment best practices. Skills must be tailored to:
- Environment (local, staging, production)
- CI/CD stage (source, build, test, security, artifact, staging, production)
- Safety primitives (sandbox, read-only, structured outputs, scoped secrets)
- Configuration (thresholds, permissions, secrets, caching, debug mode)

The implementation strategy provides a clear path to achieve systematic, repeatable skill wiring across all stages of the deployment pipeline.
