# Benchmark Comparison Tool

A specialized tool for comparing Criterion benchmark results to detect performance regressions in the Praxis game engine.

## Purpose

This tool is designed to automatically detect performance regressions in critical performance areas:

- **Multi-draw indirect batching**: Ensures draw call overhead remains minimal
- **GPU culling overhead**: Verifies GPU compute culling stays under performance targets
- **Descriptor allocation rate**: Confirms efficient caching reduces allocation pressure

## Usage

### Basic Usage

```bash
cargo run --manifest-path scripts/benchmark-compare/Cargo.toml -- \
    --baseline-dir target/criterion \
    --current-baseline main \
    --new-baseline current \
    --threshold 10.0
```

### With Output Files

```bash
cargo run --manifest-path scripts/benchmark-compare/Cargo.toml -- \
    --baseline-dir target/criterion \
    --current-baseline main \
    --new-baseline current \
    --threshold 10.0 \
    --output benchmark-results/comparison.json \
    --output-markdown benchmark-results/comparison.md
```

## Arguments

- `--baseline-dir`: Directory containing Criterion benchmark results (default: `target/criterion`)
- `--current-baseline`: Name of the baseline to compare against (default: `main`)
- `--new-baseline`: Name of the new baseline being tested (default: `current`)
- `--threshold`: Regression threshold as a percentage (default: `10.0`)
- `--output`: Optional JSON output file path
- `--output-markdown`: Optional Markdown output file path

## Exit Codes

- `0`: No critical regressions detected
- `1`: Critical regressions detected (will fail CI)

## Critical Benchmarks

The tool specifically tracks these critical performance metrics:

1. **Multi-draw indirect rendering** (`multi_draw_indirect_rendering`)
   - Measures draw call batching efficiency
   - Target: Minimize CPU overhead per object

2. **GPU culling** (`gpu_vs_cpu_culling`)
   - Measures GPU compute culling performance
   - Target: <1ms for 10,000 objects

3. **Descriptor set caching** (`descriptor_set_caching_lru`)
   - Measures allocation rate with LRU caching
   - Target: 100x+ reduction in allocations

## Output Formats

### Console Output

The tool prints a colored summary to the console:
- ❌ Red for regressions
- ✅ Green for improvements
- ⚠️ Yellow for critical benchmark names

### JSON Output

Structured data containing:
- Total benchmark count
- Lists of regressions, improvements, and unchanged benchmarks
- Regression threshold
- Critical regression flag

### Markdown Output

GitHub-friendly report format with:
- Summary statistics
- Tables for regressions and improvements
- Collapsible section for unchanged benchmarks
- Critical benchmark indicators (🚨)

## Integration with CI

This tool is automatically run by the `performance-regression.yml` GitHub Actions workflow on every PR. See `.github/workflows/performance-regression.yml` for details.

## Development

### Building

```bash
cd scripts/benchmark-compare
cargo build --release
```

### Testing Locally

1. Run benchmarks on main branch:
   ```bash
   cargo bench --bench graphics_optimization -- "multi_draw_indirect" --save-baseline main
   ```

2. Make changes to the code

3. Run benchmarks on current branch:
   ```bash
   cargo bench --bench graphics_optimization -- "multi_draw_indirect" --save-baseline current
   ```

4. Compare:
   ```bash
   cargo run --manifest-path scripts/benchmark-compare/Cargo.toml
   ```
