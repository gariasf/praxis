# Benchmark Quick Start Guide

Quick reference for running and interpreting Praxis engine benchmarks.

## 🚀 Quick Commands

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench mesh_upload
cargo bench --bench render_loop  
cargo bench --bench physics_step
cargo bench --bench transform_propagation
cargo bench --bench asset_loading
cargo bench --bench scene_serialization

# Run with pattern matching
cargo bench -- physics          # All physics-related benchmarks
cargo bench -- raycast          # Just raycast benchmarks

# Save baseline for comparison
cargo bench -- --save-baseline main

# Compare against baseline
cargo bench -- --baseline main
```

## 📊 Quick Interpretation

### Good Results
```
mesh_upload/1000    time:   [1.23 ms 1.25 ms 1.27 ms]
                    change: [-2.5% -1.2% +0.3%]  ✓ No significant change
```
- ✅ Low variance (narrow confidence interval)
- ✅ No regression (change near 0%)
- ✅ Few outliers

### Bad Results  
```
physics_step/100    time:   [25.1 ms 28.7 ms 32.4 ms]
                    change: [+15.2% +18.9% +22.1%]  ⚠️ Performance regressed
```
- ❌ High variance (wide confidence interval)
- ❌ Significant regression (+18.9%)
- ❌ Exceeds frame budget (> 16.67ms for 60 FPS)

## 🎯 Performance Targets

| Benchmark | Target | Critical Threshold |
|-----------|--------|-------------------|
| mesh_upload/10000 | < 5ms | < 10ms |
| camera_matrix_updates/10 | < 100μs | < 500μs |
| physics_step/100 | < 16ms | < 25ms |
| transform_propagation/1000 | < 1ms | < 2ms |
| obj_parsing/5000 | < 10ms | < 20ms |
| gltf_parsing/5000 | < 20ms | < 40ms |
| scene_serialization/100 | < 5ms | < 10ms |
| scene_deserialization/100 | < 10ms | < 20ms |

## 🔍 Finding Bottlenecks

### 1. Run benchmarks
```bash
cargo bench --bench physics_step
```

### 2. Check HTML report
```bash
# Open in browser
target/criterion/physics_step/report/index.html
```

### 3. Look for
- Highest mean times
- Worst-case outliers
- Non-linear scaling patterns
- Unexpected regressions

### 4. Profile the slow benchmark
```bash
# Linux with perf
cargo bench --bench physics_step -- --profile-time=10
perf record -F 99 -g target/release/deps/physics_step-*
perf report

# Or use flamegraph
cargo install flamegraph
cargo flamegraph --bench physics_step
```

## 🛠️ Common Issues

### "No device available" error
**Cause:** Vulkan not available (mesh_upload benchmark)  
**Fix:** Ensure Vulkan drivers installed or skip: `cargo bench --bench render_loop`

### High variance in results
**Cause:** Background processes, thermal throttling  
**Fix:** 
- Close other applications
- Disable CPU frequency scaling
- Run multiple times: `cargo bench -- --sample-size 500`

### Benchmarks too slow
**Cause:** Debug build or unoptimized dependencies  
**Fix:** Benchmarks automatically use release mode, but check:
```bash
# Should use release
cargo bench --verbose
```

## 📈 Optimization Workflow

1. **Establish baseline**
   ```bash
   cargo bench -- --save-baseline before
   ```

2. **Make changes** to code

3. **Compare results**
   ```bash
   cargo bench -- --baseline before
   ```

4. **Check for regressions** in other benchmarks
   ```bash
   cargo bench
   ```

5. **Document** what you changed and why

## 📝 Quick Benchmark Checklist

Before committing optimizations:
- [ ] All benchmarks still pass
- [ ] No regressions in other subsystems
- [ ] Performance gain is significant (> 5%)
- [ ] Results are consistent across runs
- [ ] Optimization is documented

## 🔗 More Information

- Full guide: `docs/benchmarking.md`
- Benchmark README: `benches/README.md`
- Criterion docs: https://bheisler.github.io/criterion.rs/book/
