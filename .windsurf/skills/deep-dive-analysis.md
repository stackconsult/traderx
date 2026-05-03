# Deep-Dive Analysis Skill (Auto-generated from uncertainty gap)

## Description
Systematic deep-dive analysis when certainty < 0.80 on critical decisions. Gathers comprehensive data, queries multiple sources, and only proceeds when certainty > 0.99.

## Trigger
Certainty < 0.80 on any critical decision (architectural, security, deployment, etc.)

## Action

### Step 1: Acknowledge Uncertainty
1. Log current certainty level
2. Identify the decision requiring certainty
3. Declare uncertainty state explicitly
4. Do NOT proceed with low certainty

### Step 2: Gather More Data Points
1. Read additional files in the codebase
2. Query additional API endpoints
3. Check related documentation
4. Review similar past decisions
5. Consult domain experts if available

### Step 3: Query Additional Sources
1. Check GitHub issues/PRs for context
2. Search for similar patterns in the codebase
3. Review commit history for related changes
4. Check documentation for guidance
5. Query external resources if needed

### Step 4: Run Additional Tests
1. Create test cases for the decision
2. Run tests to validate assumptions
3. Check edge cases
4. Verify performance implications
5. Validate security implications

### Step 5: Calculate New Certainty
1. Weigh all gathered evidence
2. Consider edge cases
3. Map dependencies
4. Assess risks
5. Calculate revised certainty score

### Step 6: Only Proceed If > 0.99
If new certainty >= 0.99:
- Document the analysis
- Proceed with decision
- Log the reasoning

If new certainty < 0.99:
- Return to Step 2
- Gather more data
- Or escalate for human decision

## Verification Checklist

- [ ] Data completeness > 95% ✅
- [ ] Edge cases considered ✅
- [ ] Dependencies mapped ✅
- [ ] Security implications assessed ✅
- [ ] Performance implications assessed ✅
- [ ] Certainty >= 0.99 ✅

## Example Usage

```rust
pub async fn deep_dive_analysis<T>(
    decision: &str,
    initial_certainty: f64,
    gather_data: impl Fn() -> Vec<DataPoint>,
    run_tests: impl Fn() -> Vec<TestResult>,
) -> Result<AnalysisResult<T>> {
    if initial_certainty >= 0.99 {
        return Ok(AnalysisResult::Proceed {
            certainty: initial_certainty,
        });
    }

    tracing::warn!(
        "Low certainty ({}) on decision: {}. Initiating deep-dive analysis.",
        initial_certainty, decision
    );

    // Step 2: Gather more data
    let data_points = gather_data();
    let data_completeness = calculate_completeness(&data_points);

    // Step 3: Query additional sources
    let additional_context = query_additional_sources(decision).await?;

    // Step 4: Run additional tests
    let test_results = run_tests();
    let test_coverage = calculate_coverage(&test_results);

    // Step 5: Calculate new certainty
    let new_certainty = calculate_certainty(
        initial_certainty,
        data_completeness,
        test_coverage,
        &additional_context,
    );

    // Step 6: Only proceed if > 0.99
    if new_certainty >= 0.99 {
        Ok(AnalysisResult::Proceed {
            certainty: new_certainty,
        })
    } else {
        Ok(AnalysisResult::Escalate {
            certainty: new_certainty,
            reason: "Insufficient certainty after deep-dive analysis".to_string(),
        })
    }
}
```

## Success Criteria

- Data completeness > 95%
- Edge cases identified and addressed
- Dependencies fully mapped
- Security implications assessed
- Performance implications assessed
- Final certainty >= 0.99

## Anti-Patterns

❌ **DO NOT** proceed with certainty < 0.80
❌ **DO NOT** skip data gathering
❌ **DO NOT** ignore edge cases
❌ **DO NOT** assume without verification
❌ **DO NOT** proceed with partial analysis

## Escalation Criteria

If after deep-dive analysis certainty is still < 0.99:
- Document all gathered evidence
- Present uncertainty to human decision-maker
- Recommend human intervention
- Do not make the decision autonomously

## Origin

Auto-generated from autonomous-upskilling workflow after decisions were made with insufficient certainty, leading to suboptimal outcomes.
