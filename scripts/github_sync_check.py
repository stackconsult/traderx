#!/usr/bin/env python3
"""
GitHub Sync Check - CRITICAL SAFETY GUARD

MUST run before any work session.
MUST run after every commit.
NEVER proceed if unsynced.

This script is LAW - obey it unconditionally.
"""

import subprocess
import sys
import time

def run_cmd(cmd, timeout=10):
    """Run shell command and return output"""
    try:
        result = subprocess.run(
            cmd, shell=True, capture_output=True, text=True, timeout=timeout
        )
        return result.returncode, result.stdout, result.stderr
    except Exception as e:
        return -1, '', str(e)

def check_git_status():
    """Check for rebase/merge in progress"""
    code, out, err = run_cmd('git status --porcelain --branch')
    
    if 'rebase in progress' in out or 'rebase in progress' in err:
        print("🚨 CRITICAL: Rebase in progress!")
        print("   Run: git rebase --abort")
        return False
    
    if 'merge in progress' in out or 'merge in progress' in err:
        print("🚨 CRITICAL: Merge in progress!")
        print("   Resolve or run: git merge --abort")
        return False
    
    return True

def check_unpushed_commits():
    """Check for unpushed commits"""
    code, out, err = run_cmd('git log origin/main..main --oneline')
    
    commits = [l for l in out.split('\n') if l.strip()]
    
    if commits:
        print(f"🚨 CRITICAL: {len(commits)} unpushed commit(s)!")
        print("   Commits waiting to be pushed:")
        for commit in commits[:5]:
            print(f"     - {commit}")
        if len(commits) > 5:
            print(f"     ... and {len(commits) - 5} more")
        print()
        print("   ⚠️  PUSH IMMEDIATELY:")
        print("   git push origin main")
        return False
    
    return True

def check_detached_head():
    """Check if in detached HEAD state"""
    code, out, err = run_cmd('git symbolic-ref --short HEAD')
    
    if code != 0 or not out.strip():
        print("🚨 CRITICAL: Detached HEAD state!")
        print("   Run: git checkout main")
        return False
    
    return True

def verify_github_online():
    """Verify GitHub shows recent commits"""
    import requests
    
    try:
        url = 'https://api.github.com/repos/stackconsult/traderx/commits?per_page=1'
        r = requests.get(url, timeout=5)
        
        if r.status_code == 200:
            commits = r.json()
            if commits:
                latest = commits[0]
                sha = latest['sha'][:7]
                msg = latest['commit']['message'][:50]
                date = latest['commit']['committer']['date']
                
                # Check if commit is recent (within 1 hour)
                from datetime import datetime
                commit_time = datetime.fromisoformat(date.replace('Z', '+00:00'))
                now = datetime.now(commit_time.tzinfo)
                age_hours = (now - commit_time).total_seconds() / 3600
                
                if age_hours > 1:
                    print(f"⚠️  WARNING: GitHub last commit is {age_hours:.1f} hours old!")
                    print(f"   Latest: {sha} - {msg}")
                    return False
                else:
                    print(f"✅ GitHub verified: {sha} - {msg[:40]} ({age_hours:.1f}h ago)")
                    return True
    except Exception as e:
        print(f"⚠️  Could not verify GitHub: {e}")
        return True  # Don't block on API failure
    
    return True

def main():
    print("=" * 70)
    print("GITHUB SYNC CHECK - CRITICAL SAFETY GUARD")
    print("=" * 70)
    print()
    
    all_good = True
    
    # Check 1: Git status
    print("1. Checking git status...")
    if not check_git_status():
        all_good = False
    else:
        print("   ✅ No rebase/merge in progress")
    print()
    
    # Check 2: Unpushed commits
    print("2. Checking for unpushed commits...")
    if not check_unpushed_commits():
        all_good = False
    else:
        print("   ✅ No unpushed commits")
    print()
    
    # Check 3: Detached HEAD
    print("3. Checking branch status...")
    if not check_detached_head():
        all_good = False
    else:
        print("   ✅ On valid branch")
    print()
    
    # Check 4: GitHub online
    print("4. Verifying GitHub online status...")
    if not verify_github_online():
        all_good = False
    print()
    
    # Final verdict
    print("=" * 70)
    if all_good:
        print("✅ SYNC VERIFIED - Safe to proceed")
        print("=" * 70)
        return 0
    else:
        print("🚨 SYNC ISSUES FOUND - FIX BEFORE PROCEEDING")
        print("=" * 70)
        print()
        print("REQUIRED ACTIONS:")
        print("1. Push any unpushed commits: git push origin main")
        print("2. Abort any rebase: git rebase --abort")
        print("3. Checkout main: git checkout main")
        print("4. Re-run this check")
        return 1

if __name__ == "__main__":
    sys.exit(main())
