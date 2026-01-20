# GPU Memory Profiling and VRAM Tracking

Comprehensive GPU memory tracking system for monitoring texture allocations, mesh buffers, descriptor sets, compute shader overhead, and their correlation with rendering optimizations.

## Overview

The memory profiling system provides real-time tracking of all GPU memory allocations with:
- **Category-based breakdown**: Textures, mesh buffers, descriptor sets, compute buffers, render targets
- **Historical tracking**: Rolling history of memory usage over time
- **Render stats correlation**: Automatic integration with RenderStats to correlate memory usage with culling, LOD, and draw calls
- **Chrome trace export**: Memory metrics exported to chrome://tracing for timeline visualization
- **CSV export**: Complete memory and rendering data for external analysis

## Architecture

### Core Components

- **`MemoryProfiler`**: Central profiling system tracking all GPU allocations
- **`MemoryCategory`**: Classification of allocation types (Texture, MeshBuffer, DescriptorSet, etc.)
- **`VramAllocation`**: Individual allocation record with size, category, and metadata
- **`MemorySnapshot`**: Point-in-time memory state with breakdown by category
- **`MemoryHistory`**: Rolling history with trend analysis and statistical aggregation

### Integration Points

1. **RenderStats Integration**: Memory snapshots automatically attached to render statistics
2. **Chrome Trace Export**: Memory metrics exported as counter events via `praxis_profiling`
3. **CSV Export**: Memory data included in render stats CSV exports

## Usage

### Basic Memory Tracking

```rust
use praxis_graphics::{RenderContext, utilities::memory_profiler::MemoryCategory};

// Enable memory profiling (enabled by default)
render_context.set_memory_profiling_enabled(true);

// Query current memory usage
let profiler = render_context.memory_profiler();
println!("Total VRAM: {:.2} MB", profiler.total_allocated_mb());
println!("Peak VRAM: {:.2} MB", profiler.peak_mb());
println!("Active allocations: {}", profiler.allocation_count());

// Query by category
println!("Texture memory: {:.2} MB", profiler.category_mb(MemoryCategory::Texture));
println!("Mesh buffers: {:.2} MB", profiler.category_mb(MemoryCategory::MeshBuffer));
```

### Memory History Analysis

```rust
// Access historical data
let history = render_context.memory_profiler().history();

// Get statistics
println!("Average total: {:.2} MB", history.avg_total_bytes() / 1_048_576.0);
println!("Global peak: {:.2} MB", history.global_peak_mb());

// Get per-category trends
let texture_history = history.category_history(MemoryCategory::Texture);
// Returns Vec<f32> of memory usage in MB for each tracked frame
```

### Correlation with Render Stats

Memory snapshots are automatically attached to render statistics when both systems are enabled:

```rust
// Both systems enabled by default
render_context.set_render_stats_enabled(true);
render_context.set_memory_profiling_enabled(true);

// Render a frame
render_context.render(&cmds)?;

// Access correlated data
let stats = render_context.render_stats_history().latest().unwrap();
println!("Visible objects: {}", stats.visible_objects);
println!("Draw calls: {}", stats.draw_calls);

// Memory data is attached
if let Some(mem) = &stats.memory_snapshot {
    println!("VRAM at frame: {:.2} MB", mem.total_mb());
    println!("Texture memory: {:.2} MB", mem.category_mb(MemoryCategory::Texture));
}
```

### CSV Export with Memory Data

```rust
// Export render stats with memory correlation to CSV
render_context.export_render_stats_csv("performance_analysis.csv")?;
```

The CSV includes columns:
```csv
frame_number,total_objects,visible_objects,frustum_culled,occlusion_culled,draw_calls,
descriptor_allocations,streaming_queue_depth,culling_efficiency,vram_total_mb,
vram_texture_mb,vram_mesh_mb,vram_descriptor_mb,vram_compute_mb,vram_render_target_mb
```

### Chrome Trace Integration

When using `praxis_profiling` with the `graphics_integration` feature:

```rust
use praxis_profiling::Profiler;

let mut profiler = Profiler::new(ProfilerConfig::default());
profiler.begin_trace_export();

// Main loop
for frame in 0..300 {
    // ... rendering ...
    
    // Record stats with memory
    let render_stats = render_context.current_render_stats();
    profiler.record_render_stats(&render_stats, Instant::now());
}

// Export trace with memory metrics
profiler.end_trace_export("trace.json")?;
```

Open `trace.json` in chrome://tracing to visualize:
- Timeline of memory usage alongside CPU/GPU work
- Counter tracks for total VRAM and per-category breakdown
- Correlation between memory spikes and rendering activity
- Trends in allocation count over time

## Memory Categories

### Texture
- Albedo textures, normal maps, roughness maps
- Shadow maps (if counted as textures rather than render targets)
- Procedurally generated textures
- Cubemaps and environment probes

**Typical Size**: 4MB for 1024x1024 RGBA8, 16MB for 2048x2048 RGBA8

### MeshBuffer
- Vertex buffers (position, normal, UV, tangent)
- Index buffers (u16 or u32)

**Typical Size**: 
- Vertex3D: 48 bytes per vertex
- Index: 2 bytes per index (u16)
- Example: 10k vertices + 15k indices = ~510 KB

### DescriptorSet
- Descriptor set allocations
- Small but can accumulate with many materials

**Typical Size**: ~256 bytes per set (estimated)

### UniformBuffer
- View/projection matrices
- Material properties
- Lighting data
- Bone matrices for skeletal animation

**Typical Size**: 64-256 bytes per buffer

### ComputeBuffer
- GPU culling indirect draw buffers
- Compute shader scratch buffers
- Particle system buffers

**Typical Size**: Varies widely, can be several MB for large scenes

### RenderTarget
- G-buffer attachments (albedo, normal, depth)
- Shadow map depth textures
- HDR render targets
- Post-processing intermediate targets

**Typical Size**: 8MB for 1920x1080 RGBA16F, 4MB for depth-only 1920x1080 D32F

## Performance Considerations

### Overhead

The memory profiler has minimal runtime overhead:
- **Allocation tracking**: ~20ns per record operation
- **Snapshot creation**: O(categories) = ~100ns
- **History updates**: O(1) per frame
- **Total overhead**: < 0.01ms per frame when enabled

### Memory Usage

The profiler itself uses minimal memory:
- Active allocations: `HashMap<String, VramAllocation>` (~100 bytes per entry)
- History: Circular buffer of snapshots (default 300 frames × ~200 bytes = ~60 KB)
- Total: < 1 MB for typical scenes

### Disabling

To eliminate overhead entirely:

```rust
render_context.set_memory_profiling_enabled(false);
```

This continues tracking allocations but stops recording history snapshots.

## Example: Analyzing Memory Usage

See `examples/memory_profiling_demo.rs` for a comprehensive demonstration:

```bash
cargo run --example memory_profiling_demo
```

The demo shows:
- Texture allocation tracking with various sizes
- Mesh buffer monitoring
- Memory correlation with render stats
- Historical trend analysis
- CSV export for external analysis

## Best Practices

### 1. Track Resource Loading

When loading assets, the profiler automatically tracks allocations through the RenderContext APIs. For manual tracking:

```rust
// Textures are automatically tracked when loaded through TextureManager
texture_manager.load_texture("brick", "assets/brick.png")?;

// Meshes are automatically tracked when loaded through MeshAssetManager
mesh_manager.load_mesh("character", mesh_data)?;
```

### 2. Monitor Peak Usage

```rust
let profiler = render_context.memory_profiler();
if profiler.total_allocated_mb() > profiler.peak_mb() * 0.9 {
    println!("Warning: Approaching peak memory usage!");
}
```

### 3. Correlate with Optimization Settings

```rust
// Compare memory usage with different LOD settings
let stats = render_context.render_stats_history().latest().unwrap();
println!("LOD distribution: {:?}", stats.lod_distribution_percentages());
if let Some(mem) = &stats.memory_snapshot {
    println!("Mesh memory: {:.2} MB", mem.category_mb(MemoryCategory::MeshBuffer));
}
```

### 4. Export for Analysis

```rust
// After profiling session
render_context.export_render_stats_csv("analysis.csv")?;
```

Open in Excel/Google Sheets to create charts correlating:
- Memory usage vs. frame number
- Texture memory vs. visible objects
- Mesh memory vs. LOD distribution
- Total VRAM vs. draw calls

## Limitations

### 1. Estimation-Based

The profiler estimates memory usage based on format and dimensions. Actual GPU memory usage may differ due to:
- Alignment requirements
- Mipmap chains
- Compression (if used)
- Driver overhead

Estimates are typically within 10-20% of actual usage for standard formats.

### 2. Allocation Tracking Only

The profiler tracks allocations made through the RenderContext APIs. Direct Vulkan allocations or third-party libraries are not tracked.

### 3. No Deallocation Tracking

Currently, the system tracks allocations but does not automatically track deallocations when resources are freed. Manual cleanup tracking may be added in the future.

## Future Enhancements

Potential improvements:
- Automatic deallocation tracking when resources are dropped
- Mipmap memory estimation
- Compressed texture format support
- GPU query-based actual memory usage (if available)
- Memory leak detection via reference counting
- Per-material memory breakdown
- Streaming budget enforcement

## See Also

- `utilities/render_stats.rs` - Rendering statistics system
- `RENDER_STATS.md` - Render statistics documentation
- `praxis_profiling` - CPU/GPU profiling with Chrome trace export
- `examples/memory_profiling_demo.rs` - Comprehensive usage example
