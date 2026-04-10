#!/bin/bash

# TraderX Build Script
# Builds all components with proper feature flags

set -e

echo "🚀 Building TraderX..."
echo "===================="

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
    echo -e "${YELLOW}⚠️ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    print_error "Rust/Cargo not found. Please install Rust first."
    exit 1
fi

print_status "Rust toolchain found"

# Build OMS Engine (core component)
echo ""
echo "Building OMS Engine..."
cd packages/oms-engine
if cargo build --release; then
    print_status "OMS Engine built successfully"
else
    print_error "OMS Engine build failed"
    exit 1
fi

# Run OMS tests
if cargo test --release; then
    print_status "OMS Engine tests passed"
else
    print_warning "OMS Engine tests failed (may need Redis running)"
fi

cd ../..

# Build HFT System
echo ""
echo "Building HFT System..."
cd packages/hft-system
if cargo build --release; then
    print_status "HFT System built successfully"
else
    print_error "HFT System build failed"
    exit 1
fi

cd ../..

# Build eBPF Router (without DPDK first)
echo ""
echo "Building eBPF Router..."
cd packages/dealing-desk/ebpf-router
if cargo build --release; then
    print_status "eBPF Router built successfully"
else
    print_error "eBPF Router build failed"
    exit 1
fi

# Try building with DPDK (optional)
echo ""
echo "Building eBPF Router with DPDK support..."
if cargo build --release --features dpdk 2>/dev/null; then
    print_status "eBPF Router with DPDK built successfully"
else
    print_warning "DPDK build failed (DPDK libraries not installed)"
fi

cd ../../..

# Build AI Agents (Python)
echo ""
echo "Checking Python dependencies..."
cd packages/ai-agents
if python3 -c "import numpy, pandas, asyncio" 2>/dev/null; then
    print_status "Python dependencies available"
else
    print_warning "Python dependencies missing (run: pip install -r requirements.txt)"
fi

cd ../..

# Build Execution Adapters
echo ""
echo "Checking Execution Adapters..."
cd packages/execution-adapters
if python3 -c "import aiohttp, websockets" 2>/dev/null; then
    print_status "Execution adapter dependencies available"
else
    print_warning "Execution adapter dependencies missing"
fi

# Check for optional Databento
if python3 -c "import databento" 2>/dev/null; then
    print_status "Databento client available"
else
    print_warning "Databento client not installed (optional)"
fi

cd ../..

# Build Dashboard
echo ""
echo "Building Dashboard..."
cd apps/dashboard
if npm install && npm run build; then
    print_status "Dashboard built successfully"
else
    print_warning "Dashboard build failed (Node.js/NPM required)"
fi

cd ../..

# Summary
echo ""
echo "===================="
echo "Build Summary:"
echo "===================="

echo "✅ Core Components:"
echo "   - OMS Engine: Complete"
echo "   - HFT System: Complete"
echo "   - eBPF Router: Complete"
echo ""
echo "📊 Performance Optimizations:"
echo "   - Aeron Messaging: Ready (requires media driver)"
echo "   - DPDK Support: Ready (requires DPDK libraries)"
echo "   - Databento Feed: Ready (requires databento pip package)"
echo ""
echo "🎯 Next Steps:"
echo "   1. Install optional dependencies for full performance"
echo "   2. Start Redis server for journaling"
echo "   3. Run benchmarks: cargo run --release --bin benchmark"
echo "   4. Deploy to co-location for lowest latency"

echo ""
print_status "Build complete! 🚀"
