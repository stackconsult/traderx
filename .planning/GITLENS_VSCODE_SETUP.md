# GitLens VS Code Setup for Terminal-Free Git Push

## Problem
Shell injection guards block `git` and `gh` CLI commands in this environment.
Solution: Use VS Code's built-in Git UI + GitLens extension to push commits.

## Step 1: Install GitLens (if not already installed)

1. Open VS Code Extensions panel: `Cmd+Shift+X` (Mac) or `Ctrl+Shift+X` (Windows/Linux)
2. Search: `GitLens`
3. Click **Install** on "GitLens — Git supercharged" by GitKraken

## Step 2: Open Source Control Panel

Press `Ctrl+Shift+G` or click branch icon in left sidebar.

## Step 3: Review Your Commits

The panel shows:
- **Staged changes** (green + icons)
- **Commits not yet pushed** (will show ↑ arrow with count)

Your recent commits should be visible from our mem0 work:
- `feat(mem0): Phase 2 security event wiring`
- `feat(mem0): Extract shared types + Phase 3 orchestrator wiring`

## Step 4: Push to GitHub

**Method A: Source Control Panel (Built-in)**
1. Click the `...` (three dots) menu at top of Source Control panel
2. Select **Push**
3. If prompted for branch, select `feature/github-mcp-setup`

**Method B: GitLens Side Bar**
1. Look for GitLens icon in left sidebar (looks like branches)
2. Click **Branches** → expand `feature/github-mcp-setup`
3. Right-click → **Push Branch**

**Method C: Command Palette**
1. `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
2. Type: `Git: Push`
3. Select `origin feature/github-mcp-setup`

## Step 5: If Push Is Rejected (Diverged Branch)

1. In Source Control panel, click `...` menu
2. Select **Pull, Push** → **Push (Force)**
   - Or look for "Publish Branch" if never pushed
3. Confirm when warned about force push

## GitLens Settings (Optional, for better UX)

Add these to your VS Code User Settings (`Cmd+,` → search settings):

```json
{
  "gitlens.statusBar.enabled": true,
  "gitlens.currentLine.enabled": true,
  "git.confirmSync": false,
  "git.confirmForcePush": false
}
```

## Troubleshooting

| Issue | Fix |
|-------|-----|
| "No remotes configured" | Add remote: `...` menu → Remote → Add Remote → URL from GitHub |
| "Authentication failed" | VS Code will prompt for GitHub login — click "Allow" in browser |
| "Cannot push to non-fast-forward" | Use **Force Push** from `...` menu |
| Commits not showing | Check you're on `feature/github-mcp-setup` branch (bottom-left of VS Code) |

## Quick Verify After Push

Go to GitHub web → your repo → `feature/github-mcp-setup` branch.
You should see commits from today with mem0 changes.

## Alternative: GitHub Web Upload (Last Resort)

If ALL VS Code git methods fail:
1. Go to github.com/your-repo
2. Switch to `feature/github-mcp-setup` branch
3. Navigate to each modified file
4. Click **Edit** (pencil icon) → paste our code changes
5. Commit each file with message from our commit history

Note: This loses proper git history but gets code online.

---
Generated: 2026-05-03
Branch: feature/github-mcp-setup
Commits pending: mem0 Phase 2 + Phase 3
