# LLM Modelling Skill

## When to activate
Load this skill when the task involves: LLM integration, prompt engineering, RAG pipelines,
embedding strategies, model selection, fine-tuning, context window management, or AI agent wiring.

## Model Selection Decision Tree

```
Is the task local-only? (no internet, cost-sensitive, < 4k tokens)
  YES → ollama/gemma3:1b  (routing, classification, yes/no)
      → ollama/qwen2.5-coder:1.5b  (code generation)
      → ollama/nomic-embed-text  (embeddings, mem0)

Does it require reasoning, architecture, or multi-step planning?
  YES → google/gemini-2.5-pro-preview-05-06  (think)

Default / most tasks?
  → google/gemini-2.0-flash  (balanced)

Bulk / background / summarisation?
  → google/gemini-2.0-flash-lite  (cheap)

Long context (>32k tokens)?
  → moonshot-v1-32k via Kimi API  (kimi)

Fallback if cloud unavailable?
  → ollama/gemma3:1b → google/gemini-2.0-flash-lite
```

## Context Window Management Rules

1. **Never flood context** — load only the SKILL.md relevant to the current phase
2. **Summarise before handoff** — when switching models, compress context to < 2k tokens
3. **Embed before storing** — use `nomic-embed-text` locally before writing to mem0/pgvector
4. **Token budget per call**:
   - fast (local): 2048 tokens max
   - coder (local): 8192 tokens max
   - balanced (cloud): 32768 tokens max
   - think (cloud): 128k tokens max — reserve for architecture only

## Prompt Engineering Standards

```python
# Structure every prompt:
SYSTEM = """
Role: [specific agent role]
Constraints: [hard rules — never bypass]
Output format: [exact format required]
"""

USER = """
Context: [minimal relevant context]
Task: [one sentence]
Success criteria: [measurable done state]
"""
# No rambling preambles. No "as an AI". Direct task framing only.
```

## RAG Pipeline Pattern

```
Query → embed (nomic-embed-text local) → pgvector similarity search
     → top-k chunks → rerank (optional) → inject into context
     → model call → response → store result embedding → mem0
```

## Rate Limiting (enforced)

| Provider | RPM limit | TPM limit | Strategy |
|----------|-----------|-----------|----------|
| Ollama local | unlimited | RAM-bound | queue depth ≤ 3 concurrent |
| Google Gemini Flash | 1000 RPM | 1M TPM | exponential backoff, retry × 3 |
| Google Gemini Pro | 60 RPM | 32k TPM | reserve for think tasks only |
| Kimi | 20 RPM | 128k TPM | long-context only, no polling |

**Backoff formula**: `wait = min(2^attempt * 0.5s, 30s) + jitter(0-1s)`

## Information Security

- **Never include** API keys, secrets, passwords, PII in model prompts
- **Never log** model responses containing credentials
- **Redact before send**: `re.sub(r'[A-Za-z0-9+/]{40,}', '[REDACTED]', text)`
- Context passed between models must be sanitised — strip file paths containing usernames
- All prompts stored in mem0 must have `sensitive: false` flag verified

## Self-Upskill Trigger

When this skill is insufficient for the task:
1. Search `skills.chat` or `github.com/addyosmani/agent-skills` for relevant SKILL.md
2. Download to `.windsurf/skills/[skill-name]/SKILL.md`
3. Load the new skill and retry
4. Log the gap to `GENESIS_ROADMAP.md` under `## Skill Gaps Resolved`
