# Praxis Engine Benchmarks

This directory contains Criterion-based benchmark suites for measuring the performance of key engine subsystems, including comprehensive graphics optimization benchmarks.

## Benchmark Suites

### 1. Mesh Upload (`mesh_upload.rs`)

Measures the performance of uploading mesh data from CPU to GPU using Vulkan.

**Benchmarks:**
- `mesh_upload`: Tests mesh upload performance across various vertex counts (100 to 50,000)
- `mesh_upload_textured`: Tests mesh upload with texture coordinates
- `primitive_generation_and_upload`: Benchmarks built-in primitive generators (cube, pyramid, quad)

**Metrics:**
- Throughput measured in vertices per second
- Upload time for different mesh sizes
- Primitive generation overhead

### 2. Render Loop (`render_loop.rs`)

Benchmarks camera system operations and frame timing utilities.

**Benchmarks:**
- `camera_matrix_updates`: Measures camera matrix computation performance (1 to 50 cameras)
- `camera_query_primary`: Tests primary camera selection from multiple cameras
- `camera_query_sorted`: Benchmarks sorting cameras by priority
- `frame_timer_update`: Measures frame timing overhead

**Metrics:**
- Camera update time per frame
- Query overhead for camera selection
- Frame timer tick performance

### 3. Physics Step (`physics_step.rs`)

Measures physics simulation performance using Rapier3D.

**Benchmarks:**
- `physics_step`: Basic physics simulation (10 to 500 objects)
- `physics_step_with_collisions`: Physics with collision event detection
- `physics_raycast`: Raycasting query performance
- `physics_point_inside`: Point-in-collider query performance
- `transform_sync_to_physics`: ECS-to-Rapier transform synchronization (10 to 1,000 objects)

**Metrics:**
- Physics step duration per frame
- Collision detection overhead
- Spatial query performance
- Transform synchronization cost

### 4. Transform Propagation (`transform_propagation.rs`)

Benchmarks the hierarchical transform system that propagates transforms through parent-child relationships.

**Benchmarks:**
- `transform_propagation_flat`: Flat hierarchy (no parents, 10 to 1,000 entities)
- `transform_propagation_hierarchical`: Tree hierarchies with varying depth and breadth
- `transform_propagation_with_rotation`: Transform propagation with rotation and scale
- `transform_propagation_deep_hierarchy`: Linear chains (5 to 50 levels deep)
- `parent_child_sync`: Parent-child relationship maintenance
- `transform_modification_propagation`: Incremental updates when transforms change

**Metrics:**
- Propagation time for different hierarchy shapes
- Overhead of parent-child sync
- Change detection performance

### 5. Asset Loading (`asset_loading.rs`)

Measures the performance of loading and parsing 3D model file formats (OBJ and GLTF).

**Benchmarks:**
- `obj_parsing`: OBJ file parsing across various vertex counts (100 to 10,000)
- `obj_file_io`: Raw file I/O overhead for OBJ files
- `gltf_parsing`: GLTF file parsing and buffer extraction (100 to 5,000 vertices)
- `obj_with_normals_and_uvs`: Full OBJ parsing with all attributes
- `obj_positions_only`: Minimal OBJ parsing (positions only)
- `obj_load_real_cube_asset`: Loading the actual cube.obj asset from assets/

**Metrics:**
- Parsing time measured in vertices per second
- File I/O vs parsing overhead
- Impact of different attribute combinations

### 6. Scene Serialization (`scene_serialization.rs`)

Benchmarks scene definition serialization and deserialization using RON format.

**Benchmarks:**
- `scene_serialization`: Serialize scenes with varying entity counts (10 to 1,000)
- `scene_deserialization`: Deserialize and validate scenes
- `scene_roundtrip`: Full serialize-deserialize cycle
- `scene_hierarchy_serialization`: Serialize hierarchical scenes with different depths
- `scene_hierarchy_deserialization`: Deserialize hierarchical scenes
- `scene_with_editor_data_serialization`: Serialize scenes with editor metadata
- `scene_with_editor_data_deserialization`: Deserialize editor-enhanced scenes
- `scene_to_runtime`: Convert editor scene to runtime scene (strips editor data)
- `scene_metadata_serialization`: Serialize scenes with heavy metadata
- `minimal_scene_serialization`: Minimal scene (empty) serialization baseline
- `complex_scene_with_all_features`: Complex scene with cameras, lights, hierarchy, and editor data

**Metrics:**
- Serialization/deserialization time per entity
- Impact of hierarchy depth on performance
- Editor data overhead
- RON format efficiency

### 7. Descriptor Set Allocation (`descriptor_set_allocation.rs`)

Comprehensive benchmarks for Vulkan descriptor set allocation patterns and optimization strategies. Designed to measure the impact of planned graphics optimizations, with special focus on LRU caching performance.

**Key Benchmarks:**
- `descriptor_caching_with_lru`: **Main benchmark** - Measures allocation rates with/without LRU caching over 1000 frames with 100 unique materials
  - Without caching: 100,000 allocations (100 materials × 1000 frames)
  - With LRU caching: 100 allocations (only on first frame)
  - Validates 100x+ reduction in allocations
  - Tracks cache hit rate (expected >99.9%)

- `descriptor_allocation_with_tracking`: Detailed per-frame allocation tracking with statistics
  - Validates frame-by-frame allocation patterns
  - First frame: 100 allocations, subsequent frames: 0 allocations
  - Verifies 100x+ reduction with detailed metrics

- `cache_hit_rate_analysis`: Measures steady-state cache efficiency after warmup
  - 10-frame warmup period
  - 990 frames steady-state measurement
  - Expected: 100% cache hit rate after warmup

- `varying_material_counts`: Tests scalability (10, 50, 100, 200, 500 materials)
  - Validates cache efficiency scales properly
  - Ensures >99.9% hit rate regardless of material count

**Legacy Benchmarks:**
- `descriptor_set_single_allocation`: Single descriptor set allocation overhead
- `descriptor_set_batch_allocation`: Batch allocation (10-1000 sets) with throughput metrics
- `descriptor_reuse_vs_recreation`: Compare reusing existing sets vs recreating each frame
- `descriptor_pooling_patterns`: Per-frame vs material-based pooling strategies
- `allocator_configurations`: Test different allocator configurations
- `descriptor_write_patterns`: Single vs multiple buffer writes per set
- `frame_by_frame_allocation`: Simulate per-frame allocation with varying object counts

**Metrics:**
- Total allocations over 1000 frames
- Cache hit rate percentage
- Allocation reduction factor (with vs without caching)
- Per-frame allocation counts
- Descriptor sets allocated per second
- Memory efficiency (sets created vs sets needed)

**Performance targets:**
- **100x+ reduction** in allocations with LRU caching (primary target)
- **>99.9% cache hit rate** after first frame
- **100% cache hit rate** in steady-state (after warmup)
- <50μs per descriptor set with optimal pooling
- 10-20x reduction in allocations through material batching

### 8. Staging Buffer (`staging_buffer.rs`)

Benchmarks for staging buffer allocation, write, and copy performance for GPU uploads. Validates staging buffer optimization strategies.

**Benchmarks:**
- `staging_buffer_allocation`: Allocation overhead for various buffer sizes (256B-256KB)
- `staging_buffer_write`: CPU write performance to staging buffers
- `staging_to_device_copy`: Complete staging-to-device transfer pipeline
- `persistent_staging_buffer`: Reuse single buffer vs create new each time
- `batch_staging_upload`: Batch multiple copies into single command buffer (1-100 buffers)
- `ring_buffer_staging`: Ring buffer pattern for frames in flight (3 frames)
- `direct_write_vs_staging`: Compare host-visible vs staging approach
- `staging_buffer_sizes`: Impact of staging buffer size on upload performance

**Metrics:**
- Bytes per second throughput
- Allocation + write + copy latency
- CPU vs GPU time breakdown
- Buffer reuse efficiency

**Performance targets:**
- Target 5-15% improvement through persistent staging buffers
- >500 MB/s throughput for typical uploads
- <1ms latency for 64KB uploads

### 9. Graphics Optimization (`graphics_optimization.rs`)

Integrated benchmarks simulating complete frame rendering with realistic optimization scenarios. This is the primary benchmark for validating the 5-15% performance improvement target.

**Benchmarks:**
- `complete_frame_render_pattern`: Full frame simulation (10-200 objects)
  - Staging buffer uploads
  - Descriptor set creation
  - Material batching
  
- `material_batching_optimization`: Compare no batching vs 5-10 materials
  - Measures descriptor set allocation reduction
  - Validates target performance improvements

- `dynamic_uniform_buffer_pattern`: Dynamic buffer with offsets (10-500 objects)
  - Simulates engine's actual dynamic uniform buffer implementation
  - Measures write + bind overhead per object

- `descriptor_set_caching`: Cache descriptor sets across 60 frames
  - Quantifies benefit of material descriptor set caching
  - Measures frame-to-frame reuse patterns

- `staging_buffer_pooling`: Pooled vs per-frame staging buffers (10 frames)
  - Ring buffer pattern with 3-frame pool
  - Measures allocation overhead reduction

- `integrated_optimization_scenarios`: Complete before/after comparison
  - **Baseline**: Per-object descriptor sets + per-object staging uploads
  - **Optimized**: Material batching (10:100 ratio) + pooled staging buffers
  - **Target**: Validate 5-15% overall improvement

**Metrics:**
- Total frame time (end-to-end)
- Descriptor operations per frame
- GPU upload throughput
- Memory allocations per frame

**Validation criteria:**
- Optimized approach should show 5-15% improvement over baseline
- Material batching should reduce descriptor allocations by 10-20x
- Staging buffer pooling should reduce allocations by 3-10x (frame count dependent)

## Running Benchmarks

### Run all benchmarks:
```bash
cargo bench
```

### Run a specific benchmark suite:
```bash
cargo bench --bench mesh_upload
cargo bench --bench render_loop
cargo bench --bench physics_step
cargo bench --bench transform_propagation
cargo bench --bench asset_loading
cargo bench --bench scene_serialization
cargo bench --bench descriptor_set_allocation
cargo bench --bench staging_buffer
cargo bench --bench graphics_optimization
```

### Run a specific benchmark within a suite:
```bash
cargo bench --bench physics_step -- physics_raycast
cargo bench --bench descriptor_set_allocation -- batch_allocation
cargo bench --bench graphics_optimization -- material_batching
```

### Run graphics optimization benchmarks:
```bash
# Run all graphics optimization benchmarks
cargo bench --bench descriptor_set_allocation
cargo bench --bench staging_buffer
cargo bench --bench graphics_optimization

# Or run the complete integrated benchmark
cargo bench --bench graphics_optimization -- integrated_optimization_scenarios
```

### Generate HTML reports:
```bash
cargo bench --bench graphics_optimization
# Reports generated in target/criterion/
open target/criterion/report/index.html  # macOS
xdg-open target/criterion/report/index.html  # Linux
start target/criterion/report/index.html  # Windows
```

## Interpreting Results

Criterion generates detailed reports including:
- Mean execution time with confidence intervals
- Throughput measurements (where applicable)
- Performance comparison across runs
- Statistical analysis of variance
- HTML visualizations with plots

### Key Metrics to Watch:

**Mesh Upload:**
- Target: < 5ms for 10k vertices
- Watch for: Linear scaling with vertex count

**Render Loop:**
- Target: < 100μs for camera updates with 10 cameras
- Watch for: Constant-time camera queries

**Physics Step:**
- Target: < 16ms for 100 objects (60 FPS budget)
- Watch for: Quadratic scaling with collision detection

**Transform Propagation:**
- Target: < 1ms for 100-entity hierarchy
- Watch for: Linear scaling with entity count, no unnecessary updates

**Asset Loading:**
- Target: < 10ms for 5k vertex OBJ, < 20ms for 5k vertex GLTF
- Watch for: Linear scaling with vertex count, I/O bottlenecks

**Scene Serialization:**
- Target: < 5ms to serialize 100-entity scene, < 10ms to deserialize
- Watch for: Linear scaling with entity count, hierarchy depth impact

**Descriptor Set Allocation:**
- **Primary Target**: 100x+ reduction in allocations with LRU caching
- Target: < 50μs per allocation with pooling
- Watch for: >99.9% cache hit rate after first frame
- Compare: "without_caching" vs "with_lru_caching" in `descriptor_caching_with_lru`
- Verify: Frame 1 = 100 allocations, Frames 2-1000 = 0 allocations

**Staging Buffer:**
- Target: > 500 MB/s throughput
- Watch for: 2-3x improvement with persistent buffers
- Compare: "create_new_buffer_each_time" vs "reuse_persistent_buffer"

**Graphics Optimization (Integrated):**
- **Primary Target**: 5-15% improvement in "integrated_optimization_scenarios"
- Compare: "baseline_current_approach" vs "optimized_batching_and_pooling"
- Watch for: Combined effect of all optimizations

### Reading Graphics Optimization Results

1. **Descriptor Set Allocation (LRU Caching)**: 
   - Look at "descriptor_caching_with_lru" group
   - Compare "without_caching" vs "with_lru_caching"
   - **Should see 1000x reduction in allocations** (100,000 → 100)
   - **Should see >99.9% cache hit rate**
   - Verify assertions pass (they validate the 100x+ reduction)
   
2. **Descriptor Set Allocation (Legacy)**: 
   - Look at "material_batching_optimization" group
   - Compare "no_batching" vs "with_batching_10_materials"
   - Should see 10x fewer allocations

3. **Staging Buffer Performance**:
   - Look at "persistent_staging_buffer" group
   - Compare "create_new_buffer_each_time" vs "reuse_persistent_buffer"
   - Should see 2-3x throughput improvement

4. **Integrated Performance** (Most Important):
   - Look at "integrated_optimization_scenarios" group
   - Compare "baseline_current_approach" vs "optimized_batching_and_pooling"
   - **Target: 5-15% faster frame time**
   - This validates the overall optimization strategy

5. **Frame-to-Frame Efficiency**:
   - "descriptor_set_caching" should show near-zero cost after first frame
   - "staging_buffer_pooling" should show consistent performance

## Using Benchmarks for Optimization Validation

### Before implementing optimizations:
```bash
cargo bench --bench graphics_optimization -- baseline > results_before.txt
cargo bench --bench descriptor_set_allocation > results_desc_before.txt
cargo bench --bench staging_buffer > results_stage_before.txt

# Save baseline for comparison
cargo bench -- --save-baseline before_optimization
```

### After implementing optimizations:
```bash
cargo bench --bench graphics_optimization -- optimized > results_after.txt
cargo bench --bench descriptor_set_allocation > results_desc_after.txt
cargo bench --bench staging_buffer > results_stage_after.txt

# Compare against baseline
cargo bench -- --baseline before_optimization
```

### Validate the 5-15% improvement target:
```bash
# The most important benchmark for validation
cargo bench --bench graphics_optimization -- integrated_optimization_scenarios

# Look for the comparison in the HTML report:
# "baseline_current_approach" vs "optimized_batching_and_pooling"
# Should show 5-15% faster execution time
```

## Performance Optimization Tips

1. **Mesh Upload**: Batch uploads, reuse buffers, minimize vertex attributes
2. **Render Loop**: Cache camera matrices, minimize camera count, use dirty flags
3. **Physics**: Use spatial partitioning, reduce collider complexity, tune timestep
4. **Transforms**: Flatten hierarchies where possible, batch updates, use change detection
5. **Asset Loading**: Cache parsed assets, use async loading, prefer binary formats (GLB over GLTF)
6. **Scene Serialization**: Minimize editor data in runtime builds, use binary formats for faster loading, cache deserialized scenes
7. **Descriptor Sets**: Use material batching, cache descriptor sets, prefer dynamic uniform buffers
8. **Staging Buffers**: Reuse persistent buffers, use ring buffer pattern, batch uploads

## Continuous Benchmarking

For CI/CD integration, run benchmarks with:
```bash
cargo bench -- --save-baseline main
```

Compare against baseline:
```bash
cargo bench -- --baseline main
```

## Adding New Benchmarks

1. Create a new `.rs` file in `benches/`
2. Add `[[bench]]` section to `Cargo.toml`
3. Use Criterion's API to define benchmarks
4. Document the new benchmark in this README

Example structure:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn my_benchmark(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        b.iter(|| {
            // Code to benchmark
            black_box(my_function());
        });
    });
}

criterion_group!(benches, my_benchmark);
criterion_main!(benches);
```

## Dependencies

- `criterion = "0.5"` - Statistical benchmarking framework
- `vulkano = "0.35.1"` - For graphics benchmarks
- `base64 = "0.22"` - For GLTF embedded buffer generation
- HTML reports feature enabled for visualization
- Access to Vulkan-capable GPU for graphics benchmarks
- All engine crates (praxis_ecs, praxis_graphics, praxis_physics, praxis_assets, praxis_scene, etc.)

## Notes on Graphics Optimization Benchmarks

The descriptor set allocation, staging buffer, and integrated graphics optimization benchmarks are specifically designed to:

1. **Measure realistic workloads**: Object counts and patterns match typical game scenes (10-200 objects)
2. **Test optimization strategies**: Each benchmark compares naive vs optimized approaches
3. **Validate performance targets**: The 5-15% improvement target is directly measurable
4. **Provide actionable insights**: Results inform implementation decisions

**Key optimization strategies validated**:
- **Material batching**: Group objects by material to reduce descriptor set allocations
- **Descriptor set caching**: Reuse descriptor sets across frames when materials don't change
- **Dynamic uniform buffers**: Use single buffer with dynamic offsets instead of per-object buffers
- **Staging buffer pooling**: Reuse persistent staging buffers in a ring buffer pattern
- **Batch GPU uploads**: Combine multiple uploads into single command buffer submission

**Expected improvements**:
- Material batching: 10-20x reduction in descriptor set allocations
- Staging buffer pooling: 2-3x improvement in upload throughput
- Combined optimizations: 5-15% overall frame time improvement

These benchmarks provide concrete validation that the optimization strategies meet their performance targets before full implementation in the engine.
