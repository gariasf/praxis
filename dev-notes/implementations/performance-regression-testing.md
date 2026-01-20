# Performance Regression Testing Implementation

## Overview

Implemented a comprehensive automated performance regression testing system for the Praxis game engine, running on GitHub Actions. The system automatically detects performance regressions in critical areas and fails CI if regressions exceed 10%.

## Components Implemented

### 1. GitHub Actions Workflow

**File:** `.github/workflows/performance-regression.yml`

- Runs on every PR and push to main
- Executes critical benchmarks on both PR and main branch
- Compares results using custom comparison tool
- Posts formatted results as PR comments
- Fails CI if critical benchmarks regress >10%

**Critical Benchmarks Tracked:**
- Multi-draw indirect batching (draw call reduction)
- GPU culling overhead (compute shader performance)
- Descriptor allocation rate (LRU caching efficiency)

### 2. Benchmark Comparison Tool

**Location:** `scripts/benchmark-compare/`

A Rust CLI tool that:
- Parses Criterion JSON benchmark results
- Compares two baselines (main vs current)
- Identifies regressions, improvements, and unchanged benchmarks
- Generates human-readable reports (console, JSON, Markdown)
- Exits with error code if critical regressions detected

**Key Features:**
- Configurable regression threshold (default: 10%)
- Critical benchmark identification (marked with 🚨)
- Statistical analysis of performance changes
- Colored console output for easy visual scanning

### 3. Helper Scripts

**Bash Script:** `scripts/run-performance-regression.sh`
**PowerShell Script:** `scripts/run-performance-regression.ps1`

Local testing scripts that:
- Run benchmarks on current code
- Optionally create baseline from main branch
- Compare results using the comparison tool
- Display formatted output
- Exit with appropriate status codes

### 4. Documentation

**Main Documentation:** `docs/performance-regression-testing.md`
- Complete system overview
- Workflow execution details
- Critical benchmark descriptions with targets
- Local testing instructions
- Troubleshooting guide

**Tool Documentation:** `scripts/benchmark-compare/README.md`
- Tool usage instructions
- Command-line arguments
- Output formats
- Integration details

**Workflow Documentation:** `.github/workflows/README.md`
- Overview of all CI workflows
- Configuration details
- Troubleshooting tips

### 5. Updated .gitignore

Added entries for performance testing artifacts:
- `benchmark-results/` - Output directory for comparisons
- `*.bench` - Benchmark data files
- `comparison.json` / `comparison.md` - Generated reports
- `regressions-detected` - Marker file for CI

## Critical Benchmarks

### 1. Multi-Draw Indirect Batching

**Benchmark:** `graphics_optimization::multi_draw_indirect_rendering`

Measures draw call batching efficiency with varying object counts (500, 750, 1000) and material counts (20). Tests both traditional individual draw calls and multi-draw indirect rendering.

**Target:** Minimize CPU overhead per rendered object

**Why Critical:** Draw call batching is fundamental to rendering performance. Even small regressions can compound with large object counts, directly impacting frame rates.

### 2. GPU Culling Overhead

**Benchmark:** `graphics_optimization::gpu_vs_cpu_culling/gpu_culling`

Measures the overhead of GPU compute-based frustum culling for 1000, 5000, and 10000 objects. Tests both CPU-side preparation and GPU execution time.

**Target:** <1ms for 10,000 objects

**Why Critical:** GPU culling enables massive scenes. Excessive overhead negates the benefits of parallel processing and can become a bottleneck.

### 3. Descriptor Allocation Rate

**Benchmark:** `descriptor_set_allocation::descriptor_set_caching_lru`

Measures descriptor set allocation efficiency with LRU caching over 1000 frames with 100 unique materials. Tracks allocation count and cache hit rate.

**Target:** 100x+ reduction (100,000 → 100 allocations)

**Why Critical:** Descriptor allocation is a major CPU bottleneck. Poor caching causes frame time spikes and visible stutter.

## Regression Threshold

**Configured at:** 10%

### Rationale

- **CI Variance:** GitHub Actions runners have ~5% performance variance
- **Meaningful Changes:** 10% is significant enough to impact user experience
- **False Positives:** Tighter thresholds would fail too often due to noise
- **Industry Standard:** Most performance-critical projects use 5-15%

### Threshold Behavior

- **>10% slower:** Regression (fails CI)
- **>10% faster:** Improvement (passes with praise)
- **±10%:** Unchanged (acceptable variance)

## Workflow Execution

### On Pull Request

```
1. Checkout PR branch
2. Run critical benchmarks → save as "current"
3. Checkout main branch
4. Run critical benchmarks → save as "main"
5. Return to PR branch
6. Run comparison tool
7. Post PR comment with results
8. Fail if critical regressions detected
```

**Duration:** ~10-15 minutes

### On Main Branch Push

```
1. Checkout main branch
2. Run critical benchmarks → save as baseline
3. Cache results for future comparisons
```

**Duration:** ~5-8 minutes (no comparison needed)

## PR Comment Format

The bot posts a structured comment with:

```markdown
## 🔬 Performance Benchmark Results

### 📊 Benchmark Results (45 total)

#### ⚠️ CRITICAL REGRESSIONS DETECTED
- 🚨 gpu_vs_cpu_culling/gpu_culling/10000: +12.45% slower

#### ❌ Regressions (2)
[Table with benchmark names, times, and changes]

#### ✅ Improvements (3)
[Table with benchmark names, times, and changes]

#### ➡️ Unchanged (within ±10%) - 40
<details>
[Collapsible list of unchanged benchmarks]
</details>

---
**Critical Benchmarks Tracked:**
- Multi-draw indirect batching
- GPU culling overhead
- Descriptor allocation rate
```

## Local Testing

### Quick Test

```bash
# Linux/Mac
./scripts/run-performance-regression.sh

# Windows
.\scripts\run-performance-regression.ps1
```

### Manual Test

```bash
# 1. Run benchmarks on main
git checkout main
cargo bench --bench graphics_optimization -- "multi_draw_indirect" --save-baseline main

# 2. Run benchmarks on feature branch
git checkout feature-branch
cargo bench --bench graphics_optimization -- "multi_draw_indirect" --save-baseline current

# 3. Compare
cargo run --manifest-path scripts/benchmark-compare/Cargo.toml -- \
    --baseline-dir target/criterion \
    --current-baseline main \
    --new-baseline current \
    --threshold 10.0 \
    --output-markdown comparison.md
```

## CI Integration

### Caching Strategy

The workflow uses `rust-cache` to cache:
- Cargo build artifacts
- Criterion baseline results
- Compiled benchmark binaries

This reduces subsequent runs from ~15 minutes to ~5-8 minutes.

### System Dependencies

The workflow installs:
- `libvulkan-dev` - Vulkan development headers
- `vulkan-validationlayers` - Vulkan validation layers
- `mesa-vulkan-drivers` - Software Vulkan rendering (for headless CI)

## Technical Details

### Criterion Integration

The system leverages Criterion's built-in baseline feature:
- `--save-baseline NAME` saves results under `target/criterion/{benchmark}/NAME/`
- Each baseline contains `estimates.json` with statistical data
- Baseline directories include mean, median, and standard error

### JSON Format

Each benchmark's `estimates.json` contains:

```json
{
  "mean": {
    "point_estimate": 123456.78,
    "standard_error": 1234.56
  },
  "median": {
    "point_estimate": 123400.00,
    "standard_error": 1200.00
  }
}
```

The comparison tool parses these files to calculate percentage changes.

### Error Handling

The system includes robust error handling:
- Missing baseline detection with helpful messages
- Graceful handling of incomplete benchmark runs
- Clear error messages for debugging
- Automatic retry logic for transient failures

## Benefits

1. **Early Detection:** Catches performance regressions before merge
2. **Objective Metrics:** Removes guesswork from performance reviews
3. **Historical Tracking:** Maintains baseline for comparison
4. **Developer Feedback:** Clear reports help identify causes
5. **Quality Assurance:** Prevents accidental performance degradation

## Future Enhancements

Potential improvements:

1. **Historical Tracking:** Store results in database for trend analysis
2. **Performance Graphs:** Generate charts showing trends over time
3. **Automatic Baselines:** Update main baseline after merge
4. **Platform-Specific:** Different thresholds for different runners
5. **Selective Benchmarks:** Run different sets based on changed files
6. **Flamegraph Integration:** Automatic profiling on regressions

## Files Created

```
.github/workflows/
  └── performance-regression.yml         # GitHub Actions workflow

scripts/
  ├── run-performance-regression.sh      # Bash helper script
  ├── run-performance-regression.ps1     # PowerShell helper script
  └── benchmark-compare/                 # Comparison tool
      ├── Cargo.toml                     # Tool manifest
      ├── README.md                      # Tool documentation
      └── src/
          └── main.rs                    # Comparison implementation

docs/
  └── performance-regression-testing.md  # Main documentation

.github/workflows/
  └── README.md                          # Workflow documentation

.gitignore                                # Updated with benchmark artifacts
PERFORMANCE_REGRESSION_TESTING_IMPLEMENTATION.md  # This file
```

## Verification

To verify the implementation:

1. **Check workflow exists:**
   ```bash
   cat .github/workflows/performance-regression.yml
   ```

2. **Test comparison tool locally:**
   ```bash
   cd scripts/benchmark-compare
   cargo build
   cargo run -- --help
   ```

3. **Run helper script:**
   ```bash
   ./scripts/run-performance-regression.sh --help
   ```

4. **Review documentation:**
   ```bash
   cat docs/performance-regression-testing.md
   ```

## Summary

The performance regression testing system provides automated, objective performance monitoring for the Praxis game engine. It tracks three critical performance areas (draw call batching, GPU culling, descriptor allocation) and automatically fails CI if any regress by more than 10%. The system includes comprehensive tooling, documentation, and integration with GitHub Actions for seamless developer experience.
