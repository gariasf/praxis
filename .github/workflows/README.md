# GitHub Actions Workflows

This directory contains CI/CD workflows for the Praxis game engine.

## Active Workflows

### `rust-ci.yml` - Standard CI Pipeline

Runs on every PR and push to main.

**Jobs:**
- `check`: Cargo check, format, and clippy
- `test`: Run test suite
- `build_examples`: Build all examples

**Duration:** ~5-10 minutes

### `performance-regression.yml` - Performance Testing

Runs on every PR and push to main.

**Jobs:**
- `benchmark`: Run critical performance benchmarks and compare against main branch

**Critical Benchmarks Tracked:**
- Multi-draw indirect batching performance
- GPU culling overhead  
- Descriptor allocation rate with LRU caching

**Regression Threshold:** 10%

**Duration:** ~10-15 minutes

**Failure Conditions:**
- Any critical benchmark regresses by >10% compared to main branch
- Benchmark execution fails

**PR Comments:**
- Automatically posts benchmark comparison results to PR
- Highlights regressions and improvements
- Marks critical benchmarks with 🚨

## Workflow Configuration

### Triggers

Both workflows trigger on:
- Pull requests to `main` branch
- Direct pushes to `main` branch

### Caching

All workflows use `Swatinem/rust-cache@v2` to cache:
- Cargo registry
- Cargo build artifacts
- Target directory

The performance workflow also caches Criterion baseline results.

### System Dependencies

Both workflows install:
- `libasound2-dev` (audio support)
- `libudev-dev` (device support)
- `pkg-config` (build tool)

Performance workflow additionally installs:
- `libvulkan-dev` (Vulkan development headers)
- `vulkan-validationlayers` (Vulkan validation)
- `mesa-vulkan-drivers` (software Vulkan rendering)

## Local Testing

### Test CI Checks Locally

```bash
# Run checks
cargo check --all --features headless
cargo fmt --all -- --check
cargo clippy --all --features headless -- -D warnings

# Run tests
cargo test --workspace --features headless

# Build examples
cargo build --examples --features headless
```

### Test Performance Benchmarks Locally

See [Performance Regression Testing Documentation](../../docs/performance-regression-testing.md) for detailed instructions.

Quick version:
```bash
# Run critical benchmarks
cargo bench --bench graphics_optimization -- "multi_draw_indirect"
cargo bench --bench graphics_optimization -- "gpu_vs_cpu_culling"
cargo bench --bench descriptor_set_allocation -- "descriptor_set_caching_lru"
```

## Adding New Workflows

When adding a new workflow:

1. Create `.github/workflows/your-workflow.yml`
2. Use existing workflows as templates
3. Add caching with `rust-cache` action
4. Install necessary system dependencies
5. Document in this README
6. Test locally using `act` or similar tools

## Troubleshooting

### Workflow Fails on System Dependencies

If system dependency installation fails:
1. Check Ubuntu package availability
2. Update package names if needed
3. Add PPA repositories if required

### Cache Issues

If caching causes problems:
1. Clear cache via GitHub UI (Actions → Caches)
2. Update `rust-cache` action version
3. Adjust cache key in workflow

### Performance Benchmark Variance

If benchmarks show high variance:
1. Re-run the workflow (variance is expected)
2. Check for system load on GitHub runners
3. Consider adjusting regression threshold
4. Use local benchmarks for comparison

## Related Documentation

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Performance Regression Testing](../../docs/performance-regression-testing.md)
- [Benchmark Comparison Tool](../../scripts/benchmark-compare/README.md)
- [Praxis CI/CD Setup](../../docs/ci-cd-setup.md) (if exists)
