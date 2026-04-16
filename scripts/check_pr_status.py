#!/usr/bin/env python3
"""Check PR status including check runs - PRODUCTION GUARD"""
import urllib.request
import json
import ssl
import sys

TOKEN = "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"

def get_pr_check_runs(owner, repo, ref):
    """Get check runs for a specific ref (branch)"""
    ctx = ssl.create_default_context()
    headers = {'Authorization': f'token {TOKEN}', 'Accept': 'application/vnd.github.v3+json'}
    
    url = f'https://api.github.com/repos/{owner}/{repo}/commits/{ref}/check-runs'
    req = urllib.request.Request(url, headers=headers)
    
    try:
        with urllib.request.urlopen(req, context=ctx) as response:
            return json.loads(response.read())
    except Exception as e:
        print(f"Error checking PR status: {e}")
        return None

def get_pr_details(owner, repo, pr_number):
    """Get PR details including mergeable status"""
    ctx = ssl.create_default_context()
    headers = {'Authorization': f'token {TOKEN}', 'Accept': 'application/vnd.github.v3+json'}
    
    url = f'https://api.github.com/repos/{owner}/{repo}/pulls/{pr_number}'
    req = urllib.request.Request(url, headers=headers)
    
    try:
        with urllib.request.urlopen(req, context=ctx) as response:
            return json.loads(response.read())
    except Exception as e:
        print(f"Error getting PR details: {e}")
        return None

def main():
    owner = "stackconsult"
    repo = "traderx"
    branch = "fix/oms-engine-compilation-errors"
    
    print("=" * 60)
    print("PRODUCTION GUARD: PR Check Status Validation")
    print("=" * 60)
    print()
    
    # Get check runs for the branch
    print(f"Checking check runs for branch: {branch}")
    check_data = get_pr_check_runs(owner, repo, branch)
    
    if not check_data:
        print("❌ FAILED: Could not retrieve check runs")
        return 1
    
    total = check_data.get('total_count', 0)
    print(f"Total check runs: {total}")
    print()
    
    if total == 0:
        print("⚠️ WARNING: No check runs found - Actions may not have triggered")
        return 2
    
    # Analyze each check run
    failed_checks = []
    passed_checks = []
    pending_checks = []
    
    for check in check_data.get('check_runs', []):
        name = check.get('name', 'Unknown')
        status = check.get('status', 'unknown')
        conclusion = check.get('conclusion', 'unknown')
        html_url = check.get('html_url', 'N/A')
        
        print(f"Check: {name}")
        print(f"  Status: {status}")
        print(f"  Conclusion: {conclusion}")
        
        if status == 'completed':
            if conclusion == 'success':
                passed_checks.append(name)
                print(f"  ✅ PASSED")
            elif conclusion == 'failure':
                failed_checks.append({'name': name, 'url': html_url})
                print(f"  ❌ FAILED")
            else:
                pending_checks.append(name)
                print(f"  ⚠️ {conclusion}")
        else:
            pending_checks.append(name)
            print(f"  ⏳ {status}")
        print()
    
    # Summary
    print("=" * 60)
    print("VALIDATION SUMMARY")
    print("=" * 60)
    print(f"Passed: {len(passed_checks)}")
    print(f"Failed: {len(failed_checks)}")
    print(f"Pending: {len(pending_checks)}")
    print()
    
    if failed_checks:
        print("❌ CRITICAL: PR CHECKS FAILED")
        print("Failed checks:")
        for check in failed_checks:
            print(f"  - {check['name']}: {check['url']}")
        print()
        print("DO NOT MERGE - Fixes required")
        return 1
    
    if pending_checks:
        print("⏳ PENDING: Checks still running")
        print("Wait for completion before merging")
        return 2
    
    if passed_checks and not failed_checks and not pending_checks:
        print("✅ ALL PR CHECKS PASSED")
        print("Ready for merge")
        return 0
    
    return 1

if __name__ == '__main__':
    sys.exit(main())
