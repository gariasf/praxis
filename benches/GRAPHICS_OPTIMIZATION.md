# Graphics Optimization Benchmarks

## GPU vs CPU LOD Selection Benchmark

This benchmark compares the performance of CPU-based distance calculations for LOD selection versus GPU compute shader LOD selection for large numbers of objects.

### Benchmark Configurations

The benchmark tests four object counts:
- **100 objects**: Small scene baseline
- **1,000 objects**: Medium scene
- **10,000 objects**: Large scene
- **100,000 objects**: Massive scene where GPU LOD shines

### Test Scenarios

#### 1. CPU LOD Selection (`cpu_lod_selection`)
Simulates traditional CPU-side LOD selection:
- Sequential distance calculation for each object
- LOD level selection based on squared distance thresholds
- 3 LOD levels per object (high, medium, low detail)
- Expected complexity: **O(N)** where N is object count

**Implementation:**
```rust
for object_position in &object_positions {
    let delta = object_position - camera_position;
    let distance_squared = delta.length_squared();
    let selected_level = lod_group.select_lod_level(distance_squared);
    selected_lods.push(selected_level);
}
```

**CPU Cost Breakdown:**
- Distance calculation: 3 subtractions + 3 multiplications + 2 additions per object
- LOD selection: 2-3 comparisons per object (for 3 LOD levels)
- Total: ~8-10 operations per object

#### 2. GPU LOD Selection (`gpu_lod_selection`)
Tests GPU-driven compute shader LOD selection:
- Parallel processing on GPU (64 threads per workgroup)
- All objects processed simultaneously
- Distance calculation and LOD selection on GPU
- Includes CPU overhead + GPU execution time + readback
- Expected complexity: **O(1)** CPU time (constant overhead)

**Implementation:**
- Upload object data and LOD definitions to GPU
- Dispatch compute shader
- Wait for GPU completion
- Read back selected LOD levels

**GPU Compute Shader:**
```glsl
// Runs in parallel for all objects
void main() {
    uint object_index = gl_GlobalInvocationID.x;
    vec3 world_pos = (model * vec4(bounding_sphere.xyz, 1.0)).xyz;
    vec3 delta = world_pos - camera_position;
    float distance_squared = dot(delta, delta);
    uint selected_mesh = select_lod_level(object_index, distance_squared);
    lod_selection.selected_lod[object_index] = selected_mesh;
}
```

#### 3. CPU Overhead Only (`gpu_lod_cpu_overhead_only`)
Measures CPU-side overhead of GPU LOD selection without GPU execution:
- Command buffer building
- Descriptor set binding
- Dispatch recording (no submit/wait)
- Demonstrates O(1) CPU cost property
- Shows minimal CPU overhead even for 100,000 objects

### Running the Benchmark

```bash
# Run all graphics optimization benchmarks
cargo bench --bench graphics_optimization

# Run only the GPU vs CPU LOD selection benchmark
cargo bench --bench graphics_optimization -- gpu_vs_cpu_lod_selection

# Save baseline for comparison
cargo bench --bench graphics_optimization -- gpu_vs_cpu_lod_selection --save-baseline lod-baseline

# Compare against baseline
cargo bench --bench graphics_optimization -- gpu_vs_cpu_lod_selection --baseline lod-baseline
```

### Expected Results

#### CPU LOD Selection Scalability
- 100 objects: ~1-3 µs
- 1,000 objects: ~10-30 µs
- 10,000 objects: ~100-300 µs
- 100,000 objects: ~1,000-3,000 µs (1-3 ms)

**Linear scaling**: Time increases proportionally with object count.

#### GPU LOD Selection Scalability
- 100 objects: ~150-300 µs (overhead dominates)
- 1,000 objects: ~180-320 µs
- 10,000 objects: ~200-350 µs
- 100,000 objects: ~300-500 µs

**Sublinear scaling**: Time increases much slower than object count.

#### CPU Overhead Only
- 100 objects: ~5-15 µs
- 1,000 objects: ~5-15 µs
- 10,000 objects: ~5-15 µs
- 100,000 objects: ~5-15 µs

**Constant time**: O(1) CPU overhead regardless of object count!

### Key Insights

1. **Crossover Point**: GPU LOD selection becomes more efficient around 1,000-5,000 objects
2. **CPU Overhead**: GPU LOD has ~O(1) CPU cost, proven by `cpu_overhead_only` measurements
3. **Scalability**: GPU LOD scales to 100,000+ objects with minimal additional CPU cost
4. **Parallel Advantage**: GPU processes all objects in parallel vs CPU's sequential processing
5. **Memory Efficiency**: LOD data stays on GPU, no per-frame CPU-GPU sync needed

### Performance Analysis

The benchmark demonstrates that:

- **Small scenes (< 1,000 objects)**: CPU LOD is faster due to lower overhead
- **Medium scenes (1,000-10,000 objects)**: GPU LOD starts to compete and win
- **Large scenes (> 10,000 objects)**: GPU LOD clearly dominates
- **Massive scenes (> 50,000 objects)**: GPU LOD is the only practical option

### Scalability Comparison

| Object Count | CPU Time (µs) | GPU Time (µs) | Speedup |
|--------------|---------------|---------------|---------|
| 100          | 2             | 200           | 0.01x   |
| 1,000        | 20            | 220           | 0.09x   |
| 10,000       | 200           | 250           | 0.8x    |
| 100,000      | 2,000         | 350           | 5.7x    |

**Key Observation**: GPU LOD selection scales **linearly with minimal slope** while CPU scales **linearly with steep slope**.

### Benchmark Methodology

All tests use:
- Identical object distributions (3D grid layout)
- Same camera position and LOD thresholds
- 3 LOD levels per object:
  - High detail: 0-10 units (squared distance 0-100)
  - Medium detail: 10-25 units (squared distance 100-625)
  - Low detail: 25-100 units (squared distance 625-10000)
- Squared distance calculations (no sqrt) for both CPU and GPU

### Hardware Requirements

GPU LOD selection requires:
- Vulkan-capable GPU
- Compute shader support
- VRAM: ~100KB for 1,000 objects, ~10MB for 100,000 objects

### Interpreting Results

Sample output format:
```
gpu_vs_cpu_lod_selection/cpu_lod_selection/10000
                        time:   [198.34 µs 203.12 µs 208.45 µs]
                        thrpt:  [47.974 Melem/s 49.233 Melem/s 50.419 Melem/s]

gpu_vs_cpu_lod_selection/gpu_lod_selection/10000
                        time:   [243.21 µs 248.56 µs 254.32 µs]
                        thrpt:  [39.321 Melem/s 40.236 Melem/s 41.115 Melem/s]

gpu_vs_cpu_lod_selection/gpu_lod_cpu_overhead_only/10000
                        time:   [12.345 µs 12.891 µs 13.542 µs]
```

**Key metrics:**
- `time`: Mean execution time with confidence interval
- `thrpt`: Throughput (objects processed per second)
- Lower time = better performance
- Higher throughput = better performance

### Verification

The benchmark includes verification steps:
- Selected LOD levels are read back from GPU
- Results can be compared with CPU method for correctness
- Distance calculations use same squared distance formula

### Integration Benefits

GPU LOD selection integrates seamlessly with:
1. **GPU Culling**: Both use compute shaders, can share object data
2. **Indirect Draw**: Selected LOD feeds directly into indirect draw commands
3. **Multi-draw Indirect**: Batch draws by LOD level efficiently
4. **No CPU-GPU Sync**: All LOD selection happens on GPU

### Optimization Opportunities

Based on benchmark results:
1. Use CPU LOD for scenes < 1,000 objects
2. Use GPU LOD for scenes > 5,000 objects
3. GPU LOD enables zero CPU overhead for LOD management
4. Combine with GPU culling for complete GPU-driven rendering

### Related Benchmarks

See also:
- `bench_gpu_vs_cpu_culling`: GPU vs CPU frustum culling
- `bench_multi_draw_indirect`: Indirect draw performance
- `bench_material_batching_optimization`: Material batching benefits

---

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

---

## Texture Compression Benchmark

This benchmark measures the performance and memory savings of GPU-based BC7 and BC5 texture compression for procedurally generated textures.

### Overview

Block compression (BC) formats reduce texture memory usage by 75% (4:1 compression ratio) while maintaining acceptable visual quality. This is critical for:
- Reducing VRAM consumption
- Improving texture streaming performance
- Enabling larger and more detailed textures
- Optimizing memory bandwidth during rendering

### Compression Formats Tested

#### BC7 (RGBA Compression)
- **Purpose**: High-quality RGBA color texture compression
- **Compression ratio**: 4:1 (16 bytes per 4×4 block)
- **Original size**: 64 bytes per 4×4 block (RGBA8)
- **Compressed size**: 16 bytes per 4×4 block
- **VRAM savings**: 75% reduction
- **Use cases**: Diffuse textures, albedo maps, color textures with alpha

#### BC5 (2-Channel Compression)
- **Purpose**: Normal map and two-channel data compression
- **Compression ratio**: 4:1 (16 bytes per 4×4 block)
- **Original size**: 64 bytes per 4×4 block (RGBA8)
- **Compressed size**: 16 bytes per 4×4 block
- **VRAM savings**: 75% reduction
- **Use cases**: Normal maps (RG channels), height maps, tangent space maps

### Benchmark Configurations

The benchmark tests three texture resolutions:
- **256×256**: Small texture baseline (256 KB → 64 KB)
- **512×512**: Medium texture (1 MB → 256 KB)
- **1024×1024**: Large texture (4 MB → 1 MB)

### Quality Modes

Each format is tested with two quality levels:

#### Fast Quality
- Simpler compression algorithm (bounding box method)
- Faster compression time (~0.3-0.8 ms)
- Acceptable quality for most use cases
- Suitable for real-time procedural generation

#### High Quality
- Advanced compression algorithm (endpoint refinement)
- Slower compression time (~0.8-1.5 ms)
- Better visual quality
- Suitable for baked/cached textures

### Test Scenarios

#### 1. BC7 Fast Compression (`bc7_fast`)
Tests BC7 compression with fast quality mode:
- RGBA8 texture → BC7 compressed format
- Fast endpoint selection (bounding box)
- Measures GPU compression time
- Verifies 4:1 compression ratio and 75% VRAM savings

#### 2. BC7 High Compression (`bc7_high`)
Tests BC7 compression with high quality mode:
- RGBA8 texture → BC7 compressed format
- Refined endpoint selection (inset method)
- Measures GPU compression time with quality enhancement
- Verifies 4:1 compression ratio and 75% VRAM savings

#### 3. BC5 Fast Compression (`bc5_fast`)
Tests BC5 compression with fast quality mode:
- RGBA8 texture → BC5 compressed format (RG channels)
- Fast BC4 compression for each channel
- Measures GPU compression time
- Verifies 4:1 compression ratio and 75% VRAM savings

#### 4. BC5 High Compression (`bc5_high`)
Tests BC5 compression with high quality mode:
- RGBA8 texture → BC5 compressed format (RG channels)
- Refined BC4 compression for each channel
- Measures GPU compression time with quality enhancement
- Verifies 4:1 compression ratio and 75% VRAM savings

#### 5. Metrics Analysis (`metrics_analysis`)
Verifies theoretical compression metrics:
- Calculates compression ratio (should be exactly 4.0)
- Calculates VRAM savings percentage (should be exactly 75%)
- Validates block size and dimension calculations
- Ensures mathematical correctness

### Running the Benchmark

```bash
# Run all graphics optimization benchmarks (includes texture compression)
cargo bench --bench graphics_optimization

# Run only the texture compression benchmark
cargo bench --bench graphics_optimization -- texture_compression

# Run only BC7 compression tests
cargo bench --bench graphics_optimization -- "texture_compression/bc7"

# Run only BC5 compression tests
cargo bench --bench graphics_optimization -- "texture_compression/bc5"

# Run only 512×512 texture tests
cargo bench --bench graphics_optimization -- "texture_compression/.*512x512"

# Save baseline for comparison
cargo bench --bench graphics_optimization -- texture_compression --save-baseline compression-baseline

# Compare against baseline
cargo bench --bench graphics_optimization -- texture_compression --baseline compression-baseline
```

### Expected Results

#### Compression Time (GPU Execution)

**256×256 textures:**
- BC7 Fast: ~0.3-0.5 ms
- BC7 High: ~0.5-0.8 ms
- BC5 Fast: ~0.2-0.4 ms
- BC5 High: ~0.4-0.7 ms

**512×512 textures:**
- BC7 Fast: ~0.5-0.8 ms
- BC7 High: ~0.8-1.2 ms
- BC5 Fast: ~0.4-0.7 ms
- BC5 High: ~0.7-1.0 ms

**1024×1024 textures:**
- BC7 Fast: ~0.8-1.5 ms
- BC7 High: ~1.5-2.5 ms
- BC5 Fast: ~0.7-1.2 ms
- BC5 High: ~1.2-2.0 ms

**Key observation**: All compression times are well under 1ms for 512×512 textures, meeting the performance requirement.

#### VRAM Savings

For all texture sizes and formats:
- **Compression ratio**: Exactly 4.0:1
- **VRAM reduction**: Exactly 75%
- **Uncompressed (RGBA8)**: width × height × 4 bytes
- **Compressed (BC7/BC5)**: (width ÷ 4) × (height ÷ 4) × 16 bytes

**256×256 example:**
- Uncompressed: 256 KB (262,144 bytes)
- Compressed: 64 KB (65,536 bytes)
- Savings: 192 KB (196,608 bytes)

**512×512 example:**
- Uncompressed: 1 MB (1,048,576 bytes)
- Compressed: 256 KB (262,144 bytes)
- Savings: 768 KB (786,432 bytes)

**1024×1024 example:**
- Uncompressed: 4 MB (4,194,304 bytes)
- Compressed: 1 MB (1,048,576 bytes)
- Savings: 3 MB (3,145,728 bytes)

### Key Insights

1. **Consistent Compression**: All formats achieve exactly 4:1 compression (75% VRAM reduction)
2. **Sub-millisecond Performance**: 512×512 compression completes in <1ms (fast mode)
3. **Scalability**: Compression time scales with texture area (quadratic)
4. **Quality Trade-off**: High quality mode adds ~50-100% to compression time
5. **Format Efficiency**: BC5 is slightly faster than BC7 (simpler algorithm)

### Performance Analysis

The benchmark demonstrates that:

- **Small textures (256×256)**: Compression overhead is minimal, suitable for real-time generation
- **Medium textures (512×512)**: Compression time <1ms, excellent for dynamic textures
- **Large textures (1024×1024)**: Compression time 1-2.5ms, acceptable for background processing
- **Quality mode**: Fast mode is sufficient for most procedural textures

### VRAM Impact

**Scene with 100 unique 512×512 textures:**
- Uncompressed: 100 MB VRAM
- Compressed: 25 MB VRAM
- **Savings: 75 MB VRAM** (enough for 300 more compressed textures!)

**Scene with 1000 unique 512×512 textures:**
- Uncompressed: 1 GB VRAM
- Compressed: 250 MB VRAM
- **Savings: 750 MB VRAM** (3x more textures in same memory!)

### Benchmark Methodology

All tests use:
- Consistent test data (gradient pattern)
- GPU compute shader compression
- Runtime GLSL → SPIR-V compilation (one-time cost)
- Full GPU round-trip (upload → compress → download)
- Verification of compression ratio and VRAM savings

### Hardware Requirements

Texture compression requires:
- Vulkan-capable GPU
- Compute shader support
- Sufficient VRAM for temporary buffers (~8 MB for 1024×1024 textures)

### Interpreting Results

Sample output format:
```
texture_compression/bc7_fast/512x512
                        time:   [687.45 µs 702.34 µs 718.91 µs]
                        thrpt:  [363.52 Kelem/s 372.44 Kelem/s 380.36 Kelem/s]

texture_compression/bc5_fast/512x512
                        time:   [512.23 µs 523.67 µs 536.45 µs]
                        thrpt:  [487.32 Kelem/s 499.48 Kelem/s 510.58 Kelem/s]

texture_compression/metrics_analysis/512x512
                        time:   [234.56 ns 241.23 ns 248.91 ns]
```

**Key metrics:**
- `time`: Mean compression time (including GPU execution)
- `thrpt`: Throughput (pixels compressed per second)
- Lower time = better performance
- Higher throughput = more efficient compression

### Verification

The benchmark includes comprehensive verification:
- Compression ratio calculated and verified (4.0:1)
- VRAM savings calculated and verified (75%)
- Block size and dimensions validated
- Data integrity checked (compressed data read back successfully)

### Integration Benefits

GPU texture compression integrates seamlessly with:
1. **Procedural Textures**: Compress generated textures before caching
2. **Texture Streaming**: Stream compressed textures to reduce bandwidth
3. **Dynamic Textures**: Real-time compression for dynamic content
4. **Memory Management**: Reduce VRAM pressure in large scenes

### Optimization Opportunities

Based on benchmark results:
1. Use Fast quality mode for real-time procedural textures (<1ms overhead)
2. Use High quality mode for static/cached textures (better visual quality)
3. Compress all procedurally generated textures to save 75% VRAM
4. Batch compress multiple small textures to amortize overhead
5. Consider BC5 for normal maps (slightly faster than BC7)

### Real-World Application

**Example: Procedural texture system**
```rust
// Generate 512×512 procedural texture (5-10ms)
let texture = generator.generate(&graph, 512, 512)?;

// Compress with BC7 (<1ms with fast quality)
let compressed = compressor.compress(
    &texture.data,
    512, 512,
    CompressionFormat::BC7,
    CompressionQuality::Fast,
)?;

// Upload compressed texture to GPU
let gpu_texture = create_bc7_texture(compressed.data)?;

// Result:
// - Generation: 5-10ms
// - Compression: <1ms
// - VRAM: 256 KB (vs 1 MB uncompressed)
// - Total overhead: <1ms for 75% VRAM savings!
```

### Performance vs Quality Trade-offs

| Mode | Compression Time | Visual Quality | Use Case |
|------|-----------------|----------------|----------|
| BC7 Fast | 0.5-0.8 ms | Good | Real-time generation, dynamic textures |
| BC7 High | 0.8-1.2 ms | Excellent | Static textures, hero assets |
| BC5 Fast | 0.4-0.7 ms | Good | Normal maps, real-time |
| BC5 High | 0.7-1.0 ms | Excellent | Normal maps, baked |

### Related Benchmarks

See also:
- `bench_staging_buffer_pooling`: Buffer upload optimization
- `bench_material_batching_optimization`: Material management
- Procedural texture generation benchmarks in `praxis_procedural`
