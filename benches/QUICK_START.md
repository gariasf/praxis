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

# Graphics optimization benchmarks (NEW)
cargo bench --bench descriptor_set_allocation
cargo bench --bench staging_buffer
cargo bench --bench graphics_optimization

# Run with pattern matching
cargo bench -- physics          # All physics-related benchmarks
cargo bench -- raycast          # Just raycast benchmarks
cargo bench -- material_batching # Just material batching

# Save baseline for comparison
cargo bench -- --save-baseline main

# Compare against baseline
cargo bench -- --baseline main
```

## 🎨 Graphics Optimization Quick Start

To validate the 5-15% performance improvement target:

```bash
# Run the primary validation benchmark
cargo bench --bench graphics_optimization -- integrated_optimization_scenarios

# View results
open target/criterion/integrated_optimization_scenarios/report/index.html
```

Look for "baseline_current_approach" vs "optimized_batching_and_pooling" - optimized should be 5-15% faster.

Full graphics optimization suite:
```bash
# Run all three graphics benchmarks
cargo bench --bench descriptor_set_allocation && \
cargo bench --bench staging_buffer && \
cargo bench --bench graphics_optimization
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

### Graphics Optimization Results

Look for these specific improvements:
```
material_batching_optimization/no_batching         time: [3.0 ms]
material_batching_optimization/with_batching_10    time: [0.3 ms]  ✓ 10x improvement
```

```
integrated_optimization_scenarios/baseline         time: [5.0 ms]
integrated_optimization_scenarios/optimized        time: [4.5 ms]  ✓ 10% faster (target: 5-15%)
```

## 🎯 Performance Targets

### Core Engine

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

### Graphics Optimizations

| Benchmark | Target | Notes |
|-----------|--------|-------|
| descriptor_set_single_allocation | < 50μs | With pooling |
| descriptor_pooling_patterns/material | 10-20x faster | vs per-object |
| persistent_staging_buffer/reuse | 2-3x faster | vs create new |
| descriptor_set_caching/with_caching | Near-zero | After first frame |
| **integrated_optimization_scenarios/optimized** | **5-15% faster** | **Primary target** |

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
**Cause:** Vulkan not available (graphics benchmarks)  
**Fix:** Ensure Vulkan drivers installed
```bash
# Check Vulkan support
vulkaninfo
```

### High variance in results
**Cause:** Background processes, thermal throttling  
**Fix:** 
- Close other applications
- Disable CPU frequency scaling
- Run multiple times: `cargo bench -- --sample-size 500`

### Graphics benchmarks show no improvement
**Cause:** Object count too low, GPU bottleneck  
**Fix:**
- Increase object count (test with 200-500 objects)
- Use RenderDoc to profile GPU
- Verify optimizations are actually applied

### Benchmarks too slow
**Cause:** Debug build or unoptimized dependencies  
**Fix:** Benchmarks automatically use release mode, but check:
```bash
# Should use release
cargo bench --verbose
```

## 📈 Optimization Workflow

### Standard Workflow

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

### Graphics Optimization Workflow

1. **Baseline before optimizations**
   ```bash
   cargo bench --bench graphics_optimization -- --save-baseline before_opt
   ```

2. **Implement optimizations**
   - Material batching
   - Staging buffer pooling
   - Descriptor set caching
   - Dynamic uniform buffers

3. **Validate improvements**
   ```bash
   cargo bench --bench graphics_optimization -- --baseline before_opt
   ```

4. **Check component benchmarks**
   ```bash
   cargo bench --bench descriptor_set_allocation
   cargo bench --bench staging_buffer
   ```

5. **Verify 5-15% target met**
   - Look at "integrated_optimization_scenarios"
   - Compare "baseline" vs "optimized"
   - Should show 5-15% faster execution

## 📝 Quick Benchmark Checklist

### Before committing optimizations:
- [ ] All benchmarks still pass
- [ ] No regressions in other subsystems
- [ ] Performance gain is significant (> 5%)
- [ ] Results are consistent across runs
- [ ] Optimization is documented

### Before committing graphics optimizations:
- [ ] Descriptor set benchmarks show 10-20x reduction with batching
- [ ] Staging buffer benchmarks show 2-3x improvement with pooling
- [ ] Integrated benchmark shows 5-15% overall improvement
- [ ] Results validated on target hardware (discrete GPU preferred)
- [ ] No regressions in other graphics operations

## 🎨 Graphics Optimization Validation Checklist

Run these benchmarks and verify targets:

```bash
# 1. Material batching (target: 10-20x reduction)
cargo bench --bench descriptor_set_allocation -- pooling_patterns
# ✅ Check: material_pooling_pattern is 10x faster than per_frame_allocation_pattern

# 2. Staging buffer reuse (target: 2-3x improvement)
cargo bench --bench staging_buffer -- persistent
# ✅ Check: reuse_persistent_buffer is 2-3x faster than create_new_buffer_each_time

# 3. Descriptor set caching (target: near-zero after first frame)
cargo bench --bench graphics_optimization -- descriptor_set_caching
# ✅ Check: with_caching shows minimal per-frame cost

# 4. Integrated performance (target: 5-15% improvement)
cargo bench --bench graphics_optimization -- integrated_optimization_scenarios
# ✅ Check: optimized_batching_and_pooling is 5-15% faster than baseline_current_approach
```

## 🔗 More Information

- Full benchmark guide: `benches/README.md`
- Graphics optimization details: `benches/GRAPHICS_OPTIMIZATION.md`
- Criterion docs: https://bheisler.github.io/criterion.rs/book/
- Vulkan optimization: See `docs/optimization/graphics.md` (if exists)

## 💡 Tips

### For accurate results:
- Run benchmarks on idle system
- Use consistent hardware
- Close background applications
- Monitor CPU/GPU temperatures
- Run multiple iterations

### For graphics benchmarks:
- Prefer discrete GPU over integrated
- Ensure latest graphics drivers
- Test with realistic object counts (100-200)
- Validate on target hardware
- Use RenderDoc for GPU profiling

### For optimization work:
- Save baseline before starting
- Make incremental changes
- Validate each optimization independently
- Test integrated performance
- Document assumptions and results
