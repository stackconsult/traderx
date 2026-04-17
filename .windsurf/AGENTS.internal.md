# TraderX Internal Agent Governance (Agent-Only)

**Classification**: INTERNAL - For Authorized Agents Only  
**Branch**: `feature/github-mcp-setup`  
**Repository**: https://github.com/stackconsult/traderx  
**Mode**: ABSOLUTE GUARDRAILED PRODUCTION ENGINEERING

---

## ⚠️ CONFIDENTIAL

This file contains internal agent workflows, MCP configurations, and skill references. Do not expose to public repositories or external users.

---

## Quick Start (Every Session)

```
1. /session-start          ← MANDATORY - Do not skip
2. /preflight-checklist    ← Binary validation
3. Confirm GO status
4. Begin production work
```

---

## MCP Configuration

### GitHub MCP Server
```json
// ~/.windsurf/mcp_config.json
{
  "mcpServers": {
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "${env:GITHUB_TOKEN}"
      }
    }
  }
}
```

### Required Environment
```bash
export GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxx
```

---

## Upstream Skill Sources

| Source | Repository | Sync Command | Sub-Skills |
|--------|-----------|--------------|------------|
| agentic-learning | FavioVazquez/agentic-learning | `/sync-upstream-skills` | Core learning |
| impeccable | pbakaus/impeccable | `/sync-upstream-skills` | 21 sub-skills |

### Impeccable Sub-Skills (21 total)
- adapt, animate, arrange, audit, bolder, clarify, colorize, critique, delight, distill, extract, frontend-design, harden, normalize, onboard, optimize, overdrive, polish, quieter, teach-impeccable, typeset

---

## Internal Workflow Stack

### Mandatory (Every Session)
| Workflow | Purpose | Location |
|----------|---------|----------|
| `/session-start` | Initialization | `.windsurf/workflows/session-start.md` |
| `/preflight-checklist` | Binary validation | `.windsurf/workflows/preflight-checklist.md` |

### Quality (Every Commit)
| Workflow | Purpose | Gates |
|----------|---------|-------|
| `/quality-guardian` | Quality enforcement | 5 gates, A-F grading |

### Strategic (Phase Planning)
| Workflow | Purpose | When |
|----------|---------|------|
| `/ideate` | Research-backed ideation | New features |
| `/challenge` | Scope validation | Before commitment |
| `/plan-phase` | Wave-based planning | Phase start |

### Knowledge (Continuous)
| Workflow | Purpose | Trigger |
|----------|---------|---------|
| `/compound` | Capture solutions | After solving |
| `/review` | Multi-persona review | Before PR |
| `/sync-upstream-skills` | Skill updates | When outdated |

---

## 7 Absolute Laws (Internal)

1. **Session Start Mandate** - `/session-start` MUST complete before work
2. **Production-Only Code** - NO pseudo-code, NO mimics
3. **Recursive Up-Engineering** - Continuous improvement cycles
4. **Workflow Absolutism** - Guardrails ALWAYS active
5. **Pre-Task Intelligence** - Analyze/grade past work before starting
6. **Roadmap Clarity** - Zero confusion about status/direction
7. **Validation Benchmark** - A-grade standards for all work

---

## Branch Protection (Enforced)

**Main Branch**:
- ✅ Require PR (no direct push)
- ✅ Require 1 reviewer minimum
- ✅ Require conversation resolution
- ✅ Enforce for admins
- ❌ No force pushes
- ❌ No deletions

**Current Branch**: `feature/github-mcp-setup`
**Merge Strategy**: PR → Review → Merge to main

---

## Grading System

| Grade | Criteria | Action |
|-------|----------|--------|
| A | All gates pass, 90%+ coverage, no issues | Accept and compound |
| B | Minor issues (<5), 85%+ coverage | Accept with notes |
| C | Some issues, 80%+ coverage | Review and improve |
| D | Multiple failures | Fix before commit |
| F | Critical failures | Complete redo |

---

## Self-Learning & Self-Healing

### Self-Learning Triggers
- After every solution: `/compound`
- Pattern recognition: Compare to past solutions
- Knowledge aggregation: Update `.planning/KNOWLEDGE.md`

### Self-Healing Systems
- Circuit breakers on external calls
- Exponential backoff retries
- Health monitoring with auto-recovery
- Anomaly detection with context

---

## Multi-Agent Coordination

### Agent Roles
| Agent | Role | Tasks |
|-------|------|-------|
| Cascade (Windsurf) | Local orchestration | IDE integration, file ops |
| Claude 4.6 | Planning & architecture | Complex decisions |
| Gemma 4 | Execution & testing | Implementation |
| Meta-Coordinator | Handoff management | State sync |

### Handoff Protocol
```
HandoffPackage:
  - Task boundaries defined
  - Context compressed (TurboQuant)
  - Deterministic plan with rollback
  - Expected outcomes & validation
  - Timeout & error handling
```

---

## Proof Artifact Requirements

Every commit requires:
```
proofs/
├── test-[component]-[timestamp].json
├── coverage-[component]-[timestamp].json
├── security-[component]-[timestamp].json
├── perf-[component]-[timestamp].json
└── docs-[component]-[timestamp].json
```

---

## Emergency Protocols

### If `/session-start` Fails
1. STOP all work
2. Check: GitHub token, MCP config, skill directories
3. Run `/sync-upstream-skills` if skills missing
4. Re-run `/session-start`
5. Only proceed when GO status confirmed

### If Quality Degrades
1. Halt current work
2. Run `/review` for assessment
3. Identify root cause
4. Apply `/compound` to capture fix
5. Resume only after quality restored

### If Upstream Skills Outdated
1. Check: `git ls-remote` for latest commits
2. Execute `/sync-upstream-skills`
3. Validate all 21 impeccable sub-skills present
4. Re-run installer if needed

---

## Secret Management

### Tokens (NEVER hardcode)
- `GITHUB_TOKEN` - GitHub API access
- `BINANCE_API_KEY` - Exchange access
- `BINANCE_SECRET_KEY` - Exchange secrets
- `DATABASE_URL` - PostgreSQL connection
- `REDIS_URL` - Redis connection

### Location
- Local: `.env` (gitignored)
- CI/CD: Repository secrets
- Never commit to git

---

## Validation Checklist (Pre-Commit)

- [ ] `/session-start` completed this session
- [ ] `/quality-guardian` passed (grade A or B)
- [ ] All proof artifacts generated
- [ ] JOURNAL.md updated
- [ ] Tests passing (90%+ coverage)
- [ ] Security scan clean (zero critical/high)
- [ ] Performance within 10% baseline
- [ ] Documentation complete
- [ ] Conventional commit format

---

## Success Criteria

Internal agent workflow successful when:
- ✅ All 7 laws followed
- ✅ `/session-start` every session
- ✅ `/quality-guardian` every commit
- ✅ Proof artifacts current
- ✅ Knowledge compounded
- ✅ No secrets in code
- ✅ A-grade quality maintained
- ✅ Ready for PR to main

---

**ACKNOWLEDGMENT REQUIRED**: By using these workflows, you confirm understanding of the 7 Absolute Laws and commit to production-grade agentic manufacturing standards.
