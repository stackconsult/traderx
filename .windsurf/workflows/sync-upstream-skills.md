---
description: Sync agentic-learning and impeccable skills from their upstream repos (FavioVazquez/agentic-learning + pbakaus/impeccable) — run this when upstream skills have been updated
---

# /sync-upstream-skills — Pull Latest Skills from Upstream

Run this when `addyosmani/agent-skills` has new releases or when new skill repos are identified.

---

## Step 1 — Check for upstream updates

```bash
# Check agent-skills for new releases
curl -s https://api.github.com/repos/addyosmani/agent-skills/releases/latest \
  | python3 -c "import json,sys; r=json.load(sys.stdin); print(r['tag_name'], r['published_at'])"

# Check current installed version
cat .windsurf/skills/.version 2>/dev/null || echo "version not tracked"
```

---

## Step 2 — Fetch updated skills

```bash
SKILLS_DIR=".windsurf/skills"
BASE_URL="https://raw.githubusercontent.com/addyosmani/agent-skills/main/skills"

SKILLS=(
  "idea-refine"
  "spec-driven-development"
  "planning-and-task-breakdown"
  "incremental-implementation"
  "context-engineering"
  "source-driven-development"
  "frontend-ui-engineering"
  "test-driven-development"
  "api-and-interface-design"
  "browser-testing-with-devtools"
  "debugging-and-error-recovery"
  "code-review-and-quality"
  "code-simplification"
  "security-and-hardening"
  "performance-optimization"
  "git-workflow-and-versioning"
  "ci-cd-and-automation"
  "deprecation-and-migration"
  "documentation-and-adrs"
  "shipping-and-launch"
  "using-agent-skills"
)

for skill in "${SKILLS[@]}"; do
  mkdir -p "$SKILLS_DIR/$skill"
  curl -fsSL "$BASE_URL/$skill/SKILL.md" -o "$SKILLS_DIR/$skill/SKILL.md"
  echo "Updated: $skill"
done
```

---

## Step 3 — Fetch updated personas

```bash
AGENTS_DIR=".windsurf/agents"
AGENTS_URL="https://raw.githubusercontent.com/addyosmani/agent-skills/main/agents"

for agent in code-reviewer security-auditor test-engineer; do
  curl -fsSL "$AGENTS_URL/${agent}.md" -o "$AGENTS_DIR/${agent}.md"
  echo "Updated persona: $agent"
done
```

---

## Step 4 — Fetch updated references

```bash
REF_DIR=".windsurf/references"
REF_URL="https://raw.githubusercontent.com/addyosmani/agent-skills/main/references"

for ref in testing-patterns security-checklist performance-checklist accessibility-checklist orchestration-patterns; do
  curl -fsSL "$REF_URL/${ref}.md" -o "$REF_DIR/${ref}.md"
  echo "Updated reference: $ref"
done
```

---

## Step 5 — Record version and commit

```bash
curl -s https://api.github.com/repos/addyosmani/agent-skills/releases/latest \
  | python3 -c "import json,sys; r=json.load(sys.stdin); print(r['tag_name'])" \
  > .windsurf/skills/.version

git add .windsurf/
git commit -m "chore(skills): sync upstream agent-skills to $(cat .windsurf/skills/.version)"
git push origin feature/github-mcp-setup
```

---

## Step 6 — Run /self-audit after sync

New skill versions may close existing gaps or introduce new capabilities. Run `/self-audit` immediately after syncing to identify what changed and what to wire into `.windsurfrules`.
