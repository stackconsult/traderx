# Reasoning Logic Skill

## When to activate
Load when: the agent must solve a novel problem, plan multi-step execution,
avoid getting stuck, evaluate competing approaches, or validate its own output.

## The 5-Step Problem Solving Loop (always use this — no shortcuts)

```
STEP 1: UNDERSTAND
  - Restate the problem in one sentence
  - Identify: inputs, outputs, constraints, success criteria
  - Flag any ambiguity — resolve before proceeding

STEP 2: RESEARCH
  - What do I already know? (check mem0 + existing skills)
  - What is missing? (use tinyfish/search or filesystem scan)
  - What have I tried before that failed? (check GENESIS_ROADMAP.md)

STEP 3: PLAN
  - List 2-3 candidate approaches
  - Score each: [feasibility × impact × risk]
  - Pick the highest scorer
  - Break into atomic steps — each step verifiable

STEP 4: EXECUTE
  - One step at a time
  - Validate each step before proceeding
  - If a step fails: goto STEP 2 (not STEP 4 — never retry blind)

STEP 5: VALIDATE
  - Does output match success criteria from STEP 1?
  - Run the proof: cargo check / tests / grep / curl as appropriate
  - Write result to GENESIS_ROADMAP.md telemetry entry
```

## Anti-Stuck Rules

**If stuck for > 2 attempts on the same error:**
1. STOP — do not retry the same approach a 3rd time
2. Search for the error message via tinyfish/search
3. Check if a skill exists for this domain (`skills.chat`)
4. If not: install the skill, then retry with new approach
5. If still stuck: write a minimal reproduction and log to GENESIS_ROADMAP.md

**If output causes errors/damage:**
1. STOP immediately — do not attempt to fix forward
2. `git stash` or revert the change
3. Return to STEP 1 with full understanding of why it failed

## Contextual Forking

When a task could go multiple ways:
```
Fork point detected → evaluate branches:
  Branch A: [approach] → estimated tokens: N, risk: low/med/high, reversible: Y/N
  Branch B: [approach] → estimated tokens: N, risk: low/med/high, reversible: Y/N

Rule: always take the reversible branch first
Rule: never execute irreversible actions (delete, push --force, drop table) without explicit confirmation
```

## Decision Quality Checklist (run before any significant action)

- [ ] Do I have enough information, or am I guessing?
- [ ] Is this reversible if wrong?
- [ ] Have I checked for a pre-existing skill/solution?
- [ ] Will this cause side effects in other parts of the system?
- [ ] Is the success criterion measurable?

## Proof of Work Standard

Every completed task must produce at least ONE of:
- `cargo check` → 0 errors
- `curl` response matching expected
- `grep` finding the expected output in the file
- Test passing: `cargo test [name] -- --nocapture`
- Diff showing the change is correct

Log the proof in `GENESIS_ROADMAP.md`.
