#!/bin/bash
# Performance Regression Testing Script
# Runs critical benchmarks and compares against a baseline

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
BASELINE_NAME="main"
CURRENT_NAME="current"
THRESHOLD=10.0
OUTPUT_DIR="benchmark-results"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --baseline)
            BASELINE_NAME="$2"
            shift 2
            ;;
        --current)
            CURRENT_NAME="$2"
            shift 2
            ;;
        --threshold)
            THRESHOLD="$2"
            shift 2
            ;;
        --output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --baseline NAME     Baseline name (default: main)"
            echo "  --current NAME      Current baseline name (default: current)"
            echo "  --threshold PCT     Regression threshold percentage (default: 10.0)"
            echo "  --output-dir DIR    Output directory (default: benchmark-results)"
            echo "  --help              Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

echo -e "${GREEN}🔬 Performance Regression Testing${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Baseline: $BASELINE_NAME"
echo "Current:  $CURRENT_NAME"
echo "Threshold: ${THRESHOLD}%"
echo ""

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Function to run a benchmark
run_benchmark() {
    local bench_name=$1
    local filter=$2
    local baseline=$3
    
    echo -e "${YELLOW}Running $bench_name benchmark (baseline: $baseline)...${NC}"
    cargo bench --bench "$bench_name" -- "$filter" --save-baseline "$baseline" --noplot
    echo ""
}

# Step 1: Run current benchmarks
echo -e "${GREEN}Step 1: Running benchmarks on current code${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

run_benchmark "graphics_optimization" "multi_draw_indirect" "$CURRENT_NAME"
run_benchmark "graphics_optimization" "gpu_vs_cpu_culling/gpu_culling" "$CURRENT_NAME"
run_benchmark "descriptor_set_allocation" "descriptor_set_caching_lru" "$CURRENT_NAME"

# Step 2: Check if baseline exists or needs to be created
if [ ! -d "target/criterion" ] || [ -z "$(find target/criterion -name "$BASELINE_NAME" -type d 2>/dev/null)" ]; then
    echo -e "${YELLOW}Baseline '$BASELINE_NAME' not found. Creating baseline...${NC}"
    echo ""
    
    # Save current state
    CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
    
    # Checkout baseline branch (usually main)
    if [ "$BASELINE_NAME" = "main" ]; then
        echo "Checking out main branch for baseline..."
        git stash push -u -m "Performance regression test - stashing changes"
        git checkout main
        
        echo -e "${GREEN}Step 2: Running benchmarks on baseline (main branch)${NC}"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        
        run_benchmark "graphics_optimization" "multi_draw_indirect" "$BASELINE_NAME"
        run_benchmark "graphics_optimization" "gpu_vs_cpu_culling/gpu_culling" "$BASELINE_NAME"
        run_benchmark "descriptor_set_allocation" "descriptor_set_caching_lru" "$BASELINE_NAME"
        
        # Return to original branch
        git checkout "$CURRENT_BRANCH"
        git stash pop || true
    else
        echo -e "${RED}Error: Baseline '$BASELINE_NAME' not found and automatic creation only works for 'main'${NC}"
        echo "Please create the baseline manually by running:"
        echo "  cargo bench --bench <benchmark_name> -- --save-baseline $BASELINE_NAME"
        exit 1
    fi
else
    echo -e "${GREEN}Baseline '$BASELINE_NAME' found, skipping baseline creation${NC}"
    echo ""
fi

# Step 3: Compare results
echo -e "${GREEN}Step 3: Comparing results${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cargo run --manifest-path scripts/benchmark-compare/Cargo.toml -- \
    --baseline-dir target/criterion \
    --current-baseline "$BASELINE_NAME" \
    --new-baseline "$CURRENT_NAME" \
    --threshold "$THRESHOLD" \
    --output "$OUTPUT_DIR/comparison.json" \
    --output-markdown "$OUTPUT_DIR/comparison.md"

# Step 4: Show results
echo ""
echo -e "${GREEN}Results saved to:${NC}"
echo "  JSON:     $OUTPUT_DIR/comparison.json"
echo "  Markdown: $OUTPUT_DIR/comparison.md"
echo ""

# Check for regressions
if [ -f "$OUTPUT_DIR/regressions-detected" ]; then
    echo -e "${RED}❌ Performance regressions detected!${NC}"
    echo ""
    cat "$OUTPUT_DIR/comparison.md"
    exit 1
else
    echo -e "${GREEN}✅ No performance regressions detected${NC}"
    echo ""
    cat "$OUTPUT_DIR/comparison.md"
    exit 0
fi
