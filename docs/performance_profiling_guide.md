# Performance Profiling Guide

This guide explains how to use the comprehensive performance profiling system in Praxis to validate optimization improvements and ensure there are no regressions.

## Overview

The Praxis engine includes several optimization techniques:

1. **GPU Frustum Culling** - Culls objects outside the camera view using compute shaders
2. **GPU LOD Selection** - Automatically selects level-of-detail based on distance
3. **Hi-Z Occlusion Culling** - Culls objects hidden behind other objects
4. **Mesh Instancing** - Renders many identical objects efficiently
5. **Mesh Streaming** - Loads meshes asynchronously in the background
6. **Texture Caching** - Reuses textures to minimize memory usage

## Running the Comprehensive Performance Test

The `performance_profiling_comprehensive` example creates a large scene (10,000+ objects) and measures performance with different optimization combinations.

### Basic Usage

```bash
# Run in release mode for accurate measurements
cargo run --release --example performance_profiling_comprehensive
```

**Important**: Always use `--release` mode for performance profiling. Debug builds are 10-100x slower and don't reflect actual game performance.

### Controls

| Key | Action |
|-----|--------|
| 1-7 | Switch between optimization levels |
| Space | Reset to baseline (no optimizations) |
| P | Print performance comparison report |
| E | Export/save Chrome trace |
| I | Print current optimization state |
| W/A/S/D | Move camera |
| Q/E | Move camera up/down |
| Arrow Keys | Rotate camera |
| ESC | Exit |

### Optimization Levels

Press the number keys to test different optimization combinations:

| Key | Level | Optimizations Enabled |
|-----|-------|-----------------------|
| 1 | Baseline | None (worst case) |
| 2 | Frustum Culling | GPU frustum culling only |
| 3 | Frustum + LOD | + LOD system |
| 4 | Frustum + LOD + Occlusion | + Occlusion culling |
| 5 | + Instancing | + Mesh instancing |
| 6 | + Streaming | + Mesh streaming |
| 7 | Full Stack | All optimizations (best case) |

## Expected Performance Results

### Mid-Range GPU Baseline

Reference hardware: NVIDIA GTX 1060 6GB / AMD RX 580 8GB

| Optimization Level | Expected FPS | Frame Time | Speedup | Notes |
|-------------------|--------------|------------|---------|-------|
| Baseline | 10-15 FPS | 66-100ms | 1.0x | All 10,000 objects drawn |
| + Frustum Culling | 30-40 FPS | 25-33ms | 2.5x | ~70% objects culled |
| + LOD | 45-55 FPS | 18-22ms | 3.5x | Reduced triangle count |
| + Occlusion | 60-70 FPS | 14-16ms | 5.0x | Additional 20-30% culled |
| + Instancing | 90-110 FPS | 9-11ms | 7.5x | 90% fewer draw calls |
| + Streaming | 100-120 FPS | 8-10ms | 8.5x | Lower memory pressure |
| **Full Stack** | **120-140 FPS** | **7-8ms** | **10x** | All optimizations working |

### High-End GPU

Reference hardware: NVIDIA RTX 3070 / AMD RX 6800

| Optimization Level | Expected FPS | Frame Time | Notes |
|-------------------|--------------|------------|-------|
| Baseline | 25-35 FPS | 28-40ms | GPU bound on draw calls |
| Full Stack | 240-300 FPS | 3.3-4.2ms | Smooth 240Hz capable |

### Low-End GPU / Integrated Graphics

Reference hardware: Intel UHD Graphics 630 / AMD Vega 8

| Optimization Level | Expected FPS | Frame Time | Notes |
|-------------------|--------------|------------|-------|
| Baseline | 5-8 FPS | 125-200ms | Unplayable |
| Full Stack | 45-60 FPS | 16-22ms | Playable with optimizations |

## Validation Criteria

### 1. No Performance Regressions

Each optimization should show **measurable improvement**:

- Each level should be >= previous level's performance
- Full stack should be at least **8-10x faster** than baseline
- No optimization should reduce FPS

### 2. Correct Culling Behavior

Objects should never be **falsely culled** (visible objects incorrectly hidden):

- Move camera around the scene
- All visible objects should render correctly
- No "popping" artifacts (objects appearing/disappearing incorrectly)

### 3. LOD Transitions

LOD system should work smoothly:

- Objects should transition between detail levels based on distance
- No abrupt visual "pops" when LOD changes (with transitions enabled)
- High detail near camera, low detail far away

### 4. Memory Stability

Memory usage should remain stable:

- No continuous memory growth over time
- Memory should stabilize after initial warmup
- Streaming should not cause memory spikes

### 5. Culling Efficiency

Expected culling percentages:

| Optimization | Typical Culling % | Range |
|--------------|------------------|-------|
| Frustum Culling | 60-75% | 50-80% |
| + Occlusion | 75-85% | 70-90% |
| Full Stack | 80-90% | 75-95% |

## Profiling Workflow

### Step 1: Baseline Measurement

Start the demo and let it run for 60 frames to stabilize:

```
Press 1 - Start baseline measurement
Wait 2-3 seconds for warmup
Press P - Print performance snapshot
```

Record the FPS and frame time.

### Step 2: Test Each Optimization

For each optimization level (2-7):

```
Press 2-7 - Switch to optimization level
Wait 2-3 seconds for warmup
Press P - Print performance snapshot
```

### Step 3: Generate Comparison Report

After testing all levels:

```
Press P - Print full comparison report
```

This shows a table comparing all tested optimization levels.

### Step 4: Export Chrome Trace (Optional)

For detailed analysis:

```
Press E - Start trace export
Run the scene for 5-10 seconds
Press E - Save trace to performance_trace.json
```

Open the trace file in Chrome:

1. Open Chrome browser
2. Navigate to `chrome://tracing`
3. Click "Load" and select `performance_trace.json`
4. Analyze CPU/GPU timelines, memory usage, and bottlenecks

## Interpreting Results

### Good Performance Profile

✓ Each optimization shows measurable improvement (10-50% FPS gain)  
✓ Full stack achieves 8-10x speedup over baseline  
✓ Memory usage is stable (<5% variation over 1000 frames)  
✓ Culling efficiency is 60-90% depending on camera view  
✓ No visual artifacts or false culling  

### Warning Signs

⚠️ **Optimization shows no improvement** - May not be working correctly  
⚠️ **FPS decreases with optimization** - Implementation bug or overhead too high  
⚠️ **Memory continuously grows** - Memory leak in streaming system  
⚠️ **Visible objects disappear** - False culling bug in frustum/occlusion  
⚠️ **LOD pops are visible** - Transition system not working  

## Troubleshooting

### Low FPS Even With Optimizations

**Problem**: Full stack performance is much lower than expected.

**Possible causes**:
1. Running in debug mode instead of release
   - Solution: Use `cargo run --release`
2. VSync is enabled (capping at 60 FPS)
   - Solution: Check graphics settings
3. CPU bottleneck (too many objects on CPU side)
   - Solution: Check CPU usage in profiler
4. Driver issues or old GPU
   - Solution: Update graphics drivers

### Culling Not Working

**Problem**: Frustum/occlusion culling shows little or no improvement.

**Possible causes**:
1. All objects are already visible (camera seeing entire scene)
   - Solution: Move camera to look at occluders
2. Culling compute shader not running
   - Solution: Check GPU profiler for compute pass
3. Culling results not being used in render pass
   - Solution: Verify indirect draw buffers are bound

### Memory Leaks

**Problem**: Memory usage grows continuously over time.

**Possible causes**:
1. Streaming system not releasing old meshes
   - Solution: Check mesh cache eviction policy
2. Texture cache growing without bounds
   - Solution: Set maximum cache size
3. Profiler accumulating too much data
   - Solution: Reset profiler periodically

### False Culling

**Problem**: Visible objects disappear incorrectly.

**Possible causes**:
1. Frustum calculation is incorrect
   - Solution: Enable frustum visualization (F key)
2. Bounding volumes are too small
   - Solution: Check object bounds in debug view
3. Occlusion test is too aggressive
   - Solution: Increase Hi-Z bias threshold

## Performance Optimization Tips

Based on profiling results, here are optimization strategies:

### If CPU-Bound (High CPU Time, Low GPU Time)

1. Enable GPU frustum culling to move work to GPU
2. Use mesh instancing to reduce draw calls
3. Batch objects with same material
4. Use LOD to reduce number of objects processed

### If GPU-Bound (High GPU Time, Low CPU Time)

1. Enable LOD to reduce triangle count
2. Use occlusion culling to skip hidden geometry
3. Reduce shader complexity
4. Lower resolution or disable expensive effects

### If Memory-Bound

1. Enable texture caching and compression
2. Use mesh streaming to load on-demand
3. Implement more aggressive LOD bias
4. Share materials between objects

### If Draw Call Bound

1. Enable instancing for duplicate objects
2. Batch objects by material
3. Use multi-draw indirect rendering
4. Merge static geometry

## Advanced Profiling

### GPU Profiling

To enable GPU timestamp queries:

```rust
let mut profiler = Profiler::new(config);
profiler.setup_gpu_profiler(gpu_profiler);
```

This tracks exact GPU execution time for each render pass.

### System Profiling

To profile individual ECS systems:

```rust
fn my_system() {
    let _scope = ProfileScope::new("my_system");
    // System logic here
}
```

The profiler automatically tracks time spent in each system.

### Memory Profiling

To track specific allocations:

```rust
let tracker = profiler.memory_tracker();
let alloc_id = tracker.track_allocation(
    size_in_bytes,
    "allocation_name".to_string(),
    "category".to_string()
);

// Later, when freeing:
tracker.track_deallocation(alloc_id);
```

### Bottleneck Detection

The profiler automatically identifies bottlenecks:

```rust
let bottlenecks = profiler.system_profiler().identify_bottlenecks();
for bottleneck in bottlenecks {
    println!("Bottleneck: {} ({:.1}% of frame time)", 
             bottleneck.name, bottleneck.percentage);
    println!("Recommendation: {}", bottleneck.recommendation);
}
```

## Automated Testing

For CI/CD integration, you can run headless performance tests:

```bash
# Run automated performance test
cargo test --release --test performance_validation

# Check that optimizations meet minimum performance targets
cargo test --release --test optimization_regression
```

These tests verify that:
- Each optimization improves performance
- No regressions from previous builds
- Performance meets minimum acceptable thresholds

## Reporting Performance Issues

When reporting performance problems, include:

1. **Hardware specs** (GPU, CPU, RAM)
2. **Driver version** (graphics driver)
3. **Performance snapshot** (output from P key)
4. **Chrome trace file** (if available)
5. **Scene complexity** (object count, triangle count)
6. **Optimization level** (which optimizations were enabled)

Example:

```
GPU: NVIDIA GTX 1060 6GB (Driver 531.61)
CPU: Intel i5-8400
Scene: 10,000 objects, 5M triangles

Baseline: 12 FPS (83ms)
Full Stack: 45 FPS (22ms)
Expected Full Stack: 120+ FPS

Culling efficiency: 35% (expected 80%+)
```

## Best Practices

### Do's

✓ Always test in release mode  
✓ Let scene warm up before measuring  
✓ Test on target hardware (mid-range GPU)  
✓ Compare before/after when making changes  
✓ Profile regularly during development  
✓ Export Chrome traces for deep analysis  

### Don'ts

✗ Don't profile in debug mode  
✗ Don't test with tiny scenes (< 1000 objects)  
✗ Don't ignore memory profiling  
✗ Don't assume optimizations work without measuring  
✗ Don't optimize without profiling first  
✗ Don't compare across different hardware  

## Conclusion

The comprehensive performance profiling system allows you to:

1. **Measure** - Accurate FPS, frame time, and resource usage
2. **Validate** - Confirm optimizations provide expected improvements
3. **Compare** - See impact of each optimization technique
4. **Debug** - Identify bottlenecks and regressions
5. **Optimize** - Make data-driven performance improvements

Regular profiling ensures the engine maintains excellent performance as features are added.
