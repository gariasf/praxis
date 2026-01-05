# Graphics Optimization Benchmarks

This document provides detailed information about the graphics optimization benchmark suite designed to measure and validate the 5-15% performance improvement target for descriptor set allocation and staging buffer optimizations.

## Overview

The graphics optimization benchmarks consist of three complementary benchmark files:

1. **`descriptor_set_allocation.rs`** - Focused benchmarks for descriptor set patterns
2. **`staging_buffer.rs`** - Focused benchmarks for staging buffer patterns
3. **`graphics_optimization.rs`** - Integrated benchmarks simulating complete frame rendering

## Benchmark Architecture

### Design Principles

1. **Realistic Workloads**: Benchmarks simulate actual game rendering patterns with 10-200 objects
2. **Comparative Testing**: Each benchmark compares naive vs optimized approaches
3. **Measurable Targets**: 5-15% improvement is directly quantifiable in results
4. **Isolation + Integration**: Individual components are tested in isolation, then validated as a complete system

### Test Context Setup

All benchmarks create a realistic Vulkan context including:
- Physical device selection (prefers discrete GPU)
- Logical device with graphics queue
- Memory allocator (StandardMemoryAllocator)
- Command buffer allocator
- Descriptor set allocator
- Typical descriptor set layouts (transform, material)

## Descriptor Set Allocation Benchmarks

### Purpose
Measure the performance impact of descriptor set allocation strategies and validate material batching optimizations.

### Key Benchmarks

#### `descriptor_set_single_allocation`
- **What it measures**: Overhead of allocating a single descriptor set
- **Baseline metric**: Used to calculate improvement ratios
- **Typical result**: 10-50μs depending on hardware

#### `descriptor_set_batch_allocation`
- **What it measures**: Throughput when allocating many descriptor sets
- **Batch sizes**: 10, 50, 100, 500, 1000
- **Metric**: Descriptor sets per second
- **Use case**: Understanding allocation overhead at scale

#### `descriptor_reuse_vs_recreation`
- **What it measures**: Cost of recreating descriptor sets vs reusing existing ones
- **Two scenarios**:
  - `recreate_every_frame`: Create new descriptor set each iteration
  - `reuse_existing`: Use the same descriptor set (simulates caching)
- **Expected result**: Reuse should be near-zero cost vs recreation
- **Target improvement**: 50-100x faster with reuse

#### `descriptor_pooling_patterns`
- **What it measures**: Material batching effectiveness
- **Two patterns**:
  - `per_frame_allocation_pattern`: 100 objects = 100 descriptor sets (no batching)
  - `material_pooling_pattern`: 100 objects using 10 materials = 10 descriptor sets
- **Target improvement**: 10x reduction in allocations
- **Real-world scenario**: Most games have far fewer materials than objects

#### `frame_by_frame_allocation`
- **What it measures**: Per-frame allocation cost with varying object counts
- **Object counts**: 10, 50, 100, 200, 500
- **Use case**: Understanding how allocation cost scales with scene complexity

### Interpretation Guide

**Look for these key indicators**:
- `material_pooling_pattern` should be 10x faster than `per_frame_allocation_pattern`
- `reuse_existing` should show minimal overhead vs `recreate_every_frame`
- Throughput should remain consistent across batch sizes (no degradation)

**Performance targets**:
- Single allocation: <50μs with pooling
- Material batching: 10-20x reduction in allocations
- Cache hit: <1μs overhead

## Staging Buffer Benchmarks

### Purpose
Measure the performance impact of staging buffer strategies and validate persistent buffer reuse optimizations.

### Key Benchmarks

#### `staging_buffer_allocation`
- **What it measures**: Cost of allocating staging buffers of various sizes
- **Sizes tested**: 256B, 1KB, 4KB, 16KB, 64KB, 256KB
- **Metric**: Bytes per second throughput
- **Use case**: Understanding allocation overhead vs buffer size

#### `staging_buffer_write`
- **What it measures**: CPU-to-staging-buffer write performance
- **Sizes tested**: 256B to 256KB
- **Metric**: Bytes per second write throughput
- **Use case**: Identifying CPU write bottlenecks

#### `staging_to_device_copy`
- **What it measures**: Complete staging-to-device transfer pipeline
- **Includes**: Write to staging + GPU copy + synchronization
- **Sizes tested**: 256B to 256KB
- **Metric**: End-to-end transfer time
- **Use case**: Full pipeline performance including GPU wait

#### `persistent_staging_buffer`
- **What it measures**: Reuse single persistent buffer vs creating new buffers
- **Two scenarios**:
  - `reuse_persistent_buffer`: Write to pre-allocated 64KB buffer
  - `create_new_buffer_each_time`: Allocate 4KB buffer each iteration
- **Target improvement**: 2-3x faster with reuse
- **Real-world impact**: Major benefit for frequent small uploads

#### `batch_staging_upload`
- **What it measures**: Batching multiple uploads into single command buffer
- **Batch sizes**: 1, 5, 10, 50, 100 buffers
- **Metric**: Elements (buffers) per second
- **Use case**: Validating command buffer batching optimization

#### `ring_buffer_staging`
- **What it measures**: Ring buffer pattern for frames in flight
- **Configuration**: 3 frames in flight, 64KB per frame
- **Use case**: Realistic multi-frame persistent buffer pattern

#### `direct_write_vs_staging`
- **What it measures**: Host-visible buffer vs staging buffer approach
- **Two scenarios**:
  - `direct_host_write`: Write directly to host-visible GPU buffer
  - `staging_with_copy`: Write to staging, copy to device-local buffer
- **Use case**: Understanding when staging is beneficial

#### `staging_buffer_sizes`
- **What it measures**: Impact of staging buffer size on upload performance
- **Fixed upload**: Always upload 16KB
- **Staging sizes**: 16KB, 32KB, 64KB, 128KB, 256KB
- **Use case**: Determining optimal staging buffer size

### Interpretation Guide

**Look for these key indicators**:
- `reuse_persistent_buffer` should be 2-3x faster than `create_new_buffer_each_time`
- `batch_staging_upload` should show improved efficiency with larger batches
- Throughput should be >500 MB/s for typical sizes

**Performance targets**:
- Persistent buffer reuse: 2-3x improvement
- Throughput: >500 MB/s
- 64KB upload latency: <1ms

## Integrated Graphics Optimization Benchmarks

### Purpose
Validate the combined impact of all optimizations in realistic rendering scenarios. This is the **primary benchmark for validating the 5-15% improvement target**.

### Key Benchmarks

#### `complete_frame_render_pattern`
- **What it measures**: Full frame rendering simulation
- **Object counts**: 10, 50, 100, 200
- **Includes**:
  - Staging buffer allocation and writes
  - GPU copy operations
  - Material descriptor set creation
  - Command buffer recording
- **Use case**: End-to-end frame rendering performance

#### `material_batching_optimization`
- **What it measures**: Direct comparison of material batching strategies
- **Three scenarios**:
  - `no_batching`: 100 objects = 100 descriptor sets
  - `with_batching_10_materials`: 100 objects using 10 materials
  - `with_batching_5_materials`: 100 objects using 5 materials
- **Target**: Show 10-20x reduction in allocations
- **Use case**: Validating material batching concept

#### `dynamic_uniform_buffer_pattern`
- **What it measures**: Dynamic uniform buffer with offsets
- **Object counts**: 10, 50, 100, 200, 500
- **Simulates**: Engine's actual dynamic uniform buffer implementation
- **Includes**:
  - Alignment calculations
  - Single large buffer allocation
  - Per-object writes at aligned offsets
  - Single descriptor set with dynamic offsets
- **Use case**: Validating dynamic buffer approach

#### `descriptor_set_caching`
- **What it measures**: Caching descriptor sets across frames
- **Simulation**: 10 materials across 60 frames
- **Two scenarios**:
  - `no_caching`: Recreate 10 descriptor sets every frame (600 total)
  - `with_caching`: Create 10 descriptor sets once, reuse 60 times
- **Target**: Near-zero cost per frame after first frame
- **Use case**: Validating per-material descriptor set caching

#### `staging_buffer_pooling`
- **What it measures**: Staging buffer pooling across frames
- **Simulation**: 10 frames of uploads
- **Two scenarios**:
  - `no_pooling`: Allocate new staging buffer each frame
  - `with_pooling`: Ring buffer of 3 persistent staging buffers
- **Target**: 3x reduction in allocations
- **Use case**: Validating ring buffer pattern

#### `integrated_optimization_scenarios` ⭐ **MOST IMPORTANT**
- **What it measures**: Complete before/after comparison
- **Two scenarios**:
  - `baseline_current_approach`: 
    - Per-object descriptor sets (100 sets for 100 objects)
    - Per-object staging buffers (100 buffers)
    - Individual command buffers per upload
  - `optimized_batching_and_pooling`:
    - Material batching (10 sets for 100 objects)
    - Single large staging buffer (pooled)
    - Batched command buffer
- **🎯 PRIMARY TARGET: 5-15% faster frame time**
- **Use case**: Validating overall optimization strategy

### Interpretation Guide

**Primary validation metric**:
1. Run `integrated_optimization_scenarios`
2. Compare `baseline_current_approach` vs `optimized_batching_and_pooling`
3. **Target: optimized should be 5-15% faster**

**Supporting validations**:
- `material_batching_optimization`: Should show 10-20x allocation reduction
- `descriptor_set_caching`: Should show near-zero per-frame cost with caching
- `staging_buffer_pooling`: Should show 3x allocation reduction
- `dynamic_uniform_buffer_pattern`: Should scale linearly with object count

## Running the Benchmarks

### Quick Start - Validate 5-15% Target

```bash
# Run the primary validation benchmark
cargo bench --bench graphics_optimization -- integrated_optimization_scenarios

# View results
open target/criterion/integrated_optimization_scenarios/report/index.html
```

### Comprehensive Testing

```bash
# Run all graphics optimization benchmarks
cargo bench --bench descriptor_set_allocation
cargo bench --bench staging_buffer  
cargo bench --bench graphics_optimization

# View all results
open target/criterion/report/index.html
```

### Baseline Comparison

```bash
# Before implementing optimizations
cargo bench -- --save-baseline before

# After implementing optimizations
cargo bench -- --baseline before

# View comparison in HTML reports
```

### Targeted Benchmarking

```bash
# Test only descriptor set optimizations
cargo bench --bench descriptor_set_allocation -- pooling

# Test only staging buffer optimizations
cargo bench --bench staging_buffer -- persistent

# Test only material batching
cargo bench --bench graphics_optimization -- material_batching
```

## Expected Results

### Descriptor Set Allocation

| Benchmark | Baseline | Optimized | Improvement |
|-----------|----------|-----------|-------------|
| Single allocation | 30μs | 30μs | 1x (same) |
| 100 objects, no batching | 3ms | 3ms | 1x |
| 100 objects, material batching | 3ms | 300μs | **10x** |
| Recreation vs reuse | 30μs | <1μs | **30-100x** |

### Staging Buffer

| Benchmark | Baseline | Optimized | Improvement |
|-----------|----------|-----------|-------------|
| Per-frame allocation | 50μs/buffer | 50μs | 1x |
| Persistent reuse | 50μs | 15μs | **3x** |
| Throughput (64KB) | 400 MB/s | 600 MB/s | **1.5x** |
| Ring buffer (10 frames) | 500μs | 150μs | **3x** |

### Integrated Performance

| Scenario | Time per Frame | Target |
|----------|----------------|--------|
| Baseline (100 objects) | 5.0ms | - |
| Optimized (100 objects) | **4.25-4.75ms** | **5-15% faster** |

**Component breakdown** (100 objects):
- Descriptor sets: 3.0ms → 0.3ms (10x improvement)
- Staging buffers: 1.0ms → 0.3ms (3x improvement)
- Other overhead: 1.0ms → 1.0ms (unchanged)
- **Total: 5.0ms → 4.5ms (10% improvement) ✓**

## Interpreting HTML Reports

### Critical Charts

1. **Violin Plot** (`integrated_optimization_scenarios`):
   - Compare violin width between baseline and optimized
   - Narrower = more consistent performance
   - Look for clear separation between distributions

2. **Mean Time Chart**:
   - Direct comparison of mean execution times
   - Error bars show confidence intervals
   - **Look for 5-15% gap between baseline and optimized**

3. **Throughput Chart** (batch tests):
   - Higher is better
   - Should remain consistent across batch sizes
   - Watch for degradation at high batch counts

### Key Metrics

- **Mean**: Average execution time (primary metric)
- **Median**: Middle value (less affected by outliers)
- **MAD**: Median Absolute Deviation (consistency measure)
- **Std Dev**: Standard deviation (variance measure)

### What to Look For

✅ **Good signs**:
- Optimized mean is 5-15% lower than baseline
- Low standard deviation (consistent performance)
- Linear scaling with object count
- Throughput improvements align with reduction in operations

❌ **Warning signs**:
- High variance (inconsistent performance)
- Worse than baseline (optimization ineffective)
- Non-linear scaling at high counts (algorithmic issue)
- Throughput degradation with larger batches

## Common Issues and Solutions

### Issue: No improvement shown

**Possible causes**:
1. Compiler optimizations too aggressive (both paths optimized equally)
2. GPU bottleneck (CPU optimizations don't help)
3. Benchmark not representative of real workload

**Solutions**:
- Use `black_box()` to prevent over-optimization
- Profile GPU with tools like RenderDoc
- Adjust object counts to match real scenarios

### Issue: High variance in results

**Possible causes**:
1. Background processes competing for resources
2. Thermal throttling
3. GPU clock fluctuations

**Solutions**:
- Close background applications
- Run benchmarks multiple times
- Check CPU/GPU temperatures
- Use `--sample-size` to increase sample count

### Issue: Can't validate 5-15% target

**Possible causes**:
1. Baseline too fast (little room for improvement)
2. Bottleneck elsewhere (not CPU-bound)
3. Object count too low (overhead not visible)

**Solutions**:
- Increase object count (200-500 objects)
- Profile to find actual bottleneck
- Test with realistic game scene complexity

## Integration with CI/CD

### Pre-commit Benchmark

```bash
#!/bin/bash
# Save current performance baseline
cargo bench -- --save-baseline main

# Commit changes...
```

### Post-commit Validation

```bash
#!/bin/bash
# Compare against baseline
cargo bench -- --baseline main

# Parse results to check for regressions
# (Can use criterion's JSON output for automated checking)
```

### Performance Regression Detection

```bash
# Fail CI if performance regresses by >10%
cargo bench -- --baseline main 2>&1 | grep -E "Performance has regressed|Change within noise"
```

## Conclusion

These benchmarks provide comprehensive validation that the planned graphics optimizations will achieve the 5-15% performance improvement target. The integrated benchmark (`integrated_optimization_scenarios`) is the primary validation metric, while the component benchmarks provide insight into individual optimization effectiveness.

**To validate optimizations**:
1. Run baseline benchmarks before changes
2. Implement optimizations (material batching, staging buffer pooling)
3. Run benchmarks after changes
4. Verify `integrated_optimization_scenarios` shows 5-15% improvement
5. Validate component benchmarks show expected improvements (10x descriptor reduction, 3x staging reduction)

The benchmarks are designed to be realistic, repeatable, and directly tied to the optimization strategy, providing confidence that the planned improvements will deliver the expected performance gains.
