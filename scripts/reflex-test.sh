#!/bin/bash
# Reflex Testing Script for TraderX
# Blocks commits that lack corresponding proof artifacts

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Log file for tracking
LOG_FILE=".planning/logs/reflex-$(date +%Y%m%d-%H%M%S).log"
mkdir -p .planning/logs

echo "=== TraderX Reflex Test ===" | tee $LOG_FILE
echo "Timestamp: $(date)" | tee -a $LOG_FILE
echo "" | tee -a $LOG_FILE

# Function to check if file exists and is not empty
check_proof() {
    local proof_file=$1
    local description=$2
    
    if [ ! -f "$proof_file" ]; then
        echo -e "${RED}❌ FAIL: $description - Proof file $proof_file not found${NC}" | tee -a $LOG_FILE
        return 1
    fi
    
    if [ ! -s "$proof_file" ]; then
        echo -e "${RED}❌ FAIL: $description - Proof file $proof_file is empty${NC}" | tee -a $LOG_FILE
        return 1
    fi
    
    # Check if JSON is valid
    if [[ $proof_file == *.json ]]; then
        if ! python3 -m json.tool "$proof_file" > /dev/null 2>&1; then
            echo -e "${RED}❌ FAIL: $description - Invalid JSON in $proof_file${NC}" | tee -a $LOG_FILE
            return 1
        fi
    fi
    
    echo -e "${GREEN}✓ PASS: $description${NC}" | tee -a $LOG_FILE
    return 0
}

# Function to check if milestone is marked as completed
check_milestone() {
    local milestone=$1
    local proof_file=$2
    
    # Check if milestone is marked as completed in MILESTONES.md
    if grep -q "Status: ✅ COMPLETED" MILESTONES.md && grep -q "$milestone" MILESTONES.md; then
        echo -e "${GREEN}✓ PASS: $milestone marked as completed${NC}" | tee -a $LOG_FILE
        return 0
    else
        echo -e "${YELLOW}⚠️  WARN: $milestone not marked as completed in MILESTONES.md${NC}" | tee -a $LOG_FILE
        return 0  # Warning, not failure
    fi
}

# Check for recent changes
echo "Checking for recent changes..." | tee -a $LOG_FILE
CHANGED_FILES=$(git diff --cached --name-only 2>/dev/null || git diff --name-only HEAD~1 2>/dev/null || echo "")

if [ -z "$CHANGED_FILES" ]; then
    echo -e "${YELLOW}⚠️  No changes detected${NC}" | tee -a $LOG_FILE
    exit 0
fi

echo "Files changed:" | tee -a $LOG_FILE
echo "$CHANGED_FILES" | tee -a $LOG_FILE
echo "" | tee -a $LOG_FILE

# Track overall status
OVERALL_PASS=true

# Check core files for corresponding proofs
if echo "$CHANGED_FILES" | grep -q "src/core/"; then
    echo "Checking core engine proofs..." | tee -a $LOG_FILE
    check_proof "proofs/engine-tests.json" "Core Engine Tests" || OVERALL_PASS=false
    check_milestone "M1.3" "proofs/engine-tests.json"
fi

if echo "$CHANGED_FILES" | grep -q "src/risk/"; then
    echo "Checking risk manager proofs..." | tee -a $LOG_FILE
    check_proof "proofs/risk-validation.json" "Risk Manager Validation" || OVERALL_PASS=false
    check_milestone "M1.4" "proofs/risk-validation.json"
fi

if echo "$CHANGED_FILES" | grep -q "src/exchanges/"; then
    echo "Checking exchange integration proofs..." | tee -a $LOG_FILE
    check_proof "proofs/paper-exchange-load.json" "Paper Exchange Load Tests" || OVERALL_PASS=false
    check_milestone "M2.1" "proofs/paper-exchange-load.json"
fi

if echo "$CHANGED_FILES" | grep -q "src/strategies/"; then
    echo "Checking strategy proofs..." | tee -a $LOG_FILE
    check_proof "proofs/ma-backtest.json" "MA Strategy Backtest" || OVERALL_PASS=false
    check_milestone "M3.1" "proofs/ma-backtest.json"
fi

# Check if JOURNAL.md is updated for significant changes
SIGNIFICANT_CHANGES=$(echo "$CHANGED_FILES" | grep -E "(src/|config/|main\.py)" | wc -l)
if [ "$SIGNIFICANT_CHANGES" -gt 0 ]; then
    echo "Checking for journal entry..." | tee -a $LOG_FILE
    if [ ! -f "JOURNAL.md" ]; then
        echo -e "${RED}❌ FAIL: JOURNAL.md not found${NC}" | tee -a $LOG_FILE
        OVERALL_PASS=false
    else
        # Check if JOURNAL.md has been updated recently (within last hour)
        JOURNAL_AGE=$(($(date +%s) - $(stat -c %Y JOURNAL.md)))
        if [ $JOURNAL_AGE -gt 3600 ]; then
            echo -e "${YELLOW}⚠️  WARN: JOURNAL.md not updated recently (consider adding entry for changes)${NC}" | tee -a $LOG_FILE
        else
            echo -e "${GREEN}✓ PASS: JOURNAL.md recently updated${NC}" | tee -a $LOG_FILE
        fi
    fi
fi

# Check for .env changes (security)
if echo "$CHANGED_FILES" | grep -q "\.env"; then
    echo -e "${RED}❌ FAIL: .env file should not be committed${NC}" | tee -a $LOG_FILE
    echo "Add .env to .gitignore if not already present" | tee -a $LOG_FILE
    OVERALL_PASS=false
fi

# Final status
echo "" | tee -a $LOG_FILE
if [ "$OVERALL_PASS" = true ]; then
    echo -e "${GREEN}✅ Reflex Test PASSED - All required proofs found${NC}" | tee -a $LOG_FILE
    echo "" | tee -a $LOG_FILE
    echo "Proof artifacts validated:" | tee -a $LOG_FILE
    find proofs/ -name "*.json" -newer .git/index 2>/dev/null | tee -a $LOG_FILE
    exit 0
else
    echo -e "${RED}❌ Reflex Test FAILED - Missing proof artifacts${NC}" | tee -a $LOG_FILE
    echo "" | tee -a $LOG_FILE
    echo "To fix:" | tee -a $LOG_FILE
    echo "1. Run the appropriate test suite" | tee -a $LOG_FILE
    echo "2. Generate proof artifacts in proofs/ directory" | tee -a $LOG_FILE
    echo "3. Update MILESTONES.md to mark completed milestones" | tee -a $LOG_FILE
    echo "" | tee -a $LOG_FILE
    echo "Example:" | tee -a $LOG_FILE
    echo "  python test_system.py > proofs/engine-tests.json" | tee -a $LOG_FILE
    exit 1
fi
