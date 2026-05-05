---
description: Self-audit workflow — Genesis reviews its own skill coverage, identifies gaps, and proposes upskill additions. Run after any session with friction or repeated mistakes.
---

# /self-audit — Autonomous Skill Gap Analysis

Genesis runs this against itself to identify what skills it's missing, which ones it's drifting from, and what new domain knowledge the project needs.

---

## Step 1 — Inventory current skills

```bash
ls .windsurf/skills/          # all installed skills
cat .windsurfrules | grep "^name:" | sed 's/name: //'   # always-active skills
```

Expected: 20+ skills from agent-skills + any custom project skills.

---

## Step 2 — Audit recent session for drift

Review last 5 git commits for patterns that indicate skill-skipping:

```bash
git log --oneline -10
git show HEAD --stat | head -20
```

Check for red flags:
- [ ] Commits >300 lines → `incremental-implementation` drift
- [ ] No test files in commits with logic changes → `test-driven-development` drift
- [ ] `unwrap()` additions in production code → guarded-lines violation
- [ ] Secrets or API keys in diff → `security-and-hardening` violation
- [ ] Direct implementation without spec file → `spec-driven-development` skipped

---

## Step 3 — Domain gap analysis

Check if new codebase areas lack skill coverage:

```bash
ls packages/                   # Rust crates — covered by rust-analyzer + TDD skills?
ls src/                        # Python strategies — covered by Python skills?
grep -r "vhdl\|fpga\|aeron\|questdb" .planning/ | wc -l  # new domains?
```

For each domain without a skill:
1. Check `https://github.com/addyosmani/agent-skills` for existing skill
2. If found → install into `.windsurf/skills/`
3. If not found → draft new SKILL.md (see Step 5)

---

## Step 4 — CVE and dependency audit

```bash
cargo audit 2>&1 | grep -E "(error|warning|RUSTSEC)" | head -20
cat package.json 2>/dev/null | python3 -c "import json,sys; d=json.load(sys.stdin); [print(k,v) for k,v in d.get('dependencies',{}).items()]"
```

New CVE categories not yet in `/security-gate` workflow → add them.

---

## Step 5 — Draft new skill (if gap found)

Create `.windsurf/skills/<skill-name>/SKILL.md` following this anatomy:

```markdown
---
name: skill-name
description: One-line description. Use when [triggering condition].
---

# Skill Name

## Overview
What this skill does and why it matters for this project.

## When to Use
- [Specific trigger condition 1]
- [Specific trigger condition 2]

## Process
1. [Step with concrete action]
2. [Step with verification gate]
3. [Step with evidence requirement]

## Common Rationalizations
| Excuse | Reality |
|--------|---------|
| "I'll add this later" | Later never comes. Do it now. |

## Red Flags
- [Sign that skill is being skipped]

## Verification
- [ ] [Specific evidence that skill was followed]
```

---

## Step 6 — Add to .windsurfrules if always-active

If the new skill should activate on every session (not just on demand), append it to `.windsurfrules`.

---

## Step 7 — Update GENESIS_AGENT.md

Update the "Self-Upskilling Targets" table:
- Move completed gaps to "Installed" section
- Add newly discovered gaps
- Commit: `chore(skills): add <skill-name> — closes skill gap from self-audit`

---

## Output format

```
## Self-Audit Report — [date]

### Skills in good standing
- [skill]: last used [when], no drift detected

### Skills with drift
- [skill]: [evidence of skipping] → ACTION: [specific fix]

### New gaps identified
- [domain/area]: [why it needs a skill] → ACTION: draft SKILL.md or install from upstream

### CVE categories not covered
- [category]: [evidence] → ACTION: add to /security-gate

### Proposed additions to .windsurfrules
- [skill]: [justification for always-active]
```
