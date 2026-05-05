# Neural Context Engineering Skill

## When to activate
Load when: managing context across model calls, sessions, or agent handoffs.
Critical for multi-agent orchestration, session continuity, and memory efficiency.

## Context Architecture — This System

```
┌─────────────────────────────────────────────────────┐
│                  GENESIS HOARD AGENT                 │
│                                                       │
│  ┌─────────────┐   ┌─────────────┐   ┌───────────┐  │
│  │  Working    │   │   mem0      │   │ pgvector  │  │
│  │  Context    │──▶│  (local     │──▶│  (long    │  │
│  │  (session)  │   │   cache)    │   │   term)   │  │
│  └─────────────┘   └─────────────┘   └───────────┘  │
│         │                                    │        │
│         ▼                                    ▼        │
│  ┌─────────────┐              ┌──────────────────┐   │
│  │ Model Call  │              │ GENESIS_ROADMAP   │   │
│  │ (Ollama or  │              │ .md (telemetry    │   │
│  │  Gemini)    │              │  journal)         │   │
│  └─────────────┘              └──────────────────┘   │
└─────────────────────────────────────────────────────┘
```

## Context Budgeting

| Context type | Max size | Storage | Eviction |
|-------------|----------|---------|----------|
| Working (current task) | 8k tokens | RAM | end of task |
| Session (current chat) | 32k tokens | RAM | end of session |
| Skill files | 2k each, max 4 loaded | RAM | manual unload |
| mem0 cache | 1k per entry, 100 entries | Redis | LRU |
| pgvector long-term | unlimited | PostgreSQL | never |

## Handoff Protocol (cross-model/session)

When switching from Cascade → Genesis or local → cloud:
```markdown
## HANDOFF SNAPSHOT
Task: [one sentence]
State: [what has been done]
Pending: [what remains]
Files modified: [list]
Last error (if any): [error]
Next action: [exact next step]
Model needed: [fast/coder/balanced/think]
```

## Context Injection Order (optimised for token efficiency)

```
1. SYSTEM: agent role + hard constraints (always, ~500 tokens)
2. SKILL: relevant SKILL.md for current phase (~1-2k tokens)
3. CONTEXT: minimal project state (AGENTS.md excerpt, ~1k tokens)
4. MEMORY: top-3 mem0 results for this task (~600 tokens)
5. TASK: the actual request (as small as possible)
```

Never inject:
- Full file contents (use line ranges)
- All skills simultaneously (load on-demand)
- Raw git history (summarise first)
- Unrelated agent persona files

## Contextual Forking

When the agent reaches a decision point with multiple valid paths:
```
Fork detected at: [line/function/decision]
  Branch A: [description] — token cost: N, reversible: Y/N
  Branch B: [description] — token cost: N, reversible: Y/N

Selection rule:
  - Prefer reversible branch
  - Prefer lower token cost when outcomes are equal
  - Never fork into irreversible action without user confirmation
  - Record fork decision in GENESIS_ROADMAP.md
```

## Information Leak Prevention

### What NEVER enters model context:
- `GOOGLE_API_KEY`, `KIMI_API_KEY`, `GITHUB_TOKEN` or any env var value
- SSH private keys, JWT tokens, bearer tokens
- Database passwords or connection strings with credentials
- Personal user data (emails, names, IDs from production)

### Sanitisation before model call:
```python
import re
REDACT_PATTERNS = [
    r'[A-Za-z0-9+/]{40,}={0,2}',  # base64 secrets
    r'sk-[A-Za-z0-9]{40,}',        # OpenAI-style keys
    r'(?:password|secret|token|key)\s*[:=]\s*\S+',  # key=value patterns
]
def sanitise(text):
    for pattern in REDACT_PATTERNS:
        text = re.sub(pattern, '[REDACTED]', text, flags=re.IGNORECASE)
    return text
```

## Session Continuity

At end of every session, write to `GENESIS_ROADMAP.md`:
```markdown
## Session Snapshot — [YYYY-MM-DD HH:MM]
Branch: [git branch]
Last commit: [hash]
Files modified: [list]
Errors resolved: [list]
Pending: [list]
Next session start: [exact command or task]
```

This allows any session (local model, Cascade, or Genesis) to resume without context loss.
