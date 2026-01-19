# GPU LOD Selection Benchmark

This benchmark compares CPU-based LOD selection against GPU compute shader-based LOD selection to demonstrate the performance benefits of GPU-driven LOD management.

## Overview

The benchmark tests LOD (Level of Detail) selection performance for 100 to 100,000 objects, comparing:

1. **CPU LOD Selection**: Sequential distance calculations and LOD level selection on CPU
2. **GPU LOD Selection**: Parallel compute shader performing all calculations on GPU
3. **GPU CPU Overhead**: Measures just the CPU-side overhead of dispatching GPU work

## Key Findings

### Expected Performance Characteristics

- **CPU LOD Selection**: O(N) complexity - time scales linearly with object count
- **GPU LOD Selection**: Near O(1) CPU complexity - minimal scaling with object count
- **Crossover Point**: GPU becomes faster around 1,000-5,000 objects

### Benchmark Configurations

| Object Count | Use Case                  |
|--------------|---------------------------|
| 100          | Small scene baseline      |
| 1,000        | Medium scene              |
| 10,000       | Large scene               |
| 100,000      | Massive scene (stress test)|

## Implementation Details

### CPU LOD Selection

For each object:
1. Calculate squared distance from camera: `delta.length_squared()`
2. Select LOD level based on distance thresholds
3. Store selected LOD level

**Characteristics:**
- Simple, straightforward implementation
- No GPU setup overhead
- Scales linearly: O(N) operations

### GPU LOD Selection

Setup phase (once per frame):
1. Upload object transforms and LOD definitions to GPU buffers
2. Update camera position uniform

Compute phase (GPU parallel):
1. Transform object center to world space
2. Calculate squared distance from camera
3. Select LOD level based on thresholds with bias support
4. Write selected LOD to output buffer

Readback phase (for verification):
1. Read selected LOD levels from GPU

**Characteristics:**
- Parallel execution on GPU (64 objects per workgroup)
- Constant CPU overhead: O(1)
- GPU time increases sublinearly

### GPU Compute Shader

Located at: `crates/praxis_graphics/src/shaders/lod_selection.comp`

Key features:
- 64 threads per workgroup for coalesced memory access
- Squared distance calculations (no sqrt needed)
- LOD bias support for quality/performance tuning
- Per-object LOD level arrays for flexibility

## Running the Benchmark

```bash
# Run the GPU LOD benchmark only
cargo bench --bench graphics_optimization -- gpu_vs_cpu_lod_selection

# Run all graphics optimization benchmarks
cargo bench --bench graphics_optimization

# Save baseline for future comparisons
cargo bench --bench graphics_optimization -- gpu_vs_cpu_lod_selection --save-baseline lod-v1

# Compare against saved baseline
cargo bench --bench graphics_optimization -- gpu_vs_cpu_lod_selection --baseline lod-v1
```

## Expected Results

### Small Scenes (100 objects)
- **CPU**: ~2 µs (faster due to no GPU overhead)
- **GPU**: ~200 µs (GPU setup overhead dominates)
- **Winner**: CPU

### Medium Scenes (1,000 objects)
- **CPU**: ~20 µs
- **GPU**: ~220 µs
- **Winner**: CPU (but gap closing)

### Large Scenes (10,000 objects)
- **CPU**: ~200 µs
- **GPU**: ~250 µs
- **Winner**: GPU starts to compete

### Massive Scenes (100,000 objects)
- **CPU**: ~2,000 µs (2 ms)
- **GPU**: ~350 µs
- **Winner**: GPU (5-6x faster!)

### CPU Overhead Only
- All object counts: ~5-15 µs (constant!)
- Demonstrates O(1) CPU complexity

## Performance Scalability

The benchmark demonstrates:

1. **Linear CPU Scaling**: CPU time doubles when object count doubles
2. **Sublinear GPU Scaling**: GPU time increases minimally with more objects
3. **Constant CPU Overhead**: GPU dispatch overhead stays flat regardless of object count
4. **Crossover Analysis**: Identifies the point where GPU becomes more efficient

## Integration with Rendering Pipeline

GPU LOD selection integrates with:

1. **GPU Culling**: Share object data buffers between LOD and culling passes
2. **Indirect Draw**: Feed selected LODs directly to indirect draw buffer generation
3. **Material Batching**: Group objects by LOD level for efficient rendering
4. **No Stalls**: All LOD calculations happen on GPU, no CPU-GPU sync needed

## Verification

The benchmark includes correctness verification:
- Selected LOD levels are read back from GPU
- Same LOD thresholds used for both CPU and GPU tests
- Distance calculations use identical squared distance formula

## Interpreting Output

Example output:
```
gpu_vs_cpu_lod_selection/cpu_lod_selection/10000
                        time:   [203.12 µs 208.45 µs 215.23 µs]
                        thrpt:  [46.458 Melem/s 47.974 Melem/s 49.233 Melem/s]

gpu_vs_cpu_lod_selection/gpu_lod_selection/10000
                        time:   [248.56 µs 254.32 µs 261.45 µs]
                        thrpt:  [38.248 Melem/s 39.321 Melem/s 40.236 Melem/s]

gpu_vs_cpu_lod_selection/gpu_lod_cpu_overhead_only/10000
                        time:   [12.891 µs 13.542 µs 14.321 µs]
```

**Key Metrics:**
- `time`: Execution time (lower is better)
- `thrpt`: Throughput in millions of elements per second (higher is better)
- Confidence intervals show measurement variance

## When to Use GPU LOD

**Use GPU LOD when:**
- Scene has > 5,000 objects with LOD
- Using GPU culling (can share data)
- Need consistent frame times regardless of object count
- Building a fully GPU-driven rendering pipeline

**Use CPU LOD when:**
- Scene has < 1,000 objects
- Simplicity is more important than peak performance
- Not using other GPU-driven techniques

## Implementation Notes

### Object Data Structure (96 bytes)
```rust
struct GpuObjectData {
    model: Mat4,              // 64 bytes - transform matrix
    bounding_sphere: Vec4,    // 16 bytes - center + radius
    mesh_id: u32,             // 4 bytes - base mesh ID
    lod_count: u32,           // 4 bytes - number of LOD levels
    lod_offset: u32,          // 4 bytes - offset in LOD array
    padding: u32,             // 4 bytes - alignment
}
```

### LOD Level Structure (16 bytes)
```rust
struct GpuLodLevel {
    mesh_id: u32,             // 4 bytes - mesh for this LOD
    min_distance_sq: f32,     // 4 bytes - min distance threshold
    max_distance_sq: f32,     // 4 bytes - max distance threshold
    padding: u32,             // 4 bytes - alignment
}
```

### Memory Requirements

| Objects | Object Data | LOD Levels (3/obj) | Total VRAM |
|---------|-------------|-------------------|------------|
| 100     | 9.6 KB      | 4.8 KB            | ~15 KB     |
| 1,000   | 96 KB       | 48 KB             | ~150 KB    |
| 10,000  | 960 KB      | 480 KB            | ~1.5 MB    |
| 100,000 | 9.6 MB      | 4.8 MB            | ~15 MB     |

## Related Benchmarks

- **`bench_gpu_vs_cpu_culling`**: Frustum culling comparison
- **`bench_multi_draw_indirect`**: Indirect draw performance
- **`bench_material_batching_optimization`**: Material system optimization

## Technical References

- Shader code: `crates/praxis_graphics/src/shaders/lod_selection.comp`
- LOD system: `crates/praxis_graphics/src/lod.rs`
- GPU LOD example: `examples/lod_gpu_demo.rs`

## Conclusion

This benchmark demonstrates that GPU LOD selection:
1. **Scales efficiently** to 100,000+ objects
2. **Has minimal CPU overhead** (constant time)
3. **Integrates naturally** with GPU-driven rendering
4. **Becomes advantageous** at moderate to large object counts

The O(1) CPU overhead is particularly important for maintaining consistent frame times in large, dynamic scenes.
