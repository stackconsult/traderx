#!/bin/bash

# Architectural Compliance Check Script
# This script enforces the same rules as the CI/CD pipeline locally

set -e

echo "🎯 TRADERX ARCHITECTURAL COMPLIANCE CHECK"
echo "======================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# 1. Binary compilation check (CRITICAL)
echo ""
echo "1. Binary Compilation Check"
echo "--------------------------"

echo "Checking binary compilation..."
if cargo check --package oms-engine --bins 2>&1 | tee compile_output.txt; then
    error_count=$(grep "^error" compile_output.txt | wc -l || true)
    if [ $error_count -ne 0 ]; then
        print_error "Binary compilation failed with $error_count errors"
        grep "^error" compile_output.txt || true
        rm -f compile_output.txt
        exit 1
    else
        print_status "All binaries compile successfully"
    fi
else
    print_error "Binary compilation check failed"
    rm -f compile_output.txt
    exit 1
fi

rm -f compile_output.txt

# 2. Integration module usage check
echo ""
echo "2. Integration Module Usage Check"
echo "-------------------------------"

if grep -q "integration::" packages/oms-engine/src/bin/main.rs; then
    print_status "main.rs uses integration module"
else
    print_error "main.rs must use integration module"
    exit 1
fi

# 3. API drift detection
echo ""
echo "3. API Drift Detection"
echo "--------------------"

# Check for direct component construction in binaries
if grep -q "OmsEngine::new" packages/oms-engine/src/bin/main.rs && ! grep -q "create_trading_system" packages/oms-engine/src/bin/main.rs; then
    print_error "main.rs must use integration factory functions instead of direct construction"
    exit 1
else
    print_status "No direct component construction detected"
fi

# Check for other anti-patterns
if grep -q "RiskBus::new" packages/oms-engine/src/bin/main.rs && ! grep -q "create_trading_system\|create_risk_bus" packages/oms-engine/src/bin/main.rs; then
    print_error "main.rs should use factory functions for RiskBus"
    exit 1
fi

print_status "No architectural drift detected"

# 4. Integration tests
echo ""
echo "4. Integration Tests"
echo "-------------------"

echo "Running integration tests..."
if cargo test --package oms-engine --test integration_tests -- --nocapture; then
    print_status "Integration tests passed"
else
    print_error "Integration tests failed"
    exit 1
fi

echo "Running integration module test..."
if cargo test --package oms-engine --test integration_module_test -- --nocapture; then
    print_status "Integration module test passed"
else
    print_error "Integration module test failed"
    exit 1
fi

# 5. Performance validation
echo ""
echo "5. Performance Validation"
echo "------------------------"

echo "Testing complete trading flow performance..."
if timeout 30s cargo test --package oms-engine test_complete_order_lifecycle --release -- --nocapture; then
    print_status "Performance test passed"
else
    print_error "Performance test failed or timed out"
    exit 1
fi

# 6. Security check
echo ""
echo "6. Security Check"
echo "----------------"

echo "Checking for hardcoded secrets..."
if grep -r "password\|secret\|token\|api_key" packages/oms-engine/src/ --include="*.rs" | grep -v "//" | grep -q "\""; then
    print_error "Potential hardcoded secrets found"
    grep -r "password\|secret\|token\|api_key" packages/oms-engine/src/ --include="*.rs" | grep -v "//" | grep "\"" || true
    exit 1
else
    print_status "No hardcoded secrets detected"
fi

# 7. Documentation check
echo ""
echo "7. Documentation Check"
echo "---------------------"

echo "Checking documentation..."
if cargo doc --package oms-engine --no-deps; then
    print_status "Documentation builds successfully"
else
    print_error "Documentation build failed"
    exit 1
fi

# Final summary
echo ""
echo "🎉 ALL CHECKS PASSED"
echo "==================="
echo "✅ Binary compilation"
echo "✅ Integration module usage"
echo "✅ No architectural drift"
echo "✅ Integration tests"
echo "✅ Performance validation"
echo "✅ Security check"
echo "✅ Documentation"
echo ""
echo "Ready for commit!"
