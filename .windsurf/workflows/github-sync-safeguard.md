# GitHub Sync Safeguard - NEVER FORGET

**Priority**: CRITICAL - HIGHEST  
**Rule**: GitHub online repo is the ONLY source of truth  
**Frequency**: Check before EVERY action, after EVERY commit  
**Enforcement**: Automatic + Manual  

---

## 🚨 THE LAW

> **"Last commits to the GitHub repo branches online (THIS IS WHAT MATTERS AND MUST ALWAYS BE SYNCED)"**

### **Absolute Rules**:

1. **NEVER** let commits sit unpushed > 5 minutes
2. **NEVER** perform rebases without --force-with-lease backup plan
3. **NEVER** assume push succeeded - ALWAYS verify
4. **NEVER** work on detached HEAD for extended periods
5. **ALWAYS** check `git status` and `git log origin/main..main` before any work

---

## ⚡ PRE-COMMAND CHECKLIST

Before **EVERY** command that changes code:

```bash
# 1. Check current status
git status

# 2. Check for unpushed commits
git log origin/main..main --oneline

# 3. If unpushed commits exist, PUSH FIRST
# DO NOT PROCEED until push succeeds
```

---

## 🔧 SAFE WORKFLOW

### **Step 1: Pre-Flight Check** (5 seconds)
```bash
git status
git log origin/main..main --oneline
```
**If output shows unpushed commits → STOP → PUSH NOW**

### **Step 2: Make Changes**
Edit files, make improvements

### **Step 3: Stage & Commit** (10 seconds)
```bash
git add <files>
git commit -m "type: clear description"
```

### **Step 4: VERIFY PUSH** (CRITICAL - 5 seconds)
```bash
git push origin main
# Watch for "Everything up-to-date" or successful push output
```

### **Step 5: VERIFY ONLINE** (10 seconds)
```bash
# Check GitHub directly
python scripts/github_online_check.py
# OR
gh run list --limit 1
# OR open https://github.com/stackconsult/traderx/commits/main
```

### **Step 6: Document**
Update JOURNAL.md with commit hash

---

## 🛡️ SAFEGUARD AUTOMATION

### **Pre-Commit Hook** (Automatic)
```bash
#!/bin/sh
# .git/hooks/pre-commit

# Check for unpushed commits
unpushed=$(git log origin/main..main --oneline 2>/dev/null | wc -l)
if [ "$unpushed" -gt "5" ]; then
    echo "ERROR: $unpushed unpushed commits! Push before committing."
    exit 1
fi
```

### **Pre-Action Script** (Manual/Automated)
```python
# scripts/github_sync_check.py
import subprocess
import sys

def check_sync():
    # Check for unpushed commits
    result = subprocess.run(
        ['git', 'log', 'origin/main..main', '--oneline'],
        capture_output=True, text=True
    )
    
    unpushed = len([l for l in result.stdout.split('\n') if l.strip()])
    
    if unpushed > 0:
        print(f"🚨 CRITICAL: {unpushed} unpushed commits!")
        print("PUSH IMMEDIATELY before any other work.")
        return False
    
    print("✅ GitHub sync verified - no unpushed commits")
    return True

if __name__ == "__main__":
    if not check_sync():
        sys.exit(1)
```

---

## 🚨 EMERGENCY RECOVERY

### **If Rebase Blocks Commits**:

1. **ABORT IMMEDIATELY**:
   ```bash
   git rebase --abort
   ```

2. **CHECKOUT MAIN**:
   ```bash
   git checkout main
   ```

3. **FIND LOST COMMITS**:
   ```bash
   git reflog | head -20
   ```

4. **RECOVER CHANGES**:
   ```bash
   git cherry-pick <commit-hash>
   # OR re-apply changes manually
   ```

5. **COMMIT & PUSH**:
   ```bash
   git add -A
   git commit -m "RECOVERY: Re-apply lost changes after rebase abort"
   git push origin main
   ```

---

## 📊 SYNC VERIFICATION COMMANDS

```bash
# Quick check
alias sync-check='git log origin/main..main --oneline'

# Detailed check
alias sync-verify='git status && git log origin/main..main --oneline && echo "If any commits shown above, PUSH NOW"'

# Force push check (emergency)
alias sync-emergency='git push origin main --force-with-lease'
```

---

## 🎯 INTEGRATION WITH WORKFLOWS

### **Before /session-start**:
```bash
# 1. Check sync status
python scripts/github_sync_check.py

# 2. If unsynced, push first
git push origin main

# 3. Verify
python scripts/github_online_check.py
```

### **Before Every Commit**:
```bash
# Run sync check
if ! python scripts/github_sync_check.py; then
    echo "Fix sync before committing!"
    exit 1
fi
```

### **After Every Commit**:
```bash
# Immediate push
git push origin main

# Verify online
python scripts/github_online_check.py
```

---

## 🔒 HARDENED CHECKLIST

**Before ANY work session**:
- [ ] Run `git status`
- [ ] Check `git log origin/main..main`
- [ ] If unpushed commits: **PUSH FIRST**
- [ ] Verify online: `python scripts/github_online_check.py`

**After EVERY commit**:
- [ ] Push immediately: `git push origin main`
- [ ] Verify online within 1 minute
- [ ] Document commit hash in JOURNAL.md

**If ANY git operation fails**:
- [ ] STOP immediately
- [ ] Check status: `git status`
- [ ] Do NOT proceed until resolved
- [ ] Verify sync before continuing

---

## ⚠️ COMMON FAILURE MODES

### **Rebase Blocking**:
- **Symptom**: `git status` shows "interactive rebase in progress"
- **Fix**: `git rebase --abort && git checkout main`
- **Prevent**: Never start long rebases without escape plan

### **Detached HEAD**:
- **Symptom**: `git status` shows "HEAD detached"
- **Fix**: `git checkout main`
- **Prevent**: Check branch before every commit

### **Merge Conflicts**:
- **Symptom**: `git status` shows "Unmerged paths"
- **Fix**: Resolve or `git merge --abort`
- **Prevent**: Pull before push, sync frequently

### **Force Push Needed**:
- **Symptom**: "rejected - non-fast-forward"
- **Fix**: `git pull origin main` first, then push
- **Prevent**: Use `git push --force-with-lease` only if sure

---

## 🎓 MEMORY AID

**Remember**: 
- GitHub online = Truth
- Local = Ephemeral
- Unpushed commits = At risk
- **PUSH EARLY, PUSH OFTEN**

**Never forget**: 
- Check sync status every 5 minutes
- Push after every meaningful change
- Verify online immediately after push
- Rebase = Danger (avoid unless necessary)

---

## ✅ VERIFICATION

**After reading this workflow**:
1. Run `git status` → Verify no rebase in progress
2. Run `git log origin/main..main` → Verify no unpushed commits
3. If clean: ✅ Ready to work
4. If unclean: 🚨 PUSH FIRST

**Status**: SAFEGUARD ACTIVE - NEVER FORGET AGAIN
