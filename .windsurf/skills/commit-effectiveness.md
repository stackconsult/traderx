# Commit Effectiveness Skill (Auto-generated from commit gap)

## Description
Ensures commits are actually effective by verifying they exist both locally and remotely. Prevents false claims of "committed" when changes are not pushed to GitHub.

## Trigger
Any commit operation (git commit, amend, squash, etc.)

## Mandatory Steps

### Step 1: Stage All Changes
```bash
git add -A
```
Verify: `git status --porcelain` shows no unstaged changes

### Step 2: Create Commit
```bash
git commit -m "descriptive message"
```
Verify: `git log --oneline -1` shows the new commit

### Step 3: Push to Remote
```bash
git push origin <branch-name>
```
Verify: No authentication errors, no rejections

### Step 4: Verify Remote SHA
```bash
git log origin/<branch-name> --oneline -1
```
Verify: Local SHA == Remote SHA

### Step 5: Verify GitHub Web
- Navigate to repository on GitHub
- Check the branch
- Verify commit appears in the commit history
- Verify commit message matches

### Step 6: Only Then Claim "Done"
Only after all 5 verification steps pass, claim the operation is complete

## Verification Checklist

- [ ] Local SHA == Remote SHA ✅
- [ ] Commit visible on GitHub ✅
- [ ] Actions triggered (if applicable) ✅
- [ ] No authentication errors ✅
- [ ] No push rejections ✅

## Error Handling

If any step fails:
- Report which step failed
- Report specific error message
- Do not claim "committed"
- Provide remediation steps
- Retry from the failing step

## Example Usage

```rust
pub async fn commit_and_verify(
    message: &str,
    branch: &str,
) -> Result<CommitResult> {
    // Step 1: Stage all changes
    Command::new("git")
        .args(&["add", "-A"])
        .status()
        .await?;

    // Step 2: Create commit
    Command::new("git")
        .args(&["commit", "-m", message])
        .status()
        .await?;

    let local_sha = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .await?
        .stdout;

    // Step 3: Push to remote
    Command::new("git")
        .args(&["push", "origin", branch])
        .status()
        .await?;

    // Step 4: Verify remote SHA
    let remote_sha = Command::new("git")
        .args(&["rev-parse", &format!("origin/{}", branch)])
        .output()
        .await?
        .stdout;

    if local_sha != remote_sha {
        return Err(CommitError::ShaMismatch {
            local: String::from_utf8_lossy(&local_sha).to_string(),
            remote: String::from_utf8_lossy(&remote_sha).to_string(),
        });
    }

    // Step 5: Verify GitHub web (via API)
    let github_commit = github_api
        .get_commit(branch, &String::from_utf8_lossy(&local_sha))
        .await?;

    if github_commit.sha != String::from_utf8_lossy(&local_sha) {
        return Err(CommitError::GitHubMismatch);
    }

    Ok(CommitResult::Success {
        sha: String::from_utf8_lossy(&local_sha).to_string(),
    })
}
```

## Success Criteria

- All 6 steps completed successfully
- Local and remote SHAs match
- Commit visible on GitHub web interface
- CI/CD actions triggered (if configured)
- No authentication or push errors

## Anti-Patterns

❌ **DO NOT** claim "committed" after only local commit
❌ **DO NOT** skip push verification
❌ **DO NOT** assume push succeeded without checking
❌ **DO NOT** claim "done" without GitHub verification

## Origin

Auto-generated from autonomous-upskilling workflow after commit was claimed but not pushed to GitHub, causing changes to not be visible on the remote repository.
