# Performance Profiling Quick Reference

## Running the Test

```bash
# Automated test (recommended)
./scripts/run_performance_test.sh --release      # Linux/Mac
.\scripts\run_performance_test.ps1 -Release      # Windows

# Manual test
cargo run --release --example performance_profiling_comprehensive
```

## Controls

| Key | Action |
|-----|--------|
| **1** | Baseline (no optimizations) |
| **2** | Frustum culling |
| **3** | Frustum + LOD |
| **4** | Frustum + LOD + Occlusion |
| **5** | + Instancing |
| **6** | + Streaming |
| **7** | Full stack (all optimizations) |
| **Space** | Reset to baseline |
| **P** | Print performance report |
| **E** | Export/save Chrome trace |
| **I** | Print current state |
| **W/A/S/D** | Move camera |
| **Q/E** | Move camera up/down |
| **Arrow Keys** | Rotate camera |
| **ESC** | Exit |

## Expected Results (Mid-Range GPU)

| Level | FPS | Frame Time | Speedup |
|-------|-----|------------|---------|
| 1. Baseline | 10-15 | 66-100ms | 1.0x |
| 2. Frustum | 30-40 | 25-33ms | 2.5x |
| 3. + LOD | 45-55 | 18-22ms | 3.5x |
| 4. + Occlusion | 60-70 | 14-16ms | 5.0x |
| 5. + Instancing | 90-110 | 9-11ms | 7.5x |
| 6. + Streaming | 100-120 | 8-10ms | 8.5x |
| 7. **Full Stack** | **120-140** | **7-8ms** | **10x** |

Reference: GTX 1060 6GB / RX 580 8GB

## Validation Checklist

### ✅ Performance

- [ ] Each optimization improves FPS by 10-50%
- [ ] Full stack achieves 8-10x speedup
- [ ] No optimization decreases performance

### ✅ Correctness

- [ ] No visible objects disappear (false culling)
- [ ] LOD transitions are smooth
- [ ] No visual artifacts or popping

### ✅ Memory

- [ ] Memory usage is stable (<5% variation)
- [ ] No continuous memory growth
- [ ] Streaming doesn't cause spikes

### ✅ Culling Efficiency

- [ ] Frustum culling: 60-75% culled
- [ ] + Occlusion: 75-85% culled
- [ ] Full stack: 80-90% culled

## Common Issues

### Low FPS

**Symptom**: FPS much lower than expected

**Fixes**:
- Use `--release` mode (not debug)
- Update graphics drivers
- Check VSync settings
- Verify GPU is being used (not integrated)

### No Improvement from Optimizations

**Symptom**: Optimization levels show similar FPS

**Fixes**:
- Move camera to see occluders/LOD differences
- Verify compute shaders are running
- Check profiler shows culling is active

### False Culling

**Symptom**: Visible objects disappear

**Fixes**:
- Check frustum calculation
- Increase bounding volume sizes
- Reduce occlusion bias threshold

### Memory Leaks

**Symptom**: Memory grows continuously

**Fixes**:
- Check streaming mesh cache eviction
- Set texture cache size limit
- Reset profiler periodically

## Performance Analysis

### Chrome Trace Export

1. Press **E** to start trace
2. Run for 5-10 seconds
3. Press **E** to save
4. Open `chrome://tracing` in Chrome
5. Load `performance_trace.json`

### What to Look For

- **CPU Timeline**: System execution order and duration
- **GPU Timeline**: Render passes and compute shaders
- **Memory Counters**: Allocation patterns over time
- **Frame Markers**: Frame boundaries and timing

### Bottleneck Identification

Check profiler output for:
- Systems taking >15% of frame time
- GPU passes taking >5ms
- Memory allocations per frame
- Draw call count

## Quick Optimization Guide

### CPU-Bound (High CPU, Low GPU)

✓ Enable GPU frustum culling  
✓ Use mesh instancing  
✓ Batch materials  
✓ Reduce object count  

### GPU-Bound (High GPU, Low CPU)

✓ Enable LOD system  
✓ Use occlusion culling  
✓ Reduce shader complexity  
✓ Lower triangle count  

### Memory-Bound

✓ Enable texture compression  
✓ Use mesh streaming  
✓ Increase LOD bias  
✓ Share materials  

### Draw Call Bound

✓ Enable instancing  
✓ Batch by material  
✓ Use multi-draw indirect  
✓ Merge static geometry  

## Testing Workflow

### Quick Test (5 minutes)

1. Run with `--release`
2. Test baseline (1)
3. Test full stack (7)
4. Press **P** for report
5. Verify 8-10x speedup

### Full Test (15 minutes)

1. Run with `--release`
2. Test all levels (1-7)
3. Wait 3s warmup per level
4. Press **P** after each level
5. Export trace with **E**
6. Verify all optimizations
7. Analyze trace in Chrome

### Regression Test (2 minutes)

```bash
# Run automated test
cargo test --release --test performance_validation

# Check exit code
if [ $? -eq 0 ]; then
    echo "✅ Performance validated"
else
    echo "❌ Performance regression detected"
fi
```

## Hardware Tiers

### Low-End (Integrated Graphics)

Intel UHD 630 / AMD Vega 8

- Baseline: 5-8 FPS
- Full Stack: 45-60 FPS

### Mid-Range (Discrete GPU)

GTX 1060 / RX 580

- Baseline: 10-15 FPS
- Full Stack: 120-140 FPS

### High-End (Enthusiast)

RTX 3070 / RX 6800

- Baseline: 25-35 FPS
- Full Stack: 240-300 FPS

## Profiling Best Practices

### ✓ Do

- Always use `--release` mode
- Let scene warm up (60 frames)
- Test on target hardware
- Compare before/after
- Profile regularly
- Export traces for analysis

### ✗ Don't

- Don't use debug mode
- Don't test tiny scenes
- Don't ignore memory
- Don't assume without measuring
- Don't optimize prematurely
- Don't compare different hardware

## Getting Help

If results don't match expectations:

1. Check docs/performance_profiling_guide.md
2. Verify hardware meets minimum specs
3. Update graphics drivers
4. Export Chrome trace for analysis
5. Report issue with:
   - Hardware specs
   - Performance snapshot (P key)
   - Chrome trace (if available)
   - Expected vs actual results

## Related Documentation

- [Full Guide](performance_profiling_guide.md) - Detailed profiling documentation
- [Optimization Guide](../crates/praxis_graphics/README.md) - Graphics optimization techniques
- [Profiling Crate](../crates/praxis_profiling/README.md) - Profiler API documentation
