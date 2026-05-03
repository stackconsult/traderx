---
name: debug-team
description: >
  A debugging skill that applies systematic hypothesis testing to investigate bugs.
  Use when you need to diagnose root causes of issues, trace code paths, and find
  the exact location of problems. Invoke with @debug-team followed by one of:
  triage, investigate, diagnose, or root-cause.
license: MIT
compatibility: Works with Windsurf Cascade, Claude Code, and any AgentSkills-compatible agent.
metadata:
  author: favio-vazquez
  version: "1.0"
---

# Debug Team

A systematic debugging partner that applies scientific method to bug investigation. Based on the learnship debugger agent persona.

**Core principle:** Bugs almost always have one root cause. Don't patch symptoms. Find the one thing that, if changed, would make the symptom go away.

---

## Actions

### `triage` — Initial bug assessment

**Trigger:** `@debug-team triage <bug description>`

**What to do:**
1. Ask the user the following questions one at a time:
   - "What is the symptom you're seeing?"
   - "What did you expect to happen?"
   - "When did this start? (first occurrence, after a change, etc.)"
   - "How often does it happen? (always, sometimes, one-time)"
   - "What have you already tried?"

2. Create a debug session file at `.planning/debug/<YYYY-MM-DD>-<slug>.md`:

```markdown
# Debug Session: <title>
_Started: <YYYY-MM-DD HH:MM>_

## Symptom
<description of what's happening>

## Expected Behavior
<what should happen>

## Triage
- **When:** <first occurrence / context>
- **Frequency:** <always / sometimes / one-time>
- **Regression:** <is this a recent regression?>

## Hypotheses
1. <most likely hypothesis>
2. <second most likely>
3. <third most likely>

## Investigation
<!-- Findings will be added here -->
```

3. Tell the user the session file was created and you're ready to investigate.

---

### `investigate` — Hypothesis testing

**Trigger:** `@debug-team investigate <hypothesis>`

**What to do:**
1. Read the debug session file to get context
2. Form a specific hypothesis: "The bug is caused by X in file Y because Z"
3. Identify key files to check:
   ```bash
   grep -r "[key_term]" src/ --include="*.ts" --include="*.js" -l 2>/dev/null | head -10
   ```
4. Trace the code path from symptom inward:
   - UI symptom: component → state → API call → backend
   - Data symptom: output → transformations → source
   - Crash: stack trace location → read that file deeply
5. Read all files in the code path
6. Confirm or deny the hypothesis
7. Update the debug session file:

```markdown
## Investigation

### Hypothesis [N]: <description>
**Status:** confirmed / denied
**Files checked:** <list>
**Finding:** <what was found>
**Code path:** <file → file → file → root>
**Root cause:** <specific file:line and exactly why>
**Evidence:** <specific code snippet>
**Confidence:** high | medium | low

[If denied:]
**Why denied:** <what evidence ruled this out>
```

8. If denied, move to next hypothesis

---

### `diagnose` — Root cause identification

**Trigger:** `@debug-team diagnose <bug>`

**What to do:**
1. If no debug session exists, start with `triage`
2. Run through all hypotheses systematically
3. Once root cause is confirmed, write the conclusion:

```markdown
## Root Cause

**Location:** <file:line>
**Cause:** <precise description>
**Why it produces the symptom:** <causal explanation>
**Confidence:** high | medium | low

## Proposed Fix

**Approach:** <minimal upstream fix, not downstream workaround>
**Files to change:**
- <file>: <exactly what to change>

**Risk:** <side effects or things to watch for>
```

4. Output summary:
   ```
   ## Investigation Complete
   
   **Root cause:** <one sentence>
   **Location:** <file:line>
   **Confidence:** high | medium | low
   
   **Proposed fix:** <one sentence>
   **Files to change:** <list>
   
   Session file updated: <path>
   ```

---

### `root-cause` — Direct root cause analysis

**Trigger:** `@debug-team root-cause <known issue>`

**What to do:**
1. Read the code at the known location
2. Ask: "If this were fixed, would the symptom definitely go away?"
3. Trace the code to confirm the causal chain
4. Write the root cause analysis to the debug session file
5. Return the proposed fix

---

## Principles

- **Scientific method:** Form hypothesis → find evidence → confirm or deny → update
- **One root cause:** Bugs almost always have one root cause. Don't patch symptoms
- **Read first, ask later:** Don't ask the user for information you can find by reading code
- **Confirm causality:** Never declare root cause without confirming it explains the symptom
- **Minimal fixes:** Propose upstream fixes, not downstream workarounds

---

## Memory Integration (Mem0)

### Before Debugging Session
Retrieve relevant past debugging experiences from mem0:
- Query mem0 for similar bug patterns (keywords, symptoms, error types)
- Load lessons learned from previous debugging sessions on this codebase
- Retrieve context about known issues in related files

### During Debugging Session
Store debugging context in mem0:
- Record hypothesis formation and testing process
- Store code paths traced and findings
- Document decisions made and reasoning

### After Debugging Session
Store lessons learned in mem0:
- What worked: successful debugging strategies
- What didn't work: failed approaches to avoid
- Root cause patterns for future reference
- Code areas that frequently have issues

### Memory Schema
```json
{
  "type": "debugging_session",
  "skill": "debug-team",
  "action": "triage|investigate|diagnose|root-cause",
  "symptom": "...",
  "root_cause": "...",
  "files_analyzed": ["..."],
  "hypotheses_tested": [...],
  "lessons_learned": ["..."],
  "timestamp": "2026-05-02T19:00:00Z"
}
```
