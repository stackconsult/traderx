# Self-Upskill Skill

## When to activate
Load when: the agent encounters a task it has no skill for, gets stuck more than twice,
or when the user says "install skills", "upskill", "learn X", or "get better at Y".

## Skill Gap Detection

A skill gap exists when:
- The agent has attempted a task 2+ times and failed
- The task requires knowledge not in any loaded SKILL.md
- The error message references a library/pattern/language not in `.windsurf/skills/`
- The user explicitly requests a capability the agent doesn't have

## Self-Upskill Loop

```
DETECT gap → name the missing skill (e.g. "tokio-streams", "pgvector-queries")
    ↓
SEARCH sources (in order):
  1. ~/.windsurf/skills/ — already installed?
  2. skills.chat         — community skill available?
  3. github.com/addyosmani/agent-skills — upstream repo
  4. tinyfish/search     — "agent skill for [topic]"
    ↓
DOWNLOAD the SKILL.md to:
  .windsurf/skills/[skill-name]/SKILL.md
    ↓
LOAD the skill — read it fully before proceeding
    ↓
RETRY the task with the new skill active
    ↓
VALIDATE — did the task succeed?
    ↓
LOG in GENESIS_ROADMAP.md:
  ## Skill Gaps Resolved
  - [date] [skill-name]: installed from [source], resolved [task description]
```

## Skill Tailoring

After installing a skill, tailor it for this project:
1. Open the SKILL.md
2. Add a `## Project-Specific Adaptations` section at the bottom:
   - Replace generic examples with TraderX-specific ones
   - Add Rust/Python/zsh variants as needed
   - Note any constraints from `AGENTS.md`

## Skill Upgrade Triggers

Run `/sync-upstream-skills` when:
- A skill has not been updated in > 30 days
- A bug was caused by outdated skill guidance
- A new major version of a referenced library is released

## Rate Limit on Skill Installation

- Max 3 new skills per session (prevents context flooding)
- Each skill adds ~2k tokens to context — budget carefully
- If > 3 skills needed: queue them, install one per task

## Skill Index Update

After installing any skill, update `.windsurf/skills/skills_index.json`:
```json
{
  "skills": [
    {
      "name": "skill-name",
      "path": ".windsurf/skills/skill-name/SKILL.md",
      "triggers": ["keyword1", "keyword2"],
      "installed": "YYYY-MM-DD",
      "source": "url-or-source"
    }
  ]
}
```

## Never-Install Rules

Do NOT install skills that:
- Require writing secrets/keys into skill files
- Duplicate an existing skill (extend instead)
- Have not been reviewed for accuracy
- Reference deprecated APIs (check dates)
