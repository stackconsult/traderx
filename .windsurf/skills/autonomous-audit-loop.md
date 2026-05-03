# Autonomous Audit Loop Skill (Auto-generated from execution request)

## Description
Continuous validation and auto-execution based on certainty. Monitors execution results, identifies issues, and automatically triggers remediation when certainty is high enough.

## Trigger
Any execution that requires continuous validation (builds, tests, deployments, audits)

## Action

### Phase 1: Initial Execution
1. Execute the requested operation
2. Capture all output and metrics
3. Record execution time
4. Store result for analysis

### Phase 2: Continuous Validation
1. Monitor execution in real-time
2. Check intermediate results
3. Validate against expected outcomes
4. Detect anomalies early

### Phase 3: Result Analysis
1. Compare result against expected outcome
2. Identify any deviations
3. Calculate confidence in result
4. Determine if remediation needed

### Phase 4: Auto-Execution Based on Certainty
If certainty >= 0.99:
- Auto-execute remediation
- Apply fixes automatically
- Verify fix effectiveness
- Continue to next phase

If certainty < 0.99:
- Pause for human review
- Present findings
- Await decision
- Do not auto-execute

### Phase 5: Verification
1. Re-run validation after remediation
2. Verify fix effectiveness
3. Check for side effects
4. Confirm resolution

## Verification Checklist

- [ ] Execution completed ✅
- [ ] Results captured ✅
- [ ] Validation performed ✅
- [ ] Certainty calculated ✅
- [ ] Remediation executed (if high certainty) ✅
- [ ] Verification completed ✅

## Example Usage

```rust
pub struct AutonomousAuditLoop {
    certaint_threshold: f64,
    remediation_enabled: bool,
}

impl AutonomousAuditLoop {
    pub async fn execute_with_audit<T>(
        &self,
        operation: impl Fn() -> Result<T>,
        validate: impl Fn(&T) -> ValidationResult,
        remediate: impl Fn(&T) -> Result<T>,
    ) -> Result<AuditResult<T>> {
        // Phase 1: Initial execution
        let result = operation()?;
        let execution_time = Instant::now().elapsed();

        // Phase 2: Continuous validation
        let validation = validate(&result);

        // Phase 3: Result analysis
        let certainty = self.calculate_certainty(&validation, &result);

        // Phase 4: Auto-execution based on certainty
        if certainty >= self.certainty_threshold && self.remediation_enabled {
            let remediated = remediate(&result)?;
            
            // Phase 5: Verification
            let revalidation = validate(&remediated);
            if revalidation.is_valid() {
                Ok(AuditResult::Success {
                    result: remediated,
                    execution_time,
                    certainty,
                    auto_remediated: true,
                })
            } else {
                Ok(AuditResult::RemediationFailed {
                    result,
                    execution_time,
                    certainty,
                    failure_reason: revalidation.error,
                })
            }
        } else if certainty < self.certainty_threshold {
            Ok(AuditResult::LowCertainty {
                result,
                execution_time,
                certainty,
                requires_human_review: true,
            })
        } else {
            Ok(AuditResult::Success {
                result,
                execution_time,
                certainty,
                auto_remediated: false,
            })
        }
    }

    fn calculate_certainty(&self, validation: &ValidationResult, result: &T) -> f64 {
        // Calculate certainty based on validation result and context
        // Higher certainty if validation passes with strong evidence
        // Lower certainty if validation is weak or ambiguous
        match validation {
            ValidationResult::Valid => 0.99,
            ValidationResult::Weak => 0.85,
            ValidationResult::Invalid => 0.30,
        }
    }
}
```

## Success Criteria

- Execution completed successfully
- Validation performed continuously
- Certainty threshold met for auto-remediation
- Remediation effective when executed
- Verification confirms resolution

## Anti-Patterns

❌ **DO NOT** auto-execute with low certainty
❌ **DO NOT** skip validation phases
❌ **DO NOT** ignore low certainty warnings
❌ **DO NOT** proceed without verification
❌ **DO NOT** auto-execute without remediation capability

## Human Review Triggers

Auto-execution is paused and human review is required when:
- Certainty < 0.99
- Validation is ambiguous
- Remediation capability is uncertain
- Side effects are unknown
- Risk is high (security, production, data loss)

## Configuration

```toml
[audit_loop]
certainty_threshold = 0.99
remediation_enabled = true
max_auto_remediation_attempts = 3
human_review_on_low_certainty = true
```

## Origin

Auto-generated from autonomous-upskilling workflow after request for autonomous audit loop with continuous validation and auto-execution based on certainty.
