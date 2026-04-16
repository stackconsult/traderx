#!/usr/bin/env python3
"""Check GitHub Actions status for validation"""
import urllib.request
import json
import ssl
import sys
import os

def main():
    token = os.environ.get('GITHUB_TOKEN', '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')
    
    ctx = ssl.create_default_context()
    headers = {'Authorization': f'token {token}'}
    
    # Note: The & in URL needs to be encoded or handled carefully
    url = 'https://api.github.com/repos/stackconsult/traderx/actions/runs?branch=fix/oms-engine-compilation-errors&per_page=1'
    
    req = urllib.request.Request(url, headers=headers)
    
    try:
        with urllib.request.urlopen(req, context=ctx) as response:
            data = json.loads(response.read())
            
            if data.get('workflow_runs') and len(data['workflow_runs']) > 0:
                run = data['workflow_runs'][0]
                status = run.get('status', 'unknown')
                conclusion = run.get('conclusion', 'unknown')
                run_id = run.get('id', 'N/A')
                html_url = run.get('html_url', 'N/A')
                
                print(f"Status: {status}")
                print(f"Conclusion: {conclusion}")
                print(f"Run ID: {run_id}")
                print(f"URL: {html_url}")
                print()
                
                if conclusion == 'success':
                    print("✅ VALIDATION PASSED - Ready to merge")
                    return 0
                elif conclusion == 'failure':
                    print("❌ VALIDATION FAILED - Fixes required")
                    return 1
                else:
                    print("⏳ VALIDATION PENDING")
                    return 2
            else:
                print("No workflow runs found for branch")
                return 1
    except Exception as e:
        print(f"Error: {e}")
        return 1

if __name__ == '__main__':
    sys.exit(main())
