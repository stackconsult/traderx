# Validation Gate Workflow

## Description

Mandatory validation checkpoint before any merge or deployment. Ensures all GitHub Actions checks passed by querying GitHub API. Part of GitHub-first architecture where validation happens at the source of truth.

**Principle**: *"Never merge without verified Actions success"*

---

## Validation Sequence

### **Step 1: Pre-Push Validation**

Before pushing any changes:

```bash
# 1. Run local tests first (smoke check)
cargo test --all

# 2. Check code quality
cargo clippy -- -D warnings
cargo fmt -- --check

# 3. Security audit
cargo audit

# Local tests pass? → Proceed to push
```

### **Step 2: Push and Trigger**

```bash
# Push to trigger Actions
git push origin <branch>

# Actions automatically run on push
```

### **Step 3: Automated Validation (This Workflow)**

```bash
# Wait for Actions completion and validate
./scripts/validate_actions_status.sh <branch>

# Script will:
# 1. Query GitHub API for latest run
# 2. Poll until completion (up to 15 min)
# 3. Report success/failure
# 4. Exit with appropriate code
```

### **Step 4: Gate Decision**

**If validation passes:**
- ✅ Create PR if not exists
- ✅ Mark as "ready for merge"
- ✅ Proceed to merge workflow

**If validation fails:**
- ❌ Block merge
- ❌ Analyze failure logs
- ❌ Engineer fix
- ❌ Return to Step 1

---

## GitHub Actions Validation Job

```yaml
# .github/workflows/validation-gate.yml
name: Validation Gate

on:
  pull_request:
    branches: [main]
    types: [opened, synchronize, reopened]

jobs:
  validate-actions-passed:
    name: Validate Actions Passed
    runs-on: ubuntu-latest
    steps:
      - name: Check Actions Status
        id: validate
        run: |
          BRANCH="${{ github.head_ref }}"
          
          echo "Validating Actions for branch: $BRANCH"
          
          # Get latest run
          RUN_DATA=$(curl -s -H "Authorization: token ${{ secrets.GITHUB_TOKEN }}" \
            "https://api.github.com/repos/${{ github.repository }}/actions/runs?branch=$BRANCH&per_page=1")
          
          CONCLUSION=$(echo "$RUN_DATA" | jq -r '.workflow_runs[0].conclusion // "unknown"')
          
          if [ "$CONCLUSION" == "success" ]; then
            echo "✅ Actions passed"
            echo "passed=true" >> $GITHUB_OUTPUT
          else
            echo "❌ Actions not passed (conclusion: $CONCLUSION)"
            echo "passed=false" >> $GITHUB_OUTPUT
            exit 1
          fi
      
      - name: Report Status
        if: always()
        run: |
          if [ "${{ steps.validate.outputs.passed }}" == "true" ]; then
            echo "🟢 VALIDATION GATE: PASSED"
            echo "All Actions checks successful."
            echo "Branch ready for merge."
          else
            echo "🔴 VALIDATION GATE: BLOCKED"
            echo "Actions checks failed or incomplete."
            echo "Fix required before merge."
            exit 1
          fi
```

---

## Local Validation Integration

### **Script: `scripts/validate_before_merge.sh`**

```bash
#!/bin/bash
# Run before any merge operation
# Validates Actions passed on GitHub

BRANCH=$(git branch --show-current)

echo "=== Pre-Merge Validation ==="
echo "Branch: $BRANCH"
echo ""

# Check local is clean
if [ -n "$(git status --porcelain)" ]; then
    echo "❌ Local changes not committed"
    git status
    exit 1
fi

# Validate Actions passed
./scripts/validate_actions_status.sh "$BRANCH"

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Validation complete. Safe to merge."
    exit 0
else
    echo ""
    echo "❌ Validation failed. Do not merge."
    exit 1
fi
```

---

## Validation Checklist

### **Before Merge, Verify**:

- [ ] Local tests pass (`cargo test`)
- [ ] Code quality clean (`cargo clippy`)
- [ ] No hardcoded secrets (`cargo audit`)
- [ ] **Actions passed on GitHub** (via validation script)
- [ ] No merge conflicts
- [ ] PR reviewed (if required)

### **Actions Jobs Must Pass**:

- [ ] Security Audit (no CVEs)
- [ ] Code Quality (clippy clean)
- [ ] Unit Tests (all pass)
- [ ] Integration Tests (services work)
- [ ] Benchmarks (performance OK)
- [ ] Container Scan (no image CVEs)
- [ ] K8s Validation (manifests valid)

---

## Failure Handling

### **When Actions Fail**:

1. **Identify Failed Job**:
   ```bash
   # Get detailed job information
   curl -s -H "Authorization: token $GITHUB_TOKEN" \
     "https://api.github.com/repos/$OWNER/$REPO/actions/runs/$RUN_ID/jobs" | \
     jq -r '.jobs[] | select(.conclusion == "failure") | .name'
   ```

2. **Download Logs**:
   ```bash
   # Get failure logs
   curl -s -H "Authorization: token $GITHUB_TOKEN" \
     "https://api.github.com/repos/$OWNER/$REPO/actions/runs/$RUN_ID/logs" \
     -o failure-logs.zip
   unzip failure-logs.zip -d failure-logs/
   cat failure-logs/*/*.txt | grep -A 5 "error\|FAILED\|panic"
   ```

3. **Engineer Fix**:
   - Analyze failure pattern
   - Apply targeted fix
   - Test locally
   - Commit and push

4. **Re-validate**:
   ```bash
   # Trigger new Actions run
   git commit --allow-empty -m "ci: trigger re-run"
   git push
   
   # Re-run validation
   ./scripts/validate_actions_status.sh <branch>
   ```

---

## Success Metrics

| Metric | Target | Current |
|--------|--------|---------|
| **Validation time** | < 15 min | TBD |
| **False positives** | < 5% | 0% |
| **Merge without validation** | 0% | 0% |
| **Actions pass rate** | > 95% | TBD |

---

## Integration with Other Workflows

### **With Pre-Action Analysis**:
```markdown
### Pre-Action Checklist
- [ ] Goal alignment
- [ ] Risk assessment
- [ ] **Validation gate check** ← NEW
- [ ] Alternative approaches
```

### **With Meta-Cognitive**:
```markdown
### Mistake Journal
- Mistake: Merged without Actions validation
- Prevention: Mandatory validation gate
```

### **With Session-Start**:
```markdown
### Session Start Checklist
- [ ] GitHub Actions status
- [ ] Branch validation
- [ ] **Previous PR validation status** ← NEW
```

---

## Command Reference

### **Quick Validation**:
```bash
# Validate current branch
./scripts/validate_actions_status.sh $(git branch --show-current)

# Validate specific branch
./scripts/validate_actions_status.sh fix/oms-engine-compilation-errors

# Check with timeout override
TIMEOUT=30 ./scripts/validate_actions_status.sh <branch>
```

### **Monitor Continuously**:
```bash
# Watch until completion
while true; do
    ./scripts/validate_actions_status.sh <branch> && break
    sleep 60
done
```

---

## Troubleshooting

### **"No Actions runs found"**
- Push may not have triggered Actions
- Check `.github/workflows/` exists
- Verify YAML syntax valid
- Check branch matches `on:` conditions

### **"GITHUB_TOKEN not set"**
- Token required for API access
- Create: https://github.com/settings/tokens
- Set: `export GITHUB_TOKEN=ghp_...`
- Scope: `repo` (full control of private repos)

### **Actions running too long**
- Check if jobs queued
- Verify runner availability
- Check for infinite loops in tests

---

**This workflow ensures: No merge without verified Actions success.**
