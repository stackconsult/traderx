---
name: qa-team
description: >
  A quality assurance skill that reviews code changes through multiple persona lenses.
  Use when you need to review code for correctness, testing, security, performance,
  maintainability, or adversarial scenarios. Invoke with @qa-team followed by one of:
  review-correctness, review-testing, review-security, review-performance,
  review-maintainability, review-adversarial, or full-review.
license: MIT
compatibility: Works with Windsurf Cascade, Claude Code, and any AgentSkills-compatible agent.
metadata:
  author: favio-vazquez
  version: "1.0"
---

# QA Team

A quality assurance skill that applies systematic code review through specific persona lenses. Based on the learnship code-reviewer agent persona.

**Core principle:** Review code through specific lenses to catch different classes of bugs. Each lens asks different questions and catches different issues.

---

## Actions

### `review-correctness` — Logic and state verification

**Trigger:** `@qa-team review-correctness <diff or files>`

**What to do:**
1. Read the diff or files to review
2. Load project context from `./AGENTS.md`, `./CLAUDE.md`, or `./GEMINI.md`
3. Apply the correctness lens:
   - Logic errors
   - Edge cases
   - State bugs
   - Error propagation
   - Intent compliance
4. Ask: "Does this code actually do what it claims to do? What inputs would break it?"
5. Return structured findings with severity (P0-P3) and confidence (0.0-1.0):

```
## Review: correctness lens

### Findings

**[P0]** [file:line] — [title]
Confidence: [0.XX]
Evidence: [specific code and explanation]
Suggestion: [fix if obvious]

**[P1]** [file:line] — [title]
Confidence: [0.XX]
Evidence: [specific code and explanation]
Suggestion: [fix if obvious]

### Summary
Findings: [N] total ([breakdown by severity])
Coverage: [which parts were reviewed]
```

---

### `review-testing` — Test coverage and quality

**Trigger:** `@qa-team review-testing <diff or files>`

**What to do:**
1. Read the diff or files to review
2. Apply the testing lens:
   - Coverage gaps
   - Weak assertions
   - Brittle tests
   - Missing negative tests
3. Ask: "If a bug were introduced here, would the tests catch it?"
4. Return structured findings with severity and confidence

---

### `review-security` — Security vulnerability scan

**Trigger:** `@qa-team review-security <diff or files>`

**What to do:**
1. Read the diff or files to review
2. Apply the security lens:
   - Auth bypass
   - Input validation
   - Secrets exposure
   - Permission escalation
   - Unsafe deserialization
3. Ask: "How would an attacker exploit this?"
4. Return structured findings with severity and confidence

---

### `review-performance` — Performance analysis

**Trigger:** `@qa-team review-performance <diff or files>`

**What to do:**
1. Read the diff or files to review
2. Apply the performance lens:
   - N+1 queries
   - Unbounded loops
   - Missing indexes
   - Memory leaks
   - Missing pagination
3. Ask: "What happens at 10x the expected load?"
4. Return structured findings with severity and confidence

---

### `review-maintainability` — Code quality assessment

**Trigger:** `@qa-team review-maintainability <diff or files>`

**What to do:**
1. Read the diff or files to review
2. Load project context, including `.planning/codebase/CONVENTIONS.md` if it exists
3. Apply the maintainability lens:
   - Coupling
   - Complexity
   - Naming
   - Dead code
   - Premature abstraction
4. Check compliance with project conventions if CONVENTIONS.md exists
5. Ask: "Will a new team member understand this in 6 months?"
6. Return structured findings with severity and confidence

---

### `review-adversarial` — Adversarial testing mindset

**Trigger:** `@qa-team review-adversarial <diff or files>`

**What to do:**
1. Read the diff or files to review
2. Apply the adversarial lens:
   - Assume the code is wrong and prove it
   - Test with empty input
   - Test with null values
   - Test with max values
   - Test concurrent access
3. Ask: "What's the most creative way to break this?"
4. Return structured findings with severity and confidence

---

### `full-review` — Multi-lens comprehensive review

**Trigger:** `@qa-team full-review <diff or files>`

**What to do:**
1. Read the diff or files to review
2. Load project context
3. Apply all six lenses sequentially:
   - correctness
   - testing
   - security
   - performance
   - maintainability
   - adversarial
4. Aggregate findings by severity
5. Return comprehensive review:

```
## Full Code Review

### Severity Summary
- **P0:** [N] critical issues
- **P1:** [N] high-impact issues
- **P2:** [N] moderate issues
- **P3:** [N] low-impact improvements

### Findings by Lens

#### Correctness
[P0] [file:line] — [title]
Confidence: [0.XX]
Evidence: [explanation]

#### Testing
[P1] [file:line] — [title]
Confidence: [0.XX]
Evidence: [explanation]

[... other lenses ...]

### Recommendations
1. [Must fix before merge]
2. [Should fix]
3. [Nice to have]
```

---

## Severity Scale

| Level | Meaning | Action |
|-------|---------|--------|
| **P0** | Critical breakage, exploitable vulnerability, data loss | Must fix before merge |
| **P1** | High-impact defect likely hit in normal usage | Should fix |
| **P2** | Moderate issue — edge case, perf regression, maintainability trap | Fix if straightforward |
| **P3** | Low-impact, minor improvement | Discretion |

---

## Principles

- **Lens-specific:** Each lens asks different questions and catches different bugs
- **Evidence-based:** Always provide specific code snippets and explanations
- **Confidence scoring:** Rate how certain you are about each finding (0.0-1.0)
- **Severity classification:** Prioritize findings by impact
- **Fix suggestions:** Provide concrete fixes when the correct approach is obvious
- **Read-only:** Do NOT edit files during review — only analyze and report

---

## Memory Integration (Mem0)

### Before Code Review
Retrieve relevant past review experiences from mem0:
- Query mem0 for similar code patterns previously reviewed
- Load common issues found in this codebase
- Retrieve review history for the files being reviewed
- Get context about project-specific review criteria

### During Code Review
Store review context in mem0:
- Record findings with severity and confidence
- Store patterns of issues found
- Document review decisions and reasoning
- Track which lenses are most effective for this codebase

### After Code Review
Store lessons learned in mem0:
- What types of issues are common in this codebase
- Which review lenses catch the most bugs
- Code patterns that frequently have issues
- Review strategies that worked well

### Memory Schema
```json
{
  "type": "code_review",
  "skill": "qa-team",
  "action": "review-correctness|review-testing|review-security|review-performance|review-maintainability|review-adversarial|full-review",
  "files_reviewed": ["..."],
  "findings": [
    {
      "severity": "P0|P1|P2|P3",
      "lens": "...",
      "issue": "...",
      "confidence": 0.95
    }
  ],
  "lessons_learned": ["..."],
  "timestamp": "2026-05-02T19:00:00Z"
}
```
