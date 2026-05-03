# Production Guard Skill (Auto-generated from validation mistake)

## Description
Multi-layer validation system to prevent false positive validation claims. Zero tolerance for failures across all validation layers.

## Trigger
When claiming validation success for any operation (git push, merge, deployment, etc.)

## Action

### Layer 1: Individual Checks
1. Query `/commits/{ref}/check-runs` endpoint (or equivalent)
2. Verify EACH check conclusion = "success"
3. Count total checks vs passed checks
4. Fail if any single check is not "success"

### Layer 2: Mergeable State
1. Verify `mergeable_state` = "clean"
2. If "dirty", identify blocking files
3. Do not claim success until clean

### Layer 3: Required Checks
1. Identify required checks from branch protection rules
2. Verify all required checks are present and passing
3. Do not skip required checks even if optional ones pass

### Layer 4: Workflow Conclusion
1. Verify the overall workflow status = "success"
2. Check for any skipped or cancelled jobs
3. Ensure complete execution, not partial success

## Verification Checklist

- [ ] Layer 1: Individual checks ✅
- [ ] Layer 2: Mergeable state ✅
- [ ] Layer 3: Required checks ✅
- [ ] Layer 4: Workflow conclusion ✅

## Error Handling

If any layer fails:
- Report which layer failed
- Report specific check/state that failed
- Do not claim validation success
- Provide remediation steps

## Example Usage

```rust
pub async fn validate_production_state(commit_ref: &str) -> Result<ValidationResult> {
    // Layer 1: Check individual checks
    let checks = github_api.get_check_runs(commit_ref).await?;
    for check in checks {
        if check.conclusion != Some("success".to_string()) {
            return Err(ValidationError::CheckFailed {
                check_name: check.name,
                conclusion: check.conclusion,
            });
        }
    }

    // Layer 2: Check mergeable state
    let pr = github_api.get_pull_request(commit_ref).await?;
    if pr.mergeable_state != "clean" {
        return Err(ValidationError::NotMergeable {
            state: pr.mergeable_state,
        });
    }

    // Layer 3: Verify required checks
    let required_checks = github_api.get_required_checks(commit_ref).await?;
    for required in required_checks {
        if !checks.iter().any(|c| c.name == required.name && c.conclusion == Some("success".to_string())) {
            return Err(ValidationError::RequiredCheckMissing {
                check_name: required.name,
            });
        }
    }

    // Layer 4: Verify workflow conclusion
    let workflow = github_api.get_workflow_run(commit_ref).await?;
    if workflow.conclusion != Some("success".to_string()) {
        return Err(ValidationError::WorkflowFailed {
            conclusion: workflow.conclusion,
        });
    }

    Ok(ValidationResult::Valid)
}
```

## Success Criteria

- All 4 layers pass
- Zero tolerance for partial success
- No false positive validation claims
- Clear error reporting for failures

## Origin

Auto-generated from autonomous-upskilling workflow after validation mistake where success was claimed without verifying all layers.
