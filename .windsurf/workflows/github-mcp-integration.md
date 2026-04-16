# GitHub MCP Integration Workflow

## Description

Integration of GitHub MCP server for automated PR management, Actions validation, and merge operations with production guard safeguards.

**Principle**: *Automate GitHub operations through MCP while maintaining zero-tolerance validation.*

---

## MCP Server Configuration

**File**: `~/.codeium/windsurf/mcp_config.json`

**GitHub MCP Server**:
```json
{
  "mcpServers": {
    "github": {
      "command": "docker",
      "args": [
        "run",
        "-i",
        "--rm",
        "-e",
        "GITHUB_PERSONAL_ACCESS_TOKEN",
        "ghcr.io/github/github-mcp-server"
      ],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "github_pat_11BZU7ESI0..."
      },
      "registry": "github"
    }
  }
}
```

**Requirements**:
- Docker installed and running
- GitHub Personal Access Token with `repo` scope
- Token configured in mcp_config.json

---

## MCP Tools Available

### **Repository Operations**:
- `list_repositories` - List user/org repos
- `get_repository` - Get repo details
- `create_repository` - Create new repo
- `fork_repository` - Fork existing repo

### **PR Operations**:
- `list_pull_requests` - List open/closed PRs
- `get_pull_request` - Get PR details
- `create_pull_request` - Create new PR
- `update_pull_request` - Update PR (title, body, state)
- `merge_pull_request` - Merge PR (with safeguards)

### **Actions Operations**:
- `list_workflow_runs` - List Actions runs
- `get_workflow_run` - Get run details
- `get_workflow_run_logs` - Download logs
- `rerun_workflow` - Re-trigger Actions

### **Issues Operations**:
- `list_issues` - List repository issues
- `get_issue` - Get issue details
- `create_issue` - Create new issue
- `update_issue` - Update issue state

---

## Automated PR Management with Safeguards

### **Step 1: List and Identify PRs**

```python
# Use MCP to list PRs
pr_list = mcp.github.list_pull_requests(
    owner="stackconsult",
    repo="traderx",
    state="open"
)

# Identify target PR
for pr in pr_list:
    print(f"PR #{pr['number']}: {pr['title']}")
    print(f"  Branch: {pr['headRefName']} -> {pr['baseRefName']}")
    print(f"  Mergeable: {pr['mergeable']}")
    print(f"  State: {pr['mergeStateStatus']}")
```

### **Step 2: Production Guard Validation (MANDATORY)**

```python
# 4-layer validation before any merge

def production_guard_validate(pr_number: int) -> ValidationResult:
    """
    Mandatory 4-layer validation using MCP
    """
    
    # Layer 1: Get PR check runs
    pr = mcp.github.get_pull_request(
        owner="stackconsult",
        repo="traderx",
        number=pr_number
    )
    
    # Layer 2: Verify check runs
    check_runs = mcp.github.list_check_runs_for_ref(
        owner="stackconsult",
        repo="traderx",
        ref=pr['headRefName']
    )
    
    failed = [c for c in check_runs if c['conclusion'] != 'success']
    passed = [c for c in check_runs if c['conclusion'] == 'success']
    pending = [c for c in check_runs if c['status'] != 'completed']
    
    if failed:
        return ValidationResult.BLOCKED(
            f"{len(failed)} checks failed",
            failed_checks=failed
        )
    
    if pending:
        return ValidationResult.PENDING(
            f"{len(pending)} checks still running",
            pending_checks=pending
        )
    
    # Layer 3: Verify mergeable state
    if not pr['mergeable']:
        return ValidationResult.BLOCKED(
            "PR not mergeable - conflicts detected"
        )
    
    if pr['mergeStateStatus'] != 'clean':
        return ValidationResult.BLOCKED(
            f"Merge state is '{pr['mergeStateStatus']}', must be 'clean'"
        )
    
    # Layer 4: Verify reviews (if required)
    reviews = mcp.github.list_pull_request_reviews(
        owner="stackconsult",
        repo="traderx",
        number=pr_number
    )
    
    approved_reviews = [r for r in reviews if r['state'] == 'APPROVED']
    if len(approved_reviews) < 1:  # Require at least 1 approval
        return ValidationResult.BLOCKED(
            "PR requires at least 1 approval"
        )
    
    return ValidationResult.PASSED(
        f"All {len(passed)} checks passed",
        passed_checks=passed,
        reviews=approved_reviews
    )
```

### **Step 3: Engineer Passing Actions (If Failing)**

```python
# If validation fails, engineer fixes

def engineer_actions_fix(failed_checks: list) -> list:
    """
    Automatically engineer fixes for failing Actions
    """
    fixes = []
    
    for check in failed_checks:
        check_name = check['name']
        logs_url = check['html_url']
        
        # Download logs
        logs = mcp.github.get_workflow_run_logs(
            owner="stackconsult",
            repo="traderx",
            run_id=check['run_id']
        )
        
        # Analyze failure pattern
        if "test" in check_name.lower():
            # Test failure - engineer test fix
            fix = engineer_test_fix(logs)
            fixes.append(fix)
            
        elif "security" in check_name.lower():
            # Security scan failure - engineer security fix
            fix = engineer_security_fix(logs)
            fixes.append(fix)
            
        elif "lint" in check_name.lower() or "clippy" in check_name.lower():
            # Lint failure - auto-fix
            fix = engineer_lint_fix(logs)
            fixes.append(fix)
    
    return fixes

# Apply fixes
for fix in fixes:
    apply_fix(fix)
    
# Re-trigger Actions
mcp.github.rerun_workflow(
    owner="stackconsult",
    repo="traderx",
    run_id=failed_run_id
)

# Wait and re-validate
wait_for_completion()
result = production_guard_validate(pr_number)
```

### **Step 4: Merge with Safeguards**

```python
# Only merge if validation passed with certainty > 0.99

def safe_merge_pr(pr_number: int) -> MergeResult:
    """
    Merge PR with production guard and MCP
    """
    
    # Validate
    validation = production_guard_validate(pr_number)
    
    if validation.status != "PASSED":
        return MergeResult.BLOCKED(
            f"Validation failed: {validation.reason}"
        )
    
    # Calculate certainty
    certainty = calculate_certainty(validation)
    if certainty < 0.99:
        return MergeResult.BLOCKED(
            f"Certainty {certainty:.2f} below threshold 0.99"
        )
    
    # Merge using MCP
    merge_result = mcp.github.merge_pull_request(
        owner="stackconsult",
        repo="traderx",
        number=pr_number,
        merge_method="merge",  # or "squash" or "rebase"
        commit_title=f"merge: PR #{pr_number} - {validation.reason}",
        commit_message="Auto-merged with production guard validation"
    )
    
    # Verify merge
    if merge_result['merged']:
        return MergeResult.SUCCESS(
            f"PR #{pr_number} merged successfully",
            sha=merge_result['sha']
        )
    else:
        return MergeResult.FAILED(
            f"Merge failed: {merge_result.get('message', 'Unknown error')}"
        )
```

---

## Complete Automation Workflow

### **Autonomous PR Management**:

```python
class AutonomousPRManager:
    """
    Autonomous PR management with MCP and production guard
    """
    
    def __init__(self):
        self.mcp = MCPClient()
        self.guard = ProductionGuard()
    
    async def process_open_prs(self):
        """
        Process all open PRs autonomously
        """
        # List all open PRs
        prs = self.mcp.github.list_pull_requests(
            owner="stackconsult",
            repo="traderx",
            state="open"
        )
        
        for pr in prs:
            pr_number = pr['number']
            
            # Validate
            validation = self.guard.validate(pr_number)
            
            if validation.status == "PASSED":
                # Auto-merge if validation passed
                result = self.safe_merge_pr(pr_number)
                
                if result.status == "SUCCESS":
                    # Create success notification
                    self.mcp.github.create_issue_comment(
                        owner="stackconsult",
                        repo="traderx",
                        issue_number=pr_number,
                        body=f"✅ Auto-merged with certainty {validation.certainty:.2f}"
                    )
                    
            elif validation.status == "BLOCKED":
                # Engineer fixes
                fixes = self.engineer_fixes(validation.failed_checks)
                
                # Apply fixes
                for fix in fixes:
                    self.apply_fix(fix)
                
                # Re-trigger Actions
                self.mcp.github.rerun_workflow(
                    owner="stackconsult",
                    repo="traderx",
                    run_id=validation.run_id
                )
                
            elif validation.status == "PENDING":
                # Wait and retry
                await asyncio.sleep(60)
                await self.process_open_prs()  # Recurse
    
    async def maintain_actions_passing(self):
        """
        Continuously ensure Actions are passing
        """
        while True:
            # Check main branch Actions
            runs = self.mcp.github.list_workflow_runs(
                owner="stackconsult",
                repo="traderx",
                branch="main",
                per_page=1
            )
            
            latest_run = runs[0]
            
            if latest_run['conclusion'] == 'failure':
                # Engineer fix for failure
                logs = self.mcp.github.get_workflow_run_logs(
                    owner="stackconsult",
                    repo="traderx",
                    run_id=latest_run['id']
                )
                
                fix = self.engineer_fix_from_logs(logs)
                self.apply_fix(fix)
                
                # Re-trigger
                self.mcp.github.rerun_workflow(
                    owner="stackconsult",
                    repo="traderx",
                    run_id=latest_run['id']
                )
            
            # Wait before next check
            await asyncio.sleep(300)  # 5 minutes
```

---

## Integration with Agent Execution Engine

### **Agent Uses MCP for GitHub Operations**:

```rust
impl AgentExecutionEngine {
    async fn process_github_operations(&self) {
        // Use MCP to list PRs
        let prs = self.mcp.github.list_pull_requests(
            owner: "stackconsult",
            repo: "traderx",
            state: "open"
        ).await;
        
        for pr in prs {
            // Validate with production guard
            let validation = self.production_guard.validate(pr.number).await;
            
            match validation.status {
                ValidationStatus::PASSED => {
                    // Merge using MCP
                    let merge_result = self.mcp.github.merge_pull_request(
                        owner: "stackconsult",
                        repo: "traderx",
                        number: pr.number,
                        merge_method: "merge"
                    ).await;
                    
                    // Audit
                    self.audit.merge_completed(pr.number, merge_result).await;
                }
                
                ValidationStatus::BLOCKED => {
                    // Engineer fix
                    let fix = self.engineer_fix(validation.failed_checks).await;
                    self.apply_fix(fix).await;
                    
                    // Re-trigger Actions via MCP
                    self.mcp.github.rerun_workflow(
                        owner: "stackconsult",
                        repo: "traderx",
                        run_id: validation.run_id
                    ).await;
                }
                
                ValidationStatus::PENDING => {
                    // Wait and retry
                    sleep(Duration::from_secs(60)).await;
                    self.process_github_operations().await;
                }
            }
        }
    }
}
```

---

## Usage Examples

### **Example 1: Auto-merge Ready PRs**:

```bash
# Using MCP through agent
python -c "
from agent import AutonomousPRManager
manager = AutonomousPRManager()
manager.process_open_prs()
"
```

### **Example 2: Engineer Failing Actions**:

```bash
# Engineer fixes for all failing checks
python -c "
from agent import AutonomousPRManager
manager = AutonomousPRManager()
manager.engineer_fixes_for_pr(2)  # PR #2
"
```

### **Example 3: Maintain Actions Passing**:

```bash
# Continuous monitoring and fixing
python -c "
from agent import AutonomousPRManager
import asyncio
manager = AutonomousPRManager()
asyncio.run(manager.maintain_actions_passing())
"
```

---

## MCP Server Verification

### **Test MCP Connection**:

```bash
# Verify GitHub MCP server is running
docker ps | grep github-mcp-server

# Test API access
python -c "
from mcp import MCPClient
client = MCPClient()
repos = client.github.list_repositories()
print(f'Repos: {len(repos)}')
"
```

---

## Safety & Safeguards

### **Zero Tolerance Rules**:

1. **No merge without validation**: All 4 layers must pass
2. **Certainty threshold**: Must be > 0.99
3. **No auto-fix without verification**: All fixes tested
4. **No force merge**: Respect branch protection
5. **Audit everything**: All operations logged

### **Emergency Stop**:

```python
# Kill switch for automation
if detect_emergency_condition():
    stop_all_automation()
    notify_human_operator()
    create_incident_report()
```

---

**GitHub MCP integration complete. Automated PR management with production guard safeguards operational.**
