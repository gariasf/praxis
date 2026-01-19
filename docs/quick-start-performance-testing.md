# Quick Start: Performance Regression Testing

A 5-minute guide to using the performance regression testing system.

## What It Does

Automatically tests if your changes make critical benchmarks >10% slower:
- **Multi-draw indirect batching** (draw calls)
- **GPU culling** (frustum culling overhead)
- **Descriptor allocation** (caching efficiency)

## For PR Authors

### Automatic Testing

When you create a PR:
1. CI automatically runs performance benchmarks
2. Compares your branch against main
3. Posts results as a comment
4. ❌ Fails if critical benchmarks regress >10%
5. ✅ Passes if performance is acceptable

**No action needed!** Just watch for the bot comment.

### Reading Results

Look for the bot comment on your PR:

```markdown
## 🔬 Performance Benchmark Results

#### ❌ Regressions (2)
| Benchmark | Baseline | Current | Change |
|-----------|----------|---------|--------|
| multi_draw_indirect_rendering 🚨 | 234.56 µs | 267.23 µs | +13.94% |

#### ✅ Improvements (3)
...
```

**🚨 = Critical benchmark** (will fail CI if regressed)

### If CI Fails

1. **Review the regression:** Is it expected? Justified by other improvements?
2. **Investigate the cause:** Profile your changes to find the bottleneck
3. **Fix or document:** Either optimize the code or explain why the regression is acceptable
4. **Re-run tests:** Push updates and let CI verify the fix

### Running Tests Locally

Before pushing:

```bash
# Quick test (Linux/Mac)
./scripts/run-performance-regression.sh

# Quick test (Windows)
.\scripts\run-performance-regression.ps1

# Manual test
cargo bench --bench graphics_optimization -- "multi_draw_indirect"
```

## For Reviewers

### Checking Performance

1. Look for the bot's PR comment
2. Check for 🚨 critical regressions
3. Verify improvements are real (not just noise)
4. Ask author to explain unexpected changes

### Acceptable Regressions

Sometimes regressions are justified:
- New features that add necessary overhead
- Code clarity improvements worth minor performance cost
- Temporary regressions during refactoring

**Always discuss with the author!**

## Common Scenarios

### Scenario 1: All Green ✅

```
✅ No performance regressions detected
```

**Action:** Approve the PR (from a performance perspective)

### Scenario 2: Minor Changes

```
Improvements: 5
Unchanged: 40
Regressions: 0
```

**Action:** Review improvements to ensure they're real gains

### Scenario 3: Critical Regression 🚨

```
❌ CRITICAL REGRESSIONS DETECTED
- 🚨 gpu_culling/10000: +12.45% slower
```

**Action:** Ask author to investigate and fix

### Scenario 4: Mixed Results

```
Improvements: 3
Regressions: 2 (none critical)
Unchanged: 40
```

**Action:** Review trade-offs with author

## Quick Commands

```bash
# Run all critical benchmarks
cargo bench --bench graphics_optimization -- "multi_draw_indirect"
cargo bench --bench graphics_optimization -- "gpu_vs_cpu_culling"
cargo bench --bench descriptor_set_allocation

# Run specific benchmark
cargo bench --bench graphics_optimization -- "multi_draw_indirect/500_objects"

# Save baseline for comparison
cargo bench --bench graphics_optimization -- "multi_draw_indirect" --save-baseline main

# Compare two baselines
cargo run --manifest-path scripts/benchmark-compare/Cargo.toml -- \
    --current-baseline main \
    --new-baseline current
```

## Thresholds

- **>10% slower:** ❌ Regression (fails CI if critical)
- **±10%:** ➡️ Unchanged (acceptable variance)
- **>10% faster:** ✅ Improvement

## Critical vs Non-Critical

**Critical benchmarks** (marked 🚨):
- Multi-draw indirect batching
- GPU culling overhead
- Descriptor allocation rate

**Why they're critical:**
- Core rendering performance
- Direct impact on frame rate
- Compound with scale (thousands of objects)

**Non-critical benchmarks:**
- Everything else
- Still important, but won't fail CI

## Getting Help

- **Documentation:** `docs/performance-regression-testing.md`
- **Tool Help:** `./scripts/run-performance-regression.sh --help`
- **CI Issues:** Check `.github/workflows/README.md`

## Tips

1. **Test before pushing:** Run benchmarks locally to catch issues early
2. **Understand the impact:** 10% on a hot path matters more than 50% on a cold path
3. **Profile your changes:** Use `cargo flamegraph` to understand performance
4. **Compare apples to apples:** Ensure test conditions are consistent
5. **Don't over-optimize:** 10% threshold allows for reasonable variance

## What Not To Do

❌ **Don't disable tests:** They're there for a reason
❌ **Don't ignore regressions:** They compound over time
❌ **Don't micro-optimize prematurely:** Focus on critical paths
❌ **Don't trust single runs:** CI runs multiple iterations for accuracy

## Next Steps

- Read full documentation: `docs/performance-regression-testing.md`
- Explore benchmark code: `benches/graphics_optimization.rs`
- Learn about Criterion: https://bheisler.github.io/criterion.rs/
- Profile your code: `cargo install cargo-flamegraph`

---

**Remember:** Performance testing is about preventing regressions, not achieving perfection. The goal is to catch significant changes, not every microsecond.
