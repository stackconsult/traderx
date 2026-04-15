# GitHub-First Self-Healing Architecture

## Description

Treat GitHub (not local) as the source of truth. Continuously monitor GitHub Actions, automatically detect failures, research solutions, apply fixes, and validate through the same Actions pipeline. Local state is ephemeral - GitHub is the platform.

**Principle**: *"GitHub is the computer. Local is just a cache."*

---

## Architecture: GitHub as Source of Truth

### **The Reality**:
```
┌─────────────────────────────────────────────────────────────┐
│                    GITHUB (The Platform)                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │   Actions    │  │    Code      │  │    Issues    │       │
│  │   (CI/CD)    │  │ (Source of   │  │   (Bugs)     │       │
│  │              │  │   Truth)     │  │              │       │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘       │
│         │                  │                  │             │
│         ▼                  ▼                  ▼             │
│  ┌──────────────────────────────────────────────────┐       │
│  │         AUTOMATED SELF-HEALING LOOP              │       │
│  │  Monitor → Detect → Research → Fix → Validate    │       │
│  └──────────────────────────────────────────────────┘       │
└─────────────────────────────────────────────────────────────┘
                              ▲
                              │ Sync (Push/Pull)
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              LOCAL (Ephemeral Cache)                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │  Working     │  │   Cache      │  │   Build      │       │
│  │  Directory   │  │   (target/)  │  │   Artifacts  │       │
│  │  (Can be     │  │   (Can be    │  │   (Can be    │       │
│  │   erased)    │  │   erased)    │  │   erased)    │       │
│  └──────────────┘  └──────────────┘  └──────────────┘       │
│                                                              │
│  State: TEMPORARY - Can be recreated from GitHub            │
└─────────────────────────────────────────────────────────────┘
```

### **Key Insight**:
- **Local fails?** → `git clone` → Recreate in minutes
- **GitHub fails?** → Platform down → Everyone blocked
- **Therefore**: GitHub state is what matters
- **Therefore**: Actions status is the ultimate validator

---

## The Automated Loop: Monitor → Detect → Fix → Validate

### **Phase 1: Monitor (Continuous)**

```yaml
# .github/workflows/monitor-and-heal.yml
name: Continuous Monitoring & Self-Healing

on:
  schedule:
    - cron: '*/5 * * * *'  # Every 5 minutes
  workflow_run:
    workflows: ["Production Validation Pipeline"]
    types: [completed]

jobs:
  monitor:
    runs-on: ubuntu-latest
    steps:
      - name: Check Actions status
        id: check
        uses: actions/github-script@v7
        with:
          script: |
            const { data: runs } = await github.rest.actions.listWorkflowRunsForRepo({
              owner: context.repo.owner,
              repo: context.repo.repo,
              per_page: 10
            });
            
            const failed = runs.workflow_runs.filter(r => r.conclusion === 'failure');
            
            if (failed.length > 0) {
              core.setOutput('has_failures', 'true');
              core.setOutput('failed_runs', JSON.stringify(failed.map(r => ({
                id: r.id,
                name: r.name,
                head_branch: r.head_branch,
                head_sha: r.head_sha,
                failure_url: r.html_url
              }))));
            }

  analyze-failure:
    needs: monitor
    if: needs.monitor.outputs.has_failures == 'true'
    runs-on: ubuntu-latest
    steps:
      - name: Download failed run logs
        run: |
          echo '${{ needs.monitor.outputs.failed_runs }}' | jq -r '.[] | .id' | while read run_id; do
            gh run download $run_id --dir failure-$run_id
          done
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      
      - name: Analyze failure patterns
        id: analyze
        run: |
          # Categorize failure type
          if grep -r "risk_bus_atomicity" failure-*/; then
            echo "failure_type=race_condition" >> $GITHUB_OUTPUT
          elif grep -r "POSTGRES_PASSWORD" failure-*/; then
            echo "failure_type=hardcoded_secret" >> $GITHUB_OUTPUT
          elif grep -r "clippy" failure-*/; then
            echo "failure_type=lint_error" >> $GITHUB_OUTPUT
          fi

  auto-fix:
    needs: [monitor, analyze-failure]
    if: needs.analyze-failure.outputs.failure_type != ''
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
      
      - name: Apply automated fix
        id: fix
        run: |
          FAILURE_TYPE="${{ needs.analyze-failure.outputs.failure_type }}"
          
          case $FAILURE_TYPE in
            race_condition)
              echo "Applying race condition fix..."
              ./scripts/fix_race_condition.sh
              ;;
            hardcoded_secret)
              echo "Applying secret externalization..."
              ./scripts/externalize_secrets.sh
              ;;
            lint_error)
              echo "Running cargo fix..."
              cargo fix --all --allow-dirty
              ;;
          esac
      
      - name: Create fix PR
        uses: peter-evans/create-pull-request@v5
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
          commit-message: "auto-fix: ${{ needs.analyze-failure.outputs.failure_type }}"
          title: "Auto-Fix: ${{ needs.analyze-failure.outputs.failure_type }}"
          body: |
            This PR was automatically generated to fix a failed workflow.
            
            **Failure Type**: ${{ needs.analyze-failure.outputs.failure_type }}
            **Original Run**: ${{ needs.monitor.outputs.failed_runs }}
            
            Please review and merge if the fix is correct.
          branch: auto/fix-${{ needs.analyze-failure.outputs.failure_type }}

  validate-fix:
    needs: auto-fix
    runs-on: ubuntu-latest
    steps:
      - name: Wait for PR checks
        run: |
          # Poll until checks pass
          for i in {1..30}; do
            status=$(gh pr checks auto/fix-* --json state -q '.[0].state')
            if [ "$status" = "SUCCESS" ]; then
              echo "Fix validated successfully!"
              exit 0
            fi
            sleep 60
          done
          echo "Validation timeout"
          exit 1
```

---

## Phase 2: Local Agent Adaptation

### **How I (Cascade) Must Adapt**:

```rust
/// My new operational model
pub struct GitHubFirstAgent {
    /// GitHub is the source of truth
    source_of_truth: GitHubRepo,
    
    /// Local is just a working cache
    local_cache: LocalWorkspace,
    
    /// Continuous monitoring
    monitor: ActionsMonitor,
    
    /// Self-healing capabilities
    healer: AutoHealer,
}

impl GitHubFirstAgent {
    /// Before any action: Check GitHub state
    pub async fn sync_from_github(&mut self) -> Result<(), SyncError> {
        // 1. Fetch latest from origin/main
        self.local_cache.git_fetch("origin")?;
        
        // 2. Check Actions status
        let status = self.monitor.check_status().await?;
        
        // 3. If Actions failing, prioritize fix
        if status.has_failures() {
            self.healer.queue_fix(status.failures()).await?;
        }
        
        // 4. Sync local to match GitHub
        self.local_cache.git_reset_hard("origin/main")?;
        
        Ok(())
    }
    
    /// After any action: Push to GitHub immediately
    pub async fn push_to_github(&self, changes: &Changes) -> Result<(), PushError> {
        // 1. Create branch
        let branch = format!("auto/fix-{}", changes.issue_id());
        
        // 2. Commit changes
        self.local_cache.git_checkout_branch(&branch)?;
        self.local_cache.git_add_all()?;
        self.local_cache.git_commit(&changes.message())?;
        
        // 3. Push to GitHub
        self.local_cache.git_push("origin", &branch)?;
        
        // 4. Create PR
        let pr = self.source_of_truth.create_pr(&branch, changes).await?;
        
        // 5. Monitor PR until merged or failed
        self.monitor.watch_pr(pr.number()).await?;
        
        Ok(())
    }
}
```

### **Operational Rules**:

1. **Always Start with GitHub**:
   ```bash
   # Before any work
   git fetch origin
   git status  # Check if behind
   gh run list  # Check Actions status
   ```

2. **Never Trust Local State**:
   ```bash
   # Treat local as disposable
   rm -rf target/  # Rebuild from scratch
   cargo clean && cargo build  # Verify clean build
   ```

3. **GitHub Actions Are the Real Tests**:
   ```bash
   # Local tests are just smoke tests
   cargo test  # Quick check
   
   # Real validation happens in Actions
   git push origin branch
   gh run watch  # Wait for Actions
   ```

4. **Continuous Sync**:
   ```bash
   # Every 5 minutes or after any change
   git add .
   git commit -m "checkpoint"
   git push origin current-branch
   ```

---

## Phase 3: Automated Investigation Engine

### **Failure Investigation Pipeline**:

```python
# scripts/investigate_failure.py
import json
import sys
from typing import List, Dict, Optional

class FailureInvestigator:
    """Automatically investigate Actions failures"""
    
    def __init__(self, repo_owner: str, repo_name: str):
        self.repo = f"{repo_owner}/{repo_name}"
        self.patterns = self.load_patterns()
    
    def investigate(self, run_id: str) -> InvestigationResult:
        """
        1. Download logs
        2. Parse failure
        3. Match patterns
        4. Research solution
        5. Generate fix
        """
        
        # Step 1: Get logs
        logs = self.download_logs(run_id)
        
        # Step 2: Parse
        failure = self.parse_failure(logs)
        
        # Step 3: Classify
        category = self.classify(failure)
        
        # Step 4: Research
        solution = self.research_solution(category, failure)
        
        # Step 5: Generate fix
        fix = self.generate_fix(solution)
        
        return InvestigationResult(
            failure=failure,
            category=category,
            solution=solution,
            fix=fix,
            confidence=solution.confidence
        )
    
    def classify(self, failure: Failure) -> FailureCategory:
        """Classify failure type"""
        patterns = {
            "race_condition": [
                "would_breach",
                "atomicity",
                "concurrent",
                "Ordering::Relaxed"
            ],
            "hardcoded_secret": [
                "POSTGRES_PASSWORD",
                "password:",
                "credentials",
                "secret"
            ],
            "compilation_error": [
                "error[E",
                "mismatched types",
                "unresolved import"
            ],
            "test_failure": [
                "assertion failed",
                "test result: FAILED",
                "panicked"
            ],
            "lint_error": [
                "clippy::",
                "warning:",
                "fmt"
            ]
        }
        
        for category, keywords in patterns.items():
            if any(kw in failure.message for kw in keywords):
                return FailureCategory(category)
        
        return FailureCategory("unknown")
    
    def research_solution(self, category: FailureCategory, 
                         failure: Failure) -> Solution:
        """Research solution based on category"""
        
        solutions = {
            "race_condition": Solution(
                description="Use CAS (compare-and-swap) for atomic check-and-update",
                approach="Replace would_breach + update with atomic operation",
                code_template=self.load_template("race_condition_fix.rs"),
                confidence=0.95
            ),
            "hardcoded_secret": Solution(
                description="Externalize secrets to environment variables",
                approach="Move secrets to .env, use ${VAR} in compose files",
                code_template=self.load_template("secret_externalize.sh"),
                confidence=0.98
            ),
            "compilation_error": Solution(
                description="Fix type mismatch or import error",
                approach="Check compiler message, apply suggested fix",
                code_template=None,  # Requires manual analysis
                confidence=0.70
            ),
            "test_failure": Solution(
                description="Investigate test logic or implementation bug",
                approach="Run test locally, debug, fix root cause",
                code_template=None,
                confidence=0.60
            ),
            "lint_error": Solution(
                description="Apply clippy suggestion or cargo fix",
                approach="Run cargo clippy --fix or cargo fix",
                code_template="cargo fix --all --allow-dirty",
                confidence=0.90
            )
        }
        
        return solutions.get(category.value, Solution(
            description="Unknown failure - requires manual investigation",
            approach="Analyze logs, identify pattern, apply fix",
            code_template=None,
            confidence=0.30
        ))
```

---

## Phase 4: Validation Through Actions

### **The Validation Loop**:

```rust
/// Complete validation cycle
pub async fn validate_through_actions(&self, fix: &Fix) -> ValidationResult {
    // 1. Push fix to GitHub
    let branch = self.push_fix(fix).await?;
    
    // 2. Create PR
    let pr = self.create_pr(&branch, fix).await?;
    
    // 3. Wait for Actions to run
    let mut attempts = 0;
    loop {
        sleep(Duration::from_secs(30)).await;
        
        let status = self.check_pr_status(pr.number()).await?;
        
        match status {
            CheckStatus::Pass => {
                return ValidationResult::Success {
                    pr_number: pr.number(),
                    duration: attempts * 30,
                };
            }
            CheckStatus::Fail(failure) => {
                // Investigate why fix didn't work
                let investigation = self.investigate(failure).await?;
                
                // Generate improved fix
                let improved_fix = self.improve_fix(fix, &investigation).await?;
                
                // Try again
                return self.validate_through_actions(&improved_fix).await;
            }
            CheckStatus::Pending => {
                attempts += 1;
                if attempts > 60 {  // 30 minute timeout
                    return ValidationResult::Timeout;
                }
            }
        }
    }
}
```

---

## Phase 5: Resilience Against Local Loss

### **Checkpoint Strategy**:

```bash
#!/bin/bash
# scripts/auto_checkpoint.sh

# Run every 5 minutes or after significant changes

COMMIT_MSG="checkpoint: $(date '+%Y-%m-%d %H:%M:%S')"

# Check if there are changes
if [ -n "$(git status --porcelain)" ]; then
    echo "Changes detected, creating checkpoint..."
    
    # Add all
    git add -A
    
    # Commit with timestamp
    git commit -m "$COMMIT_MSG" || echo "Nothing to commit"
    
    # Push to remote (GitHub is source of truth)
    git push origin $(git branch --show-current) || echo "Push failed"
    
    echo "Checkpoint created: $COMMIT_MSG"
else
    echo "No changes to checkpoint"
fi
```

### **Recovery From Local Loss**:

```bash
#!/bin/bash
# scripts/recover_from_loss.sh

# If local state is lost, recover from GitHub

echo "Recovering from GitHub (source of truth)..."

# 1. Clone fresh
REPO_URL="https://github.com/stackconsult/traderx.git"
git clone $REPO_URL traderx-recovered
cd traderx-recovered

# 2. Fetch all branches
git fetch --all

# 3. Checkout current work branch
BRANCH="feature/github-mcp-setup"  # or get from user
git checkout $BRANCH

# 4. Verify state
git log --oneline -5
gh run list --limit 5

# 5. Check Actions status
git status

echo "Recovery complete. Local state reconstructed from GitHub."
```

---

## Phase 6: Continuous Monitoring Dashboard

### **GitHub-Based Status Dashboard**:

```yaml
# .github/workflows/status-dashboard.yml
name: Continuous Status Dashboard

on:
  schedule:
    - cron: '*/5 * * * *'  # Every 5 minutes

jobs:
  report:
    runs-on: ubuntu-latest
    steps:
      - name: Generate status report
        uses: actions/github-script@v7
        with:
          script: |
            // Get comprehensive status
            const [repo, workflows, issues, prs] = await Promise.all([
              github.rest.repos.get({ owner, repo }),
              github.rest.actions.listWorkflowRunsForRepo({ owner, repo, per_page: 20 }),
              github.rest.issues.listForRepo({ owner, repo, state: 'open', per_page: 10 }),
              github.rest.pulls.list({ owner, repo, state: 'open', per_page: 10 })
            ]);
            
            const report = {
              timestamp: new Date().toISOString(),
              repo_status: {
                default_branch: repo.data.default_branch,
                open_issues: repo.data.open_issues_count,
                open_prs: prs.data.length
              },
              actions_status: {
                total_runs: workflows.data.total_count,
                successful: workflows.data.workflow_runs.filter(r => r.conclusion === 'success').length,
                failed: workflows.data.workflow_runs.filter(r => r.conclusion === 'failure').length,
                pending: workflows.data.workflow_runs.filter(r => r.status !== 'completed').length
              },
              health_score: calculateHealthScore(workflows.data, issues.data)
            };
            
            // Post as issue comment or to external dashboard
            console.log(JSON.stringify(report, null, 2));

  alert:
    needs: report
    if: needs.report.outputs.health_score < 70
    runs-on: ubuntu-latest
    steps:
      - name: Send alert
        run: |
          echo "Health score below threshold: ${{ needs.report.outputs.health_score }}"
          # Send Slack/email notification
```

---

## Implementation: My Adaptation

### **How I Will Operate Now**:

1. **Start Every Session With**:
   ```bash
   git fetch origin
   git status
   gh run list  # Check Actions health
   ```

2. **Before Any Fix**:
   ```bash
   # Check if Actions are already failing
   gh run view --log-failed
   # Analyze failure pattern
   # Design fix
   ```

3. **During Work**:
   ```bash
   # Every 5 minutes OR after any significant change
   ./scripts/auto_checkpoint.sh
   ```

4. **After Any Fix**:
   ```bash
   # Push immediately
   git push origin fix/branch-name
   
   # Watch Actions validate
   gh run watch
   
   # Only continue if Actions pass
   ```

5. **If Local State Lost**:
   ```bash
   ./scripts/recover_from_loss.sh
   # Continue from GitHub state
   ```

---

## Success Metrics

| Metric | Before | After | Target |
|--------|--------|-------|--------|
| **Time to recover from local loss** | Hours | Minutes | < 5 min |
| **Actions failure detection** | Manual | Automatic | Real-time |
| **Fix validation** | Local only | Through Actions | Always via Actions |
| **State durability** | Local fragile | GitHub durable | 100% GitHub |
| **Auto-healing** | None | Enabled | All common failures |

---

**Adaptation Complete. Operating in GitHub-First Mode.**
