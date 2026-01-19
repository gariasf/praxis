# Rendering Pipeline Stress Test

Comprehensive stress testing example for validating the rendering pipeline's stability, performance, and resource management under extreme conditions.

## Overview

This stress test provides a suite of 7 test scenarios designed to push the rendering pipeline to its limits and validate that it remains stable, maintains acceptable performance, and properly manages resources.

## Test Scenarios

### 1. Massive Object Count (10,000+ Objects)
- **Purpose**: Test rendering throughput with extreme object counts
- **Scene**: 10,240 objects arranged in a dense 32x32x10 grid
- **Validation**: 
  - No crashes
  - FPS > 15
  - Memory growth < 500MB
- **What to observe**: Draw call batching, culling efficiency, GPU memory usage

### 2. Extreme Camera Movement (Rapid Teleports)
- **Purpose**: Test frustum culling and visibility determination under rapid viewpoint changes
- **Scene**: 5,000 objects scattered across large area (600x100x600 units)
- **Behavior**: Camera teleports to random positions every 0.5 seconds
- **Validation**:
  - No culling artifacts (visible objects not being drawn)
  - Quick visibility updates
  - No stuttering during teleports
- **What to observe**: Frustum culling performance, object visibility updates

### 3. Rapid LOD Transitions (Fast Camera Sweeps)
- **Scene**: 1,000 objects in concentric circles (20 rings, 50 objects per ring)
- **Behavior**: Camera performs fast circular motion triggering constant LOD switches
- **Validation**:
  - Smooth LOD transitions
  - No popping artifacts
  - Correct LOD selection based on distance
- **What to observe**: LOD switching frequency, transition smoothness, distance calculations

### 4. Material Instance Stress (500+ Instances)
- **Purpose**: Test material instancing system with hundreds of property overrides
- **Scene**: 500 unique material instances, 625 objects using them
- **Features tested**:
  - Material instance creation
  - Property overrides (color, metallic, roughness)
  - Descriptor set reuse
  - Memory efficiency
- **Validation**:
  - Efficient descriptor set reuse
  - No unbounded descriptor set growth
  - Correct per-object material properties
- **What to observe**: Descriptor set pool size, material instance statistics

### 5. Mesh Streaming Throughput (Continuous Load/Unload)
- **Purpose**: Test mesh streaming system under continuous load/unload cycles
- **Scene**: 2,000 objects with dynamic lifetime management
- **Behavior**: Objects are continuously spawned and despawned, triggering mesh loads/unloads
- **Validation**:
  - No mesh orphaning (unreferenced meshes stay loaded)
  - Proper resource cleanup when meshes are unloaded
  - Stable memory usage over time
- **What to observe**: Mesh load/unload counts, GPU memory usage trends

### 6. Combined Stress (All Tests Simultaneously)
- **Purpose**: Test system behavior under multiple stressors at once
- **Scene**: ~4,840 objects + 200 material instances + extreme camera movement
- **Combines**:
  - Large object count
  - Rapid camera teleports
  - Material instancing
  - Dynamic spawning/despawning
- **Validation**:
  - System remains stable under combined load
  - No cascading failures
  - Performance degradation is acceptable (>15 FPS)
- **What to observe**: Overall system stability, resource management under pressure

### 7. Resource Cleanup Validation
- **Purpose**: Validate proper cleanup and no memory leaks
- **Scene**: 1,000 objects spawned in batches
- **Behavior**: Repeatedly spawn and despawn entire batches
- **Validation**:
  - Memory returns to baseline after cleanup
  - No descriptor set leaks
  - No mesh orphaning
  - GPU resources properly released
- **What to observe**: Memory usage over multiple spawn/despawn cycles

## Controls

### Test Selection
- **1-7**: Select test scenario
- **Space**: Reset to idle state (minimal load)
- **P**: Print current statistics
- **ESC**: Exit

### Manual Camera Controls (for non-automated tests)
- **W/A/S/D**: Move camera forward/left/back/right
- **Q/E**: Move camera down/up
- **Arrow Keys**: Rotate camera

## Usage

```bash
# Run in release mode for accurate performance testing
cargo run --release --example rendering_stress_test
```

**Important**: Release mode is required for realistic performance measurements. Debug mode will show significantly worse performance and may fail validation criteria.

## Validation Criteria

Each test is evaluated against these criteria:

1. **Stability**: No crashes or panics during test duration
2. **Performance**: Minimum FPS > 15 (acceptable degradation from baseline)
3. **Memory**: Memory growth < 500MB over baseline
4. **Visual Correctness**: No flickering, culling errors, or rendering artifacts
5. **Resource Management**: Proper cleanup, no unbounded growth

## Test Duration

Each test runs for 10 seconds by default. Statistics are printed every second during the test. After completion, the test is automatically validated and results are recorded.

## Expected Results

On a mid-range GPU (GTX 1060 / RX 580 equivalent):

| Test | Expected FPS | Memory Growth | Notes |
|------|--------------|---------------|-------|
| Test 1 (Massive Objects) | 20-30 FPS | 100-200 MB | Depends on culling efficiency |
| Test 2 (Camera Movement) | 30-50 FPS | 50-100 MB | Frequent culling updates |
| Test 3 (LOD Transitions) | 40-60 FPS | 50-100 MB | LOD system overhead |
| Test 4 (Material Instances) | 25-40 FPS | 150-250 MB | Descriptor set overhead |
| Test 5 (Mesh Streaming) | 30-50 FPS | Variable | Should stabilize |
| Test 6 (Combined) | 15-25 FPS | 200-400 MB | Worst case scenario |
| Test 7 (Cleanup) | N/A | Should return to baseline | Focus on cleanup |

## Interpreting Results

### Pass/Fail Determination
- Tests automatically validate against criteria
- PASSED tests are logged with ✓
- FAILED tests include failure reason

### Common Issues

1. **Low FPS (<15)**: 
   - Check GPU utilization
   - Review draw call batching
   - Verify frustum culling is working

2. **High Memory Growth (>500MB)**:
   - Check for resource leaks
   - Verify cleanup is called
   - Look for orphaned descriptor sets

3. **Visual Artifacts**:
   - Culling too aggressive (objects disappear)
   - LOD transitions jarring
   - Material instances not applying

### Statistics Explained

- **FPS**: avg/min/max frame rates during test
- **Memory**: current/peak/growth in MB
- **Objects**: spawned/despawned/visible/culled counts
- **Draw Calls**: number of draw calls per frame
- **Material Instances**: total instances created
- **Mesh Loads/Unloads**: streaming activity
- **Descriptor Sets**: allocated descriptor sets (watch for growth)

## Final Report

After running multiple tests (or exiting), a final report is printed showing:
- Total tests passed/failed
- Success rate percentage
- Individual test results with failure reasons

Example:
```
========================================
FINAL STRESS TEST REPORT
========================================
Tests Passed: 6
Tests Failed: 1

Passed Tests:
  ✓ Massive Object Count (10,000+ Objects)
  ✓ Extreme Camera Movement (Rapid Teleports)
  ✓ Rapid LOD Transitions (Fast Sweeps)
  ✓ Material Instance Stress (500+ Instances)
  ✓ Mesh Streaming Throughput (Continuous Load/Unload)
  ✓ Resource Cleanup Validation

Failed Tests:
  ✗ Combined Stress (All Tests Simultaneously) - FPS too low (12.3 < 15.0). 

Success Rate: 85.7%
========================================
```

## Implementation Notes

- Tests use simplified frustum culling (dot product + distance check)
- LOD selection is based on distance thresholds (not screen-space size)
- Memory estimation is approximate (based on object counts)
- Descriptor set tracking requires instrumentation in RenderContext

## Extending the Tests

To add a new stress test scenario:

1. Add new variant to `StressTest` enum
2. Implement setup function: `setup_<test_name>()`
3. Add input handling in `handle_input()`
4. Update `update_camera_system()` if custom camera behavior needed
5. Document expected behavior and validation criteria

## Troubleshooting

### Test Won't Start
- Ensure meshes are loaded before spawning objects
- Check for resource initialization errors in logs

### Performance Varies Wildly
- Run in release mode (`--release`)
- Close background applications
- Check GPU temperature/throttling

### Memory Keeps Growing
- This indicates a leak - check resource cleanup
- Review descriptor set allocation patterns
- Verify mesh manager properly releases unused meshes

## Related Examples

- `performance_profiling_comprehensive.rs`: Detailed profiling with optimization comparisons
- `gpu_culling_demo.rs`: GPU-driven frustum culling demonstration
- `lod_gpu_demo.rs`: GPU LOD selection example
- `mesh_streaming_demo.rs`: Mesh streaming system demonstration
- `material_instancing_demo.rs`: Material instancing example
