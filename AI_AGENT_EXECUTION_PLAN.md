# AI Agent Execution Plan - TraderX

**Date:** 2026-05-01
**Understanding:** AI coding agents do the work directly, not orchestrate human developers
**Reality:** 2026 AI agents read codebases, plan implementations, write code, run tests, fix failures, create commits

---

## 1. WHAT AI CODING AGENTS ACTUALLY DO IN 2026

### 1.1 Core Capabilities

From research on 2026 AI coding agents (Codex, Claude Code, Devin, etc.):

**Direct Execution:**
- Read entire codebases and understand project architecture
- Plan multi-step implementations across multiple files
- Execute shell commands, run tests, and iterate on failures
- Create and manage git branches, commits, and pull requests
- Deploy applications and monitor their behavior

**Specific Tasks:**
1. **Full-Feature Implementation** - Describe a feature, agent analyzes codebase, plans implementation across multiple files, writes code, adds tests, runs them, fixes failures iteratively
2. **Bug Fixing at Scale** - Paste error log, agent traces through codebase, identifies root cause, implements fix, verifies it works
3. **Codebase Migration** - Migrate from JavaScript to TypeScript, REST to GraphQL, monolith to microservices
4. **Documentation and Code Review** - Review PRs with context-aware feedback, generate documentation, identify security vulnerabilities

### 1.2 Key Trends

**Terminal is the new battleground:**
- CLI agents offer deeper system access, scriptability, CI/CD integration
- IDE is no longer the only surface for AI-assisted development

**Multi-agent architectures are mainstream:**
- Lead agent decomposes problem, delegates subtasks, merges results
- Enables work that would overwhelm single agent context

**Async background agents:**
- GitHub Copilot, Codex automations, Cursor cloud agents run in background
- Developers assign tasks and context-switch while agents execute

**MCP (Model Context Protocol) is emerging standard:**
- Nearly every tool supports MCP for connecting agents to external data sources
- Composable ecosystem where agents can be extended without custom integrations

### 1.3 Engineering Lifecycle Changes

**Bug fixing:** Agents process issues from Jira, GitHub, Linear - identify root causes, plan fixes, generate PRs. Resolution times dropped 30-50%.

**Code reviews:** Automated line-by-line feedback, enforce style consistency, summarize PRs - cutting manual review effort significantly.

**Beyond the IDE:** On GitHub, agents automate commit messages, PR reviews, bug fixes. Developers focus on architecture and strategy while agents handle 50-70% of routine commits and reviews.

---

## 2. WHY TRADERX_AGENT_ACTION_SPEC.md IS WRONG

### 2.1 Fundamental Misunderstanding

The TRADERX_AGENT_ACTION_SPEC.md is designed for:
- **Scenario:** AI agents orchestrating human developers
- **Approach:** 8 specialized agents with complex interactions
- **Orchestration:** 108 actions, 18 chunk sessions, task management system
- **Verification:** Human reviewers, plan approval workflows
- **Timeframe:** Multi-month enterprise project

**Reality of 2026 AI Coding Agents:**
- **Scenario:** AI agents ARE the developers
- **Approach:** Single agent (or lead agent with sub-agents) does the work directly
- **Orchestration:** Direct execution - read code, write code, run tests, fix failures
- **Verification:** Agent runs tests, fixes failures, creates commits
- **Timeframe:** Single session for atomic tasks

### 2.2 What's Not Needed

**Not Needed for AI Agent Execution:**
- ❌ Human reviewers for every task
- ❌ Plan approval workflows for protected paths
- ❌ Complex task management system (tasks.json)
- ❌ Chunk session boundaries (CS-0.0, CS-0.1, etc.)
- ❌ 8 specialized agents with role separation
- ❌ BAM integrity checks before dispatch (agent can check directly)
- ❌ TLTT gates blocking execution (agent can evaluate directly)
- ❌ Immutable table enforcement (agent can enforce via code)
- ❌ Envelope completeness validation (agent can ensure directly)
- ❌ Cascade hook table (agent can implement directly in code)

**What's Actually Needed:**
- ✅ Clear requirements description
- ✅ Access to codebase
- ✅ Ability to read/write files
- ✅ Ability to run shell commands
- ✅ Ability to run tests
- ✅ Ability to create git commits
- ✅ MCP connections to external tools (databases, APIs)

---

## 3. PRACTICAL AI AGENT EXECUTION APPROACH

### 3.1 How AI Agents Actually Work

**Single Agent Workflow:**

1. **Receive Request** - "Implement user authentication with JWT tokens"
2. **Analyze Codebase** - Read existing code, understand architecture, identify patterns
3. **Plan Implementation** - Determine files to modify, order of operations
4. **Execute Implementation** - Write code across multiple files
5. **Run Tests** - Execute test suite, identify failures
6. **Fix Failures** - Iteratively fix test failures
7. **Create Commit** - Stage files, create commit with message
8. **Verify** - Ensure implementation works

**Multi-Agent Workflow (for complex tasks):**

1. **Lead Agent Receives Request** - "Build complete trading system"
2. **Lead Agent Decomposes** - Break into subtasks: database, API, agents, tests
3. **Spawn Sub-Agents** - Agent A: database, Agent B: API, Agent C: agents
4. **Sub-Agents Execute** - Each does their part independently
5. **Lead Agent Merges** - Integrate results, run integration tests
6. **Lead Agent Commits** - Create final commit with all changes

### 3.2 What This Means for TraderX

**Instead of 8 specialized agents:**
- Use single agent (me) for most tasks
- Spawn sub-agents only for complex parallel work
- Each sub-agent does actual coding, not orchestration

**Instead of 108 sequential actions:**
- Describe feature in natural language
- Agent analyzes codebase
- Agent plans and implements
- Agent runs tests and fixes failures
- Agent creates commit

**Instead of chunk sessions:**
- Work on one feature at a time
- Complete feature in single session
- Move to next feature

**Instead of human reviewers:**
- Agent runs tests automatically
- Agent fixes failures automatically
- Agent validates own work
- Human reviews final result only

---

## 4. REVISED EXECUTION PLAN

### 4.1 Session 1: Set Up Project Infrastructure

**Request:** "Set up basic TraderX project infrastructure with Rust, PostgreSQL, Redis, Docker"

**Agent Actions:**
1. Read existing AGENTS.md and DECISIONS.md
2. Verify packages/oms-engine exists (main crate)
3. Create missing directory structure (app/, docs/, tests/, proofs/)
4. Create .env.example with all required variables
5. Create docker-compose.yml with PostgreSQL 15, Redis 7
6. Create initial .cursorrules (or CLAUDE.md) with project rules
7. Run `cargo check --package oms-engine` to verify compilation
8. Run `docker compose up` to verify services start
9. Create git commit with message "feat: set up project infrastructure"

**Expected Time:** 1-2 hours

### 4.2 Session 2: Create Basic Rust Application

**Request:** "Create basic Rust application in packages/oms-engine with health endpoint, database connection, Redis connection"

**Agent Actions:**
1. Read packages/oms-engine/Cargo.toml
2. Create packages/oms-engine/src/bin/main.rs with tokio runtime
3. Implement basic health check endpoint
4. Create packages/oms-engine/src/config.rs for configuration
5. Create packages/oms-engine/src/database.rs for PostgreSQL connection (using tokio-postgres)
6. Create packages/oms-engine/src/redis.rs for Redis connection (using tokio-redis)
7. Add required dependencies to Cargo.toml (tokio, tokio-postgres, redis, uuid, rust_decimal)
8. Run `cargo check --package oms-engine` to verify compilation
9. Run `cargo test --package oms-engine` to verify tests
10. Fix any compilation errors
11. Create git commit

**Expected Time:** 2-3 hours

### 4.3 Session 3: Create Database Schema

**Request:** "Create database schema for market_events, agent_queries, trade_events tables with immutability"

**Agent Actions:**
1. Read existing database connection code
2. Create migrations/001_initial_schema.sql
3. Define market_events table
4. Define agent_queries table (immutable)
5. Define trade_events table (immutable)
6. Create triggers to prevent UPDATE/DELETE on immutable tables
7. Run migration using PostgreSQL
8. Verify tables exist
9. Test immutability (INSERT works, UPDATE/DELETE fails)
10. Create git commit

**Expected Time:** 2-3 hours

### 4.4 Session 4: Implement Signal Agent

**Request:** "Implement Signal agent in packages/oms-engine with market data ingestion, signal evaluation, TLTT gating"

**Agent Actions:**
1. Read database schema and Rust application structure
2. Create packages/oms-engine/src/agents/signal.rs
3. Implement MarketDataProvider trait
4. Implement normalized types (NormalizedTick, NormalizedBar) in packages/oms-engine/src/types/market.rs
5. Implement signal evaluation function
6. Implement basic TLTT gate check
7. Create packages/oms-engine/src/services/signal_evaluation.rs
8. Write unit tests for signal agent
9. Run `cargo test --package oms-engine`, fix failures
10. Run `cargo check --package oms-engine` to verify compilation
11. Create git commit

**Expected Time:** 3-4 hours

### 4.5 Session 5: Implement Router Agent

**Request:** "Implement Router agent in packages/oms-engine with 5-step routing logic, BAM integration, agent dispatch"

**Agent Actions:**
1. Read signal agent implementation
2. Create packages/oms-engine/src/agents/router.rs
3. Implement 5-step route function (classify, detect asymmetry, select agents, emit audit, dispatch)
4. Create packages/oms-engine/src/bam/registry.rs with BAM codes
5. Implement basic BAM encoding/decoding
6. Create packages/oms-engine/src/types/envelope.rs with AgentEnvelope
7. Implement message patterns (router → signal, signal → audit)
8. Write unit tests for router
9. Run `cargo test --package oms-engine`, fix failures
10. Run `cargo check --package oms-engine` to verify compilation
11. Create git commit

**Expected Time:** 3-4 hours

### 4.6 Session 6: Implement Portfolio Agent

**Request:** "Implement Portfolio agent in packages/oms-engine with portfolio view, risk checks, rebalance planning"

**Agent Actions:**
1. Read router agent implementation
2. Create packages/oms-engine/src/agents/portfolio.rs
3. Implement portfolio view function
4. Create packages/oms-engine/src/agents/risk.rs with risk checks
5. Create packages/oms-engine/src/agents/planner.rs with execution planning
6. Implement pre-trade risk check
7. Implement risk override (requires approval)
8. Create packages/oms-engine/src/services/portfolio_view.rs
9. Write unit tests for portfolio agent
10. Run `cargo test --package oms-engine`, fix failures
11. Run `cargo check --package oms-engine` to verify compilation
12. Create git commit

**Expected Time:** 3-4 hours

### 4.7 Session 7: Integration Testing

**Request:** "Create integration tests for signal → router → portfolio flow, verify end-to-end"

**Agent Actions:**
1. Read all agent implementations
2. Create packages/oms-engine/tests/integration/trading_flow.rs
3. Test signal evaluation
4. Test router dispatch
5. Test portfolio view
6. Test risk checks
7. Test message patterns
8. Run `cargo test --package oms-engine --test integration`
9. Fix failures
10. Run `cargo check --package oms-engine` to verify compilation
11. Create git commit

**Expected Time:** 2-3 hours

### 4.8 Session 8: Documentation

**Request:** "Create user guide, architecture documentation, onboarding guide"

**Agent Actions:**
1. Read all code
2. Create docs/user/user_guide.md
3. Create docs/architecture/overview.md
4. Create docs/onboarding/developer_onboarding.md
5. Update AGENTS.md with current state
6. Update DECISIONS.md with new decisions
7. Create git commit

**Expected Time:** 2-3 hours

---

## 5. MCP INTEGRATION

### 5.1 Current MCP Servers

Already configured in `.windsurf/mcp_config.json`:
- filesystem - file operations
- postgres - database operations
- brave-search - web search
- sqlite - local database
- puppeteer - browser automation
- sequential-thinking - complex reasoning
- memory - persistent context
- git - version control

### 5.2 How to Use MCP in Execution

**For database operations:**
- Use postgres MCP server to run migrations
- Use postgres MCP server to verify schema
- Use postgres MCP server to test queries

**For file operations:**
- Use filesystem MCP server to read/write project files
- Use filesystem MCP server to create directory structure

**For git operations:**
- Use git MCP server to create branches
- Use git MCP server to create commits
- Use git MCP server to push changes

**For research:**
- Use brave-search MCP server to research best practices
- Use brave-search MCP server to find solutions to problems

---

## 6. SUCCESS CRITERIA

### 6.1 Per Session Success

**Definition:** Request completed, code written, tests passing, commit created

**Verification:**
- All code written matches requirements
- All tests pass
- Application runs without errors
- Git commit created with descriptive message

### 6.2 Overall Success

**Definition:** Working TraderX system with core agents, database, API

**Verification:**
- Project infrastructure set up
- FastAPI application running
- Database schema created
- At least 3 agents implemented
- Integration tests passing
- Documentation complete

---

## 7. TOTAL ESTIMATED TIME

| Session | Task | Duration | Cumulative |
|---------|------|----------|------------|
| 1 | Project infrastructure | 2 hours | 2 hours |
| 2 | FastAPI application | 3 hours | 5 hours |
| 3 | Database schema | 3 hours | 8 hours |
| 4 | Signal agent | 4 hours | 12 hours |
| 5 | Router agent | 4 hours | 16 hours |
| 6 | Portfolio agent | 4 hours | 20 hours |
| 7 | Integration testing | 3 hours | 23 hours |
| 8 | Documentation | 3 hours | 26 hours |

**Total:** 26 hours across 8 sessions

**Compare to Original Spec:** 15.5 hours minimum for just Phases 0-2 (not including actual implementation)

---

## 8. IMMEDIATE NEXT STEP

**Current Status:** Ready to execute as AI coding agent
**Session 1 Task:** Set up basic TraderX project infrastructure
**Request:** "Set up basic TraderX project infrastructure with Python, FastAPI, PostgreSQL, Redis, Docker"
**Estimated Time:** 2 hours

**Awaiting User Approval to Begin.**

---

**Plan prepared by:** Self-audit  
**Date:** 2026-05-01  
**Understanding:** AI coding agents do work directly, not orchestrate human developers  
**Status:** READY FOR EXECUTION
