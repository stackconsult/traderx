---
description: Auto-sync journal and agent master system across all branches
---

# Auto Journal Sync Workflow

Automatically propagates JOURNAL.md updates and AGENT_MASTER_SYSTEM.md to all branches.
Run after every significant action. Triggered by `scripts/journal_sync.ps1`.

## Step 1: Verify clean working state
```bash
git status --short
```
If dirty → commit or stash first.

## Step 2: Add new journal entry
Edit JOURNAL.md with the standard entry format from AGENT_MASTER_SYSTEM.md §9.
Update the STATUS DASHBOARD section in AGENT_MASTER_SYSTEM.md.

## Step 3: Commit to current branch
```bash
git add JOURNAL.md AGENT_MASTER_SYSTEM.md MASTER_OPERATIONAL_CHECKLIST.md
git commit -m "journal: [timestamp] [action summary] #auto-sync"
git push origin HEAD
```

## Step 4: Run auto-sync script
```powershell
powershell -File scripts/journal_sync.ps1
```

## Step 5: Verify sync succeeded
```bash
git log --oneline --all | Select-String "journal"
```

## Notes
- The `mcp/agent-master` branch always has the latest AGENT_MASTER_SYSTEM.md
- All feature branches receive journal updates via cherry-pick
- The sync script handles conflicts by preferring the newer timestamp entry
- Never manually edit journal on multiple branches — always use this workflow
