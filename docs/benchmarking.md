# Praxis Engine Benchmarking Guide

This document provides comprehensive information about the Praxis engine's benchmark suite, including methodology, interpretation, and optimization strategies.

## Overview

The Praxis engine uses [Criterion.rs](https://github.com/bheisler/criterion.rs) for performance benchmarking. Criterion provides statistical analysis, regression detection, and HTML reports for visualizing performance characteristics.

## Architecture

### Benchmark Organization

The benchmark suite is organized into four main areas that correspond to critical engine subsystems:

1. **Mesh Upload** - Graphics memory management
2. **Render Loop** - Camera and frame timing systems
3. **Physics Step** - Rapier3D integration and spatial queries
4. **Transform Propagation** - ECS hierarchical transform system

Each benchmark suite is isolated in its own file under `benches/` and can be run independently.

## Detailed Benchmark Descriptions

### 1. Mesh Upload Performance

**File:** `benches/mesh_upload.rs`

#### Purpose
Measures the cost of transferring mesh data from CPU memory to GPU memory via Vulkan. This is a critical operation that happens during level loading, procedural generation, and dynamic mesh updates.

#### Benchmarks

##### `mesh_upload`
- **What it measures:** Time to upload vertex and index data to GPU buffers
- **Vertex counts tested:** 100, 500, 1,000, 5,000, 10,000, 50,000
- **Metrics:** Throughput (vertices/second), total upload time
- **Expected scaling:** Linear with vertex count
- **Performance target:** < 5ms for 10,000 vertices

##### `mesh_upload_textured`
- **What it measures:** Upload performance for meshes with texture coordinates
- **Vertex counts tested:** 1,000, 5,000, 10,000
- **Focus:** Additional overhead of UV coordinates
- **Performance target:** < 10% overhead vs. untextured meshes

##### `primitive_generation_and_upload`
- **What it measures:** Combined cost of generating and uploading built-in primitives
- **Primitives tested:** Cube, pyramid, quad
- **Focus:** Convenience API overhead
- **Performance target:** < 1ms per primitive

#### Technical Details

The benchmarks use a real Vulkan device and allocator to measure actual GPU upload performance. Setup includes:
- Vulkan library initialization
- Physical device selection (prefers discrete GPU)
- Logical device creation
- Standard memory allocator setup

The mesh data includes full vertex attributes:
- Positions (3D)
- Colors (RGBA)
- Normals (3D)
- UV coordinates (2D)
- Indices (u16)

#### Optimization Opportunities

1. **Batch Uploads:** Combine multiple small meshes into single upload
2. **Buffer Reuse:** Use persistent staging buffers instead of per-upload allocation
3. **Sparse Attributes:** Only include needed vertex attributes
4. **Index Format:** Use u16 vs u32 indices based on vertex count
5. **Memory Type:** Balance DEVICE_LOCAL vs HOST_VISIBLE tradeoffs

---

### 2. Render Loop Performance

**File:** `benches/render_loop.rs`

#### Purpose
Measures the overhead of camera management and frame timing utilities that run every frame. These systems are on the critical path for rendering.

#### Benchmarks

##### `camera_matrix_updates`
- **What it measures:** Time to compute view and projection matrices for cameras
- **Camera counts tested:** 1, 5, 10, 50
- **Expected scaling:** Linear with camera count
- **Performance target:** < 100μs for 10 cameras
- **Critical for:** Multi-camera rendering, split-screen, camera cuts

##### `camera_query_primary`
- **What it measures:** Cost of selecting the highest-priority active camera
- **Camera counts tested:** 1, 5, 10, 50, 100
- **Expected scaling:** Linear search O(n)
- **Performance target:** < 10μs for 100 cameras
- **Critical for:** Single-camera rendering pipeline

##### `camera_query_sorted`
- **What it measures:** Cost of retrieving all cameras sorted by priority
- **Camera counts tested:** 1, 5, 10, 50, 100
- **Expected scaling:** O(n log n) sorting
- **Performance target:** < 50μs for 100 cameras
- **Critical for:** Multi-pass rendering, compositing

##### `frame_timer_update`
- **What it measures:** Overhead of frame timing calculations (delta time, FPS)
- **Expected cost:** < 1μs per frame
- **Critical for:** Every frame update

#### Technical Details

Camera matrices computed:
- View matrix: Inverse of camera's world transform
- Projection matrix: From FOV, aspect ratio, near/far planes
- View-projection matrix: Combined for shader upload

The ECS schedule runs camera update systems including:
- Transform propagation (if cameras have parents)
- Matrix computation for changed cameras
- Change detection optimization

#### Optimization Opportunities

1. **Cache Matrices:** Only recompute when camera transforms change
2. **Cull Cameras:** Skip inactive or off-screen cameras early
3. **Priority Queue:** Use heap structure for O(1) primary camera access
4. **Parallel Updates:** Compute matrices in parallel for multiple cameras
5. **Frustum Cache:** Store computed frustum planes for culling

---

### 3. Physics Step Performance

**File:** `benches/physics_step.rs`

#### Purpose
Measures the performance of Rapier3D physics simulation integration. Physics is typically the most expensive per-frame operation in physics-heavy games.

#### Benchmarks

##### `physics_step`
- **What it measures:** Full physics simulation step including broad phase, narrow phase, and constraint solving
- **Object counts tested:** 10, 50, 100, 250, 500
- **Expected scaling:** O(n log n) broad phase, O(n²) worst-case narrow phase
- **Performance target:** < 16ms for 100 objects (60 FPS budget)
- **Configuration:** Dynamic rigid bodies with sphere colliders, static ground plane

##### `physics_step_with_collisions`
- **What it measures:** Physics step with collision event detection and distribution
- **Object counts tested:** 10, 50, 100, 250
- **Additional overhead:** Event collection, entity lookup, event receiver population
- **Performance target:** < 20% overhead vs. basic step
- **Use case:** Gameplay systems that need collision callbacks

##### `physics_raycast`
- **What it measures:** Single raycast query performance
- **Scene:** 100 physics objects
- **Expected cost:** < 100μs per raycast
- **Use case:** Line-of-sight checks, weapon firing, character controllers

##### `physics_point_inside`
- **What it measures:** Point-in-collider query performance
- **Scene:** 100 physics objects
- **Expected cost:** < 50μs per query
- **Use case:** Trigger volumes, damage zones, spawn validation

##### `transform_sync_to_physics`
- **What it measures:** Cost of synchronizing ECS transforms to Rapier rigid bodies
- **Object counts tested:** 10, 50, 100, 500, 1,000
- **Expected scaling:** Linear O(n)
- **Performance target:** < 1ms for 1,000 objects
- **Critical for:** Kinematic bodies, animated physics objects

#### Technical Details

Physics configuration:
- Fixed timestep: 1/60 second (16.67ms)
- Integration: Symplectic Euler
- Solver iterations: Rapier defaults (4 velocity, 1 position)
- Broad phase: AABB tree
- Narrow phase: GJK/SAT algorithms

The full physics pipeline includes:
1. `cleanup_physics_entities` - Remove despawned entities
2. `sync_physics_transforms_system` - ECS → Rapier
3. `physics_step_system` - Run simulation
4. `sync_physics_transforms_system` - Rapier → ECS
5. `populate_collision_events` - Distribute collision callbacks

#### Optimization Opportunities

1. **Sleeping Bodies:** Let stationary objects sleep automatically
2. **Collision Groups:** Use collision filtering to reduce pair tests
3. **Simplified Colliders:** Use primitive shapes over convex meshes
4. **Spatial Partitioning:** Tune AABB tree parameters
5. **Solver Iterations:** Reduce iterations for less critical objects
6. **Fixed Timestep Budget:** Cap max steps per frame to prevent spiral of death
7. **Dirty Flags:** Only sync transforms for kinematic bodies that moved

---

### 4. Transform Propagation Performance

**File:** `benches/transform_propagation.rs`

#### Purpose
Measures the overhead of hierarchical transform system that maintains world-space transforms for parent-child entity relationships. This is fundamental to scene graphs and skeletal animation.

#### Benchmarks

##### `transform_propagation_flat`
- **What it measures:** Transform updates for entities without parents
- **Entity counts tested:** 10, 50, 100, 500, 1,000
- **Expected scaling:** Linear O(n)
- **Performance target:** < 500μs for 1,000 entities
- **Use case:** Independent scene objects, particles

##### `transform_propagation_hierarchical`
- **What it measures:** Transform propagation through tree hierarchies
- **Test cases:**
  - Depth 3, Breadth 2: 15 entities (binary tree)
  - Depth 4, Breadth 2: 31 entities (taller binary tree)
  - Depth 3, Breadth 4: 85 entities (wider quaternary tree)
  - Depth 4, Breadth 4: 341 entities (large quaternary tree)
  - Depth 5, Breadth 3: 364 entities (deep ternary tree)
- **Expected scaling:** O(n) where n = total entities
- **Performance target:** < 1ms for 300+ entities
- **Use case:** Scene graphs, attachment systems

##### `transform_propagation_with_rotation`
- **What it measures:** Propagation cost with non-trivial transforms (rotation, scale)
- **Entity counts tested:** 10, 50, 100, 500
- **Additional cost:** Matrix composition with rotation and scale
- **Performance target:** < 20% overhead vs. translation-only
- **Use case:** Articulated objects, rotating reference frames

##### `transform_propagation_deep_hierarchy`
- **What it measures:** Linear chain propagation (worst-case depth)
- **Depths tested:** 5, 10, 20, 50 levels
- **Expected scaling:** Linear with depth
- **Performance target:** < 100μs for 50-level chain
- **Use case:** Long skeletal chains, kinematic chains

##### `parent_child_sync`
- **What it measures:** Cost of maintaining bidirectional parent-child relationships
- **Entity counts tested:** 10, 50, 100, 500
- **Focus:** Adding children to parent's Children component
- **Performance target:** < 100μs for 500 entities
- **Critical for:** Dynamic reparenting, attachment systems

##### `transform_modification_propagation`
- **What it measures:** Incremental update when a single transform changes
- **Scenario:** Change one transform in 121-entity hierarchy
- **Focus:** Change detection efficiency
- **Performance target:** Update only affected subtree
- **Critical for:** Animated transforms, player movement

#### Technical Details

Transform representation:
- Local space: `Transform { translation, rotation, scale }`
- World space: `GlobalTransform { matrix: Mat4 }`

Propagation algorithm:
1. Update root entities (no parents) whose transforms changed
2. Recursively propagate to all descendants using iterative queue
3. Handle reparented entities (Parent component changed)
4. Handle changed children (Transform changed with existing parent)

Optimizations in place:
- Change detection: Only propagate when transforms actually change
- Iterative traversal: Avoid stack overflow with work queue
- Query filtering: Separate queries for roots vs. children
- Batching: Process all roots before processing any children

#### Optimization Opportunities

1. **Dirty Flags:** Track which subtrees need updates
2. **Parallel Propagation:** Process independent subtrees in parallel
3. **Cache Locality:** Store transforms in breadth-first order
4. **SIMD:** Vectorize matrix operations for batch updates
5. **Lazy Updates:** Defer propagation until render time
6. **Pruning:** Skip propagation for off-screen hierarchies
7. **Flattening:** Collapse static hierarchies at load time

---

## Running Benchmarks

### Basic Usage

```bash
# Run all benchmarks
cargo bench

# Run specific suite
cargo bench --bench mesh_upload
cargo bench --bench render_loop
cargo bench --bench physics_step
cargo bench --bench transform_propagation

# Run specific benchmark within suite
cargo bench --bench physics_step -- physics_raycast

# Run with more samples for accuracy
cargo bench -- --sample-size 500
```

### Baseline Comparison

```bash
# Save current performance as baseline
cargo bench -- --save-baseline main

# Make changes, then compare
cargo bench -- --baseline main

# Criterion will show % change from baseline
```

### Profiling Integration

```bash
# Run benchmarks with profiler-friendly settings
cargo bench --bench physics_step -- --profile-time=10

# Then use profiler of choice:
# - flamegraph
# - perf (Linux)
# - Instruments (macOS)
# - VTune (Intel)
```

## Interpreting Results

### Criterion Output

```
mesh_upload/1000        time:   [1.2453 ms 1.2512 ms 1.2579 ms]
                        thrpt:  [795.13 Kelem/s 799.39 Kelem/s 803.16 Kelem/s]
```

- **time:** Mean with confidence interval (95% by default)
- **thrpt:** Throughput (elements per second)
- Lower time is better, higher throughput is better

### HTML Reports

Open `target/criterion/report/index.html` for:
- Detailed plots of timing distribution
- Comparison across runs
- Regression detection
- Statistical analysis

### Key Metrics

1. **Mean Time:** Primary performance indicator
2. **Standard Deviation:** Consistency of performance
3. **Outliers:** Potential GC pauses, context switches
4. **Throughput:** Work done per unit time (where applicable)
5. **Scaling:** How performance changes with input size

### Red Flags

- **Non-linear scaling:** O(n²) instead of O(n)
- **High variance:** Unpredictable performance
- **Regression:** Slower than baseline
- **Outliers:** > 5% samples significantly slower
- **Plateau:** No improvement from optimizations

## Performance Budgets

For 60 FPS (16.67ms frame budget):

| Subsystem              | Budget | Typical |
|------------------------|--------|---------|
| Transform Propagation  | 1ms    | 0.5ms   |
| Physics Simulation     | 8ms    | 5ms     |
| Mesh Upload (amortized)| 2ms    | < 1ms   |
| Camera Updates         | 0.5ms  | 0.1ms   |
| Rendering              | 10ms   | 8ms     |
| Game Logic             | 5ms    | 3ms     |

For 120 FPS (8.33ms frame budget):
- Halve all budgets
- Consider async physics updates
- Implement aggressive culling

## Optimization Workflow

1. **Measure:** Run benchmarks to establish baseline
2. **Profile:** Identify hotspots with profiler
3. **Hypothesize:** Form theory about bottleneck
4. **Optimize:** Make targeted changes
5. **Verify:** Run benchmarks again
6. **Compare:** Check for regression in other areas
7. **Document:** Record optimization and reasoning

## Continuous Integration

### GitHub Actions Example

```yaml
name: Benchmark

on:
  push:
    branches: [main]
  pull_request:

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run benchmarks
        run: cargo bench -- --save-baseline ${{ github.sha }}
      - name: Upload results
        uses: actions/upload-artifact@v2
        with:
          name: criterion-results
          path: target/criterion/
```

### Performance Regression Detection

Set up automated alerts when:
- Any benchmark regresses > 5%
- Variance increases significantly
- New outliers appear consistently

## Best Practices

### Benchmark Design

1. **Isolated:** Test one thing at a time
2. **Realistic:** Use representative data sizes
3. **Stable:** Minimize variance sources
4. **Documented:** Explain what and why
5. **Parameterized:** Test multiple input sizes

### System Configuration

For reproducible results:
- Disable CPU frequency scaling
- Close background applications
- Use dedicated benchmark machine
- Run multiple iterations
- Compare against baseline, not absolute values

### Common Pitfalls

1. **Optimizer Elimination:** Use `black_box()` to prevent DCE
2. **Cold Cache:** Warm up before measurement
3. **Setup Overhead:** Move setup outside measured section
4. **Allocation:** Pre-allocate buffers when testing algorithms
5. **I/O:** Mock or stub external dependencies

## Future Enhancements

Planned benchmark additions:
- **Asset Loading:** File I/O and parsing performance
- **Scene Serialization:** RON encode/decode speed
- **GUI Rendering:** Egui integration overhead
- **Texture Upload:** Image data transfer to GPU
- **Shader Compilation:** Vulkano shader compilation time
- **ECS Queries:** Query iteration patterns
- **Memory Allocations:** Allocation patterns and heap fragmentation

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rapier Performance Guide](https://rapier.rs/docs/user_guides/rust/performance_tuning)
- [Vulkan Best Practices](https://github.com/KhronosGroup/Vulkan-Samples/tree/master/samples/performance)
- [ECS Architecture Patterns](https://github.com/SanderMertens/ecs-faq)
