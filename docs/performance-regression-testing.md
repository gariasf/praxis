# Performance Regression Testing

Automated performance regression testing system for the Praxis game engine, running on GitHub Actions.

## Overview

The performance regression testing system automatically runs a subset of critical benchmarks on every PR and compares the results against the main branch baseline. If any critical benchmark regresses by more than the configured threshold (10%), the CI build will fail.

## Architecture

### Components

1. **GitHub Actions Workflow** (`.github/workflows/performance-regression.yml`)
   - Triggers on PR and push to main
   - Runs benchmarks on both current and baseline branches
   - Compares results and posts comments on PRs
   - Fails if critical regressions detected

2. **Benchmark Comparison Tool** (`scripts/benchmark-compare/`)
   - Rust CLI tool for comparing Criterion results
   - Parses JSON estimates from Criterion
   - Generates human-readable reports
   - Identifies critical benchmark regressions

3. **Critical Benchmarks**
   - Multi-draw indirect batching (`graphics_optimization`)
   - GPU culling overhead (`graphics_optimization`)
   - Descriptor allocation rate (`descriptor_set_allocation`)

## Workflow Details

### Execution Flow

```
PR Created/Updated
│
├─► Checkout PR branch
├─► Run critical benchmarks (save as "current")
├─► Checkout main branch
├─► Run critical benchmarks (save as "main")
├─► Return to PR branch
├─► Compare results
├─► Post PR comment with results
└─► Fail if critical regressions detected (>10%)
```

### On Main Branch Push

When changes are pushed to main, the workflow:
1. Runs benchmarks and saves as baseline
2. Caches results for future PR comparisons
3. Does not perform comparison (nothing to compare against)

### On Pull Request

When a PR is created or updated:
1. Runs benchmarks on PR branch
2. Checks out and runs benchmarks on main branch
3. Compares the two sets of results
4. Posts formatted results as PR comment
5. Fails CI if critical benchmarks regress >10%

## Critical Benchmarks

### Multi-Draw Indirect Batching

**Benchmark:** `graphics_optimization::multi_draw_indirect_rendering`

**Purpose:** Measures the efficiency of draw call batching using multi-draw indirect rendering.

**Target:** Minimize CPU overhead per rendered object

**Key Metrics:**
- Draw call count reduction (should be 10-100x fewer than traditional)
- CPU time per frame with varying object counts (500, 750, 1000 objects)
- Material batching efficiency

**Why Critical:** Draw call batching is fundamental to rendering performance. Regressions here directly impact frame rates in complex scenes.

### GPU Culling Overhead

**Benchmark:** `graphics_optimization::gpu_vs_cpu_culling/gpu_culling`

**Purpose:** Measures the overhead of GPU compute-based frustum culling.

**Target:** <1ms for 10,000 objects

**Key Metrics:**
- GPU compute dispatch time
- CPU-side preparation overhead
- Total culling time including readback

**Why Critical:** GPU culling is essential for rendering large scenes. Excessive overhead negates the benefits of parallel culling.

### Descriptor Allocation Rate

**Benchmark:** `descriptor_set_allocation::descriptor_set_caching_lru`

**Purpose:** Measures descriptor set allocation efficiency with LRU caching.

**Target:** 100x+ reduction in allocations (100,000 → 100 over 1000 frames)

**Key Metrics:**
- Total allocations over 1000 frames
- Cache hit rate (should be >99.9% after warmup)
- Steady-state allocation rate (should be 0 after warmup)

**Why Critical:** Descriptor set allocation is a CPU bottleneck. Poor caching significantly impacts frame times and causes stutter.

## Regression Threshold

The current regression threshold is **10%**.

This means:
- If a critical benchmark is >10% slower than the baseline, the CI fails
- If a benchmark is >10% faster, it's marked as an improvement
- Changes within ±10% are considered noise/acceptable variance

### Rationale for 10% Threshold

- **Accounts for CI variance:** GitHub Actions runners have some performance variance
- **Meaningful regressions:** 10% is significant enough to matter in practice
- **Avoids false positives:** Tighter thresholds would fail too often due to noise
- **Industry standard:** Many performance-critical projects use 5-15% thresholds

## PR Comment Format

When benchmarks complete, the bot posts a comment like this:

```markdown
## 🔬 Performance Benchmark Results

### 📊 Benchmark Results (45 total)

#### ❌ Regressions (2)

| Benchmark | Baseline | Current | Change |
|-----------|----------|---------|--------|
| multi_draw_indirect_rendering/500_objects_20_materials_traditional | 234.56 µs | 267.23 µs | +13.94% |
| gpu_vs_cpu_culling/gpu_culling/1000 🚨 | 156.78 µs | 178.92 µs | +14.12% |

#### ✅ Improvements (3)

| Benchmark | Baseline | Current | Change |
|-----------|----------|---------|--------|
| descriptor_set_caching_lru/with_lru_caching | 89.45 µs | 76.23 µs | -14.79% |

#### ➡️ Unchanged (within ±10%) - 40

<details>
<summary>View unchanged benchmarks</summary>

[Collapsed list of unchanged benchmarks]

</details>

---

**Critical Benchmarks Tracked:**
- Multi-draw indirect batching (🎯 target: minimize draw call overhead)
- GPU culling overhead (🎯 target: <1ms for 10k objects)
- Descriptor allocation rate (🎯 target: 100x reduction with caching)

---

*Benchmarks run on: `Linux`*
*Threshold for regression: 10%*
```

## Local Testing

### Running Benchmarks Locally

```bash
# Run all critical benchmarks
cargo bench --bench graphics_optimization -- "multi_draw_indirect"
cargo bench --bench graphics_optimization -- "gpu_vs_cpu_culling/gpu_culling"
cargo bench --bench descriptor_set_allocation -- "descriptor_set_caching_lru"

# Run with baseline saving
cargo bench --bench graphics_optimization -- "multi_draw_indirect" --save-baseline main
```

### Comparing Baselines Locally

```bash
# 1. Save baseline from main branch
git checkout main
cargo bench --bench graphics_optimization -- "multi_draw_indirect" --save-baseline main

# 2. Make changes and save new baseline
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

## Benchmark Configuration

### Sample Size and Duration

The benchmarks use Criterion's default configuration:
- **Sample size:** 100 iterations (50 for slow benchmarks)
- **Measurement time:** 5 seconds per benchmark
- **Warm-up time:** 3 seconds

For GPU benchmarks, sample sizes are reduced due to longer execution times.

### Variance Reduction

To reduce variance in CI:
- **Consistent workload:** Fixed object counts and scene complexity
- **No randomization:** Deterministic object placement and data
- **CPU-only when possible:** Measure CPU overhead separately from GPU execution
- **Multiple runs:** Criterion automatically performs statistical analysis

## CI Performance

### Benchmark Duration

- **Multi-draw indirect:** ~2-3 minutes
- **GPU culling:** ~3-4 minutes  
- **Descriptor allocation:** ~2-3 minutes
- **Total runtime:** ~10-15 minutes

### Caching Strategy

The workflow uses `rust-cache` to cache:
- Cargo build artifacts
- Criterion baseline results
- Compiled benchmarks

This reduces subsequent runs to ~5-8 minutes.

## Troubleshooting

### CI Fails with "No Vulkan Device Available"

The workflow installs Mesa Vulkan drivers for software rendering. If this fails:
1. Check system dependency installation step
2. Verify `mesa-vulkan-drivers` package is available
3. Consider adding `LIBGL_ALWAYS_SOFTWARE=1` environment variable

### False Positives (Noise)

If you get false positives due to CI variance:
1. Re-run the workflow (GitHub Actions allows re-runs)
2. Check if the regression is consistent across multiple runs
3. Consider adjusting the threshold if false positives persist

### Baseline Not Found

If comparison fails with "baseline not found":
1. Ensure benchmarks ran successfully on main branch
2. Check that `--save-baseline` was used correctly
3. Verify Criterion output directory structure

## Future Enhancements

Potential improvements to the system:

1. **Historical tracking:** Store benchmark results over time
2. **Performance graphs:** Generate trend graphs for critical metrics
3. **Automatic baseline updates:** Update main baseline automatically
4. **Additional benchmarks:** Track more performance-critical operations
5. **Platform-specific baselines:** Different thresholds for different runners
6. **Benchmark subsets:** Run different benchmark sets based on changed files

## Related Documentation

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [GitHub Actions Workflow Syntax](https://docs.github.com/en/actions/reference/workflow-syntax-for-github-actions)
- [Praxis Performance Profiling](../docs/guides/profiling.md)
- [Benchmark Comparison Tool README](../scripts/benchmark-compare/README.md)
