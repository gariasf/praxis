# Graphics Optimization Benchmarks

## GPU vs CPU Culling Benchmark

This benchmark compares the performance of CPU-based frustum culling versus GPU compute shader culling for large numbers of objects.

### Benchmark Configurations

The benchmark tests three object counts:
- **1,000 objects**: Small scene baseline
- **5,000 objects**: Medium-sized scene
- **10,000 objects**: Large scene where GPU culling shines

### Test Scenarios

#### 1. CPU Frustum Culling (`cpu_culling`)
Simulates traditional CPU-side frustum culling:
- Sequential testing of each object against frustum planes
- Sphere-frustum intersection tests (6 plane tests per object)
- Models real-world CPU culling overhead
- Expected complexity: **O(N)** where N is object count

**Implementation:**
```rust
for (center, radius) in &objects {
    for plane in &frustum_planes {
        distance = dot(plane, center) + plane.w;
        if distance < -radius { culled; break; }
    }
}
```

#### 2. GPU Compute Culling (`gpu_culling`)
Tests GPU-driven compute shader culling:
- Parallel processing on GPU (64 threads per workgroup)
- All objects tested simultaneously
- Includes CPU overhead + GPU execution time
- Expected complexity: **O(1)** CPU time (constant overhead)

**Implementation:**
- Upload draw commands to GPU
- Dispatch compute shader
- Wait for GPU completion
- Read back visible count

#### 3. CPU Overhead Only (`cpu_overhead_only`)
Measures CPU-side overhead of GPU culling without GPU execution:
- Command buffer building
- Descriptor set binding
- Dispatch recording
- Demonstrates O(1) CPU cost property

### Running the Benchmark

```bash
# Run all graphics optimization benchmarks
cargo bench --bench graphics_optimization

# Run only the GPU vs CPU culling benchmark
cargo bench --bench graphics_optimization -- gpu_vs_cpu_culling

# Save baseline for comparison
cargo bench --bench graphics_optimization -- gpu_vs_cpu_culling --save-baseline gpu-culling-baseline

# Compare against baseline
cargo bench --bench graphics_optimization -- gpu_vs_cpu_culling --baseline gpu-culling-baseline
```

### Expected Results

#### CPU Culling Scalability
- 1,000 objects: ~50-100 µs
- 5,000 objects: ~250-500 µs
- 10,000 objects: ~500-1000 µs

**Linear scaling**: Time increases proportionally with object count.

#### GPU Culling Scalability
- 1,000 objects: ~200-400 µs (total, including GPU execution)
- 5,000 objects: ~300-500 µs
- 10,000 objects: ~400-600 µs

**Sublinear scaling**: Time increases much slower than object count.

#### CPU Overhead Only
- 1,000 objects: ~10-30 µs
- 5,000 objects: ~10-30 µs
- 10,000 objects: ~10-30 µs

**Constant time**: O(1) CPU overhead regardless of object count!

### Key Insights

1. **Crossover Point**: GPU culling becomes more efficient around 5,000-10,000 objects
2. **CPU Overhead**: GPU culling has ~O(1) CPU cost, proven by `cpu_overhead_only` measurements
3. **Scalability**: GPU culling scales to 50,000+ objects with minimal additional CPU cost
4. **Parallel Advantage**: GPU processes all objects in parallel vs CPU's sequential processing

### Performance Analysis

The benchmark demonstrates that:

- **Small scenes (< 1,000 objects)**: CPU culling is faster due to lower overhead
- **Medium scenes (1,000-5,000 objects)**: GPU culling starts to compete
- **Large scenes (> 5,000 objects)**: GPU culling clearly wins

### Benchmark Methodology

All tests use:
- Identical object distributions (3D grid layout)
- Same frustum configuration
- Approximately 50% visibility rate (realistic scenario)
- Sphere bounding volumes for consistency

### Hardware Requirements

GPU culling requires:
- Vulkan-capable GPU
- Compute shader support
- At least 256MB VRAM for largest tests

### Interpreting Results

Sample output format:
```
gpu_vs_cpu_culling/cpu_culling/1000
                        time:   [75.234 µs 76.891 µs 78.642 µs]
                        thrpt:  [12.719 Melem/s 13.007 Melem/s 13.290 Melem/s]

gpu_vs_cpu_culling/gpu_culling/1000
                        time:   [234.12 µs 238.45 µs 243.01 µs]
                        thrpt:  [4.115 Melem/s 4.195 Melem/s 4.271 Melem/s]

gpu_vs_cpu_culling/cpu_overhead_only/1000
                        time:   [15.234 µs 15.891 µs 16.542 µs]
```

**Key metrics:**
- `time`: Mean execution time with confidence interval
- `thrpt`: Throughput (objects processed per second)
- Lower time = better performance
- Higher throughput = better performance

### Verification

The benchmark includes verification steps:
- Visible count is compared between CPU and GPU methods
- Results should match (both see same objects as visible)
- Validates correctness of both approaches

### Optimization Opportunities

Based on benchmark results:
1. Use CPU culling for scenes < 1,000 objects
2. Use GPU culling for scenes > 5,000 objects
3. Consider hybrid approach for 1,000-5,000 object range
4. GPU culling enables indirect draw, further reducing CPU overhead

### Related Benchmarks

See also:
- `bench_multi_draw_indirect`: Tests indirect draw performance
- `bench_material_batching_optimization`: Material batching benefits
- `bench_staging_buffer_pooling`: Buffer upload optimization
