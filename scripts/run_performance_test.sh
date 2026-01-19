#!/bin/bash
# Automated performance testing script for Praxis engine
#
# Usage:
#   ./scripts/run_performance_test.sh
#
# This script runs the comprehensive performance profiling demo and
# validates that all optimizations provide expected improvements.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "======================================"
echo "Praxis Engine Performance Test"
echo "======================================"
echo ""

# Check if running in release mode
if [[ "$1" != "--release" ]]; then
    echo -e "${YELLOW}WARNING: Running in debug mode${NC}"
    echo "For accurate performance measurement, run:"
    echo "  ./scripts/run_performance_test.sh --release"
    echo ""
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
    MODE=""
else
    MODE="--release"
    echo -e "${GREEN}Running in release mode${NC}"
fi

echo ""
echo "Building example..."
cargo build $MODE --example performance_profiling_comprehensive

if [ $? -ne 0 ]; then
    echo -e "${RED}Build failed!${NC}"
    exit 1
fi

echo -e "${GREEN}Build successful${NC}"
echo ""

# System information
echo "System Information:"
echo "===================="

# Try to get GPU info (Linux)
if command -v lspci &> /dev/null; then
    GPU=$(lspci | grep -i vga | head -1 | cut -d: -f3)
    echo "GPU: $GPU"
fi

# CPU info
if [ -f /proc/cpuinfo ]; then
    CPU=$(grep "model name" /proc/cpuinfo | head -1 | cut -d: -f2 | xargs)
    echo "CPU: $CPU"
fi

# Memory info
if [ -f /proc/meminfo ]; then
    MEM=$(grep MemTotal /proc/meminfo | awk '{printf "%.1f GB", $2/1024/1024}')
    echo "RAM: $MEM"
fi

echo ""
echo "Running performance test..."
echo "============================="
echo ""
echo "Instructions:"
echo "1. The demo will start with baseline (no optimizations)"
echo "2. Wait 2-3 seconds for warmup"
echo "3. Press '1' through '7' to test each optimization level"
echo "4. Press 'P' after each level to see results"
echo "5. Press 'P' at the end for a full comparison report"
echo "6. Press 'E' to export Chrome trace (optional)"
echo "7. Press 'ESC' to exit"
echo ""
echo "Starting in 3 seconds..."
sleep 3

# Run the example
cargo run $MODE --example performance_profiling_comprehensive

echo ""
echo "Performance test complete!"
echo ""
echo "Next steps:"
echo "1. Review the performance comparison report"
echo "2. If you exported a Chrome trace, open it in chrome://tracing"
echo "3. Check that each optimization shows improvement"
echo "4. Validate against expected results in docs/performance_profiling_guide.md"
echo ""

# Check if trace file was generated
if [ -f "performance_trace.json" ]; then
    echo -e "${GREEN}Chrome trace exported: performance_trace.json${NC}"
    echo "Open in Chrome: chrome://tracing"
    echo ""
fi
