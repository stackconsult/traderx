---
description: Workflow orchestrator from AgentKit — assembles all 35 agents for any task via plain chat, no terminal needed. Routes intent to the right agent(s), runs parallel fan-out where safe, and reports back in plain language.
---

# /orchestrate — AgentKit Workflow Orchestrator

**Plain-language trigger**: Saying "orchestrate this", "use the full team", "get everyone on it",
"swarm it", "who should handle X", or "assemble the team for Y" runs this workflow.

**No terminal required.** All orchestration happens through Cascade/Genesis chat.

---

## The 35-Agent Roster

### Engineering (7) — `genesis_model: coder/think`
| Agent | Triggers | Model |
|-------|---------|-------|
| `ai-engineer` | "AI feature", "LLM integration", "ML model" | think |
| `backend-architect` | "API design", "system architecture", "database schema" | think |
| `devops-automator` | "CI/CD", "deploy", "pipeline", "automate" | balanced |
| `frontend-developer` | "UI component", "React", "frontend bug" | coder |
| `mobile-app-builder` | "iOS", "Android", "React Native", "mobile" | balanced |
| `rapid-prototyper` | "prototype", "quick demo", "proof of concept" | coder |
| `test-writer-fixer` | "write tests", "fix failing test", "coverage" | coder |

### Design (5) — `genesis_model: balanced/fast`
| Agent | Triggers | Model |
|-------|---------|-------|
| `ui-designer` | "design this", "UI layout", "component design" | balanced |
| `brand-guardian` | "is this on brand", "brand guidelines" | balanced |
| `ux-researcher` | "user research", "usability", "why users drop off" | balanced |
| `visual-storyteller` | "infographic", "data viz", "presentation" | balanced |
| `whimsy-injector` | "make it fun", "add personality", "boring UI" | fast |

### Marketing (7) — `genesis_model: fast/balanced`
| Agent | Triggers | Model |
|-------|---------|-------|
| `app-store-optimizer` | "ASO", "app store listing", "keywords" | balanced |
| `content-creator` | "write content", "blog post", "copy" | balanced |
| `growth-hacker` | "grow users", "conversion", "viral loop" | think |
| `instagram-curator` | "Instagram", "IG post", "Reels" | fast |
| `reddit-community-builder` | "Reddit", "subreddit", "community" | balanced |
| `tiktok-strategist` | "TikTok", "viral video", "short-form" | fast |
| `twitter-engager` | "tweet", "Twitter", "X post" | fast |

### Product (3) — `genesis_model: balanced/think`
| Agent | Triggers | Model |
|-------|---------|-------|
| `feedback-synthesizer` | "user feedback", "reviews", "what users want" | balanced |
| `sprint-prioritizer` | "prioritise", "backlog", "sprint plan" | think |
| `trend-researcher` | "trends", "market research", "competitive analysis" | think |

### Studio Operations (5) — `genesis_model: balanced/think`
| Agent | Triggers | Model |
|-------|---------|-------|
| `analytics-reporter` | "metrics", "analytics", "report" | balanced |
| `finance-tracker` | "budget", "costs", "runway", "finance" | balanced |
| `infrastructure-maintainer` | "infra", "outage", "scaling", "server" | think |
| `legal-compliance-checker` | "legal", "GDPR", "compliance", "privacy" | think |
| `support-responder` | "customer support", "ticket", "user complaint" | balanced |

### Project Management (3) — `genesis_model: balanced/think`
| Agent | Triggers | Model |
|-------|---------|-------|
| `experiment-tracker` | "A/B test", "experiment", "what did we learn" | balanced |
| `project-shipper` | "launch", "release", "ship this feature" | balanced |
| `studio-producer` | "coordinate team", "project status", "blocking" | think |

### Testing (5) — `genesis_model: coder/balanced`
| Agent | Triggers | Model |
|-------|---------|-------|
| `api-tester` | "test this API", "endpoint check", "contract test" | coder |
| `performance-benchmarker` | "benchmark", "how fast is", "perf regression" | coder |
| `test-results-analyzer` | "test report", "coverage gaps", "flaky tests" | balanced |
| `tool-evaluator` | "which tool", "compare X vs Y", "build or buy" | think |
| `workflow-optimizer` | "optimize workflow", "process bottleneck" | think |

---

## Routing Logic — How the Orchestrator Decides

### Step 1: Intent Classification (fast model — gemma3:1b local)
```
User message → classify intent → map to agent category → select agent(s)
```

### Step 2: Fan-out Decision
- **Single agent**: clear owner, single domain → route directly
- **Parallel fan-out**: independent sub-tasks → spawn simultaneously (max depth 1)
- **Sequential chain**: output of A feeds into B → A → B → report

### Step 3: Model Selection
```
Routing/classification  → fast    (gemma3:1b, local, free)
Code generation         → coder   (qwen2.5-coder:1.5b, local, free)
Default tasks           → balanced (gemini-2.0-flash, cloud)
Architecture/legal/strategy → think (gemini-2.5-pro, cloud)
Fallback if cloud fails → balanced → fast
```

### Step 4: Memory + Context
- Relevant past context fetched from mem0 before agent starts
- Agent writes key findings to mem0 after completing
- JOURNAL.md updated on session end

---

## Common Orchestration Patterns

### Pattern A: Full Sprint Review
**Trigger**: "review this sprint", "ship review", "pre-launch check"
```
Parallel fan-out:
  ├── code-reviewer (engineering)     — code quality + architecture
  ├── security-auditor (windsurf)     — CVE + secrets scan
  ├── test-engineer (windsurf)        — coverage gaps
  ├── performance-benchmarker         — regression check
  └── legal-compliance-checker        — privacy/compliance touch

Integrator: studio-producer collects all reports → go/no-go decision
```

### Pattern B: New Feature End-to-End
**Trigger**: "build [feature]", "implement [X] from scratch"
```
Sequential:
  1. sprint-prioritizer    — is this the right thing to build?
  2. backend-architect     — system design + API contract
  3. ui-designer           — component spec
  4. ai-engineer (if AI)   — ML/LLM integration spec
  Parallel:
  ├── rapid-prototyper     — working prototype
  ├── test-writer-fixer    — test suite skeleton
  └── devops-automator     — CI/CD pipeline
  5. project-shipper       — launch checklist
```

### Pattern C: Growth Experiment
**Trigger**: "growth experiment", "try X to improve conversion"
```
Parallel fan-out:
  ├── growth-hacker        — experiment design + hypothesis
  ├── analytics-reporter   — baseline metrics
  ├── experiment-tracker   — tracking setup
  └── content-creator      — copy variants

After experiment:
  ├── test-results-analyzer — statistical significance
  └── feedback-synthesizer  — qualitative user signals

Decision: sprint-prioritizer — ship, iterate, or kill
```

### Pattern D: Infrastructure Incident
**Trigger**: "outage", "something's down", "performance degraded"
```
Sequential (urgent):
  1. infrastructure-maintainer  — diagnose + mitigate NOW
  2. analytics-reporter         — impact assessment
  3. support-responder          — user communication
  4. devops-automator           — fix + deploy
  5. experiment-tracker         — post-mortem logging
```

### Pattern E: BAM Build Stream
**Trigger**: "start stream [N]", "G[N] transition", "gate check"
```
/gate-check → if pass:
  Parallel:
  ├── backend-architect    — stream implementation plan
  ├── test-writer-fixer    — test framework for stream
  └── devops-automator     — CI gates for stream

On completion:
  ├── performance-benchmarker — SLO validation
  └── security-auditor        — gate G security check
```

---

## How to Use — Plain Language Examples

| You say | Orchestrator does |
|---------|-----------------|
| `"build a user auth system"` | backend-architect → frontend-developer → test-writer-fixer → security-auditor |
| `"why are users dropping off at checkout"` | ux-researcher + analytics-reporter + feedback-synthesizer in parallel |
| `"get ready to launch"` | full Pattern A sprint review |
| `"we need more users"` | growth-hacker + analytics-reporter + content-creator |
| `"infra is slow"` | Pattern D incident response |
| `"what should we build this sprint"` | sprint-prioritizer + trend-researcher + feedback-synthesizer |
| `"audit the whole agent setup"` | /self-audit + tool-evaluator + workflow-optimizer |
| `"legal check before launch"` | legal-compliance-checker + security-auditor |
| `"the tests are broken"` | test-writer-fixer + test-results-analyzer + debugging-and-error-recovery skill |

---

## Constraints (enforced, never bypass)

1. **Max orchestration depth = 1** — this orchestrator routes to agents; agents do NOT spawn more agents
2. **User is the final decision-maker** — orchestrator presents findings; user decides what to action
3. **Parallel fan-out** only for truly independent tasks (no shared mutable state)
4. **All trading orders through RiskBus** — engineering agents cannot bypass this
5. **No destructive ops without confirmation** — delete/force-push always asks first
6. **Local models first** — use ollama/gemma3:1b for routing, gemini only when needed
7. **Write to JOURNAL.md** at end of every orchestrated session

---

## Wiring Check — Run This to Verify All Connections

```bash
echo "=== AGENT ROSTER ===" && ls /Users/kirtissiemens/CascadeProjects/.ai/**/*.md 2>/dev/null | wc -l
echo "=== SKILL COUNT ===" && ls /Users/kirtissiemens/CascadeProjects/traderx-repo/.windsurf/skills/*.md | wc -l
echo "=== WORKFLOW COUNT ===" && ls /Users/kirtissiemens/CascadeProjects/traderx-repo/.windsurf/workflows/*.md | wc -l
echo "=== OLLAMA ===" && curl -s http://127.0.0.1:11434/api/tags | python3 -c "import json,sys; d=json.load(sys.stdin); [print(' -', m['name']) for m in d['models']]"
echo "=== GENESIS ===" && /usr/local/Cellar/ollama/0.23.0/bin/ollama --version 2>/dev/null; genesis --version 2>/dev/null || echo "genesis: check PATH"
echo "=== LITELLM CONFIG ===" && [ -f litellm-config.yaml ] && echo "PRESENT" || echo "MISSING"
echo "=== ALL AGENTS SKILL-BOUND ===" && grep -rL "^skills:" /Users/kirtissiemens/CascadeProjects/.ai/**/*.md 2>/dev/null | grep -v INDEX | grep -v README || echo "ALL BOUND"
```
