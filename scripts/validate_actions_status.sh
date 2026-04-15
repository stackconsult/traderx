#!/bin/bash
# Validate GitHub Actions Status
# Part of GitHub-First Self-Healing Architecture
# This script validates that Actions passed before allowing merge

set -e

REPO_OWNER="stackconsult"
REPO_NAME="traderx"
BRANCH="${1:-fix/oms-engine-compilation-errors}"
GITHUB_TOKEN="${GITHUB_TOKEN:-}"

if [ -z "$GITHUB_TOKEN" ]; then
    echo "❌ ERROR: GITHUB_TOKEN not set"
    echo "Set it with: export GITHUB_TOKEN=your_token_here"
    exit 1
fi

echo "=== GitHub Actions Validation ==="
echo "Repository: $REPO_OWNER/$REPO_NAME"
echo "Branch: $BRANCH"
echo "Time: $(date)"
echo ""

# Get latest workflow run for branch
echo "🔍 Checking latest Actions run..."
RUN_DATA=$(curl -s -H "Authorization: token $GITHUB_TOKEN" \
    "https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/actions/runs?branch=$BRANCH&per_page=1")

# Extract status
RUN_ID=$(echo "$RUN_DATA" | jq -r '.workflow_runs[0].id // "null"')
STATUS=$(echo "$RUN_DATA" | jq -r '.workflow_runs[0].status // "unknown"')
CONCLUSION=$(echo "$RUN_DATA" | jq -r '.workflow_runs[0].conclusion // "unknown"')
RUN_URL=$(echo "$RUN_DATA" | jq -r '.workflow_runs[0].html_url // "unknown"')

echo "Run ID: $RUN_ID"
echo "Status: $STATUS"
echo "Conclusion: $CONCLUSION"
echo "URL: $RUN_URL"
echo ""

# Check if run exists
if [ "$RUN_ID" == "null" ] || [ -z "$RUN_ID" ]; then
    echo "❌ No Actions runs found for branch $BRANCH"
    echo "Push may not have triggered Actions yet."
    exit 1
fi

# Wait if still running
if [ "$STATUS" == "in_progress" ] || [ "$STATUS" == "queued" ]; then
    echo "⏳ Actions still running..."
    echo "Monitoring for up to 15 minutes..."
    
    for i in {1..30}; do
        sleep 30
        
        RUN_DATA=$(curl -s -H "Authorization: token $GITHUB_TOKEN" \
            "https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/actions/runs/$RUN_ID")
        
        STATUS=$(echo "$RUN_DATA" | jq -r '.status // "unknown"')
        CONCLUSION=$(echo "$RUN_DATA" | jq -r '.conclusion // "unknown"')
        
        echo "  [$i/30] Status: $STATUS | Conclusion: $CONCLUSION"
        
        if [ "$STATUS" == "completed" ]; then
            break
        fi
    done
fi

# Final check
if [ "$STATUS" != "completed" ]; then
    echo ""
    echo "❌ TIMEOUT: Actions did not complete within 15 minutes"
    echo "Check manually: $RUN_URL"
    exit 1
fi

echo ""
echo "=== Validation Result ==="

if [ "$CONCLUSION" == "success" ]; then
    echo "✅ SUCCESS: All Actions checks passed!"
    echo "Branch: $BRANCH"
    echo "Run ID: $RUN_ID"
    echo "URL: $RUN_URL"
    echo ""
    echo "Ready for merge."
    
    # Create validation stamp
    mkdir -p .validation
    cat > .validation/$BRANCH-passed.txt << EOF
Validation Time: $(date -Iseconds)
Branch: $BRANCH
Run ID: $RUN_ID
Conclusion: $CONCLUSION
Status: PASSED
Validated By: validate_actions_status.sh
EOF
    
    exit 0
elif [ "$CONCLUSION" == "failure" ]; then
    echo "❌ FAILURE: Actions checks failed"
    echo "Branch: $BRANCH"
    echo "Run ID: $RUN_ID"
    echo "URL: $RUN_URL"
    echo ""
    echo "View logs and apply fixes."
    exit 1
else
    echo "⚠️  UNKNOWN: Actions conclusion is '$CONCLUSION'"
    echo "Branch: $BRANCH"
    echo "Run ID: $RUN_ID"
    echo "URL: $RUN_URL"
    exit 1
fi
