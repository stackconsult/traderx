# Agent Execution Safety Skill

## Trigger
When the agent is looping on the same problem, context is bloated, multiple fixes have failed, or the user says "hung" / "stuck" / "overload"

## Root Causes of Agent Overload

### 1. Speculation Loops
- Guessing fixes without understanding the API
- Trying 3+ variations of the same approach
- Context accumulates failed attempts
- Confidence degrades with each failure

### 2. Batch Bloat
- Creating multiple files/skills in one cycle
- Fixing multiple unrelated errors in one edit
- Committing large batches without intermediate validation
- Context grows beyond effective reasoning capacity

### 3. Verification Skipping
- Not checking if the previous fix worked
- Assuming success without running cargo check / git status
- Proceeding to next task before confirming current one

### 4. No Abort Criteria
- Continuing to fix the same error after 3+ failed attempts
- Not switching tactics when approach is clearly wrong
- Not asking the user for clarification

## Safe Execution Protocol

### Phase 1: Assess (2 minutes)
**Mandatory before any work cycle:**

```
1. What is the ONE thing I am trying to do right now?
2. What is my confidence level (0-10)?
3. How many attempts have I already made?
4. Is the context clean or bloated?
```

**Abort if:**
- Confidence < 3 after 2+ attempts → Stop and research
- Context > 10 failed attempts → Commit current state, start fresh cycle
- Multiple unrelated tasks mixed → Split into separate cycles

### Phase 2: Execute (15-30 minutes max)
**One task only. One file or one logical change.**

Rules:
- Single focus: Fix ONE error, create ONE skill, make ONE commit
- Time limit: 30 minutes maximum per cycle
- Verification: Check after EVERY single change (cargo check, git status, test)
- No batching: Do not create 3 files in one go

### Phase 3: Verify (2 minutes)
**Mandatory before declaring success:**

```
1. Did my change work? (cargo check passes, test passes, etc.)
2. Is the state clean? (no unverified changes lying around)
3. Can I explain what I did in one sentence?
4. Should I commit now before proceeding?
```

**If verification fails:**
- First failure: Try ONE more time with a different approach
- Second failure: Stop. Research the API/library docs
- Third failure: Abort cycle. Report to user. Do not guess.

### Phase 4: Checkpoint (2 minutes)
**Clean break before next cycle:**

```
1. Commit verified changes with clear message
2. Push to remote
3. Clear mental context: "Cycle N complete, moving to Cycle N+1"
4. State the next single task explicitly
```

## Work Cycle Template

```
--- Cycle Start ---
Task: [ONE specific thing]
Confidence: [X/10]
Attempt: [# of this cycle]

[Execute - max 30 min]

Verification: [PASS / FAIL]
Next action if fail: [research / ask user / try once more]

If PASS:
- Commit: "[type](scope): [description]"
- Push: git push origin [branch]
- State next cycle task
--- Cycle End ---
```

## Anti-Patterns (NEVER DO)

1. **Never create >2 new files in one cycle** → Split into separate cycles
2. **Never fix >1 compilation error per edit without checking** → Verify after each
3. **Never commit >10 files with vague message** → Commit incrementally
4. **Never continue after 3 failed attempts on same error** → Research or ask
5. **Never mix skill creation with code fixes** → Separate cycles
6. **Never let context grow >20 failed attempts** → Commit and reset

## Recovery Protocol

When user says "hung" / "stuck" / "overload":

1. **STOP all current work immediately**
2. Assess: What was I trying to do? How many attempts?
3. If >2 attempts failed: Do NOT try again. Research instead.
4. Commit any verified changes
5. Ask user: "I was trying to [X] but got stuck after [N] attempts. Should I research [Y] or do you want to clarify [Z]?"
6. Start fresh cycle with explicit single task

## PM Team Rules

### Sprint Structure
- Sprint = 1-3 cycles (max 90 minutes total)
- Each cycle = 15-30 min execute + 2 min verify + 2 min checkpoint
- Between sprints: Full commit, push, status report

### Definition of Done (per cycle)
- [ ] Single task completed
- [ ] Verification passed
- [ ] Changes committed
- [ ] Can explain in one sentence

### Escalation Criteria
- Cycle fails 3 times → Escalate to user
- Context bloat detected → Escalate to user
- Multiple unrelated tasks mixed → Escalate to user
- Confidence < 3 on critical task → Escalate to user

## QA Team Gate

Between each cycle, QA asks:
1. Did you verify the previous change? (Y/N)
2. Are you working on one task or many? (One/Many)
3. How many failed attempts so far? (0/1/2/3+)
4. Is context clean or bloated? (Clean/Bloated)

**If any answer is wrong → STOP and reassess**
