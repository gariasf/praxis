# Praxis Engine Benchmarks

This directory contains Criterion-based benchmark suites for measuring the performance of key engine subsystems.

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
```

### Run a specific benchmark within a suite:
```bash
cargo bench --bench physics_step -- physics_raycast
```

### Generate HTML reports:
```bash
cargo bench --bench mesh_upload
# Reports generated in target/criterion/
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

## Performance Optimization Tips

1. **Mesh Upload**: Batch uploads, reuse buffers, minimize vertex attributes
2. **Render Loop**: Cache camera matrices, minimize camera count, use dirty flags
3. **Physics**: Use spatial partitioning, reduce collider complexity, tune timestep
4. **Transforms**: Flatten hierarchies where possible, batch updates, use change detection
5. **Asset Loading**: Cache parsed assets, use async loading, prefer binary formats (GLB over GLTF)
6. **Scene Serialization**: Minimize editor data in runtime builds, use binary formats for faster loading, cache deserialized scenes

## Continuous Benchmarking

For CI/CD integration, run benchmarks with:
```bash
cargo bench --bench all -- --save-baseline main
```

Compare against baseline:
```bash
cargo bench --bench all -- --baseline main
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
- `base64 = "0.22"` - For GLTF embedded buffer generation in asset loading benchmarks
- HTML reports feature enabled for visualization
- Access to Vulkan-capable GPU for mesh upload benchmarks
- All engine crates (praxis_ecs, praxis_graphics, praxis_physics, praxis_assets, praxis_scene, etc.)
