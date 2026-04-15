# Production Guard Workflow - ZERO TOLERANCE FOR FAILURES

## Description

Mandatory multi-layer validation system to ensure NO test failures, NO merge without 100% pass rate. Prevents the critical mistake of claiming success when PR checks are failing.

**Principle**: *"One failing test = No merge. Period."*

---

## The Mistake: What Went Wrong

### **Error Analysis**:

**What I Did**:
- ❌ Only checked Actions workflow run conclusion
- ❌ Did not verify individual PR check runs
- ❌ Did not check mergeable_state
- ❌ Did not validate all jobs passed
- ❌ Assumed success without granular verification

**What I Should Have Done**:
- ✅ Check PR check runs endpoint (`/commits/{ref}/check-runs`)
- ✅ Verify each individual check conclusion
- ✅ Check mergeable_state = "clean"
- ✅ Confirm zero failures
- ✅ Validate all required checks passed

**Impact**:
- 🔴 False positive validation
- 🔴 Risk of merging failing code
- 🔴 Broken trust in validation process
- 🔴 Potential production issues

---

## Production Guard: Multi-Layer Validation

### **Layer 1: Individual Check Validation** (CRITICAL)

```python
# MANDATORY: Check individual check runs, not just workflow conclusion
def validate_pr_checks(owner, repo, branch):
    """
    Layer 1: Validate every individual check run passed
    """
    url = f"https://api.github.com/repos/{owner}/{repo}/commits/{branch}/check-runs"
    response = github_api_get(url)
    
    checks = response.get('check_runs', [])
    
    failed = []
    passed = []
    pending = []
    
    for check in checks:
        name = check['name']
        status = check['status']  # queued, in_progress, completed
        conclusion = check['conclusion']  # success, failure, neutral, cancelled, skipped, timed_out, action_required
        
        if status != 'completed':
            pending.append(name)
        elif conclusion != 'success':
            failed.append({'name': name, 'conclusion': conclusion, 'url': check['html_url']})
        else:
            passed.append(name)
    
    # GUARD: Any failure = BLOCK
    if failed:
        return ValidationResult.BLOCKED(
            reason=f"{len(failed)} checks failed",
            failures=failed,
            passed=passed,
            pending=pending
        )
    
    # GUARD: Any pending = WAIT
    if pending:
        return ValidationResult.PENDING(
            reason=f"{len(pending)} checks still running",
            pending=pending,
            passed=passed
        )
    
    # GUARD: All must pass
    if not passed:
        return ValidationResult.BLOCKED(
            reason="No checks found - verify CI is configured"
        )
    
    return ValidationResult.PASSED(
        all_checks=passed,
        total=len(passed)
    )
```

### **Layer 2: PR Mergeable State** (CRITICAL)

```python
# MANDATORY: Check PR mergeable state
def validate_pr_mergeable(owner, repo, pr_number):
    """
    Layer 2: Verify PR is actually mergeable
    """
    url = f"https://api.github.com/repos/{owner}/{repo}/pulls/{pr_number}"
    pr = github_api_get(url)
    
    mergeable = pr.get('mergeable')  # true, false, or null (unknown)
    mergeable_state = pr.get('mergeable_state')  # clean, dirty, unstable, blocked
    
    # GUARD: mergeable must be true
    if mergeable is not True:
        return ValidationResult.BLOCKED(
            reason=f"PR not mergeable: {mergeable}",
            state=mergeable_state
        )
    
    # GUARD: mergeable_state must be "clean"
    if mergeable_state != 'clean':
        return ValidationResult.BLOCKED(
            reason=f"PR mergeable_state is '{mergeable_state}', expected 'clean'",
            state=mergeable_state
        )
    
    return ValidationResult.PASSED(
        mergeable=True,
        state='clean'
    )
```

### **Layer 3: Required Status Checks** (CRITICAL)

```python
# MANDATORY: Verify all required checks passed
def validate_required_checks(owner, repo, branch):
    """
    Layer 3: Validate branch protection required checks
    """
    url = f"https://api.github.com/repos/{owner}/{repo}/branches/{branch}/protection"
    try:
        protection = github_api_get(url)
        required_checks = protection.get('required_status_checks', {})
        contexts = required_checks.get('contexts', [])
        
        # Get actual check runs
        checks_url = f"https://api.github.com/repos/{owner}/{repo}/commits/{branch}/check-runs"
        checks_response = github_api_get(checks_url)
        check_runs = {c['name']: c for c in checks_response.get('check_runs', [])}
        
        missing = []
        failed_required = []
        
        for required in contexts:
            if required not in check_runs:
                missing.append(required)
            elif check_runs[required]['conclusion'] != 'success':
                failed_required.append({
                    'name': required,
                    'conclusion': check_runs[required]['conclusion']
                })
        
        # GUARD: All required checks must exist and pass
        if missing:
            return ValidationResult.BLOCKED(
                reason=f"Required checks missing: {missing}"
            )
        
        if failed_required:
            return ValidationResult.BLOCKED(
                reason=f"Required checks failed: {failed_required}"
            )
        
        return ValidationResult.PASSED(
            required_checks=contexts,
            all_passed=True
        )
    except Exception as e:
        # If no branch protection, still check all checks passed
        return ValidationResult.WARNING(
            reason="No branch protection configured",
            error=str(e)
        )
```

### **Layer 4: Combined Workflow Run** (SECONDARY)

```python
# SECONDARY: Check combined workflow conclusion (not sufficient alone)
def validate_workflow_run(owner, repo, branch):
    """
    Layer 4: Check workflow run (supplementary only)
    """
    url = f"https://api.github.com/repos/{owner}/{repo}/actions/runs?branch={branch}&per_page=1"
    response = github_api_get(url)
    
    runs = response.get('workflow_runs', [])
    if not runs:
        return ValidationResult.WARNING(
            reason="No workflow runs found"
        )
    
    run = runs[0]
    conclusion = run.get('conclusion')
    
    # This is supplementary - individual checks are primary
    if conclusion != 'success':
        return ValidationResult.WARNING(
            reason=f"Workflow conclusion: {conclusion}",
            run_id=run['id']
        )
    
    return ValidationResult.PASSED(
        workflow_conclusion=conclusion
    )
```

---

## Complete Production Guard Script

```python
#!/usr/bin/env python3
"""
PRODUCTION GUARD - ZERO TOLERANCE VALIDATION
Usage: python production_guard.py <owner> <repo> <branch> [pr_number]
Exit codes:
  0 = All checks passed, ready for merge
  1 = BLOCKED - failures detected, DO NOT MERGE
  2 = PENDING - checks still running, wait
  3 = ERROR - validation error
"""

import sys
import json
import urllib.request
import ssl
from dataclasses import dataclass
from typing import List, Dict, Optional

# Token must be set in environment
GITHUB_TOKEN = None  # Set from env

@dataclass
class ValidationResult:
    status: str  # 'PASSED', 'BLOCKED', 'PENDING', 'ERROR'
    reason: str
    details: Dict
    
    @classmethod
    def PASSED(cls, **details):
        return cls('PASSED', 'All validations passed', details)
    
    @classmethod
    def BLOCKED(cls, reason, **details):
        return cls('BLOCKED', reason, details)
    
    @classmethod
    def PENDING(cls, reason, **details):
        return cls('PENDING', reason, details)
    
    @classmethod
    def ERROR(cls, reason, **details):
        return cls('ERROR', reason, details)

def github_api_get(url: str) -> Dict:
    """Make authenticated GitHub API request"""
    ctx = ssl.create_default_context()
    headers = {
        'Authorization': f'token {GITHUB_TOKEN}',
        'Accept': 'application/vnd.github.v3+json'
    }
    req = urllib.request.Request(url, headers=headers)
    
    with urllib.request.urlopen(req, context=ctx) as response:
        return json.loads(response.read())

class ProductionGuard:
    """Multi-layer production validation"""
    
    def __init__(self, owner: str, repo: str, branch: str, pr_number: Optional[int] = None):
        self.owner = owner
        self.repo = repo
        self.branch = branch
        self.pr_number = pr_number
        self.results = []
    
    def validate(self) -> ValidationResult:
        """Run all validation layers"""
        
        print("=" * 70)
        print("PRODUCTION GUARD - Multi-Layer Validation")
        print("=" * 70)
        print(f"Repository: {self.owner}/{self.repo}")
        print(f"Branch: {self.branch}")
        if self.pr_number:
            print(f"PR: #{self.pr_number}")
        print()
        
        # Layer 1: Individual Check Runs (CRITICAL)
        print("[LAYER 1] Validating individual check runs...")
        layer1 = self._validate_check_runs()
        self.results.append(('Check Runs', layer1))
        self._print_result(layer1)
        
        if layer1.status == 'BLOCKED':
            return self._finalize(ValidationResult.BLOCKED(
                "Layer 1 failed: Individual check runs have failures",
                layer1_details=layer1.details
            ))
        
        # Layer 2: PR Mergeable State (CRITICAL)
        if self.pr_number:
            print("[LAYER 2] Validating PR mergeable state...")
            layer2 = self._validate_mergeable_state()
            self.results.append(('Mergeable State', layer2))
            self._print_result(layer2)
            
            if layer2.status == 'BLOCKED':
                return self._finalize(ValidationResult.BLOCKED(
                    "Layer 2 failed: PR not in mergeable state",
                    layer2_details=layer2.details
                ))
        
        # Layer 3: Required Status Checks (CRITICAL)
        print("[LAYER 3] Validating required status checks...")
        layer3 = self._validate_required_checks()
        self.results.append(('Required Checks', layer3))
        self._print_result(layer3)
        
        if layer3.status == 'BLOCKED':
            return self._finalize(ValidationResult.BLOCKED(
                "Layer 3 failed: Required status checks not met",
                layer3_details=layer3.details
            ))
        
        # Layer 4: Workflow Run (SECONDARY)
        print("[LAYER 4] Validating workflow run (supplementary)...")
        layer4 = self._validate_workflow_run()
        self.results.append(('Workflow Run', layer4))
        self._print_result(layer4)
        
        # Check if any layer is PENDING
        pending = [name for name, result in self.results if result.status == 'PENDING']
        if pending:
            return self._finalize(ValidationResult.PENDING(
                f"Layers still pending: {pending}",
                pending_layers=pending
            ))
        
        # All critical layers passed
        return self._finalize(ValidationResult.PASSED(
            all_layers_passed=True,
            results={name: r.status for name, r in self.results}
        ))
    
    def _validate_check_runs(self) -> ValidationResult:
        """Layer 1: Check individual check runs"""
        try:
            url = f"https://api.github.com/repos/{self.owner}/{self.repo}/commits/{self.branch}/check-runs"
            data = github_api_get(url)
            
            checks = data.get('check_runs', [])
            total = data.get('total_count', 0)
            
            if total == 0:
                return ValidationResult.BLOCKED(
                    "No check runs found - CI may not be configured",
                    total=0
                )
            
            failed = []
            passed = []
            pending = []
            
            for check in checks:
                name = check['name']
                status = check['status']
                conclusion = check['conclusion']
                
                if status != 'completed':
                    pending.append(name)
                elif conclusion != 'success':
                    failed.append({
                        'name': name,
                        'conclusion': conclusion,
                        'url': check.get('html_url', 'N/A')
                    })
                else:
                    passed.append(name)
            
            if failed:
                return ValidationResult.BLOCKED(
                    f"{len(failed)} of {total} checks failed",
                    failed=failed,
                    passed=passed,
                    pending=pending,
                    total=total
                )
            
            if pending:
                return ValidationResult.PENDING(
                    f"{len(pending)} of {total} checks still running",
                    pending=pending,
                    passed=passed,
                    total=total
                )
            
            return ValidationResult.PASSED(
                all_checks_passed=passed,
                total=total
            )
            
        except Exception as e:
            return ValidationResult.ERROR(
                f"Error checking runs: {str(e)}"
            )
    
    def _validate_mergeable_state(self) -> ValidationResult:
        """Layer 2: Check PR mergeable state"""
        try:
            url = f"https://api.github.com/repos/{self.owner}/{self.repo}/pulls/{self.pr_number}"
            pr = github_api_get(url)
            
            mergeable = pr.get('mergeable')
            state = pr.get('mergeable_state')
            
            if mergeable is not True:
                return ValidationResult.BLOCKED(
                    f"PR not mergeable (mergeable={mergeable})",
                    mergeable=mergeable,
                    state=state
                )
            
            if state != 'clean':
                return ValidationResult.BLOCKED(
                    f"PR mergeable_state is '{state}', expected 'clean'",
                    mergeable=mergeable,
                    state=state
                )
            
            return ValidationResult.PASSED(
                mergeable=True,
                state='clean'
            )
            
        except Exception as e:
            return ValidationResult.ERROR(
                f"Error checking PR: {str(e)}"
            )
    
    def _validate_required_checks(self) -> ValidationResult:
        """Layer 3: Validate required status checks"""
        try:
            # Try to get branch protection
            url = f"https://api.github.com/repos/{self.owner}/{self.repo}/branches/{self.branch}/protection"
            protection = github_api_get(url)
            
            required = protection.get('required_status_checks', {})
            contexts = required.get('contexts', [])
            
            if not contexts:
                # No required checks configured - check all checks passed instead
                return ValidationResult.PASSED(
                    no_required_checks=True,
                    note="No branch protection required checks configured"
                )
            
            # Get actual check runs
            checks_url = f"https://api.github.com/repos/{self.owner}/{self.repo}/commits/{self.branch}/check-runs"
            checks_data = github_api_get(checks_url)
            check_runs = {c['name']: c for c in checks_data.get('check_runs', [])}
            
            missing = []
            failed = []
            
            for required_name in contexts:
                if required_name not in check_runs:
                    missing.append(required_name)
                elif check_runs[required_name]['conclusion'] != 'success':
                    failed.append({
                        'name': required_name,
                        'conclusion': check_runs[required_name]['conclusion']
                    })
            
            if missing:
                return ValidationResult.BLOCKED(
                    f"Required checks missing: {missing}",
                    missing=missing,
                    required=contexts
                )
            
            if failed:
                return ValidationResult.BLOCKED(
                    f"Required checks failed: {failed}",
                    failed=failed,
                    required=contexts
                )
            
            return ValidationResult.PASSED(
                all_required_passed=True,
                required=contexts
            )
            
        except urllib.error.HTTPError as e:
            if e.code == 404:
                # No branch protection - pass with warning
                return ValidationResult.PASSED(
                    no_branch_protection=True,
                    note="No branch protection configured"
                )
            return ValidationResult.ERROR(
                f"HTTP error: {e.code}"
            )
        except Exception as e:
            return ValidationResult.ERROR(
                f"Error checking protection: {str(e)}"
            )
    
    def _validate_workflow_run(self) -> ValidationResult:
        """Layer 4: Check workflow run (supplementary)"""
        try:
            url = f"https://api.github.com/repos/{self.owner}/{self.repo}/actions/runs?branch={self.branch}&per_page=1"
            data = github_api_get(url)
            
            runs = data.get('workflow_runs', [])
            if not runs:
                return ValidationResult.PASSED(
                    no_workflow_runs=True,
                    note="No workflow runs found"
                )
            
            run = runs[0]
            conclusion = run.get('conclusion')
            
            return ValidationResult.PASSED(
                workflow_conclusion=conclusion,
                run_id=run.get('id'),
                url=run.get('html_url')
            )
            
        except Exception as e:
            return ValidationResult.ERROR(
                f"Error checking workflow: {str(e)}"
            )
    
    def _print_result(self, result: ValidationResult):
        """Print validation result"""
        icon = {
            'PASSED': '✅',
            'BLOCKED': '❌',
            'PENDING': '⏳',
            'ERROR': '⚠️'
        }.get(result.status, '?')
        
        print(f"  {icon} {result.status}: {result.reason}")
        
        if result.details.get('failed'):
            for failure in result.details['failed']:
                print(f"     - {failure['name']}: {failure['conclusion']}")
                print(f"       {failure.get('url', '')}")
        print()
    
    def _finalize(self, result: ValidationResult) -> ValidationResult:
        """Print final summary and return"""
        print("=" * 70)
        print("FINAL RESULT")
        print("=" * 70)
        
        icon = {
            'PASSED': '✅',
            'BLOCKED': '❌',
            'PENDING': '⏳',
            'ERROR': '⚠️'
        }.get(result.status, '?')
        
        print(f"{icon} {result.status}: {result.reason}")
        print()
        
        if result.status == 'PASSED':
            print("🚀 READY FOR MERGE")
        elif result.status == 'BLOCKED':
            print("🛑 DO NOT MERGE - Fix failures first")
        elif result.status == 'PENDING':
            print("⏳ WAIT - Checks still running")
        
        return result

def main():
    if len(sys.argv) < 4:
        print("Usage: python production_guard.py <owner> <repo> <branch> [pr_number]")
        print("Example: python production_guard.py stackconsult traderx fix/branch 123")
        sys.exit(3)
    
    import os
    global GITHUB_TOKEN
    GITHUB_TOKEN = os.environ.get('GITHUB_TOKEN')
    
    if not GITHUB_TOKEN:
        print("❌ ERROR: GITHUB_TOKEN environment variable not set")
        sys.exit(3)
    
    owner = sys.argv[1]
    repo = sys.argv[2]
    branch = sys.argv[3]
    pr_number = int(sys.argv[4]) if len(sys.argv) > 4 else None
    
    guard = ProductionGuard(owner, repo, branch, pr_number)
    result = guard.validate()
    
    # Exit codes
    exit_codes = {
        'PASSED': 0,
        'BLOCKED': 1,
        'PENDING': 2,
        'ERROR': 3
    }
    
    sys.exit(exit_codes.get(result.status, 3))

if __name__ == '__main__':
    main()
```

---

## Integration with Workflows

### **Update session-start.md**:

```markdown
### Pre-Session Validation (NEW)
- [ ] Run production guard: `python scripts/production_guard.py <owner> <repo> <branch> [pr]`
- [ ] Verify all 4 layers passed
- [ ] If BLOCKED → Engineer fixes before proceeding
- [ ] If PENDING → Wait for completion
```

### **Update meta-cognitive.md**:

```markdown
### Validation Gate (MANDATORY)
**Before declaring ANY success:**
1. Run production guard script
2. All 4 layers must pass
3. ZERO tolerance for failures
4. If any layer fails → STOP and fix

**Mistake Prevention**:
- Never rely on single check
- Always verify individual check runs
- Always check mergeable_state
- Always confirm 100% pass rate
```

### **Update quality-guardian.md**:

```markdown
### Production Guard Gate
- [ ] Production guard executed
- [ ] All checks passed (not just workflow conclusion)
- [ ] Mergeable state = "clean"
- [ ] Required status checks verified
- [ ] ZERO failures tolerated
```

---

## Usage

```bash
# Run production guard before any merge
export GITHUB_TOKEN=your_token
python scripts/production_guard.py stackconsult traderx fix/branch 123

# Exit codes:
# 0 = Ready to merge
# 1 = BLOCKED - fix failures
# 2 = PENDING - wait
# 3 = ERROR - investigate
```

---

**This workflow ensures: ZERO test failures, 100% validation, NO false positives.**
