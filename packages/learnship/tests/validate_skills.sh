#!/usr/bin/env bash
# learnship — Skills validation tests
# Checks that bundled skills have the required structure


REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SKILLS_DIR="$REPO_DIR/.windsurf/skills"
PASS=0
FAIL=0
ERRORS=()

check() {
  local description="$1"
  local result="$2"
  if [ "$result" = "0" ]; then
    echo "  ✓ $description"
    PASS=$((PASS+1))
  else
    echo "  ✗ $description"
    FAIL=$((FAIL+1))
    ERRORS+=("$description")
  fi
}

echo ""
echo "─── Skills Structure Validation ────────────────────────────────────────"

IMPECCABLE_DIR="$SKILLS_DIR/impeccable"

IMPECCABLE_SKILLS=(
  "frontend-design"
  "adapt" "animate" "audit" "bolder" "clarify" "colorize"
  "critique" "delight" "distill" "extract" "harden"
  "normalize" "onboard" "optimize" "polish" "quieter"
  "teach-impeccable"
)

FRONTEND_DESIGN_REFS=(
  "typography.md" "color-and-contrast.md" "spatial-design.md"
  "motion-design.md" "interaction-design.md" "responsive-design.md"
  "ux-writing.md"
)

# Check agentic-learning skill (top-level)
echo ""
echo "  Checking skill: agentic-learning"
if [ -d "$SKILLS_DIR/agentic-learning" ]; then
  echo "    ✓ Directory exists"
  PASS=$((PASS+1))
else
  echo "    ✗ Directory missing"
  FAIL=$((FAIL+1))
  ERRORS+=("Missing skill directory: agentic-learning")
fi
if [ -f "$SKILLS_DIR/agentic-learning/SKILL.md" ]; then
  echo "    ✓ SKILL.md present"
  PASS=$((PASS+1))
else
  echo "    ✗ SKILL.md missing"
  FAIL=$((FAIL+1))
  ERRORS+=("Missing SKILL.md: agentic-learning")
fi

# Check impeccable/ container exists
echo ""
echo "  Checking impeccable/ container"
if [ -d "$IMPECCABLE_DIR" ]; then
  echo "    ✓ impeccable/ directory exists"
  PASS=$((PASS+1))
else
  echo "    ✗ impeccable/ directory missing"
  FAIL=$((FAIL+1))
  ERRORS+=("Missing impeccable/ container directory")
fi

# Check each impeccable skill
for skill in "${IMPECCABLE_SKILLS[@]}"; do
  skill_dir="$IMPECCABLE_DIR/$skill"
  echo ""
  echo "  Checking impeccable/$skill"

  if [ -d "$skill_dir" ]; then
    echo "    ✓ Directory exists"
    PASS=$((PASS+1))
  else
    echo "    ✗ Directory missing"
    FAIL=$((FAIL+1))
    ERRORS+=("Missing impeccable/$skill")
    continue
  fi

  if [ -f "$skill_dir/SKILL.md" ]; then
    size=$(wc -c < "$skill_dir/SKILL.md")
    echo "    ✓ SKILL.md present ($size bytes)"
    PASS=$((PASS+1))
  else
    echo "    ✗ SKILL.md missing"
    FAIL=$((FAIL+1))
    ERRORS+=("Missing SKILL.md: impeccable/$skill")
  fi

  # Check no unresolved placeholders
  if [ -f "$skill_dir/SKILL.md" ]; then
    if grep -q '{{' "$skill_dir/SKILL.md"; then
      echo "    ✗ Unresolved {{placeholders}} in SKILL.md"
      FAIL=$((FAIL+1))
      ERRORS+=("Unresolved placeholders: impeccable/$skill/SKILL.md")
    else
      echo "    ✓ No unresolved placeholders"
      PASS=$((PASS+1))
    fi
  fi
done

# Check frontend-design reference files
echo ""
echo "  Checking impeccable/frontend-design reference files"
for ref in "${FRONTEND_DESIGN_REFS[@]}"; do
  ref_path="$IMPECCABLE_DIR/frontend-design/reference/$ref"
  if [ -f "$ref_path" ]; then
    echo "    ✓ reference/$ref"
    PASS=$((PASS+1))
  else
    echo "    ✗ reference/$ref missing"
    FAIL=$((FAIL+1))
    ERRORS+=("Missing reference file: $ref")
  fi
done

echo ""
echo "─── Results ────────────────────────────────────────────────────────────"
echo "  Passed: $PASS"
echo "  Failed: $FAIL"

if [ ${#ERRORS[@]} -gt 0 ]; then
  echo ""
  echo "  Failures:"
  for err in "${ERRORS[@]}"; do
    echo "    - $err"
  done
fi

echo ""
[ "$FAIL" -eq 0 ] && echo "  ALL TESTS PASSED ✓" || { echo "  TESTS FAILED ✗"; exit 1; }
